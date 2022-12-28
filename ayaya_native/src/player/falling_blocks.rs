use std::sync::mpsc::Receiver;

use rand::rngs::ThreadRng;

use crate::colorlib::Color;

use super::game_player::{Game, GameInputDirection, VideoCanvas};

static FRAME_COLOR: Color = Color::new(30, 40, 112);
static INNER_FRAME_COLOR: Color = Color::new(50, 64, 190);
static SPAWN_TICKS: usize = 30;
static FALL_TICKS: usize = 60;

pub struct FallingBlocks {
    blocks: Vec<Option<Block>>,
    spawn_ticks: usize,
    fall_ticks: usize,
    move_ticks: usize,
    rand: ThreadRng
}

#[derive(Debug, Clone, PartialEq)]
struct Block {
    color: Color,
    x: usize,
    y: usize,
    active: bool,
}

impl Block {
    fn new(color: Color, x: usize,y: usize) -> Self{
        Self {
            color,
            x,
            y,
            active: true,
        }
    }
}

impl Game for FallingBlocks {
    fn width(&self) -> i32 {
        640
    }

    fn height(&self) -> i32 {
        640
    }

    fn fps(&self) -> i32 {
        60    
    }

    fn new() -> Self where Self: Sized {
        Self {
            blocks: vec![None; 140],
            spawn_ticks: 0usize,
            fall_ticks: 0usize,
            move_ticks: 0usize,
            rand: rand::thread_rng()
        }
    }

    fn draw(&mut self, input_rx: &Receiver<GameInputDirection>) -> anyhow::Result<VideoCanvas> {
        let mut canvas = VideoCanvas::new(self.width() as usize, self.height() as usize, &Color::hex("464B46")?); 

        canvas.draw_square(140, 20, 580, 40, &FRAME_COLOR);
        canvas.draw_square(140, 40, 160, 600, &FRAME_COLOR);
        canvas.draw_square(140, 600, 580, 620, &FRAME_COLOR);
        canvas.draw_square(560, 40, 580, 600, &FRAME_COLOR);
        canvas.draw_square(160, 40, 560, 600, &INNER_FRAME_COLOR);

        if self.spawn_ticks == SPAWN_TICKS {
            if self.blocks.iter().find(|x| x.is_some() && x.as_ref().unwrap().active == true).is_none(){
                let color = Color::random(&mut self.rand);

                //self.blocks[0] = Some(Block::new(color, 0, 0));
                self.blocks[3] = Some(Block::new(color, 3, 0));
                self.blocks[4] = Some(Block::new(color, 4, 0));
                self.blocks[13] = Some(Block::new(color, 3, 1));
                self.blocks[14] = Some(Block::new(color, 4, 1)); 

                self.spawn_ticks = 0;
                self.fall_ticks = 0;
            }else {
                self.spawn_ticks = 0;
            }

        } else {
            self.spawn_ticks += 1;
        }

        while let Ok(val) = input_rx.try_recv(){
            if self.move_ticks != 0 {
                continue;
            };
            match val {
                GameInputDirection::RIGHT => {
                    'block_loop: for (id, block) in self.blocks.clone()
                        .iter_mut()
                        .enumerate()
                        .rev()
                        .filter(|(_id, x)| x.is_some() && x.as_ref().unwrap().active)
                        .map(|(id, x)| (id, x.as_ref().unwrap()))
                    {
                        if block.x + 1 >= 10 {
                            continue 'block_loop;
                        }

                        if let None = self.blocks[id + 1] {
                            self.blocks[id + 1] = Some(Block::new(block.color.clone(), block.x + 1, block.y)); 
                            self.blocks[id] = None;
                        }
                    }
                    self.move_ticks = 10;
                    break;
                },
                GameInputDirection::LEFT => {
                    'block_loop: for (id, block) in self.blocks.clone()
                        .iter_mut()
                        .enumerate()
                        .filter(|(_id, x)| x.is_some() && x.as_ref().unwrap().active)
                        .map(|(id, x)| (id, x.as_ref().unwrap()))
                    {
                        if block.x == 0 {
                            continue 'block_loop;
                        }

                        if let None = self.blocks[id - 1] {
                            self.blocks[id - 1] = Some(Block::new(block.color.clone(), block.x - 1, block.y)); 
                            self.blocks[id] = None;
                        }
                    }
                    self.move_ticks = 10;
                    break;
                }
                _ => continue,
            };
        };

        if self.move_ticks != 0 {
            self.move_ticks -= 1;
        }

        if self.fall_ticks == FALL_TICKS {
            let mut block_clone = self.blocks.clone();
            let mut final_block_clone = self.blocks.clone();
            let mut allow_swap = true;

            'fall_loop: for (id, block) in &mut block_clone
                    .iter_mut()
                    .enumerate()
                    .rev()
                    .filter(|(_id, x)| x.is_some() && x.as_ref().unwrap().active == true)
                    .map(|(id, x)| (id, x.as_mut().unwrap()))
            { 
                if block.y + 1 >= 14 {
                    final_block_clone[id].as_mut().unwrap().active = false;
                    continue;
                };

                if let None = final_block_clone[id + 10] {
                    final_block_clone[id + 10] = Some(Block::new(block.color.clone(), block.x, block.y + 1)); 
                    final_block_clone[id] = None;
                }else {
                    allow_swap = false;
                    self.blocks
                        .iter_mut()
                        .filter(|x| x.is_some() && x.as_ref().unwrap().active == true)
                        .map(|x| x.as_mut().unwrap())
                        .for_each(|x| x.active = false);
                    break 'fall_loop;
                }
            }

            if allow_swap {
                self.blocks = final_block_clone;
            }

            self.fall_ticks = 0;
        } else {
            self.fall_ticks += 1;
        }

        for block in &self.blocks {
            if let Some(block) = block {
                canvas.draw_square(
                    160 + (block.x * 40),
                    40 + (block.y * 40),
                    160 + ((block.x + 1) * 40),
                    40 + ((block.y + 1) * 40), 
                    &block.color 
                );
            }
        }

        Ok(canvas)
    }
}
