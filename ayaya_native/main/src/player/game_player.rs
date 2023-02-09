use std::sync::{mpsc::{channel, Receiver, Sender}, Arc};

use anyhow::anyhow;
use font_kit::{handle::Handle, canvas::{Canvas, Format, RasterizationOptions}, hinting::HintingOptions};
use once_cell::sync::Lazy;
use pathfinder_geometry::{vector::{Vector2I, Vector2F}, transform2d::Transform2F};

use super::player_context::{NativeCommunication, VideoData, VideoPlayer};
use crate::{colorlib::Color, map_server::ServerOptions, splitting::SplittedFrame, apps::{snake::SnakeGame, falling_blocks::FallingBlocks, calculator::Calculator}};

static DEFAULT_FONT_BYTES: &'static [u8] = include_bytes!("../../WorkSans-VariableFont_wght.ttf");
static DEFAULT_FONT: Lazy<Handle> = Lazy::new(|| {
    Handle::from_memory(Arc::new(DEFAULT_FONT_BYTES.to_vec()), 0)
});

pub struct VideoCanvas {
    pub width: usize,
    pub height: usize,
    vec: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameInputDirection {
    Forward,
    Backwards,
    Left,
    Right,
    Up,
}

impl VideoCanvas {
    pub fn new(width: usize, height: usize, start_color: &Color) -> Self {
        let vec: Vec<u8> = vec![start_color.convert_to_mc(); width * height];

        Self { width, height, vec }
    }

    pub fn new_from_image(image: &BakedImage) -> Self {
        Self {
            vec: image.data.to_vec(),
            width: image.width as usize,
            height: image.height as usize
        } 
    }

    #[allow(dead_code)]
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: &Color) {
        self.vec[(y * self.width) + x] = color.convert_to_mc();
    }

    pub fn draw_square(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, color: &Color) {
        let x1 = x1.min(x2);
        let x2 = x1.max(x2);

        let y1 = y1.min(y2);
        let y2 = y1.max(y2);

        let width = x2 - x1;

        let data_to_copy: Vec<u8> = vec![color.convert_to_mc(); width + 1];

        for y in y1..=y2 {
            self.vec[((y * self.width) + x1)..=((y * self.width) + x2)]
                .copy_from_slice(&data_to_copy);
        }
    }

    /// Draws baked image
    ///
    /// X and Y are the top left coordinates of the image
    pub fn draw_image(&mut self, x: usize, y: usize, image: &BakedImage) {
        let y1 = y + image.height as usize;

        for (i, y) in (y..y1).enumerate() {
            self.vec[((y * self.width) + x)..((y * self.width) + x + image.width as usize)]
                .copy_from_slice(
                    &image.data[(i * image.width as usize)..((i + 1) * image.width as usize)],
                );
        }
    }

    //We brake the "No result rule" in this function
    //We have no idea what is going to happen
    //it might fail
    pub fn draw_default_text(
        &mut self,
        x: usize,
        y: usize,
        font_size: i32,
        font_spacing: usize,
        color: &Color,
        text: &str
    ) -> anyhow::Result<()>{
        let font = DEFAULT_FONT.load()?;
        let mut text_x_offset = 0;

        for text_char in text.chars() {
            let glyph_id = match font.glyph_for_char(text_char) {
                Some(val) => val,
                None => return Err(anyhow!(format!("This font does not support character {:?}", text_char)))
            };


            let mut canvas = Canvas::new(Vector2I::new(font_size, font_size), Format::A8);
            font.rasterize_glyph(
                &mut canvas,
                glyph_id,
                font_size as f32,
                Transform2F::from_translation(Vector2F::new(0.0, font_size as f32)),
                HintingOptions::None,
                RasterizationOptions::GrayscaleAa,
            )?;

            let font_size = font_size as usize;

            for loop_y in 0..font_size {
                for loop_x in 0..font_size {
                    if canvas.pixels[loop_y * font_size + loop_x] != 0 {
                        self.draw_pixel(x + text_x_offset + loop_x, loop_y + y, &color); 
                    }
                }
            }

            text_x_offset += font_spacing; 
        }

        Ok(())
    }

    fn draw_to_minecraft(
        &self,
        splitted_frames: &mut Vec<SplittedFrame>,
        all_frames_x: i32,
        all_frames_y: i32,
    ) -> anyhow::Result<Vec<i8>> {
        SplittedFrame::split_frames(
            bytemuck::cast_slice(self.vec.as_slice()),
            splitted_frames,
            self.width as i32,
            all_frames_x,
            all_frames_y,
        )
    }
}

#[derive(Debug)]
pub struct BakedImage {
    pub width: u32,
    pub height: u32,
    pub data: &'static [u8],
}

#[macro_export]
macro_rules! bake_image {
    (
        $NAME: ident
    ) => {{
        let data: &'static [u8] =
            include_bytes!(concat!(env!("OUT_DIR"), "/", stringify!($NAME), ".bin"));
        let dimension_arr: &'static [u8] =
            include_bytes!(concat!(env!("OUT_DIR"), "/", stringify!($NAME), ".dim"));
        //static is hard man...
        let width_arr = [
            dimension_arr[0],
            dimension_arr[1],
            dimension_arr[2],
            dimension_arr[3],
        ];
        let width = u32::from_be_bytes(width_arr);

        let height_arr = [
            dimension_arr[4],
            dimension_arr[5],
            dimension_arr[6],
            dimension_arr[7],
        ];
        let height = u32::from_be_bytes(height_arr);

        BakedImage {
            width,
            height,
            data,
        }
    }};
}

