use anyhow::anyhow;
use rayon::prelude::*;
use std::{env, num::ParseIntError, sync::atomic::Ordering};

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
                data[((y * add_width) + (x * 3))],
                data[((y * add_width) + (x * 3) + 1)],
                data[((y * add_width) + (x * 3) + 2)],
            )));
        }
    }

    buffer
}

#[cfg(feature = "ffmpeg")]
pub fn transform_frame_to_mc_yuv(
    y_arr: &[u8],
    cb_arr: &[u8],
    cr_arr: &[u8],
    width: usize,
    height: usize,
    fast_lookup_map: &Vec<usize>
) -> anyhow::Result<Vec<i8>> {
    let mut vec = Vec::<i8>::with_capacity(width * height);
    let buf_ptr = vec.as_mut_ptr() as usize;

    let y_iter = y_arr.par_iter();
    let cr_iter = cr_arr.par_iter();
    let cb_iter = cb_arr.par_iter();

    y_iter
        .zip(cb_iter)
        .zip(cr_iter)
        .enumerate()
        .for_each(|(index, ((y, cb), cr))| {
            let color = CONVERSION_TABLE_YUV[(*y as usize * 256 * 256) + (*cb as usize * 256) + *cr as usize] as i8;
            
            let ptr_offset = fast_lookup_map[index];
            let ptr = (buf_ptr + ptr_offset) as *mut i8;

            unsafe {
                ptr.write_volatile(color)
            }

        });

    unsafe {
        vec.set_len(width * height)
    }

    Ok(vec)

}

//Thanks to https://github.com/The0x539 for help with this
//No unsafe version: https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=815de260ce2c61db254bd79434caa396
//This version: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=5c9886ffabf102ff3d06f9495c9ad267
//Previous version: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=e51cac7c4eb4c8612157cc3ec1bc3642
pub fn unsafe_transform_and_split_frame_to_mc(
    data: &[u8],
    fast_lookup_map: &Vec<usize>,
    width: usize,
    height: usize,
    stride: usize,
) -> anyhow::Result<Vec<i8>> {
    let area = width * height;

    let mut buffer: Vec<i8> = Vec::with_capacity(area);
    let buf = &mut buffer.spare_capacity_mut()[..area];
    let buf_ptr = buf.as_mut_ptr() as usize;

    let src_rows = data.par_chunks(stride);
    let index_rows = fast_lookup_map.par_chunks(width);

    src_rows
        .zip(index_rows)
        .enumerate()
        .for_each(|(y, (src_row, index_row))| {
            let pixels = src_row.array_chunks::<3>().take(width);

            for (x, ([r, g, b], index)) in pixels.zip(index_row).enumerate() {
                let color = Color::new(*r, *g, *b);
                let value = get_cached_index(&color);

                let index = &fast_lookup_map[y * width + x];

                unsafe {
                    let ptr = (buf_ptr + index) as *mut i8; //pointer arithmetic
                    ptr.write_volatile(value);
                };
            }
        });

    unsafe { buffer.set_len(area) }

    Ok(buffer)
}
