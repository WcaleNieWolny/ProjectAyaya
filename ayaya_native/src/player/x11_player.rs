use std::sync::mpsc::channel;

use anyhow::anyhow;
use ffmpeg::decoder::Video;
use ffmpeg::format::context::Input;
use ffmpeg::format::Pixel;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::Error::Eof;
use ffmpeg::{rescale, Error, Format, Dictionary};

use crate::colorlib::{get_cached_index, Color};
use crate::map_server::ServerOptions;
use crate::player::player_context::{receive_and_process_decoded_frames, VideoData, VideoPlayer};
use crate::SplittedFrame;

use super::player_context::NativeCommunication;

pub struct X11Player {
    video_stream_index: usize,
    scaler: Context,
    input: Input,
    decoder: Video,
    splitted_frames: Vec<SplittedFrame>,
    width: u32,
    height: u32,
    fps: i32,
}

impl VideoPlayer for X11Player {
    fn create(_file_name: String, server_options: ServerOptions) -> anyhow::Result<Self> {
        if server_options.use_server {
            return Err(anyhow!("Single video player does not support map server"));
        }

        //https://docs.rs/ffmpeg-next/latest/ffmpeg_next/format/fn.register.html
        //We propably should call this however this breaks windows compilation!
        ffmpeg::init()?;

        if cfg!(not(target_os = "linux")){
            return Err(anyhow!("You are not running linux! X11 does not work outside of linux!")); 
        };

        let mut optional_format: Option<Format> = None;
        for format in ffmpeg::device::input::video() {
            if format.name() == "x11grab" {
                optional_format = Some(format);
                break;
            }
        }

        let format = match optional_format {
            Some(val) => val,
            None => return Err(anyhow!("Unable to find x11grab format! Reffer to the wiki for help"))
        };

        let mut dictionary = Dictionary::new();
        dictionary.set("framerate", "60"); //TODO: dynamic
        dictionary.set("video_size", "3440x1440");
        dictionary.set("probesize", "100M");

        if let Ok(conext) = ffmpeg::format::open_with(&":0.0", &format, dictionary){
            let ictx = conext.input();
            let input = ictx
                .streams()
                .best(Type::Video)
                .ok_or(Error::StreamNotFound)?;

            let video_stream_index = input.index();

            let context_decoder =
                ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
            let decoder = context_decoder.decoder();
            let decoder = decoder.video()?;

            let width = decoder.width();
            let height = decoder.height();

            let fps = input.rate().0 / input.rate().1;
            println!("WIDTH, HEIGHT, FPS: {} {} {}", width, height, fps);
            let scaler = Context::get(
                decoder.format(),
                width,
                height,
                Pixel::RGB24,
                width,
                height,
                Flags::BILINEAR,
            )?;

            let (seek_tx, seek_rx) = channel::<i32>();

            let single_video_player = Self {
                video_stream_index,
                scaler,
                input: ictx,
                decoder,
                splitted_frames: SplittedFrame::initialize_frames(width as i32, height as i32)?,
                width,
                height,
                fps,
            };

            return Ok(single_video_player);
        }

        Err(anyhow::Error::new(Error::StreamNotFound))
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        while let Some((stream, packet)) = self.input.packets().next() {
            if stream.index() == self.video_stream_index {
                self.decoder.send_packet(&packet)?;
                let frame_data = receive_and_process_decoded_frames(
                    &mut self.decoder,
                    &mut self.scaler,
                    &packet,
                )?;

                let transformed_frame =
                    Self::transform_frame_to_mc(frame_data.data(0), self.width, self.height, frame_data.stride(0));

                let transformed_frame = SplittedFrame::split_frames(
                    transformed_frame.as_slice(),
                    &self.splitted_frames,
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

    fn destroy(self: Box<Self>) -> anyhow::Result<()> {
        Ok(()) //Nothing to do
    }

    fn handle_jvm_msg(&self, msg: NativeCommunication) -> anyhow::Result<()> {
        return Err(anyhow!("X11 player does not support native messages!"));
    }
}

impl X11Player {
    //We need a custom implemenetaion due to ffmpeg not using the same linewidth as the width. It
    //makes the whole calculation wrong. That is why we have "add_width"
    pub fn transform_frame_to_mc(data: &[u8], width: u32, height: u32, add_width: usize) -> Vec<i8> {
        let mut buffer = Vec::<i8>::with_capacity((width * height) as usize);

        for y in 0..height as usize {
            for x in 0..width as usize {
                buffer.push(get_cached_index(&Color::new(
                    data[((y * add_width) + (x * 3)) as usize],
                    data[((y * add_width) + (x * 3) + 1) as usize],
                    data[((y * add_width) + (x * 3) + 2) as usize],
                )));
            }
        }

        buffer
    }
}
