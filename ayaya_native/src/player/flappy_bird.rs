use crate::colorlib::Color;

use super::game_player::{Game, VideoCanvas, GamePlayer};

pub struct FlappyBirdGame {
    
}

impl Game for FlappyBirdGame {
    //5 x 5 @ 15 FPS
    //This is to save bandwith both server and client side
    //Also make this efficient - it is not going to be multithreaded!
    fn width(&self) -> i32 {
        640
    }

    fn height(&self) -> i32 {
        640
    }

    fn fps(&self) -> i32 {
        15
    }

    fn new() -> Self {
        Self {}
    }

    fn draw(&self, player: &GamePlayer) -> anyhow::Result<super::game_player::VideoCanvas> {
        let c = Color::hex("464B46")?;
        println!("C: {:?}", c);
        let canvas = VideoCanvas::new(player.width, player.height, c);

        Ok(canvas)
    }
}
