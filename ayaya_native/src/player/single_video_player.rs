use ffmpeg::decoder::Video;
use ffmpeg::format::context::Input;
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::Error;
use ffmpeg::Error::Eof;

use crate::colorlib::transform_frame_to_mc;
use crate::player::player_context::{
    receive_and_process_decoded_frames, PlayerContext, VideoData, VideoPlayer,
};
use crate::{ffmpeg_set_multithreading, SplittedFrame};

pub struct SingleVideoPlayer {
    video_stream_index: usize,
    scaler: Context,
    input: Input,
    decoder: Video,
    splitted_frames: Vec<SplittedFrame>,
    width: u32,
    height: u32,
    fps: i32,
}

impl VideoPlayer for SingleVideoPlayer {
    fn create(file_name: String) -> anyhow::Result<PlayerContext> {
        ffmpeg::init()?;

        if let Ok(ictx) = input(&file_name) {
            let input = ictx
                .streams()
                .best(Type::Video)
                .ok_or(Error::StreamNotFound)?;

            let video_stream_index = input.index();

            let context_decoder =
                ffmpeg::codec::context::Context::from_parameters(input.parameters())?;

            let mut decoder = context_decoder.decoder();
            ffmpeg_set_multithreading(&mut decoder, file_name);

            let decoder = decoder.video()?;

            let width = decoder.width();
            let height = decoder.height();

            let fps = input.rate().0 / input.rate().1;

            let scaler = Context::get(
                decoder.format(),
                width,
                height,
                Pixel::RGB24,
                width,
                height,
                Flags::BILINEAR,
            )?;

            let mut single_video_player = Self {
                video_stream_index,
                scaler,
                input: ictx,
                decoder,
                splitted_frames: SplittedFrame::initialize_frames(width as i32, height as i32)?,
                width,
                height,
                fps,
            };

            SingleVideoPlayer::init(&mut single_video_player)?;

            return Ok(PlayerContext::from_single_video_player(single_video_player));
        }

        Err(anyhow::Error::new(Error::StreamNotFound))
    }

    fn init(&mut self) -> anyhow::Result<()> {
        //Do nothing - we do not to initialize a single threaded video player (This fn is mainly for multi threaded player)
        Ok(())
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        while let Some((stream, packet)) = self.input.packets().next() {
            if stream.index() == self.video_stream_index as usize {
                self.decoder.send_packet(&packet)?;
                let frame_data = receive_and_process_decoded_frames(
                    &mut self.decoder,
                    &mut self.scaler,
                    &packet,
                )?;
                let transformed_frame =
                    transform_frame_to_mc(frame_data.data(0), self.width, self.height);

                let transformed_frame = SplittedFrame::split_frames(
                    transformed_frame.as_slice(),
                    &mut self.splitted_frames,
                    self.width as i32,
                )?;

                return Ok(transformed_frame);
            }
        }

        Err(anyhow::Error::new(Eof))
    }

    fn video_data(&self) -> anyhow::Result<VideoData> {
        Ok(VideoData {
            width: self.width as i32,
            height: self.height as i32,
            fps: self.fps,
        })
    }

    fn destroy(self) -> anyhow::Result<()> {
        todo!()
    }
}
