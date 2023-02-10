use std::sync::mpsc::Receiver;

use anyhow::anyhow;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

use crate::{
    bake_image,
    colorlib::Color,
    player::game_player::{BakedImage, Game, GameInputDirection, VideoCanvas},
};

static FRAME_COLOR: Color = Color::new(30, 40, 112);
static INNER_FRAME_COLOR: Color = Color::new(60, 60, 60);
static SPAWN_TICKS: usize = 30;
static FALL_TICKS: usize = 60;
static FAST_FALL_TICKS: usize = 4;

static NUMBER_0: BakedImage = bake_image!(tetris_0);
static NUMBER_1: BakedImage = bake_image!(tetris_1);
static NUMBER_2: BakedImage = bake_image!(tetris_2);
static NUMBER_3: BakedImage = bake_image!(tetris_3);
static NUMBER_4: BakedImage = bake_image!(tetris_4);
static NUMBER_5: BakedImage = bake_image!(tetris_5);
static NUMBER_6: BakedImage = bake_image!(tetris_6);
static NUMBER_7: BakedImage = bake_image!(tetris_7);
static NUMBER_8: BakedImage = bake_image!(tetris_8);
static NUMBER_9: BakedImage = bake_image!(tetris_9);
static GAME_OVER_SCREEN: BakedImage = bake_image!(tetris_gameover);

static RANDOM_COLORS: [Color; 7] = [
    Color::new(0, 240, 240),
    Color::new(0, 0, 240),
    Color::new(240, 160, 0),
    Color::new(240, 240, 0),
    Color::new(0, 240, 0),
    Color::new(160, 0, 240),
    Color::new(120, 0, 0),
];

pub struct FallingBlocks {
    blocks: Vec<Option<Block>>,
    spawn_ticks: usize,
    fall_ticks: usize,
    move_ticks: usize,
    fast_fall_ticks: usize,
    current_block: BlockType,
    last_color: Color,
    lines_cleared: usize,
    game_over: bool,
    rand: ThreadRng,
}

#[derive(Debug, Clone, PartialEq, Copy)]
struct Block {
    color: Color,
    x: usize,
    y: usize,
    active: bool,
    falling: bool,
}

#[derive(Debug, PartialEq)]
enum BlockType {
    O,
    T,
    I,
    S,
    Z,
    J,
    L,
}

impl BlockType {
    fn from_i8(int: i8) -> anyhow::Result<BlockType> {
        return match int {
            0 => Ok(BlockType::O),
            1 => Ok(BlockType::T),
            2 => Ok(BlockType::I),
            3 => Ok(BlockType::S),
            4 => Ok(BlockType::Z),
            5 => Ok(BlockType::J),
            6 => Ok(BlockType::L),
            _ => Err(anyhow!("Invalid intiger")),
        };
    }
}

impl Block {
    fn new(color: Color, x: usize, y: usize) -> Self {
        Self {
            color,
            x,
            y,
            active: true,
            falling: false,
        }
    }

    fn partial_clone(&self, x: usize, y: usize) -> Self {
        Self {
            color: self.color,
            x,
            y,
            active: self.active,
            falling: self.falling,
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

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            blocks: vec![None; 140],
            spawn_ticks: 0usize,
            fall_ticks: 0usize,
            move_ticks: 0usize,
            fast_fall_ticks: 0usize,
            current_block: BlockType::I, //We do not want to have a I block as the first one
            last_color: Color::BLACK,    //Does not matter
            lines_cleared: 0,
            game_over: false,
            rand: rand::thread_rng(),
        }
    }

