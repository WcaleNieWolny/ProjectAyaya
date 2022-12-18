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
        let mut canvas = VideoCanvas::new(player.width as usize, player.height as usize, Color::hex("464B46")?);
        
        canvas.draw_pixel(0, 0, Color::RED);
        canvas.draw_pixel(639, 639, Color::RED);
        canvas.draw_square(50, 0, 100, 100, Color::RED);

        Ok(canvas)
    }
}
