use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender, SyncSender};
use std::thread;
use ffmpeg::decoder::{Video};
use ffmpeg::Error;
use ffmpeg::Error::Eof;
use ffmpeg::format::{input, Pixel};
use ffmpeg::format::context::Input;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{oneshot};
use crate::{ffmpeg_set_multithreading, init, PlayerContext, VideoPlayer};
use crate::colorlib::transform_frame_to_mc;
use crate::player::player_context::receive_and_process_decoded_frames;

pub struct MultiVideoPlayer{
    pub width: Option<i32>,
    pub height: Option<i32>,
    thread_pool_size: i32,
    file_name: String,
    receiver: Option<Arc<Receiver<Vec<i8>>>>,
    runtime: Arc<Runtime>
}

impl MultiVideoPlayer {
    fn decode_frame(input: &mut Input, video_stream_index: usize, decoder: &mut Video, scaler: &mut Context) -> anyhow::Result<ffmpeg_next::util::frame::video::Video> {
        while let Some((stream, packet)) = input.packets().next() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                let frame_data = receive_and_process_decoded_frames(decoder, scaler, &packet)?;
                return Ok(frame_data)
            }
        };

        Err(anyhow::Error::new(Eof))
    }
}

impl VideoPlayer for MultiVideoPlayer{
    fn create(file_name: String) -> anyhow::Result<PlayerContext> {
        let thread_pool_size = 4;
        let runtime = Builder::new_multi_thread()
            .worker_threads(thread_pool_size as usize)
            .thread_name("ProjectAyaya native worker thread")
            .thread_stack_size(3840 as usize * 2160 as usize * 4) //Big stack due to memory heavy operations (4k is max resolution for now)
            .build()
            .expect("Couldn't create tokio runtime");

        let mut multi_video_player = MultiVideoPlayer {
            width: None,
            height: None,
            thread_pool_size,
            file_name,
            receiver: None,
            runtime: Arc::new(runtime)
        };

        multi_video_player.init().expect("Couldn't initialize multithreaded player");

        Ok(PlayerContext::from_multi_video_player(multi_video_player))
    }

    fn init(&mut self) -> anyhow::Result<()> {

        let (global_tx, global_rx) = mpsc::sync_channel::<Vec<i8>>(50);
        let (len_tx, len_rx) = mpsc::sync_channel::<i32>(2);

        self.receiver = Some(Arc::new(global_rx));

        let handle = self.runtime.handle().clone();

        let file_name = self.file_name.clone();
        let thread_pool_size = self.thread_pool_size.clone() - 1;

        thread::spawn(move || {
            if let Ok(mut ictx) = input(&file_name) {
                let input = ictx
                    .streams()
                    .best(Type::Video)
                    .ok_or(Error::StreamNotFound).expect("Couldn't create async video stream");

                let video_stream_index = input.index();

                let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters()).expect("Couldn't get context from parametrs async");
                let mut decoder = context_decoder.decoder();
                ffmpeg_set_multithreading(&mut decoder, file_name);

                let mut decoder = decoder.video().expect("Couldn't get async decoder");

                let width = decoder.width();
                let height = decoder.height();

                len_tx.send(width as i32).unwrap();
                len_tx.send(height as i32).unwrap();

                let mut scaler = Context::get(
                    decoder.format(),
                    width,
                    height,
                    Pixel::RGB24,
                    width,
                    height,
                    Flags::BILINEAR,
                ).expect("Couldn't get async scaler");

                let mut frames_channels: Vec<oneshot::Receiver<Vec<i8>>> = Vec::with_capacity((thread_pool_size + 1) as usize);

                for _ in 0..thread_pool_size {

                    println!("loop en");

                    let frame = MultiVideoPlayer::decode_frame(&mut ictx, video_stream_index, &mut decoder, &mut scaler).expect("Couldn't create async frame");

                    let (tx, rx) = oneshot::channel::<Vec<i8>>();
                    frames_channels.push(rx);

                    handle.spawn(async move {
                        let vec = transform_frame_to_mc(frame.data(0), width, height);
                        tx.send(vec)
                    });
                };

                loop {
                    for i in 0..thread_pool_size {
                        let frame = MultiVideoPlayer::decode_frame(&mut ictx, video_stream_index, &mut decoder, &mut scaler).expect("Couldn't create async frame");
                        let (tx, rx) = oneshot::channel::<Vec<i8>>();
                        frames_channels.push(rx);

                        handle.spawn(async move {
                            let vec = transform_frame_to_mc(frame.data(0), width, height);
                            tx.send(vec)
                        });

                        //Note: This is suboptimal behavior! We should have some NONE space in the vector that we will later replace with new chanel - TODO!

                        let rx = frames_channels.swap_remove(i as usize);
                        global_tx.send(rx.blocking_recv().unwrap()).expect("Couldn't send global async message");
                    }
                }

            }else {
                panic!("Couldn't create async video input")
            }
        });

        self.width = Some(len_rx.recv().unwrap());
        self.height = Some(len_rx.recv().unwrap());

        Ok(())

    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        Ok(self.receiver.as_ref().unwrap().recv().expect("Couldn't receive async global message"))
    }

    fn width(&self) -> i32 {
        self.width.unwrap()
    }

    fn height(&self) -> i32 {
        self.height.unwrap()
    }

    fn destroy(self) -> anyhow::Result<()> {
        let runtime = Arc::try_unwrap(self.runtime).expect("Couldn't get ownership to async runtime");
        runtime.shutdown_background();
        Ok(())
    }
}