use std::{collections::LinkedList, sync::mpsc::Receiver};

use crate::{colorlib::Color, bake_image, player::game_player::{BakedImage, Game, GameInputDirection, VideoCanvas}};
use anyhow::anyhow;
use rand::{rngs::ThreadRng, Rng};

static BOARD_WIDTH: usize = 10;
static BOARD_HEIGHT: usize = 10;

static CANVAS_WIDTH: usize = 640;
static CANVAS_HEIGHT: usize = 640;

static HEAD_OFFSET: usize = 6;

static CELL_SIZE_X: usize = CANVAS_WIDTH / BOARD_WIDTH;
static CELL_SIZE_Y: usize = CANVAS_HEIGHT / BOARD_HEIGHT;
static DRAW_CELL_SIZE_X: usize = CELL_SIZE_X / 2;
static DRAW_CELL_SIZE_Y: usize = CELL_SIZE_Y / 2;
static DRAW_X_OFFSET: usize = DRAW_CELL_SIZE_X / 2;
static DRAW_Y_OFFSET: usize = DRAW_CELL_SIZE_Y / 2;

static SNAKE_COLOR: Color = Color::new(35, 90, 35);
static APPLE_COLOR: Color = Color::new(85, 27, 27);

static DEATH_FRAMES: i8 = 1;

static SNAKE_LOSE_SCREEN: BakedImage = bake_image!(snake_lose);
static SNAKE_WIN_SCREEN: BakedImage = bake_image!(snake_win);

enum SnakeDirection {
    Up,
    Down,
    Right,
    Left,
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

#[derive(Clone, Debug)]
struct SnakeCell {
    x: usize,
    y: usize,
}

impl SnakeCell {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

enum SnakeGameState {
    Win,
    Lose,
    Playing,
}

pub struct SnakeGame {
    snake: LinkedList<SnakeCell>,
    direction: SnakeDirection,
    apple_x: usize,
    apple_y: usize,
    rand: ThreadRng,
    death_timer: i8,
    game_state: SnakeGameState,
}

impl Game for SnakeGame {
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
        Self: Sized,
    {
        let mut rand = rand::thread_rng();
        let apple_x = rand.gen_range(0..BOARD_WIDTH);
        let apple_y = rand.gen_range(0..BOARD_HEIGHT);

        //For now static start possition
        Self {
            snake: LinkedList::from([SnakeCell::new(0, 0)]),
            direction: SnakeDirection::Right,
            apple_x,
            apple_y,
            rand,
            death_timer: DEATH_FRAMES,
            game_state: SnakeGameState::Playing,
        }
    }

    fn draw(&mut self, input_rx: &Receiver<GameInputDirection>) -> anyhow::Result<VideoCanvas> {
        if matches!(self.game_state, SnakeGameState::Lose) {
            return self.draw_lose_screen();
        }

        if matches!(self.game_state, SnakeGameState::Win) {
            return self.draw_win_screen();
        }

        while let Ok(val) = input_rx.try_recv() {
            match val {
                GameInputDirection::Forward => {
                    if !matches!(self.direction, SnakeDirection::Down) {
                        self.direction = SnakeDirection::Up
                    }
                }
                GameInputDirection::Backwards => {
                    if !matches!(self.direction, SnakeDirection::Up) {
                        self.direction = SnakeDirection::Down
                    }
                }
                GameInputDirection::Left => {
                    if !matches!(self.direction, SnakeDirection::Right) {
                        self.direction = SnakeDirection::Left
                    }
                }
                GameInputDirection::Right => {
                    if !matches!(self.direction, SnakeDirection::Left) {
                        self.direction = SnakeDirection::Right
                    }
                }
                GameInputDirection::Up => {}
            }
        }

        let head_cell = match self.snake.front() {
            Some(val) => val,
            None => return Err(anyhow!("Snake is empty")),
        };

        let head_x = head_cell.x;
        let head_y = head_cell.y;

        //edge cases
        if let SnakeDirection::Left = self.direction {
            if head_x == 0 {
                return self.draw_lose_screen();
            }
        }
        if let SnakeDirection::Right = self.direction {
            if head_x == BOARD_WIDTH - 1 {
                return self.draw_lose_screen();
            }
        }
        if let SnakeDirection::Up = self.direction {
            if head_y == 0 {
                return self.draw_lose_screen();
            }
        }
        if let SnakeDirection::Down = self.direction {
            if head_y == BOARD_HEIGHT - 1 {
                return self.draw_lose_screen();
            }
        }

        let new_head_x = (head_x as i32 + self.direction.to_x_diff() as i32) as usize;
        let new_head_y = (head_y as i32 + self.direction.to_y_diff() as i32) as usize;

        let apple_eaten = new_head_x == self.apple_x && new_head_y == self.apple_y;

        //We might lose at some point and then we need the original snake
        let mut snake_clone = self.snake.clone();
        snake_clone.push_front(SnakeCell::new(new_head_x, new_head_y));

        if !apple_eaten {
            snake_clone.pop_back();
        } else {
            let mut recurstion = 0;
            'apple_loop: loop {
                if recurstion == CANVAS_WIDTH * CANVAS_HEIGHT {
                    return self.draw_win_screen();
                }

                self.apple_x = self.rand.gen_range(0..BOARD_WIDTH);
                self.apple_y = self.rand.gen_range(0..BOARD_HEIGHT);

                if snake_clone
                    .iter()
                    .find(|cell| cell.x == self.apple_x && cell.y == self.apple_y)
                    .is_none()
                {
                    break 'apple_loop;
                }
                recurstion += 1;
            }
        }

