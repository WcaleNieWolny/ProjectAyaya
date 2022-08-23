use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Debug, Clone)]
pub struct SplittedFrame {
    start_x: i32,
    start_y: i32,
    width: i32,
    height: i32,
    x_margin: i32,
    y_margin: i32,
    pub data: Vec<i8>,
    pub frame_length: i32,
}

static FRAME_SPLITTER_ALL_FRAMES_X: AtomicI32 = AtomicI32::new(0);
static FRAME_SPLITTER_ALL_FRAMES_Y: AtomicI32 = AtomicI32::new(0);

impl SplittedFrame {
    pub fn initialize_frames(width: i32, height: i32) -> anyhow::Result<Vec<SplittedFrame>> {
        let mut frames: Vec<SplittedFrame> = Vec::new();

        if width % 2 != 0 {
            return Err(anyhow::Error::msg("asymmetrical width is not supported"));
        }
        if height % 2 != 0 {
            return Err(anyhow::Error::msg("asymmetrical height is not supported"));
        }

        let frames_x = width as f32 / 128.0;
        let frames_y = height as f32 / 128.0;

        let x_margin = if width % 128 == 0 {
            0
        } else {
            128 - (width - (frames_x as i32 * 128))
        };
        let y_margin = if height % 128 == 0 {
            0
        } else {
            128 - (height - (frames_y as i32 * 128))
        };

        let all_frames_x = frames_x.ceil() as i32;
        let all_frames_y = frames_y.ceil() as i32;

        FRAME_SPLITTER_ALL_FRAMES_X.store(all_frames_x, Relaxed);
        FRAME_SPLITTER_ALL_FRAMES_Y.store(all_frames_y, Relaxed);

        let mut i = 0;

        for x in 0..all_frames_x {
            for y in 0..all_frames_y {
                let x_frame_margin = if x == 0 {
                    x_margin / 2
                } else {
                    0
                };
                let y_frame_margin = if y == 0 {
                    y_margin / 2
                } else {
                    0
                };

                let frame_width = if x != all_frames_x - 1 {
                    128 - x_frame_margin
                } else {
                    128 - (x_margin / 2)
                };
                let frame_height = if y != (all_frames_y - 1) {
                    128 - y_frame_margin
                } else {
                    128 - (y_margin / 2)
                };

                let frame_length = frame_height * frame_width;

                //println!("DAT: {}, {}, {}, {}, {}, {}", x_frame_margin, y_frame_margin, width, height, x_margin, y_margin);
                i = i + 1;

                frames.push(
                    SplittedFrame {
                        start_x: x_frame_margin,
                        start_y: y_frame_margin,
                        width: frame_width,
                        height: frame_height,
                        x_margin,
                        y_margin,
                        data: Vec::with_capacity(frame_length as usize),
                        frame_length,
                    }
                )
            }
        }

        Ok(frames)
    }

    pub fn split_frames(data: Vec<i8>, frames: &mut Vec<SplittedFrame>, width: i32) -> anyhow::Result<()> {
        let all_frames_x = FRAME_SPLITTER_ALL_FRAMES_X.load(Relaxed);
        let all_frames_y = FRAME_SPLITTER_ALL_FRAMES_Y.load(Relaxed);

        if all_frames_y * all_frames_x != frames.len() as i32 {
            return Err(anyhow::Error::msg("Frame list size does not match required lenght"));
        }

        let mut i = 0;
        let mut y_i = 0;

        for y in 0..all_frames_y {
            let mut x_i = 0;
            for _x in 0..all_frames_x {
                let frame = &mut frames[i];
                let frame_data = &mut frame.data;

                for y1 in 0..frame.height {
                    frame_data.extend_from_slice(&data[(y_i * width + x_i) as usize + (y1 * width) as usize..(y_i * width + x_i) as usize + (y1 * width) as usize + frame.width as usize])
                }

                x_i += frame.width;
                i = i + 1;
            }
            y_i += frames[(y * all_frames_x) as usize].height
        };

        Ok(())
    }
}

//    pub fn split_frames(data: Vec<i8>, mut frames: &mut Vec<SplittedFrame>, width: i32, height: i32) -> anyhow::Result<Vec<i8>>{
//         let all_frames_x = FRAME_SPLITTER_ALL_FRAMES_X.load(Relaxed);
//         let all_frames_y = FRAME_SPLITTER_ALL_FRAMES_Y.load(Relaxed);
//
//         let mut index: usize = 0;
//         let mut vec: Vec<i8> = Vec::with_capacity((all_frames_x * all_frames_y * 128 * 128) as usize);
//
//         for frame in frames.iter() {
//             for y in 0..frame.height {
//                 for x in 0..frame.width {
//                     vec.push(data[index]);
//                     index = index + 1;
//                 }
//             }
//         }
//
//         Ok(vec)
//     }