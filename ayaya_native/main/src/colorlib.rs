use anyhow::anyhow;
use rayon::prelude::*;
use std::{env, num::ParseIntError, sync::atomic::Ordering};

use crate::splitting::ExternalSplitFrameMemCopyRange;

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

    pub const BLACK: Color = Color::new(0, 0, 0);

    pub fn hex(hex: &str) -> anyhow::Result<Self> {
        let hex = hex.to_string();
        let hex = hex.replace('#', "");

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

    pub fn convert_to_mc(&self) -> u8 {
        get_cached_index(self) as u8
    }
}

// static CONVERSION_TABLE_DIR: String = format!("{}/cached_color.hex", env::var("OUT_DIR").unwrap());
#[cfg(not(feature = "skip_buildrs"))]
pub static CONVERSION_TABLE: &[u8; 16777216] =
    include_bytes!(concat!(env!("OUT_DIR"), "/cached_color.hex"));

#[cfg(feature = "skip_buildrs")]
pub static CONVERSION_TABLE: &[u8; 1] =
    include_bytes!(concat!(env!("OUT_DIR"), "/cached_color.hex"));

pub static CONVERSION_TABLE_YUV: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/cached_color_yuv.hex"));

pub fn get_cached_index(color: &Color) -> i8 {
    CONVERSION_TABLE
        [(color.red as usize * 256 * 256) + (color.green as usize * 256) + color.blue as usize]
        as i8
}

#[cfg(feature = "ffmpeg")]
pub fn transform_frame_to_mc(
    data: &[u8],
    width: usize,
    height: usize,
    add_width: usize,
) -> Vec<i8> {
    let mut buffer = Vec::<i8>::with_capacity((width * height) as usize);

    for y in 0..height as usize {
        for x in 0..width as usize {
            buffer.push(get_cached_index(&Color::new(
                data[(y * add_width) + (x * 3)],
                data[(y * add_width) + (x * 3) + 1],
                data[(y * add_width) + (x * 3) + 2],
            )));
        }
    }

    buffer
}