pub trait Game {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn fps(&self) -> i32;
    fn new() -> Self
    where
        Self: Sized;
    fn draw(&mut self, input_rx: &Receiver<GameInputDirection>) -> anyhow::Result<VideoCanvas>;
}

pub struct GamePlayer {
    pub width: i32,
    pub height: i32,
    fps: i32,
    splitted_frames: Vec<SplittedFrame>,
    all_frames_x: i32,
    all_frames_y: i32,
    game: Box<dyn Game>,
    input_rx: Receiver<GameInputDirection>,
    input_tx: Sender<GameInputDirection>,
    last_frame: Vec<i8>,
    frame_counter: u8,
}

impl VideoPlayer for GamePlayer {
    fn create(file_name: String, _server_options: ServerOptions) -> anyhow::Result<Self> {
        let game: Box<dyn Game> = match file_name.as_str() {
            "falling_blocks" => Box::new(FallingBlocks::new()),
            "snake" => Box::new(SnakeGame::new()),
            "calculator" => Box::new(Calculator::new()),
            _ => return Err(anyhow!("This game is not implemented!")),
        };

        let (width, height, fps) = (game.width(), game.height(), game.fps());
        let (input_tx, input_rx) = channel::<GameInputDirection>();

        let (splitted_frames, all_frames_x, all_frames_y) =
            SplittedFrame::initialize_frames(width, height)?;

        Ok(Self {
            width,
            height,
            fps,
            splitted_frames,
            all_frames_x,
            all_frames_y,
            game,
            input_rx,
            input_tx,
            last_frame: Vec::new(),
            frame_counter: 0,
        })
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        let frame = if self.frame_counter == 0 {
            let canvas = self.game.draw(&self.input_rx)?;
            let frame = canvas.draw_to_minecraft(
                &mut self.splitted_frames,
                self.all_frames_x,
                self.all_frames_y,
            )?;
            self.last_frame = frame.clone();

            Ok(frame)
        } else {
            let new_frame = self.game.draw(&self.input_rx)?.draw_to_minecraft(
                &mut self.splitted_frames,
                self.all_frames_x,
                self.all_frames_y,
            )?;
            let mut frame_str_info = String::new();
            let mut frame_data = Vec::<i8>::with_capacity(65536);

            let mut offset: usize = 0;
            for (frame_inxex, frame) in self.splitted_frames.iter().enumerate() {
                let (mut x1, mut y1, mut x2, mut y2) = (128usize, 128usize, 0usize, 0usize);
                for y in 0..frame.height as usize {
                    for x in 0..frame.width as usize {
                        let old_pixel = self.last_frame[offset + (y * frame.width as usize) + x];
                        let new_pixel = new_frame[offset + (y * frame.width as usize) + x];

                        if old_pixel != new_pixel {
                            if x1 > x {
                                x1 = x;
                            }

                            if y1 > y {
                                y1 = y;
                            }

                            if x > x2 {
                                x2 = x;
                            }

                            if y > y2 {
                                y2 = y;
                            }
                        }
                    }
                }

                if x1 != 128 && y1 != 128 {
                    let width = x2 - x1 + 1; // + 1 due to the fact that x2 is inclusive;
                    let height = y2 - y1 + 1;

                    let mut data: Vec<i8> = Vec::with_capacity(width * height);

                    for y in y1..=y2 {
                        data.extend_from_slice(
                            &new_frame[(offset + ((y * frame.width as usize) + x1))
                                ..=(offset + ((y * frame.width as usize) + x2))],
                        )
                    }
                    //Format: {frame_inxex}_{width}_{height}_{x1}_{y1}$
                    frame_str_info.push_str(&format!(
                        "{frame_inxex:?}_{width:?}_{height:?}_{x1:?}_{y1:?}$"
                    ));
                    frame_data.extend(data);
                }

                offset += frame.frame_length as usize;
            }

            let frame_str_info = match frame_str_info.strip_suffix('$') {
                Some(string) => string,
                None => return Ok(vec![1]), //No change = no new packets
            };

            let frame_str_arr: &[i8] = bytemuck::cast_slice(frame_str_info.as_bytes());

            let mut final_data =
                Vec::<i8>::with_capacity(frame_str_arr.len() + 5 + frame_data.len());
            final_data.push(0); //Magic value
            final_data.extend_from_slice(bytemuck::cast_slice(
                &(frame_str_arr.len() as i32).to_be_bytes(),
            ));
            final_data.extend_from_slice(frame_str_arr);
            final_data.extend(frame_data);

            self.last_frame = new_frame;

            Ok(final_data)
        };

        if self.frame_counter + 1 != 3 {
            self.frame_counter += 1;
        } else {
            self.frame_counter = 0;
        };

        frame
    }

    fn video_data(&self) -> anyhow::Result<super::player_context::VideoData> {
        Ok(VideoData {
            width: self.width,
            height: self.height,
            fps: self.fps,
        })
    }

    fn handle_jvm_msg(&self, msg: NativeCommunication) -> anyhow::Result<()> {
        match msg {
            NativeCommunication::GameInput { input } => {
                for ele in &input {
                    self.input_tx.send(*ele)?;
                }
            }
            _ => {
                return Err(anyhow!(
                    "Gamep player does not accept jvm messages other than GameInputDirection"
                ))
            }
        }
        Ok(())
    }

    fn destroy(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
