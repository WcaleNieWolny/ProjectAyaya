use anyhow::anyhow;

use crate::{map_server::ServerOptions, colorlib::Color, splitting::SplittedFrame};
use super::{player_context::{VideoPlayer, VideoData, NativeCommunication}, flappy_bird::FlappyBirdGame};

pub struct VideoCanvas {
    pub width: usize,
    pub height: usize,
    vec: Vec<u8>
}

#[derive(Debug)]
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
        start_color: Color, 
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
        color: Color
    ){
        self.draw_pixel_exact(x, self.height - y - 1, color);
    }

    fn draw_pixel_exact(
        &mut self,
        x: usize,
        y: usize,
        color: Color
    ){
        self.vec[(y * self.width) + x] = color.to_mc();
    }

    pub fn draw_square(
        &mut self,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        color: Color
    ){
        let x1 = x1.min(x2);
        let x2 = x1.max(x2);

        let temp_y1 = self.height - y1;
        let temp_y2 = self.height - y2;
        let y1 = temp_y1.min(temp_y2);
        let y2 = temp_y1.max(temp_y2);

        let width = x2 - x1;

        let data_to_copy: Vec<u8> = vec![color.to_mc(); width]; 

        for y in y1..y2 {
            self.vec[((y * self.width) + x1)..((y * self.width) + x2)].copy_from_slice(&data_to_copy);
        }
    }

    /// Draws baked image
    ///
    /// X and Y are the top left coordinates of the image
    pub fn draw_image(&mut self, x: usize, y: usize, image: &BakedImage){
        let y2 = self.height as usize - y;
        let y1 = y2 - image.height as usize;

        let mut i : usize = 0;
        for y in y1..y2 {
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
    fn draw(&self, player: &GamePlayer) -> anyhow::Result<VideoCanvas>;
}

pub struct GamePlayer {
    pub width: i32,
    pub height: i32,
    fps: i32,
    splitted_frames: Vec<SplittedFrame>,
    game: Box<dyn Game>
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
            _ => return Err(anyhow!("This game is not implemented!"))
        };

        let (width, height, fps) = (game.width(), game.height(), game.fps());

        Ok(Self {
            width,
            height,
            fps,
            splitted_frames: SplittedFrame::initialize_frames(width as i32, height as i32)?,
            game
        })
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        let canvas = self.game.draw(&self)?;
        canvas.draw_to_minecraft(&mut self.splitted_frames)
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
                    println!("Rust got: {:?}", ele);
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
