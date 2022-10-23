use std::env;

pub struct MinecraftColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl MinecraftColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            red: r,
            green: g,
            blue: b,
        }
    }
}

// static CONVERSION_TABLE_DIR: String = format!("{}/cached_color.hex", env::var("OUT_DIR").unwrap());
pub static CONVERSION_TABLE: &[u8; 16777216] =
    include_bytes!(concat!(env!("OUT_DIR"), "/cached_color.hex"));

pub fn get_cached_index(color: MinecraftColor) -> i8 {
    CONVERSION_TABLE
        [(color.red as usize * 256 * 256) + (color.green as usize * 256) + color.blue as usize]
        as i8
}

pub fn transform_frame_to_mc(data: &[u8], width: u32, height: u32) -> Vec<i8> {
    //height as usize * width as usize
    let mut buffer = Vec::<i8>::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            buffer.push(get_cached_index(MinecraftColor::new(
                data[((y * width * 3) + (x * 3)) as usize],
                data[((y * width * 3) + (x * 3) + 1) as usize],
                data[((y * width * 3) + (x * 3) + 2) as usize],
            )));
        }
    }

    return buffer;
}
