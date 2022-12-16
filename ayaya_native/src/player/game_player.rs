use anyhow::anyhow;

use crate::{map_server::ServerOptions, colorlib::{Color, transform_frame_to_mc}, splitting::SplittedFrame};
use super::{player_context::{VideoPlayer, PlayerContext, VideoData}, flappy_bird::FlappyBirdGame};

pub struct VideoCanvas {
    pub width: i32,
    pub height: i32,
    vec: Vec<u8>
}

impl VideoCanvas {
    pub fn new(
        width: i32,
        height: i32,
        start_color: Color, 
    ) -> Self {
        let vec: Vec<u8> = std::iter::repeat([start_color.red, start_color.green, start_color.blue])
            .take((width * height) as usize)
            .flatten()
            .collect();

        Self {
            width,
            height,
            vec
        }
    }

    pub fn draw_to_minecraft(&self, splitted_frames: &mut Vec<SplittedFrame>) -> anyhow::Result<Vec<i8>>{
        let frame = transform_frame_to_mc(&self.vec, self.width as u32, self.height as u32);
        Ok(SplittedFrame::split_frames(&frame, splitted_frames, self.width)?)
    }
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
    ) -> anyhow::Result<PlayerContext> {
        let game: Box<dyn Game> = match file_name.as_str() {
            "bird" => {
                Box::new(FlappyBirdGame::new())     
            }
            _ => return Err(anyhow!("This game is not implemented!"))
        };

        let (width, height, fps) = (game.width(), game.height(), game.fps());

        Ok(PlayerContext::from_player(Self {
            width,
            height,
            fps,
            splitted_frames: SplittedFrame::initialize_frames(width as i32, height as i32)?,
            game
        }))
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
        msg: super::player_context::NativeCommunication,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn destroy(self: Box<Self>) -> anyhow::Result<()> {
        todo!()
    }
}