        //Late init of canvas to save memory if we quit early
        let mut canvas = VideoCanvas::new(
            self.width() as usize,
            self.height() as usize,
            &Color::hex("464B46")?,
        );

        let mut cursor = snake_clone.cursor_front();
        let mut check_for_head_collision = false;

        'cursor_loop: loop {
            let cell = match cursor.current() {
                Some(val) => val,
                None => break 'cursor_loop,
            };

            if check_for_head_collision && cell.x == new_head_x && cell.y == new_head_y {
                return self.draw_lose_screen();
            }

            if let Some(next_cell) = cursor.peek_next() {
                let first_cell = if cell.x != next_cell.x {
                    if next_cell.x > cell.x {
                        cell
                    } else {
                        next_cell
                    }
                } else {
                    if next_cell.y > cell.y {
                        cell
                    } else {
                        next_cell
                    }
                };

                let second_cell = if cell.x != next_cell.x {
                    if next_cell.x > cell.x {
                        next_cell
                    } else {
                        cell
                    }
                } else {
                    if next_cell.y > cell.y {
                        next_cell
                    } else {
                        cell
                    }
                };

                canvas.draw_square(
                    first_cell.x * CELL_SIZE_X + DRAW_X_OFFSET,
                    first_cell.y * CELL_SIZE_Y + DRAW_Y_OFFSET,
                    second_cell.x * CELL_SIZE_X + DRAW_X_OFFSET + DRAW_CELL_SIZE_X - 1,
                    second_cell.y * CELL_SIZE_Y + DRAW_Y_OFFSET + DRAW_CELL_SIZE_Y - 1,
                    &SNAKE_COLOR,
                );
            } else {
                canvas.draw_square(
                    cell.x * CELL_SIZE_X + DRAW_X_OFFSET,
                    cell.y * CELL_SIZE_Y + DRAW_Y_OFFSET,
                    cell.x * CELL_SIZE_X + DRAW_X_OFFSET + DRAW_CELL_SIZE_X - 1,
                    cell.y * CELL_SIZE_Y + DRAW_Y_OFFSET + DRAW_CELL_SIZE_Y - 1,
                    &SNAKE_COLOR,
                );
            }

            check_for_head_collision = true;
            cursor.move_next();
        }

        //We start drawing at 0. (9 + 1) * 64 = 10 * 64 * 640
        //640 is out of bounds
        canvas.draw_square(
            new_head_x * CELL_SIZE_X + HEAD_OFFSET,
            new_head_y * CELL_SIZE_Y + HEAD_OFFSET,
            (new_head_x + 1) * CELL_SIZE_X - 1 - HEAD_OFFSET,
            (new_head_y + 1) * CELL_SIZE_Y - 1 - HEAD_OFFSET,
            &SNAKE_COLOR,
        );

