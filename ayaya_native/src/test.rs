#[cfg(test)]
mod tests {
    use std::{
        thread::{self, Thread},
        time::Duration,
    };

    use crate::{player::{
        gpu_player::{self, GpuSplittedFrameInfo},
        player_context::{PlayerContext, VideoPlayer, receive_and_process_decoded_frames}, single_video_player,
    }, splitting::{SplittedFrame, FRAME_SPLITTER_ALL_FRAMES_X, FRAME_SPLITTER_ALL_FRAMES_Y}, colorlib::{transform_frame_to_mc, self, MinecraftColor}};

    use ffmpeg::{format::{input, Pixel}, color};
    use ffmpeg::media::Type;
    use ffmpeg::software::scaling::{Context, Flags};
    use ffmpeg::Error;
    use std::sync::atomic::Ordering::Relaxed;

//Width                                    : 3840 pixels
//Height                                   : 2160 pixels

//8294400



    #[test]
    fn gpu() {
        let gpu_player =
            gpu_player::GpuVideoPlayer::create("/home/wolny/Downloads/4k_test.mp4".to_string())
                .expect("Creation failed");

        let ptr = gpu_player.wrap_to_ptr();

        for i in 0..999 {
            let _ = PlayerContext::load_frame(ptr).unwrap();
        }

        assert_eq!(4, 4);
    }

    #[test]
    fn gpu_split_test_2() {
        let gpu_player =
            gpu_player::GpuVideoPlayer::create("/home/wolny/Downloads/4k_test.mp4".to_string())
                .expect("Creation failed");

        let ptr = gpu_player.wrap_to_ptr();

        let frame = PlayerContext::load_frame(ptr).unwrap();

            let mut diff = 0;
        let single = single_video_player::SingleVideoPlayer::create("/home/wolny/Downloads/4k_test.mp4".to_string())
            .expect("Creation failed");
        let single = single.wrap_to_ptr();
        let frame_single = PlayerContext::load_frame(single).unwrap();
        for i in 0..frame.len()-1 {
            if frame_single[i] != frame[i] {
                diff += 1;
                println!("O: {}, THE: {}", frame_single[i], frame[i])
            }
        };

        println!("D: {}", diff);


        assert_eq!(4, 4);
    }

    #[test]
    fn gpu_split_test(){
        if let Ok(mut ictx) = input(&"/home/wolny/Downloads/4k_test.mp4".to_string()) {
            let input = ictx
                .streams()
                .best(Type::Video)
                .ok_or(Error::StreamNotFound).unwrap();

            let video_stream_index = input.index();

            let context_decoder =
                ffmpeg::codec::context::Context::from_parameters(input.parameters()).unwrap();

            let mut decoder = context_decoder.decoder();
            let mut decoder = decoder.video().unwrap();

            let width = decoder.width();
            let height = decoder.height();

            let fps = input.rate().0 / input.rate().1;

            let mut scaler = Context::get(
                decoder.format(),
                width,
                height,
                Pixel::RGB24,
                width,
                height,
                Flags::BILINEAR,
            ).unwrap();

            let mut splitted_frames = SplittedFrame::initialize_frames(width as i32, height as i32).unwrap();

            while let Some((stream, packet)) = ictx.packets().next() {
                if stream.index() == video_stream_index as usize {
                    decoder.send_packet(&packet).unwrap();
                    let frame_data = receive_and_process_decoded_frames(
                        &mut decoder,
                        &mut scaler,
                        &packet,
                    ).unwrap();

                    let vid = frame_data.data(0);

                    let transformed_frame = transform_frame_to_mc(frame_data.data(0), width, height);
                    
                    let transformed_frame = SplittedFrame::split_frames(
                        transformed_frame.as_slice(),
                        &mut splitted_frames,
                        width as i32,
                    ).unwrap();

                    let gpu_frames = GpuSplittedFrameInfo::from_splitted_frames(&splitted_frames);

                    let all_frames_x = FRAME_SPLITTER_ALL_FRAMES_X.load(Relaxed) as u32;
                    let all_frames_y = FRAME_SPLITTER_ALL_FRAMES_Y.load(Relaxed) as u32;
                    let mut output: Vec<i8> = Vec::new();

                    for _ in 0..all_frames_x * all_frames_y * 128 * 128{
                        output.push(0)
                    }

                    let out = output.as_mut_slice();

                    for idy in 0..height{
                        for idx in 0..width{
 
                            let r = vid[(idy * width * 3) as usize + (idx * 3) as usize];
                            let g = vid[(idy * width * 3) as usize + (idx * 3) as usize + 1];
                            let b = vid[(idy * width * 3) as usize + (idx * 3) as usize + 2];

                            let mut offset_xy = 0;
                            let mut i = 0;

                            'y: for y in 0..all_frames_y {
                                for _ in 0..all_frames_x {
                                    let info = &gpu_frames[i];
                                    let frame_height = info.height_end - info.height_start;
                                    let frame_width = info.width_end - info.width_start;


                                    if info.width_start <= idx && info.width_end > idx && info.height_start <= idy && info.height_end > idy {

                                        let x1 = idx - info.width_start;
                                        let y1 = idy - info.height_start;

                                        if offset_xy as usize + (y1 * frame_width) as usize + x1 as usize == 15360 {
                                            println!("D")
                                        }

                                        out[offset_xy as usize + (y1 * frame_width) as usize + x1 as usize] = colorlib::get_cached_index(MinecraftColor::new(r, g, b));
                                        break 'y;
                                    }
                    
                                    i += 1;
                                    // offset_xy += frame_height * frame_width;
                                    offset_xy += frame_height * frame_width;
                                }
                            }
                        }
                    }

                    println!("F");

                    let mut diff = 0;
                    let single = single_video_player::SingleVideoPlayer::create("/home/wolny/Downloads/4k_test.mp4".to_string())
                        .expect("Creation failed");
                    let single = single.wrap_to_ptr();
                    let frame = PlayerContext::load_frame(single).unwrap();
                    for i in 0..frame.len()-1 {
                        if out[i] != frame[i] {
                            diff += 1;
                            println!("O: {}, THE: {}", out[i], frame[i])
                        }
                    };

                    println!("D: {}", diff);
                    break;
                }
            }
        }
    }
}
