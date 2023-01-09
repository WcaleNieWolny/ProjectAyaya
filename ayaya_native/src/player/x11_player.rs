use std::sync::mpsc::{channel, Receiver, Sender};

use anyhow::anyhow;
use ffmpeg::decoder::Video;
use ffmpeg::format::context::Input;
use ffmpeg::format::{input, Pixel, format};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::Error::Eof;
use ffmpeg::{rescale, Error, Rescale, Format, dictionary, Dictionary};

use crate::colorlib::transform_frame_to_mc;
use crate::map_server::ServerOptions;
use crate::player::player_context::{receive_and_process_decoded_frames, VideoData, VideoPlayer};
use crate::{ffmpeg_set_multithreading, SplittedFrame};

use super::player_context::NativeCommunication;

pub struct X11Player {
    video_stream_index: usize,
    scaler: Context,
    input: Input,
    decoder: Video,
    splitted_frames: Vec<SplittedFrame>,
    seek_tx: Sender<i32>,
    seek_rx: Receiver<i32>,
    width: u32,
    height: u32,
    fps: i32,
}

impl VideoPlayer for X11Player {
    fn create(_file_name: String, server_options: ServerOptions) -> anyhow::Result<Self> {
        if server_options.use_server {
            return Err(anyhow!("Single video player does not support map server"));
        }

        ffmpeg::init()?;
        ffmpeg::format::register_all();

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
        dictionary.set("framerate", "15"); //TODO: dynamic
        dictionary.set("video_size", "3440x1440");
        dictionary.set("input_format", "h264");
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
            let mut decoder = context_decoder.decoder();
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

            let (seek_tx, seek_rx) = channel::<i32>();

            let single_video_player = Self {
                video_stream_index,
                scaler,
                input: ictx,
                decoder,
                splitted_frames: SplittedFrame::initialize_frames(width as i32, height as i32)?,
                seek_tx,
                seek_rx,
                width,
                height,
                fps,
            };

            return Ok(single_video_player);
        }

        Err(anyhow::Error::new(Error::StreamNotFound))
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        while let Ok(position) = &self.seek_rx.try_recv() {
            let position = position.rescale((1, 1), rescale::TIME_BASE);
            self.input.seek(position, ..position)?;
            self.decoder.flush();
        }

        while let Some((stream, packet)) = self.input.packets().next() {
            if stream.index() == self.video_stream_index {
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
        match msg {
            NativeCommunication::VideoSeek { second } => {
                self.seek_tx.send(second)?;
            }
            _ => return Err(anyhow!("Expected VideoSeek msg")),
        };
        Ok(())
    }
}