    fn draw(&mut self, input_rx: &Receiver<GameInputDirection>) -> anyhow::Result<VideoCanvas> {
        let mut canvas = VideoCanvas::new(
            self.width() as usize,
            self.height() as usize,
            &Color::hex("464B46")?,
        );

        if self.game_over {
            self.draw_game_over_screen(&mut canvas)?;
            return Ok(canvas);
        }

        canvas.draw_square(0, 20, 580, 40, &FRAME_COLOR);
        canvas.draw_square(140, 40, 160, 600, &FRAME_COLOR);
        canvas.draw_square(140, 600, 580, 620, &FRAME_COLOR);
        canvas.draw_square(560, 40, 580, 600, &FRAME_COLOR);
        canvas.draw_square(160, 40, 560, 600, &INNER_FRAME_COLOR);
        canvas.draw_square(0, 40, 20, 100, &FRAME_COLOR);
        canvas.draw_square(20, 80, 140, 100, &FRAME_COLOR);

        self.draw_score_display(&mut canvas, 20, 40)?;

        if self.spawn_ticks == SPAWN_TICKS {
            if !self
                .blocks
                .iter()
                .any(|x| x.is_some() && x.as_ref().unwrap().active)
            {
                //Do line checks before spawing!
                'y_loop: for y in 0usize..14 {
                    for x in 0usize..10 {
                        if self.blocks[y * 10 + x].is_none() {
                            continue 'y_loop;
                        }
                    }

                    //Set line to none blocks
                    self.blocks[y * 10..(y + 1) * 10].copy_from_slice(&vec![None; 10]);

                    let mut block_copy = vec![None; 140];
                    if y + 1 != 14 {
                        block_copy[((y + 1) * 10)..self.blocks.len()]
                            .copy_from_slice(&self.blocks[((y + 1) * 10)..self.blocks.len()]);
                    }
                    if y != 0 {
                        block_copy[(10..(y + 1) * 10)].copy_from_slice(&self.blocks[0..(y * 10)]);
                    }
                    self.blocks = block_copy;
                    self.lines_cleared += 1;
                }

                let mut color = RANDOM_COLORS.choose(&mut self.rand).unwrap();
                while color == &self.last_color {
                    color = RANDOM_COLORS.choose(&mut self.rand).unwrap();
                }
                let color = *color;
                self.last_color = color;

                let mut rand_int = self.rand.gen_range(0..=6);

                //Make sure we do not get 2 the same block in a row
                //I am aware that this is not as the original tetris however this game is NOT
                //supose to be the original tetris!
                while BlockType::from_i8(rand_int)? == self.current_block {
                    rand_int = self.rand.gen_range(0..=6);
                }

                self.current_block = BlockType::from_i8(rand_int)?;

                macro_rules! set_block {
                    (
                        $X: expr,
                        $Y: expr
                    ) => {{
                        if self.blocks[$Y * 10 + $X].is_some() {
                            self.game_over = true;
                        } else {
                            self.blocks[$Y * 10 + $X] = Some(Block::new(color, $X, $Y))
                        }
                    }};
                }

                match rand_int {
                    0 => {
                        //Block O  OK
                        set_block!(3, 0);
                        set_block!(4, 0);
                        set_block!(3, 1);
                        set_block!(4, 1);
                    }
                    1 => {
                        //Block T
                        self.blocks[3] = Some(Block::new(color, 3, 0));
                        self.blocks[4] = Some(Block::new(color, 4, 0));
                        self.blocks[5] = Some(Block::new(color, 5, 0));
                        self.blocks[14] = Some(Block::new(color, 4, 1));
                    }
                    2 => {
                        //Block I
                        set_block!(3, 0);
                        set_block!(4, 0);
                        set_block!(5, 0);
                        set_block!(6, 0);
                    }
                    3 => {
                        //block S
                        set_block!(4, 0);
                        set_block!(5, 0);
                        set_block!(4, 1);
                        set_block!(3, 1);
                    }
                    4 => {
                        //Block Z OK
                        set_block!(3, 0);
                        set_block!(4, 0);
                        set_block!(4, 1);
                        set_block!(5, 1);
                    }
                    5 => {
                        //Block J
                        set_block!(3, 0);
                        set_block!(3, 1);
                        set_block!(4, 1);
                        set_block!(5, 1);
                    }
                    6 => {
                        //Block L
                        set_block!(5, 0);
                        set_block!(3, 1);
                        set_block!(4, 1);
                        set_block!(5, 1);
                    }
                    _ => {
                        return Err(anyhow!("Unexpected random int when spawing a new block"));
                    }
                }

                self.spawn_ticks = 0;
                self.fall_ticks = 0;
            } else {
                self.spawn_ticks = 0;
            }
        } else {
            self.spawn_ticks += 1;
        }

