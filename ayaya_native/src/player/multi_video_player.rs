use std::sync::Arc;
use ffmpeg::decoder::{Video};
use ffmpeg::Error;
use ffmpeg::Error::Eof;
use ffmpeg::format::{input, Pixel};
use ffmpeg::format::context::Input;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use tokio::runtime::{Runtime};
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::{Receiver, Sender};
use crate::{ffmpeg_set_multithreading, PlayerContext, VideoPlayer};
use crate::player::player_context::receive_and_process_decoded_frames;

pub struct MultiVideoPlayer{
    pub width: i32,
    pub height: i32,
    thread_pool_size: i32,
    file_name: String,
    chanel: Option<(Sender<Vec<i8>>, Receiver<Vec<i8>>)>,
    runtime: Arc<Runtime>
}

impl MultiVideoPlayer {
    fn decode_frame(mut input: Input, video_stream_index: usize, mut decoder: Video, mut scaler: Context) -> anyhow::Result<&[u8]> {
        while let Some((stream, packet)) = input.packets().next() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                let frame_data = receive_and_process_decoded_frames(&mut decoder, &mut scaler, &packet)?;
                return Ok(frame_data.data(0))
            }
        };

        Err(anyhow::Error::new(Eof))
    }
}

impl VideoPlayer for MultiVideoPlayer{
    fn create(file_name: String) -> anyhow::Result<PlayerContext> {
        todo!()
    }

    fn init(&mut self) -> anyhow::Result<()> {

        let (tx, rx) = mpsc::channel::<Vec<i8>>(50);
        self.chanel = Some((tx, rx));

        let file_name = self.file_name.clone();

        self.runtime.spawn(async move {
            if let Ok(ictx) = input(&file_name) {
                let input = ictx
                    .streams()
                    .best(Type::Video)
                    .ok_or(Error::StreamNotFound).expect("Couldn't create async video stream");

                let video_stream_index = input.index();

                let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters()).expect("Couldn't get context from parametrs async");
                let mut decoder = context_decoder.decoder();
                ffmpeg_set_multithreading(&mut decoder, file_name);

                let decoder = decoder.video().expect("Couldn't get async decoder");

                let width = decoder.width();
                let height = decoder.height();

                let scaler = Context::get(
                    decoder.format(),
                    width,
                    height,
                    Pixel::RGB24,
                    width,
                    height,
                    Flags::BILINEAR,
                ).expect("Couldn't get async scaler");

                let mut frames_channels: Vec<oneshot::Receiver<Vec<i8>>> = Vec::new();

                for _ in 0..self.thread_pool_size {

                    println!("loop en");

                    let frame = receive_and_process_decoded_frames(decoder, scaler, )

                    println!("DEC");

                    let width = self.width;
                    let height = self.height;

                    let (tx, rx) = oneshot::channel::<Vec<i8>>();
                    frames_channels.push(rx);


                    tokio::spawn(async move {
                        let vec = transform_frame_to_mc(frame.data(0), width, height);
                        tx.send(vec)
                    });
                };


            }else {
                panic!("Couldn't create async video input")
            }
        });

        Ok(())

    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        todo!()
    }

    fn width(&self) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        self.height
    }

    fn destroy(self) -> anyhow::Result<()> {
        self.runtime.shutdown_background();
        Ok(())
    }
}