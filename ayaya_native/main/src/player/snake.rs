use std::sync::mpsc::Receiver;

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

#[derive(Clone)]
enum SnakeCell {
    None,
    Up,
    Down,
    Right,
    Left
}

impl SnakeCell{
    fn to_x_diff(&self) -> i32 {
        match self {
            Self::Up | Self::Down => 0,
            Self::Right => 1,
            Self::Left => -1,
            _ => unreachable!()
        }
    }
    fn to_y_diff(&self) -> i32 {
        match self {
            Self::Left | Self::Right => 0,
            Self::Up => -1,
            Self::Down => 1,
            _ => unreachable!()
        }
    }
    fn reverse(&self) -> Self {
        match self {
            SnakeCell::None => Self::None,
            SnakeCell::Up => Self::Down,
            SnakeCell::Down => Self::Up,
            SnakeCell::Right => Self::Left,
            SnakeCell::Left => Self::Right,
        }
    }
}

pub struct SnakeGame {
    board: Vec<SnakeCell>,
    head_x: usize,
    head_y: usize,
    direction: SnakeCell, //None is unreachable
}

impl Game for  SnakeGame {
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
       
        let mut board = vec![SnakeCell::None; BOARD_WIDTH * BOARD_HEIGHT];
        board[21] = SnakeCell::Up; //Head
        board[11] = SnakeCell::Up;
        board[2] = SnakeCell::Left;
        board[1] = SnakeCell::Left;
        
        //For now static start possition
        Self {
            board,
            head_x: 1,
            head_y: 2,
            direction: SnakeCell::Right,
        }
    }

    fn draw(&mut self, _input_rx: &Receiver<GameInputDirection>) -> anyhow::Result<VideoCanvas> {
        let mut canvas = VideoCanvas::new(
            self.width() as usize,
            self.height() as usize,
            &Color::hex("464B46")?,
        );

        let mut cell = match self.board.get_mut(BOARD_WIDTH * self.head_y + self.head_x) {
            Some(val) => val.clone(),
            None => return Err(anyhow!("Unable to get none cell in snake"))
        };

        let mut cell_x = self.head_x;
        let mut cell_y = self.head_y;

        if let SnakeCell::None = cell {
            return Err(anyhow!("Snake cell is none"));
        }

        //edge cases
        if let SnakeCell::Left = self.direction{
            if self.head_x == 0 {
                println!("LEFT ED");
                return self.draw_lose_screen();
            }
        }
        if let SnakeCell::Right = self.direction{
            if self.head_x == BOARD_WIDTH - 1 {
                println!("RIGHT ED");
                return self.draw_lose_screen();
            }
        }
        if let SnakeCell::Up = self.direction{
            if self.head_y == 0 {
                println!("UP ED");
                return self.draw_lose_screen();
            }
        }
        if let SnakeCell::Down = self.direction{
            if self.head_y == BOARD_HEIGHT - 1 {
                println!("DOWN ED");
                return self.draw_lose_screen();
            }
        }

        loop {
            //This is safe, we handled edge cases
            let pointing_y = (cell_y as i32 + cell.to_y_diff()) as usize;
            let pointing_x = (cell_x as i32 + cell.to_x_diff()) as usize;
            let pointing_cell = &self.board[pointing_y * BOARD_WIDTH + pointing_x];

            if let SnakeCell::None = pointing_cell {
                self.board[BOARD_WIDTH * cell_y + cell_x] = SnakeCell::None;
                break;
            }else {
                canvas.draw_square(cell_x * CELL_SIZE_X, cell_y * CELL_SIZE_Y, (cell_x + 1) * CELL_SIZE_X, (cell_y + 1) * CELL_SIZE_Y, &SNAKE_COLOR); 

                cell_x = pointing_x;
                cell_y = pointing_y;
                cell = pointing_cell.clone();
                continue;
            }
        }

        let new_head_x = (self.head_x as i32 + self.direction.to_x_diff()) as usize;
        let new_head_y = (self.head_y as i32 + self.direction.to_y_diff()) as usize;
        let new_head_cell = self.board.get_mut(new_head_y * BOARD_WIDTH + new_head_x);

        if let Some(new_head_cell) = new_head_cell {
            if let SnakeCell::None = new_head_cell {
                *new_head_cell = self.direction.reverse();
                self.head_y = new_head_y;
                self.head_x = new_head_x;
                canvas.draw_square(new_head_x * CELL_SIZE_X, new_head_y * CELL_SIZE_Y, (new_head_x + 1) * CELL_SIZE_X, (new_head_y + 1) * CELL_SIZE_Y, &SNAKE_COLOR); 
            }else {
                println!("LOOSE SCR");
                return self.draw_lose_screen();
            }
        }else {
            return Err(anyhow!("Cannot get none value when getting new head cell"));
        }

        Ok(canvas)
    }

}

impl SnakeGame {
    fn draw_lose_screen(&mut self) -> anyhow::Result<VideoCanvas> {
        Ok(VideoCanvas::new(
            self.width() as usize,
            self.height() as usize,
            &Color::new(0, 0, 0),
        ))
    }
}