        while let Ok(val) = input_rx.try_recv() {
            if self.move_ticks != 0 {
                continue;
            };
            match val {
                GameInputDirection::Right => {
                    let mut blocks_clone = self.blocks.clone();
                    let mut allow_swap = false;
                    'block_loop: for block in self
                        .blocks
                        .clone()
                        .iter_mut()
                        .rev()
                        .filter(|x| x.is_some() && x.as_ref().unwrap().active)
                        .map(|x| x.as_ref().unwrap())
                    {
                        if block.x + 1 >= 10 {
                            continue 'block_loop;
                        }

                        if blocks_clone[block.y * 10 + (block.x + 1)].is_none() {
                            allow_swap = true;
                            blocks_clone[block.y * 10 + (block.x + 1)] =
                                Some(block.partial_clone(block.x + 1, block.y));
                            blocks_clone[block.y * 10 + block.x] = None;
                        } else {
                            allow_swap = false;
                            break 'block_loop;
                        }
                    }

                    if allow_swap {
                        self.blocks = blocks_clone;
                    }
                    self.move_ticks = 10;
                    break;
                }
                GameInputDirection::Left => {
                    let mut blocks_clone = self.blocks.clone();
                    let mut allow_swap = false;
                    'block_loop: for block in self
                        .blocks
                        .clone()
                        .iter_mut()
                        .filter(|x| x.is_some() && x.as_ref().unwrap().active)
                        .map(|x| x.as_ref().unwrap())
                    {
                        if block.x == 0 {
                            continue 'block_loop;
                        }

                        if blocks_clone[block.y * 10 + (block.x - 1)].is_none() {
                            allow_swap = true;
                            blocks_clone[block.y * 10 + (block.x - 1)] =
                                Some(block.partial_clone(block.x - 1, block.y));
                            blocks_clone[block.y * 10 + block.x] = None;
                        } else {
                            allow_swap = false;
                            break 'block_loop;
                        }
                    }

                    if allow_swap {
                        self.blocks = blocks_clone;
                    }

                    self.move_ticks = 10;
                    break;
                }
                GameInputDirection::Up => {
                    self.blocks
                        .iter_mut()
                        .rev()
                        .filter(|x| x.is_some() && x.as_ref().unwrap().active)
                        .map(|x| x.as_mut().unwrap())
                        .for_each(|block| {
                            block.falling = true;
                        });
                }
                GameInputDirection::Forward => {
                    //Right rotate
                    if self.rotate_block(false) {
                        self.move_ticks = 10;
                        break;
                    }
                }
                GameInputDirection::Backwards => {
                    //left rotate
                    if self.rotate_block(true) {
                        self.move_ticks = 10;
                        break;
                    }
                }
            };
        }

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
                .filter(|x| {
                    x.is_some() && x.as_ref().unwrap().active && x.as_ref().unwrap().falling
                })
                .map(|x| x.as_mut().unwrap())
            {
                if block.y == 13 {
                    self.blocks[block.y * 10 + block.x].as_mut().unwrap().active = false;
                    self.blocks[block.y * 10 + block.x]
                        .as_mut()
                        .unwrap()
                        .falling = false;
                    continue 'fast_fall_loop;
                };
                if final_block_clone[(block.y + 1) * 10 + block.x].is_none() {
                    allow_swap = true;
                    final_block_clone[(block.y + 1) * 10 + block.x] =
                        Some(block.partial_clone(block.x, block.y + 1));
                    final_block_clone[block.y * 10 + block.x] = None;
                } else {
                    allow_swap = false;
                    self.blocks
                        .iter_mut()
                        .filter(|x| x.is_some() && x.as_ref().unwrap().active)
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
                .filter(|x| {
                    x.is_some() && x.as_ref().unwrap().active && !x.as_ref().unwrap().falling
                })
                .map(|x| x.as_mut().unwrap())
            {
                if block.y == 13 {
                    self.blocks[block.y * 10 + block.x].as_mut().unwrap().active = false;
                    continue 'fall_loop;
                };
                if final_block_clone[(block.y + 1) * 10 + block.x].is_none() {
                    allow_swap = true;
                    final_block_clone[(block.y + 1) * 10 + block.x] =
                        Some(block.partial_clone(block.x, block.y + 1));
                    final_block_clone[block.y * 10 + block.x] = None;
                } else {
                    allow_swap = false;
                    self.blocks
                        .iter_mut()
                        .filter(|x| x.is_some() && x.as_ref().unwrap().active)
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
                        &block.color,
                    );
                }
            }
        }

