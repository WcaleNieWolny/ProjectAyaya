use anyhow::anyhow;
use std::{env, num::ParseIntError};
use rayon::prelude::*;

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

#[cfg(feature = "external_player")]
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

pub fn fast_frame_to_mc(
    data: &[u8],
    width: usize,
    height: usize,
    add_width: usize
) -> Vec<i8> {
    let mut buf = Vec::<i8>::with_capacity(width * height);
    let buf_ptr = buf.as_mut_ptr() as usize;

    data
        .par_chunks(add_width)
        .enumerate()
        .for_each(|(i, arr)| {
            arr
                .array_chunks::<3>()
                .take(width)
                .enumerate()
                .for_each(|(c_id, [r, g, b])| {
                    let color = Color::new(*r, *g, *b);
                    let value = get_cached_index(&color);
                    unsafe {
                        let write_ptr = (buf_ptr + (i * width) + c_id) as *mut i8;
                        write_ptr.write_volatile(value);
                    };
                })
        });

    unsafe {
        buf.set_len(width * height)
    }
    return buf;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use test::Bencher;
    use crate::SplittedFrame;

    #[test]
    fn test_fast_split() {
        let width = 3840usize;
        let height = 2160usize;
        let linesize = width * 3;

        let (splitted_frames, all_frames_x, all_frames_y) =
            SplittedFrame::initialize_frames(width, height).unwrap();

        let values: Vec<u8> = rand::thread_rng()
            .sample_iter(rand::distributions::Standard)
            .take(width * height * 3)
            .collect();

        let fast_conversion = fast_frame_to_mc(&values, width, height, width * 3);
        let normal_conversion = transform_frame_to_mc(&values, width, height, width * 3);

        assert!(do_vecs_match(&fast_conversion, &normal_conversion))
    }

    #[bench]
    fn bench_color_conversion(b: &mut Bencher) {
        let width = 3840usize;
        let height = 2160usize;
        let linesize = width * 3;

        let (splitted_frames, all_frames_x, all_frames_y) =
            SplittedFrame::initialize_frames(width, height).unwrap();
        let values: Vec<u8> = vec![89u8; width * height * 3];

        b.iter(|| {
            transform_frame_to_mc(&values, width, height, width * 3)
        });
    }

    #[bench]
    fn bench_fast_color_conversion(b: &mut Bencher) {
        let width = 3840usize;
        let height = 2160usize;
        let linesize = width * 3;

        let (splitted_frames, all_frames_x, all_frames_y) =
            SplittedFrame::initialize_frames(width, height).unwrap();
        let values: Vec<u8> = vec![89u8; width * height * 3];

        b.iter(|| {
            fast_frame_to_mc(&values, width, height, width * 3)
        });
    }

    fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
        let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
        matching == a.len() && matching == b.len()
    }
}
