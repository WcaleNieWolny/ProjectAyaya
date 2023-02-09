use std::sync::mpsc::Receiver;

use crate::{player::game_player::{Game, GameInputDirection, VideoCanvas}, colorlib::Color};

pub struct Calculator {}

impl Game for Calculator {
    fn width(&self) -> i32 {
        640
    }

    fn height(&self) -> i32 {
        640
    }

    fn fps(&self) -> i32 {
        10
    }

    fn new() -> Self
    where
        Self: Sized {
        Self {}
    }

    fn draw(&mut self, _input_rx: &Receiver<GameInputDirection>) -> anyhow::Result<VideoCanvas> {
        let mut canvas = VideoCanvas::new(
            self.width() as usize,
            self.height() as usize,
            &Color::hex("464B46")?,
        );

        canvas.draw_default_text(0, 0, 128, 64, &Color::BLACK, "Hi")?;

        Ok(canvas)
    }
}
