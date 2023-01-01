use std::sync::mpsc::Receiver;

use anyhow::{anyhow, bail};
use rand::{rngs::ThreadRng, Rng};

use crate::colorlib::Color;

use super::game_player::{Game, GameInputDirection, VideoCanvas};

static FRAME_COLOR: Color = Color::new(30, 40, 112);
static INNER_FRAME_COLOR: Color = Color::new(50, 64, 190);
static SPAWN_TICKS: usize = 30;
static FALL_TICKS: usize = 60;
static FAST_FALL_TICKS: usize = 4;

pub struct FallingBlocks {
    blocks: Vec<Option<Block>>,
    spawn_ticks: usize,
    fall_ticks: usize,
    move_ticks: usize,
    fast_fall_ticks: usize,
    current_block: BlockType,
    rotation_state: usize,
    rand: ThreadRng
}

#[derive(Debug, Clone, PartialEq)]
struct Block {
    color: Color,
    x: usize,
    y: usize,
    active: bool,
    falling: bool,
    center: bool
}

#[derive(Debug, PartialEq)]
enum BlockType {
    O,
    T,
    I,
    S,
    Z,
    J,
    L
}

impl BlockType {
    fn from_i8(int: i8) -> anyhow::Result<BlockType>{
        return match int {
            0 => Ok(BlockType::O),
            1 => Ok(BlockType::T),
            2 => Ok(BlockType::I),
            3 => Ok(BlockType::S),
            4 => Ok(BlockType::Z),
            5 => Ok(BlockType::J),
            6 => Ok(BlockType::L),
            _ => Err(anyhow!("Invalid intiger"))
        }
    }
}

impl Block {
    fn new(color: Color, x: usize,y: usize) -> Self{
        Self {
            color,
            x,
            y,
            active: true,
            falling: false,
            center: false
        }
    }

    fn new_center(color: Color, x: usize,y: usize) -> Self{
        println!("CE");
        Self {
            color,
            x,
            y,
            active: true,
            falling: false,
            center: true
        }
    }


