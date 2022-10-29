use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Debug, Clone)]
pub struct SplittedFrame {
    pub width: i32,
    pub height: i32,
    pub frame_length: i32,
}

pub static FRAME_SPLITTER_ALL_FRAMES_X: AtomicI32 = AtomicI32::new(0);
pub static FRAME_SPLITTER_ALL_FRAMES_Y: AtomicI32 = AtomicI32::new(0);

pub fn generate_index_cache(width: u32, height: u32, splitted_frames: Vec<SplittedFrame>) -> Vec<usize>{
    let mut output = Vec::<usize>::with_capacity(width as usize * height as usize);
    output.resize(output.capacity(), 0);

    let all_frames_x = FRAME_SPLITTER_ALL_FRAMES_X.load(Relaxed) as u32;
    let all_frames_y = FRAME_SPLITTER_ALL_FRAMES_Y.load(Relaxed) as u32;

    let mut offset_x = 0;
    let mut offset_y = 0;
    let mut i = 0;
    let mut offset_xy = 0;

    for a_y in 0..all_frames_y {
        for _ in 0..all_frames_x {
            let frame = &splitted_frames[i];
          
            let width_start = offset_x;
            let height_start = offset_y;
            let width_end = offset_x + (frame.width as u32);
            let height_end = offset_y + (frame.height as u32);
            let frame_width = frame.width;
            let frame_height = frame.height;

            for x in width_start..width_end {
                for y in height_start..height_end {
                    let x1 = x - width_start;
                    let y1 = y - height_start;

                    output[(y as usize * width as usize) + x as usize] = offset_xy as usize + (y1 as usize * frame_width as usize) as usize + x1 as usize            
                }
            };

            offset_x += frame.width as u32;
            offset_xy += frame.width as u32 * frame.height as u32;
            i += 1;
        }
        offset_y += splitted_frames[(a_y * all_frames_x) as usize].height as u32;
        offset_x = 0;
    }


    output
}

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

        for y in 0..all_frames_y {
            for x in 0..all_frames_x {
                let x_frame_margin = if x == 0 { x_margin / 2 } else { 0 };
                let y_frame_margin = if y == 0 { y_margin / 2 } else { 0 };

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

                //println!("DAT: {}, {}, {}, {}, {}, {}", x_frame_margin, y_frame_margin, frame_width, frame_height, x_margin, y_margin);
                //println!("DAT: {}", frame_width);
                i = i + 1;

                frames.push(SplittedFrame {
                    width: frame_width,
                    height: frame_height,
                    frame_length,
                })
            }
        }

        Ok(frames)
    }

    pub fn split_frames(
        data: &[i8],
        frames: &mut Vec<SplittedFrame>,
        width: i32,
    ) -> anyhow::Result<Vec<i8>> {
        let all_frames_x = FRAME_SPLITTER_ALL_FRAMES_X.load(Relaxed);
        let all_frames_y = FRAME_SPLITTER_ALL_FRAMES_Y.load(Relaxed);

        if all_frames_y * all_frames_x != frames.len() as i32 {
            return Err(anyhow::Error::msg(
                "Frame list size does not match required lenght",
            ));
        }

        //let mut final_data: Vec<i8> = Vec::with_capacity((all_frames_x * all_frames_y * 128 * 128) as usize);
        let mut final_data = vec![0i8; (all_frames_x * all_frames_y * 128 * 128) as usize];

        //println!("D SIZE: {}, {}", final_data.len(), data.len());

        let mut i = 0;
        let mut y_i = 0;

        let mut final_data_index = 0;

        for y in 0..all_frames_y {
            let mut x_i = 0;
            for _x in 0..all_frames_x {
                let frame = &mut frames[i];

                for y1 in 0..frame.height {
                    //final_data.extend_from_slice(&data[(y_i * width + x_i) as usize + (y1 * width) as usize..(y_i * width + x_i) as usize + (y1 * width) as usize + frame.width as usize])
                    final_data[final_data_index as usize
                        ..final_data_index as usize + frame.width as usize]
                        .copy_from_slice(
                            &data[(y_i * width + x_i) as usize + (y1 * width) as usize
                                ..(y_i * width + x_i) as usize
                                    + (y1 * width) as usize
                                    + frame.width as usize],
                        );

                    final_data_index = final_data_index + frame.width

                    //for x1 in 0..frame.width{
                    // ((yI * width) + xI) + ((y1 * width) + x1)
                    // final_data[f_i as usize + (y1 * frame.width) as usize + x1 as usize] = data[((y_i * width) + x_i) as usize + ((y1 * width) as usize + x1 as usize)];
                    //final_data.push(data[((y_i * width) + x_i) as usize + ((y1 * width) as usize + x1 as usize)]);
                    //final_data.push(88);
                    //}
                }

                x_i = x_i + frame.width;
                i = i + 1;
            }
            y_i += frames[(y * all_frames_x) as usize].height;
        }

        Ok(final_data)
    }

    pub fn fast_split(data: &[i8], cache: Vec<usize>) -> Vec<i8>{
        let mut out = Vec::<i8>::with_capacity(data.len());
        unsafe {
            out.set_len(out.capacity())
        } //Speed > mem safety
          
        for i in 0..data.len(){
            out[cache[i]] = data[i]
        }

        out
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
