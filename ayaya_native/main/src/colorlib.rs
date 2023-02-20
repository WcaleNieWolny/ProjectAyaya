use anyhow::anyhow;
use rayon::prelude::*;
use std::{env, num::ParseIntError, mem::{MaybeUninit, self}, sync::atomic::Ordering};

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

pub fn get_cached_index(color: &Color) -> i8 {
    CONVERSION_TABLE
        [(color.red as usize * 256 * 256) + (color.green as usize * 256) + color.blue as usize]
        as i8
}

#[cfg(feature = "ffmpeg")]
pub fn transform_frame_to_mc(data: &[u8], width: u32, height: u32, add_width: usize) -> Vec<i8> {
    let mut buffer = Vec::<i8>::with_capacity((width * height) as usize);

    for y in 0..height as usize {
        for x in 0..width as usize {
            buffer.push(get_cached_index(&Color::new(
                data[((y * add_width) + (x * 3))],
                data[((y * add_width) + (x * 3) + 1)],
                data[((y * add_width) + (x * 3) + 2)],
            )));
        }
    }

    buffer
}

#[ignore = "unused"]
pub fn fast_transform_frame_to_mc(data: &[u8], width: usize, height: usize, line_size: usize) -> Vec<i8> {
    unsafe {
        let mut buffer: Vec<i8> = vec![0i8; width * height];

        let buf_start = buffer.as_mut_ptr();

        data
            .chunks(line_size)
            .enumerate()
            .for_each(|(line, data)| 
                data.iter().take(width * 3).array_chunks::<3>().enumerate().for_each(|(pix_id, pix_data)| {
                    let ptr = buf_start.clone().add(line * width + pix_id);
                    ptr.write_volatile(get_cached_index(&Color::new(*pix_data[0], *pix_data[1], *pix_data[2])))
                })
            );

        return buffer 
    };
}

//Thanks to https://github.com/The0x539 for help with this
//No unsafe version: https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=815de260ce2c61db254bd79434caa396
//This version: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=5c9886ffabf102ff3d06f9495c9ad267
//Previous version: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=e51cac7c4eb4c8612157cc3ec1bc3642
pub fn second_fast_transform_frame_to_mc(
    data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
) -> Vec<i8> {
    let area = width * height;

    let mut buffer = Vec::with_capacity(area);
    let buf = &mut buffer.spare_capacity_mut()[..area];

    let src_rows = data.par_chunks(stride);
    let dst_rows = buf.par_chunks_mut(width);

    // feel free to remove this once you're 100% sure it works right and is stable
    #[cfg(debug_assertions)]
    let num_init = std::sync::atomic::AtomicUsize::new(0);

    src_rows.zip(dst_rows).for_each(|(src_row, dst_row)| {
        let pixels = src_row.array_chunks::<3>().take(width).copied();

        for ([r, g, b], dst) in pixels.zip(dst_row) {
            let color = Color::new(r, g, b);
            let value = get_cached_index(&color);
            dst.write(value);

            #[cfg(debug_assertions)]
            num_init.fetch_add(1, Ordering::Relaxed);
        }
    });

    unsafe {
        //debug_assert_eq!(num_init.into_inner(), area);

        // SAFETY: the above loop manually initialized
        // all the values within the spare capacity (up to area).
        buffer.set_len(area);
    }

    buffer
}