    fn partial_clone(&self, x: usize, y: usize) -> Self {
        Self {
            color: self.color,
            x,
            y,
            active: self.active,
            falling: self.falling,
            center: false
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
            fast_fall_ticks: 0usize,
            current_block: BlockType::I, //We do not want to have a I block as the first one
            rotation_state: 0,
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
                let mut rand_int = self.rand.gen_range(0..=6);

                //Make sure we do not get 2 the same block in a row
                //I am aware that this is not as the original tetris however this game is NOT
                //supose to be the original tetris!
                //TODO: FIX != to ==
                while BlockType::from_i8(rand_int)? != self.current_block {
                    rand_int = self.rand.gen_range(0..=6);
                }

                self.current_block = BlockType::from_i8(rand_int)?;
                self.rotation_state = 0; //When spawing a new piece state is always zero!
                match rand_int {
                    0 => {
                        //Block O  OK
                        self.blocks[3] = Some(Block::new(color, 3, 0));
                        self.blocks[4] = Some(Block::new(color, 4, 0));
                        self.blocks[13] = Some(Block::new(color, 3, 1));
                        self.blocks[14] = Some(Block::new(color, 4, 1));
                        println!("B O");
                    }
                    1 => {
                        //Block T  OK
                        self.blocks[3] = Some(Block::new(color, 3, 0));
                        self.blocks[4] = Some(Block::new(color, 4, 0));
                        self.blocks[5] = Some(Block::new(color, 5, 0));
                        self.blocks[14] = Some(Block::new(color, 4, 1));
                        println!("B T");

                    }
                    2 => {
                        //Block I
                        self.blocks[3] = Some(Block::new(color, 3, 0));
                        self.blocks[4] = Some(Block::new(color, 4, 0));
                        self.blocks[5] = Some(Block::new_center(color, 5, 0));
                        self.blocks[6] = Some(Block::new(color, 6, 0));
                        println!("B I");
                    }
                    3 => {
                        //block S
                        self.blocks[4] = Some(Block::new(color, 4, 0));
                        self.blocks[5] = Some(Block::new(color, 5, 0));
                        self.blocks[14] = Some(Block::new(color, 4, 1));
                        self.blocks[13] = Some(Block::new(color, 3, 1));
                        println!("B S");
                    }
                    4 => {
                        //Block Z OK
                        self.blocks[3] = Some(Block::new(color, 3, 0));
                        self.blocks[4] = Some(Block::new(color, 4, 0));
                        self.blocks[14] = Some(Block::new(color, 4, 1));
                        self.blocks[15] = Some(Block::new(color, 5, 1));
                        println!("B Z");
                    }
                    5 => {
                        //Block J
                        self.blocks[3] = Some(Block::new(color, 3, 0));
                        self.blocks[13] = Some(Block::new(color, 3, 1));
                        self.blocks[14] = Some(Block::new(color, 4, 1));
                        self.blocks[15] = Some(Block::new(color, 5, 1));
                    }
                    6 => {
                        //Block L
                        self.blocks[5] = Some(Block::new(color, 5, 0));
                        self.blocks[13] = Some(Block::new(color, 3, 1));
                        self.blocks[14] = Some(Block::new(color, 4, 1));
                        self.blocks[15] = Some(Block::new(color, 5, 1));
                    }
                    _ => {
                        return Err(anyhow!("Unexpected random int when spawing a new block"));
                    }
                }

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
                    let mut blocks_clone = self.blocks.clone();
                    let mut allow_swap = false;
                    'block_loop: for block in self.blocks.clone()
                        .iter_mut()
                        .rev()
                        .filter(|x| x.is_some() && x.as_ref().unwrap().active)
                        .map(|x|  x.as_ref().unwrap())
                    {
                        if block.x + 1 >= 10 {
                            continue 'block_loop;
                        }

                        if let None = blocks_clone[block.y * 10 + (block.x + 1)] {
                            allow_swap = true;
                            blocks_clone[block.y * 10 + (block.x + 1)] = Some(block.partial_clone(block.x + 1, block.y)); 
                            blocks_clone[block.y * 10 + block.x] = None;
                        }else {
                            allow_swap = false;
                            break 'block_loop;
                        }
                    }

                    if allow_swap {
                        self.blocks = blocks_clone;
                    }
                    self.move_ticks = 10;
                    break;
                },
                GameInputDirection::LEFT => {
                    let mut blocks_clone = self.blocks.clone();
                    let mut allow_swap = false;
                    'block_loop: for block in self.blocks.clone()
                        .iter_mut()
                        .filter(|x| x.is_some() && x.as_ref().unwrap().active)
                        .map(|x| x.as_ref().unwrap())
                    {
                        if block.x == 0 {
                            continue 'block_loop;
                        }

                        if let None = blocks_clone[block.y * 10 + (block.x - 1)] {
                            allow_swap = true;
                            blocks_clone[block.y * 10 + (block.x - 1)] = Some(block.partial_clone(block.x - 1, block.y)); 
                            blocks_clone[block.y * 10 + block.x] = None;
                        }else {
                            allow_swap = false;
                            break 'block_loop;
                        }

                    }

                    if allow_swap {
                        self.blocks = blocks_clone;
                    }

                    self.move_ticks = 10;
                    break;
                },
                GameInputDirection::UP => {

                    self.blocks.iter_mut()
                        .rev()
                        .filter(|x| x.is_some() && x.as_ref().unwrap().active == true)
                        .map(|x| x.as_mut().unwrap())
                        .for_each(|block| {
                            block.falling = true; 
                        });

                }
                GameInputDirection::FORWARD => {
                    //Right rotate
                    let prev_state = self.rotation_state;
                    if self.rotation_state + 1 != 4 {
                        self.rotation_state += 1;
                    }else {
                        self.rotation_state = 0;
                        break;
                    }

                    if !self.rotate_block_right() {
                        self.rotation_state = prev_state;
                    }
                }
                _ => continue,
            };
        };

        if self.move_ticks != 0 {
            self.move_ticks -= 1;
        }
   
        if self.fast_fall_ticks == FAST_FALL_TICKS {
            let mut block_clone = self.blocks.clone();
            let mut final_block_clone = self.blocks.clone();
            let mut allow_swap = false;

            'fast_fall_loop: for block in &mut block_clone
                    .iter_mut()
                    .rev()
                    .filter(|x| x.is_some() && x.as_ref().unwrap().active == true && x.as_ref().unwrap().falling == true)
                    .map(|x| x.as_mut().unwrap())
            { 
                if block.y == 13 {
                    self.blocks[block.y * 10 + block.x].as_mut().unwrap().active = false;
                    self.blocks[block.y * 10 + block.x].as_mut().unwrap().falling = false;
                    continue 'fast_fall_loop;
                };
                if let None = final_block_clone[(block.y + 1) * 10 + block.x] {
                    allow_swap = true;
                    final_block_clone[(block.y + 1) * 10 + block.x] = Some(block.partial_clone(block.x, block.y + 1)); 
                    final_block_clone[block.y * 10 + block.x] = None;
                }else {
                    allow_swap = false;
                    self.blocks
                        .iter_mut()
                        .filter(|x| x.is_some() && x.as_ref().unwrap().active == true)
                        .map(|x| x.as_mut().unwrap())
                        .for_each(|x| {
                            x.active = false;
                            x.falling = false;
                        });
                    break 'fast_fall_loop;
                }
            }

            if allow_swap {
                self.blocks = final_block_clone;
            }

            self.fast_fall_ticks = 0;
        } else {
            self.fast_fall_ticks += 1;
        }

        if self.fall_ticks == FALL_TICKS {
            let mut block_clone = self.blocks.clone();
            let mut final_block_clone = self.blocks.clone();
            let mut allow_swap = false;

            'fall_loop: for block in &mut block_clone
                    .iter_mut()
                    .rev()
                    .filter(|x| x.is_some() && x.as_ref().unwrap().active == true && x.as_ref().unwrap().falling == false)
                    .map(|x| x.as_mut().unwrap())
            { 
                if block.y == 13 {
                    self.blocks[block.y * 10 + block.x].as_mut().unwrap().active = false;
                    continue 'fall_loop;
                };
                if let None = final_block_clone[(block.y + 1) * 10 + block.x] {
                    allow_swap = true;
                    final_block_clone[(block.y + 1) * 10 + block.x] = Some(block.partial_clone(block.x, block.y + 1)); 
                    final_block_clone[block.y * 10 + block.x] = None;
                }else {
                    allow_swap = false;
                    self.blocks
                        .iter_mut()
                        .filter(|x| x.is_some() && x.as_ref().unwrap().active == true)
                        .map(|x| x.as_mut().unwrap())
                        .for_each(|x| {
                            x.active = false;
                        });
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

        for y in 0..14 {
            for x in 0..10 {
                if let Some(block) = &self.blocks[y * 10 + x] {
                    canvas.draw_square(
                        160 + (x * 40),
                        40 + (y * 40),
                        160 + ((x + 1) * 40),
                        40 + ((y + 1) * 40), 
                        &block.color 
                    );
                }
            }
        }

        Ok(canvas)
    }
}



impl FallingBlocks {
    fn rotate_block_right(&mut self) -> bool {
        let blocks_clone: Vec<Block> = self.blocks
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.as_ref().unwrap())
            .filter(|x| x.active)
            .map(|x| x.clone())
            .collect();

        let (mut x1, mut y1, mut x2, mut y2) = (10usize, 14usize, 0usize, 0usize);

        for block in &blocks_clone {
            let x = block.x;
            let y = block.y;

            if x1 > x {
                x1 = x;
            }

            if y1 > y {
                y1 = y;
            }

            if x > x2 {
                x2 = x;
            }

            if y > y2 {
                y2 = y;
            }
        }

        let width = x2 - x1 + 1; // + 1 due to the fact that x2 is inclusive;
        let height = y2 - y1 + 1;

        let mut data_vec: Vec<bool> = Vec::with_capacity(width * height);
        for y in y1..=y2 {
            for x in x1..=x2 {
                data_vec.push(self.blocks[y * 10 + x].is_some());
            }
        }

        for y in y1..y2 {
            println!("Y: {} = {:?}", y, data_vec[y * 10 + x1..=y * 10 + x2].to_vec())
        }

        false
    }
}