        Ok(canvas)
    }
}

impl FallingBlocks {
    fn rotate_block(&mut self, left: bool) -> bool {
        if self.current_block == BlockType::O {
            return false;
        }

        let mut blocks_clone: Vec<Option<Block>> = self
            .blocks
            .iter()
            .map(|x| {
                if x.is_none() {
                    *x
                } else if x.as_ref().unwrap().active {
                    None
                } else {
                    *x
                }
            })
            .collect();

        let any_block: &Block = match self
            .blocks
            .iter()
            .find(|x| x.is_some() && x.as_ref().unwrap().active)
        {
            Some(val) => val.as_ref().unwrap(),
            None => return false,
        };

        let (mut x1, mut y1, mut x2, mut y2) = (10usize, 14usize, 0usize, 0usize);

        self.blocks
            .iter()
            .filter(|x| x.is_some() && x.as_ref().unwrap().active)
            .map(|x| x.as_ref().unwrap())
            .for_each(|block| {
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
            });

        let mut width = x2 - x1 + 1; // + 1 due to the fact that x2 is inclusive;
        let mut height = y2 - y1 + 1;

        let mut data_vec: Vec<bool> = Vec::with_capacity(width * height);
        if !left {
            for x in x1..=x2 {
                for y in (y1..=y2).rev() {
                    let block = &self.blocks[y * 10 + x];
                    data_vec.push(block.is_some() && block.as_ref().unwrap().active);
                }
            }
        } else {
            for x in (x1..=x2).rev() {
                for y in y1..=y2 {
                    let block = &self.blocks[y * 10 + x];
                    data_vec.push(block.is_some() && block.as_ref().unwrap().active);
                }
            }
        }

        std::mem::swap(&mut width, &mut height);

        for y in 0..height {
            for x in 0..width {
                if !data_vec[y * width + x] {
                    continue;
                }

                if y1 + y >= 14 {
                    return false;
                }
                if x1 + x >= 10 {
                    return false;
                }

                let block = &blocks_clone[(y1 + y) * 10 + (x1 + x)];
                if block.is_some() {
                    return false;
                }

                blocks_clone[(y1 + y) * 10 + (x1 + x)] =
                    Some(any_block.partial_clone(x1 + x, y1 + y));
            }
        }

        self.blocks = blocks_clone;
        true
    }

    fn draw_score_display(
        &self,
        canvas: &mut VideoCanvas,
        x1: usize,
        y1: usize,
    ) -> anyhow::Result<()> {
        let chars = &mut self
            .lines_cleared
            .to_string()
            .chars()
            .map(|d| d.to_digit(10).unwrap())
            .collect::<Vec<_>>();

        while chars.len() != 3 {
            chars.push(0);
            chars.rotate_right(1);
        }

        for (id, x) in chars.iter().enumerate() {
            let img = match x {
                0 => &NUMBER_0,
                1 => &NUMBER_1,
                2 => &NUMBER_2,
                3 => &NUMBER_3,
                4 => &NUMBER_4,
                5 => &NUMBER_5,
                6 => &NUMBER_6,
                7 => &NUMBER_7,
                8 => &NUMBER_8,
                9 => &NUMBER_9,
                _ => return Err(anyhow!("Unable to draw segment screen")),
            };

            canvas.draw_image(x1 + (40 * id), y1, img);
        }
        Ok(())
    }

    fn draw_game_over_screen(&mut self, canvas: &mut VideoCanvas) -> anyhow::Result<()> {
        canvas.draw_image(0, 0, &GAME_OVER_SCREEN);
        self.draw_score_display(canvas, 320, 240)?;
        Ok(())
    }
}
