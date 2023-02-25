#[derive(Debug, Clone)]
pub struct SplittedFrame {
    pub width: usize,
    pub height: usize,
    pub frame_length: usize,
}

#[repr(C)]
pub struct ExternalSplitFrameMemCopyRange {
    src_offset: usize,
    dst_offset: usize,
    len: usize
}

impl SplittedFrame {
    pub fn initialize_frames(
        width: usize,
        height: usize,
    ) -> anyhow::Result<(Vec<SplittedFrame>, usize, usize)> {
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
            128 - (width - (frames_x as usize * 128))
        };
        let y_margin = if height % 128 == 0 {
            0
        } else {
            128 - (height - (frames_y as usize * 128))
        };

        let all_frames_x = frames_x.ceil() as usize;
        let all_frames_y = frames_y.ceil() as usize;

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

                frames.push(SplittedFrame {
                    width: frame_width,
                    height: frame_height,
                    frame_length,
                })
            }
        }

        Ok((frames, all_frames_x, all_frames_y))
    }

    pub fn split_frames(
        data: &[i8],
        frames: &Vec<SplittedFrame>,
        width: usize,
        all_frames_x: usize,
        all_frames_y: usize,
    ) -> anyhow::Result<Vec<i8>> {
        if all_frames_y * all_frames_x != frames.len() {
            return Err(anyhow::Error::msg(
                "Frame list size does not match required lenght",
            ));
        }

        //let mut final_data: Vec<i8> = Vec::with_capacity((all_frames_x * all_frames_y * 128 * 128) as usize);
        let mut final_data = vec![0i8; (all_frames_x * all_frames_y * 128 * 128) as usize];

        let mut i = 0usize;
        let mut y_i = 0usize;

        let mut final_data_index = 0;

        for y in 0..all_frames_y {
            let mut x_i = 0;
            for _x in 0..all_frames_x {
                let frame = &frames[i];

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

                    final_data_index += frame.width

                    //for x1 in 0..frame.width{
                    // ((yI * width) + xI) + ((y1 * width) + x1)
                    // final_data[f_i as usize + (y1 * frame.width) as usize + x1 as usize] = data[((y_i * width) + x_i) as usize + ((y1 * width) as usize + x1 as usize)];
                    //final_data.push(data[((y_i * width) + x_i) as usize + ((y1 * width) as usize + x1 as usize)]);
                    //final_data.push(88);
                    //}
                }

                x_i += frame.width;
                i += 1;
            }
            y_i += frames[(y * all_frames_x) as usize].height;
        }

        Ok(final_data)
    }

    pub fn prepare_fast_split(
        frames: &Vec<SplittedFrame>,
        width: usize,
        height: usize,
        all_frames_x: usize,
        all_frames_y: usize,
    ) -> anyhow::Result<Vec<usize>> {
        let mut index_table = vec![0usize; width * height];

        let mut i = 0usize;
        let mut y_i = 0usize;

        let mut final_data_index = 0;

        for y in 0..all_frames_y {
            let mut x_i = 0;
            for _x in 0..all_frames_x {
                let frame = &frames[i];

                for y1 in 0..frame.height {
                    for x1 in 0..frame.width {
                        index_table
                            [(y_i * width + x_i) as usize + (y1 * width) as usize + x1 as usize] =
                            final_data_index + x1;
                    }

                    final_data_index += frame.width
                }

                x_i += frame.width;
                i += 1;
            }
            y_i += frames[(y * all_frames_x) as usize].height;
        }

        Ok(index_table)
    }

    #[cfg(feature = "external_splitting")]
    pub fn prepare_external_ranges(
        frames: &Vec<SplittedFrame>,
        width: usize,
        height: usize,
        all_frames_x: usize,
        all_frames_y: usize,
    ) -> anyhow::Result<Vec<ExternalSplitFrameMemCopyRange>> {
        let mut ranges_table = Vec::<ExternalSplitFrameMemCopyRange>::new();

        let mut i = 0usize;
        let mut y_i = 0usize;

        let mut final_data_index = 0;

        for y in 0..all_frames_y {
            let mut x_i = 0;
            for _x in 0..all_frames_x {
                let frame = &frames[i];

                for y1 in 0..frame.height {

                    ranges_table.push(ExternalSplitFrameMemCopyRange {
                        src_offset: (y_i * width + x_i) as usize + (y1 * width) as usize,
                        dst_offset: final_data_index,
                        len: frame.width,
                    });

                    final_data_index += frame.width
                }

                x_i += frame.width;
                i += 1;
            }
            y_i += frames[(y * all_frames_x) as usize].height;
        }

        Ok(ranges_table)
    }
}
