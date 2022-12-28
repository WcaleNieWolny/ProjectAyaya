use std::sync::mpsc::{Receiver, Sender, channel};

use anyhow::anyhow;

use crate::{map_server::ServerOptions, colorlib::Color, splitting::SplittedFrame};
use super::{player_context::{VideoPlayer, VideoData, NativeCommunication}, flappy_bird::FlappyBirdGame, falling_blocks::FallingBlocks};

pub struct VideoCanvas {
    pub width: usize,
    pub height: usize,
    vec: Vec<u8>
}

#[derive(Debug, Clone, Copy)]
pub enum GameInputDirection {
    FORWARD,
    BACKWARDS,
    LEFT,
    RIGHT,
    UP
}

impl VideoCanvas {
    pub fn new(
        width: usize,
        height: usize,
        start_color: &Color, 
    ) -> Self {
        let vec: Vec<u8> = vec![start_color.to_mc() as u8; width*height];

        Self {
            width,
            height,
            vec
        }
    }

    pub fn draw_pixel(
        &mut self,
        x: usize,
        y: usize,
        color: &Color
    ){
        self.vec[(y * self.width) + x] = color.to_mc();
    }

    pub fn draw_square(
        &mut self,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        color: &Color
    ){
        let x1 = x1.min(x2);
        let x2 = x1.max(x2);

        let y1 = y1.min(y2);
        let y2 = y1.max(y2);

        let width = x2 - x1;

        let data_to_copy: Vec<u8> = vec![color.to_mc(); width]; 

        for y in y1..y2+1{
            self.vec[((y * self.width) + x1)..((y * self.width) + x2)].copy_from_slice(&data_to_copy);
        }
    }

    /// Draws baked image
    ///
    /// X and Y are the top left coordinates of the image
    pub fn draw_image(&mut self, x: usize, y: usize, image: &BakedImage){
        let y1 = y - image.height as usize;
        let mut i : usize = 0;

        for y in y1..y {
            self.vec[((y * self.width) + x)..((y * self.width) + x + image.width as usize)].copy_from_slice(&image.data[(i * image.width as usize)..((i + 1) * image.width as usize)]);
            i += 1;
        }
    }

    fn draw_to_minecraft(&self, splitted_frames: &mut Vec<SplittedFrame>) -> anyhow::Result<Vec<i8>>{
        Ok(SplittedFrame::split_frames(bytemuck::cast_slice(&self.vec.as_slice()), splitted_frames, self.width as i32)?)
    }
}

#[derive(Debug)]
pub struct BakedImage {
    pub width: u32,
    pub height: u32,
    pub data: &'static [u8]
}

#[macro_export]
macro_rules! bake_image {
    (
        $NAME: ident
    ) => {
        {
            let data: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/", stringify!($NAME), ".bin"));
            let dimension_arr: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/", stringify!($NAME), ".dim"));
            //static is hard man...
            let width_arr = [dimension_arr[0], dimension_arr[1], dimension_arr[2], dimension_arr[3]];
            let width = u32::from_be_bytes(width_arr);

            let height_arr = [dimension_arr[4], dimension_arr[5], dimension_arr[6], dimension_arr[7]];
            let height = u32::from_be_bytes(height_arr);

            BakedImage{
                width,
                height,
                data
            }
        } 
    };
}

pub trait Game {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn fps(&self) -> i32;
    fn new() -> Self where Self: Sized;
    fn draw(&mut self, input_rx: &Receiver<GameInputDirection>) -> anyhow::Result<VideoCanvas>;
}

pub struct GamePlayer {
    pub width: i32,
    pub height: i32,
    fps: i32,
    splitted_frames: Vec<SplittedFrame>,
    game: Box<dyn Game>,
    input_rx: Receiver<GameInputDirection>,
    input_tx: Sender<GameInputDirection>,
    last_frame: Vec<i8>,
    frame_counter: u8,
}

impl VideoPlayer for GamePlayer {
    fn create(
        file_name: String,
        _server_options: ServerOptions,
    ) -> anyhow::Result<Self> {
        let game: Box<dyn Game> = match file_name.as_str() {
            "flappy_bird" => {
                Box::new(FlappyBirdGame::new())     
            }
            "falling_blocks" => {
                Box::new(FallingBlocks::new())     
            }
            _ => return Err(anyhow!("This game is not implemented!"))
        };

        let (width, height, fps) = (game.width(), game.height(), game.fps());
        let (input_tx, input_rx) = channel::<GameInputDirection>();

        Ok(Self {
            width,
            height,
            fps,
            splitted_frames: SplittedFrame::initialize_frames(width as i32, height as i32)?,
            game,
            input_rx,
            input_tx,
            last_frame: Vec::new(),
            frame_counter: 0
        })
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        let frame = if self.frame_counter == 0 {
            let canvas = self.game.draw(&self.input_rx)?;
            let frame = canvas.draw_to_minecraft(&mut self.splitted_frames)?;
            self.last_frame = frame.clone();

            Ok(frame)
        }else {
            let new_frame = self.game.draw(&self.input_rx)?.draw_to_minecraft(&mut self.splitted_frames)?;
            let mut frame_str_info = String::new();
            let mut frame_data = Vec::<i8>::with_capacity(65536);

            let mut offset: usize = 0;
            let mut frame_inxex = 0;
            for frame in &self.splitted_frames {
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
                        data.extend_from_slice(&new_frame[(offset + ((y * frame.width as usize) + x1))..=(offset + ((y * frame.width as usize) + x2))]) 
                    };
                    //Format: {frame_inxex}_{width}_{height}_{x1}_{y1}$
                    frame_str_info.push_str(&format!("{:?}_{:?}_{:?}_{:?}_{:?}$", frame_inxex, width, height, x1, y1));
                    frame_data.extend(data);
                }
                
                offset += frame.frame_length as usize;
                frame_inxex += 1;
            }

            let frame_str_info = match frame_str_info.strip_suffix("$"){
                Some(string) => string,
                None => return Ok(vec![1]), //No change = no new packets
            };

            let frame_str_arr: &[i8] = bytemuck::cast_slice(frame_str_info.as_bytes());

            let mut final_data = Vec::<i8>::with_capacity(frame_str_arr.len() + 5 + frame_data.len());
            final_data.push(0); //Magic value
            final_data.extend_from_slice(bytemuck::cast_slice(&(frame_str_arr.len() as i32).to_be_bytes()));
            final_data.extend_from_slice(frame_str_arr);
            final_data.extend(frame_data);

            self.last_frame = new_frame.clone();

            Ok(final_data)
        };


        if self.frame_counter + 1 != 3 {
            self.frame_counter += 1;
        }else {
            self.frame_counter = 0;
        };

        return frame;
    }

    fn video_data(&self) -> anyhow::Result<super::player_context::VideoData> {
        Ok(VideoData {
            width: self.width,
            height: self.height,
            fps: self.fps,
        })
    }

    fn handle_jvm_msg(
        &self,
        msg: NativeCommunication,
    ) -> anyhow::Result<()> {
        match msg{
            NativeCommunication::GameInput { input } => {
                for ele in &input{
                    self.input_tx.send(*ele)?;
                }
            },
            _ => return Err(anyhow!("Gamep player does not accept jvm messages other than GameInputDirection"))
        }
        Ok(())
    }

    fn destroy(self: Box<Self>) -> anyhow::Result<()> {
        todo!()
    }
}
