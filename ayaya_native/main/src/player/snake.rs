use std::{sync::mpsc::Receiver, collections::LinkedList};

use anyhow::anyhow;
use crate::colorlib::Color;

use super::game_player::{Game, GameInputDirection, VideoCanvas};

static BOARD_WIDTH: usize = 10;
static BOARD_HEIGHT: usize = 10;

static CANVAS_WIDTH: usize = 640;
static CANVAS_HEIGHT: usize = 640;

static CELL_SIZE_X: usize = CANVAS_WIDTH / BOARD_WIDTH;
static CELL_SIZE_Y: usize = CANVAS_HEIGHT / BOARD_HEIGHT;

static SNAKE_COLOR: Color = Color::new(50, 95, 95);

static DEATH_FRAMES: i8 = 1;

enum SnakeDirection {
    Up,
    Down,
    Right,
    Left
}

impl SnakeDirection {
    fn to_x_diff(&self) -> i32 {
        match self {
            Self::Up | Self::Down => 0,
            Self::Right => 1,
            Self::Left => -1,
        }
    }
    fn to_y_diff(&self) -> i32 {
        match self {
            Self::Left | Self::Right => 0,
            Self::Up => -1,
            Self::Down => 1,
        }
    }
}

#[derive(Clone)]
struct SnakeCell {
    x: usize,
    y: usize
}

impl SnakeCell {
    fn new(x: usize, y: usize) -> Self{
        Self { x, y }
    }
}

pub struct SnakeGame {
    snake: LinkedList<SnakeCell>,
    direction: SnakeDirection,
    death_timer: i8,
    is_game_over: bool
}

impl Game for  SnakeGame {
    fn width(&self) -> i32 {
        640
    }

    fn height(&self) -> i32 {
        640
    }

    fn fps(&self) -> i32 {
        6
    }

    fn new() -> Self
    where
        Self: Sized {
       
        
        //For now static start possition
        Self {
            snake: LinkedList::from([SnakeCell::new(0, 0)]),
            direction: SnakeDirection::Right,
            death_timer: DEATH_FRAMES, 
            is_game_over: false
        }
    }

    fn draw(&mut self, input_rx: &Receiver<GameInputDirection>) -> anyhow::Result<VideoCanvas> {
        if self.is_game_over {
            return self.draw_lose_screen();
        }

        while let Ok(val) = input_rx.try_recv() {
            match val {
                GameInputDirection::Forward => self.direction = SnakeDirection::Up,
                GameInputDirection::Backwards => self.direction = SnakeDirection::Down,
                GameInputDirection::Left =>  self.direction = SnakeDirection::Left,
                GameInputDirection::Right =>  self.direction = SnakeDirection::Right,
                GameInputDirection::Up => {},
            }
        }

        let head_cell = match self.snake.front() {
            Some(val) => val,
            None => return Err(anyhow!("Snake is empty"))
        };

        let head_x = head_cell.x;
        let head_y = head_cell.y;

        //edge cases
        if let SnakeDirection::Left = self.direction{
            if head_x == 0 {
                println!("LEFT ED");
                return self.draw_lose_screen();
            }
        }
        if let SnakeDirection::Right = self.direction{
            if head_x == BOARD_WIDTH - 1 {
                println!("RIGHT ED");
                return self.draw_lose_screen();
            }
        }
        if let SnakeDirection::Up = self.direction{
            if head_y == 0 {
                println!("UP ED");
                return self.draw_lose_screen();
            }
        }
        if let SnakeDirection::Down = self.direction{
            if head_y == BOARD_HEIGHT - 1 {
                println!("DOWN ED");
                return self.draw_lose_screen();
            }
        }

        let new_head_x = (head_x as i32 + self.direction.to_x_diff() as i32) as usize;
        let new_head_y = (head_y as i32 + self.direction.to_y_diff() as i32) as usize;

        println!("NEW HEAD: {}, {}", new_head_x, new_head_y);

        //We might lose at some point and then we need the original snake
        let mut snake_clone = self.snake.clone();
        snake_clone.push_front(SnakeCell::new(new_head_x, new_head_y));
        snake_clone.pop_back();

        //Late init of canvas to save memory if we quit early
        let mut canvas = VideoCanvas::new(
            self.width() as usize,
            self.height() as usize,
            &Color::hex("464B46")?,
        );

        for cell in snake_clone.iter().skip(1) {
            if cell.x == new_head_x && cell.y == new_head_y {
                println!("Colis");
                return self.draw_lose_screen();
            }

            canvas.draw_square(cell.x * CELL_SIZE_X, cell.y * CELL_SIZE_Y, (cell.x + 1) * CELL_SIZE_X - 1, (cell.y + 1) * CELL_SIZE_Y - 1, &SNAKE_COLOR);
        }

        //We start drawing at 0. (9 + 1) * 64 = 10 * 64 * 640
        //640 is out of bounds
        canvas.draw_square(new_head_x * CELL_SIZE_X, new_head_y * CELL_SIZE_Y, (new_head_x + 1) * CELL_SIZE_X - 1, (new_head_y + 1) * CELL_SIZE_Y - 1, &SNAKE_COLOR);
       
        self.death_timer = DEATH_FRAMES;
        self.snake = snake_clone;

        Ok(canvas)
    }

}

impl SnakeGame {
    fn draw_lose_screen(&mut self) -> anyhow::Result<VideoCanvas> {
        if self.death_timer == 0 {
            self.is_game_over = true;
            return Ok(VideoCanvas::new(
                self.width() as usize,
                self.height() as usize,
                &Color::new(0, 0, 0),
            ));
        }else {
            let mut canvas = VideoCanvas::new(
                self.width() as usize,
                self.height() as usize,
                &Color::hex("464B46")?,
            );

            for cell in &self.snake {
                canvas.draw_square(cell.x * CELL_SIZE_X, cell.y * CELL_SIZE_Y, (cell.x + 1) * CELL_SIZE_X - 1, (cell.y + 1) * CELL_SIZE_Y - 1, &SNAKE_COLOR);
            }

            self.death_timer -= 1;
            return Ok(canvas);
        }
    }
}


