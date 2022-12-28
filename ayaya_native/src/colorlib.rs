use std::{env, num::ParseIntError};
use anyhow::anyhow;
use rand::{rngs::ThreadRng, Rng};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            red: r,
            green: g,
            blue: b,
        }
    }

    pub const RED: Color = Color::new(255, 0, 0);
    pub const BLACK: Color = Color::new(0, 0, 0);

    pub fn hex(hex: &str) -> anyhow::Result<Self> {
        let hex = hex.to_string();
        let hex = hex.replace("#", "");

        if hex.len() != 6 {
           return Err(anyhow!("Invalid hex")); 
        }

        let hex: Result<Vec<u8>, ParseIntError> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
            .collect();
        let hex = hex?;

        Ok(Self::new(hex[0], hex[1], hex[2]))
    }

    pub fn to_mc(&self) -> u8{
        get_cached_index(self) as u8
    }

    pub fn random(rng: &mut ThreadRng) -> Self {
        Self {
           red: rng.gen(),
           green: rng.gen(),
           blue: rng.gen()
        }
    }
}

// static CONVERSION_TABLE_DIR: String = format!("{}/cached_color.hex", env::var("OUT_DIR").unwrap());
pub static CONVERSION_TABLE: &[u8; 16777216] =
    include_bytes!(concat!(env!("OUT_DIR"), "/cached_color.hex"));


pub fn get_cached_index(color: &Color) -> i8 {
    CONVERSION_TABLE
        [(color.red as usize * 256 * 256) + (color.green as usize * 256) + color.blue as usize]
        as i8
}

pub fn transform_frame_to_mc(data: &[u8], width: u32, height: u32) -> Vec<i8> {
    //height as usize * width as usize
    let mut buffer = Vec::<i8>::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            buffer.push(get_cached_index(&Color::new(
                data[((y * width * 3) + (x * 3)) as usize],
                data[((y * width * 3) + (x * 3) + 1) as usize],
                data[((y * width * 3) + (x * 3) + 2) as usize],
            )));
        }
    }

    return buffer;
}