        //Draw apple
        canvas.draw_square(
            self.apple_x * CELL_SIZE_X,
            self.apple_y * CELL_SIZE_Y,
            (self.apple_x + 1) * CELL_SIZE_X - 1,
            (self.apple_y + 1) * CELL_SIZE_Y - 1,
            &APPLE_COLOR,
        );

        self.death_timer = DEATH_FRAMES;
        self.snake = snake_clone;

        Ok(canvas)
    }
}

impl SnakeGame {
    fn draw_lose_screen(&mut self) -> anyhow::Result<VideoCanvas> {
        if self.death_timer == 0 {
            self.game_state = SnakeGameState::Lose;
            return Ok(VideoCanvas::new_from_image(&SNAKE_LOSE_SCREEN));
        } else {
            let mut canvas = VideoCanvas::new(
                self.width() as usize,
                self.height() as usize,
                &Color::hex("464B46")?,
            );

            let mut cursor = self.snake.cursor_front();

            'cursor_loop: loop {
                let cell = match cursor.current() {
                    Some(val) => val,
                    None => break 'cursor_loop,
                };

                if let Some(next_cell) = cursor.peek_next() {
                    let first_cell = if cell.x != next_cell.x {
                        if next_cell.x > cell.x {
                            cell
                        } else {
                            next_cell
                        }
                    } else {
                        if next_cell.y > cell.y {
                            cell
                        } else {
                            next_cell
                        }
                    };

                    let second_cell = if cell.x != next_cell.x {
                        if next_cell.x > cell.x {
                            next_cell
                        } else {
                            cell
                        }
                    } else {
                        if next_cell.y > cell.y {
                            next_cell
                        } else {
                            cell
                        }
                    };

                    canvas.draw_square(
                        first_cell.x * CELL_SIZE_X + DRAW_X_OFFSET,
                        first_cell.y * CELL_SIZE_Y + DRAW_Y_OFFSET,
                        second_cell.x * CELL_SIZE_X + DRAW_X_OFFSET + DRAW_CELL_SIZE_X - 1,
                        second_cell.y * CELL_SIZE_Y + DRAW_Y_OFFSET + DRAW_CELL_SIZE_Y - 1,
                        &SNAKE_COLOR,
                    );
                } else {
                    canvas.draw_square(
                        cell.x * CELL_SIZE_X + DRAW_X_OFFSET,
                        cell.y * CELL_SIZE_Y + DRAW_Y_OFFSET,
                        cell.x * CELL_SIZE_X + DRAW_X_OFFSET + DRAW_CELL_SIZE_X - 1,
                        cell.y * CELL_SIZE_Y + DRAW_Y_OFFSET + DRAW_CELL_SIZE_Y - 1,
                        &SNAKE_COLOR,
                    );
                }

                cursor.move_next();
            }

            let head_cell = match self.snake.front() {
                Some(val) => val,
                None => return Err(anyhow!("Empty snake")),
            };

            let head_x = head_cell.x;
            let head_y = head_cell.y;

            //We start drawing at 0. (9 + 1) * 64 = 10 * 64 * 640
            //640 is out of bounds
            canvas.draw_square(
                head_x * CELL_SIZE_X + HEAD_OFFSET,
                head_y * CELL_SIZE_Y + HEAD_OFFSET,
                (head_x + 1) * CELL_SIZE_X - 1 - HEAD_OFFSET,
                (head_y + 1) * CELL_SIZE_Y - 1 - HEAD_OFFSET,
                &SNAKE_COLOR,
            );

            //Draw apple
            canvas.draw_square(
                self.apple_x * CELL_SIZE_X,
                self.apple_y * CELL_SIZE_Y,
                (self.apple_x + 1) * CELL_SIZE_X - 1,
                (self.apple_y + 1) * CELL_SIZE_Y - 1,
                &APPLE_COLOR,
            );

            self.death_timer -= 1;
            return Ok(canvas);
        }
    }

    fn draw_win_screen(&mut self) -> anyhow::Result<VideoCanvas> {
        self.game_state = SnakeGameState::Win;
        return Ok(VideoCanvas::new_from_image(&SNAKE_WIN_SCREEN));
    }
}
