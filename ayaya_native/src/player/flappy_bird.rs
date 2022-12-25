use std::sync::mpsc::Receiver;

use rand::{rngs::ThreadRng, Rng};

use crate::{colorlib::Color, bake_image};

use super::game_player::{Game, VideoCanvas, BakedImage, GameInputDirection};

struct Pipe {
    x: isize,
    y_space: usize,
    to_remove: bool
}

pub struct FlappyBirdGame {
    pipes: Vec<Pipe>,
    tick: i32,
    bird_y: usize,
    jump_tick: i8,
    thread_rng: ThreadRng
}

static TEST_IMAGE: BakedImage = bake_image!(test);
static PIPE_COLOR: Color = Color::new(117, 192, 47);
static HOLE_HEIGHT: usize = 192;
static PIPE_WIDTH: usize = 96;
static JUMP_HEIGHT: usize = 96;
static JUMP_TICKS: i8 = 4;

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
        30
    }

    fn new() -> Self {
        Self {
            pipes: vec![Pipe {
                x: 639,
                to_remove: false,
                y_space: 50
            }],
            tick: 0,
            bird_y: 287,
            jump_tick: -1,
            thread_rng: rand::thread_rng()
        }
    }

    fn draw(&mut self, input_rx: &Receiver<GameInputDirection>) -> anyhow::Result<super::game_player::VideoCanvas> {
        let mut canvas = VideoCanvas::new(self.width() as usize, self.height() as usize, &Color::hex("464B46")?);

        while let Ok(val) = input_rx.try_recv(){
            match val {
                GameInputDirection::UP => {
                    println!("Jump!");
                    self.jump_tick = 0;
                }
                _ => continue,
            };
        };

        for pipe in self.pipes.iter_mut() {
            if pipe.x > 542 {
                let width = 639 - pipe.x as usize;
                canvas.draw_square(639 - width, 0, 639, pipe.y_space, &PIPE_COLOR);
                canvas.draw_square(639 - width, pipe.y_space + HOLE_HEIGHT, 639, 639, &PIPE_COLOR);
                pipe.x -= 4;
            }else if pipe.x > 0{
                canvas.draw_square(pipe.x as usize, 0, pipe.x as usize + PIPE_WIDTH, pipe.y_space, &PIPE_COLOR);
                canvas.draw_square(pipe.x as usize, pipe.y_space + HOLE_HEIGHT, pipe.x as usize + PIPE_WIDTH, 639, &PIPE_COLOR);
                pipe.x -= 4;
            }else if pipe.x > -95 {
                let width = PIPE_WIDTH + pipe.x as usize;
                canvas.draw_square(0, 0, width,  pipe.y_space, &PIPE_COLOR);
                canvas.draw_square(0, pipe.y_space + HOLE_HEIGHT, width, 639, &PIPE_COLOR);
                pipe.x -= 4;
            }else {
                pipe.to_remove =  true;
                continue;
            }
        }

        self.pipes.retain(|x| !x.to_remove);

        if self.tick == 70{
            self.pipes.push(Pipe {
                x: 639,
                to_remove: false,
                y_space: self.thread_rng.gen_range(70..(self.width() as usize - 70 - HOLE_HEIGHT))
            })
        };

        if self.jump_tick >= 0 && self.jump_tick < JUMP_TICKS {
            if self.bird_y >= 24 {
                self.bird_y -= JUMP_HEIGHT / JUMP_TICKS as usize;
            }else {
                self.bird_y = 0;
            }
            self.jump_tick += 1;
        }else if self.jump_tick == JUMP_TICKS{
            self.jump_tick = -1;
        }

        canvas.draw_square(287, self.bird_y, 351, self.bird_y + 64, &Color::RED);

        if self.bird_y + 8 <= 575 && self.jump_tick == -1 {
            self.bird_y += 8;
        }else if self.jump_tick == -1{
            self.bird_y = 575;
        }

        if self.tick < 70 {
            self.tick += 1;
        }else {
            self.tick = 0;
        } 
        Ok(canvas)
    }
}
