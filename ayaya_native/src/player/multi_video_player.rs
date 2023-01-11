use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::TrySendError;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::thread;

use anyhow::anyhow;
use ffmpeg::decoder::Video;
use ffmpeg::format::context::Input;
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::Error::Eof;
use ffmpeg::{rescale, Error, Rescale};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{broadcast, oneshot};

use crate::colorlib::transform_frame_to_mc;
use crate::map_server::{MapServer, MapServerData, ServerOptions};
use crate::player::player_context::{receive_and_process_decoded_frames, VideoData};
use crate::{ffmpeg_set_multithreading, SplittedFrame, VideoPlayer};

use super::player_context::NativeCommunication;

pub struct MultiVideoPlayer {
    width: i32,
    height: i32,
    fps: i32,
    pub frame_index: Arc<AtomicI64>,
    receiver: Option<Arc<Mutex<tokio::sync::mpsc::Receiver<FrameWithIdentifier>>>>,
    map_server: MapServerData,
    stop_tx: oneshot::Sender<bool>,
    seek_tx: broadcast::Sender<i32>,
    seek_rx: broadcast::Receiver<i32>,
    runtime: Arc<Runtime>,
}

pub struct FrameWithIdentifier {
    pub id: i64,
    pub data: Vec<i8>,
}

//No data field
impl Debug for FrameWithIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameWithIdentifier")
            .field("id", &self.id)
            .finish()
    }
}

impl MultiVideoPlayer {
    pub fn decode_frame(
        input: &mut Input,
        video_stream_index: usize,
        decoder: &mut Video,
        scaler: &mut Context,
    ) -> anyhow::Result<ffmpeg_next::util::frame::video::Video> {
        while let Some((stream, packet)) = input.packets().next() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                let frame_data = receive_and_process_decoded_frames(decoder, scaler, &packet)?;
                return Ok(frame_data);
            }
        }

        Err(anyhow::Error::new(Eof))
    }
}

impl VideoPlayer for MultiVideoPlayer {
    fn create(file_name: String, map_server_options: ServerOptions) -> anyhow::Result<Self> {
        let thread_pool_size = 24;
        let runtime = Builder::new_multi_thread()
            .worker_threads(thread_pool_size as usize)
            .thread_name("ProjectAyaya native worker thread")
            .thread_stack_size(3840_usize * 2160_usize * 4) //Big stack due to memory heavy operations (4k is max resolution for now)
            .enable_io()
            .enable_time()
            .build()
            .expect("Couldn't create tokio runtime");

        let handle = runtime.handle().clone();
        let frame_index = Arc::new(AtomicI64::new(0));

        let (global_tx, global_rx) = tokio::sync::mpsc::channel::<FrameWithIdentifier>(100);
        let (data_tx, data_rx) = mpsc::sync_channel::<i32>(3);
        let (frames_tx, frames_rx) = mpsc::sync_channel::<FrameWithIdentifier>(100);

        let mut reciver: Option<Arc<Mutex<tokio::sync::mpsc::Receiver<FrameWithIdentifier>>>> =
            None;
        let (server_tx, server_rx) = oneshot::channel::<anyhow::Result<MapServerData>>();

        match map_server_options.use_server {
            true => {
                let frame_index_clone = frame_index.clone();
                handle.spawn(async move {
                    let result = MapServer::create(
                        &map_server_options.clone(),
                        frame_index_clone,
                        global_rx,
                    )
                    .await;
                    server_tx
                        .send(result)
                        .expect("Cannot send map server creation result");
                });
            }
            false => {
                reciver = Some(Arc::new(Mutex::new(global_rx)));
                server_tx.send(Ok(None)).unwrap();
            }
        };

        let map_server = server_rx.blocking_recv()??;
        let (processing_sleep_tx, processing_sleep_rx) = mpsc::sync_channel::<bool>(3);

        let (stop_tx, mut stop_rx) = oneshot::channel::<bool>();
        let (seek_tx, seek_rx) = broadcast::channel::<i32>(5);
        let mut seek_rx_clone = seek_rx.resubscribe();

        thread::spawn(move || {
            ffmpeg::init().expect("Couldn't init ffmpeg!");
            if let Ok(mut ictx) = input(&file_name) {
                let input = ictx
                    .streams()
                    .best(Type::Video)
                    .ok_or(Error::StreamNotFound)
                    .expect("Couldn't create async video stream");

                let video_stream_index = input.index();

                let context_decoder =
                    ffmpeg::codec::context::Context::from_parameters(input.parameters())
                        .expect("Couldn't get context from parametrs async");
                let mut decoder = context_decoder.decoder();
                ffmpeg_set_multithreading(&mut decoder, file_name);

                let mut decoder = decoder.video().expect("Couldn't get async decoder");

                let width = decoder.width();
                let height = decoder.height();

                data_tx.send(width as i32).unwrap();
                data_tx.send(height as i32).unwrap();
                data_tx.send(input.rate().0 / input.rate().1).unwrap();

                let splitted_frames = SplittedFrame::initialize_frames(width as i32, height as i32)
                    .expect("Couldn't initialize frame splitting");

                let mut scaler = Context::get(
                    decoder.format(),
                    width,
                    height,
                    Pixel::RGB24,
                    width,
                    height,
                    Flags::BILINEAR,
                )
                .expect("Couldn't get async scaler");

                let mut frame_id: i64 = 0;

                'main: loop {
                    if processing_sleep_rx.try_recv().is_ok() {
                        while processing_sleep_rx.try_recv().is_err() {
                            thread::sleep(Duration::from_millis(50));
                            if stop_rx.try_recv().is_ok() {
                                break 'main;
                            }
                        }
                    }

                    if stop_rx.try_recv().is_ok() {
                        break 'main;
                    }

                    if let Ok(position) = seek_rx_clone.try_recv() {
                        let position = position.rescale((1, 1), rescale::TIME_BASE);
                        if ictx.seek(position, ..position).is_err() {
                            println!("Cannot seek in async context! Quiting!");
                            break 'main;
                        }
                        //We do not flush the decoder due to some wierd bug that causes "End of
                        //file" when we do. It SHOULD be fine! (It causes a small lag but it is ok)
                        frame_id = 0;
                    }

                    let frame = MultiVideoPlayer::decode_frame(
                        &mut ictx,
                        video_stream_index,
                        &mut decoder,
                        &mut scaler,
                    )
                    .expect("Couldn't create async frame");
                    let splitted_frames = splitted_frames.clone();

                    let sender = frames_tx.clone();

                    handle.spawn(async move {
                        let vec = transform_frame_to_mc(frame.data(0), width, height);
                        let vec = SplittedFrame::split_frames(vec.as_slice(), &splitted_frames, width as i32).expect("Couldn't split frames async");

                        let frame_with_id = FrameWithIdentifier {
                            id: frame_id,
                            data: vec,
                        };

                        match sender.try_send(frame_with_id) {
                            Ok(_) => {}
                            Err(err) => {

                                //processing_sleep_tx_copy.clone().send(true).unwrap();
                                //should_break.store(true, Relaxed)
                                if matches!(err, TrySendError::Full(_)) {
                                    panic!("CRITICAL ERROR ACCURED!!! COULDN'T SEND DATA TO GLOBAL PROCESSING THREAD!! THIS IS NOT RECOVERABLE AS IT WILL RESULT IN MEM LEAK!!!")
                                }
                            }
                        }

                    });
                    frame_id += 1
                }
            } else {
                panic!("Couldn't create async video input")
            }
        });

        let frame_index_clone = frame_index.clone();

        thread::spawn(move || {
            let mut frame_hash_map: HashMap<u64, FrameWithIdentifier> = HashMap::new();
            let mut last_id: i64 = 0;

            loop {
                if last_id - frame_index.load(Relaxed) > 80 {
                    processing_sleep_tx
                        .send(true)
                        .expect("Couldnt send sleep request");
                    while last_id - frame_index.load(Relaxed) > 80 {
                        thread::sleep(Duration::from_millis(50))
                    }
                    processing_sleep_tx
                        .send(false)
                        .expect("Couldnt send sleep disable request");
                }

                let cached_frame = frame_hash_map.remove(&(last_id as u64 + 1_u64));

                if let Some(cached_frame) = cached_frame {
                    last_id = cached_frame.id;
                    global_tx
                        .blocking_send(cached_frame)
                        .expect("Couldn't send cached global frame");

                    continue;
                }

                let frame = frames_rx.recv().expect("Couldn't recive with identifier");

                if frame.id == last_id + 1 || frame.id == 0 {
                    if frame.id == 0 {
                        frame_hash_map.clear();
                    }

                    last_id = frame.id;
                    match global_tx.blocking_send(frame) {
                        Ok(_) => {}
                        Err(_) => {
                            println!("[AyayaNative] Couldn't send frame data! Exiting!");
                            break;
                        }
                    }
                    continue;
                }

                frame_hash_map.insert(frame.id as u64, frame);
            }
        });

        let width = data_rx.recv().unwrap();
        let height = data_rx.recv().unwrap();
        let fps = data_rx.recv().unwrap();

        let multi_video_player = MultiVideoPlayer {
            width,
            height,
            fps,
            frame_index: frame_index_clone,
            receiver: reciver,
            map_server,
            stop_tx,
            seek_tx,
            seek_rx,
            runtime: Arc::new(runtime),
        };
        Ok(multi_video_player)
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        if self.map_server.is_some() {
            return Err(anyhow!(
                "You cannot use JVM and native map server at the same time!"
            ));
        }

        let reciver = self.receiver.as_ref().unwrap();
        let mut reciver = match reciver.lock() {
            Ok(val) => val,
            Err(_) => return Err(anyhow!("Unable to lock JVM frame mutex")),
        };

        //Recive so we can do next frame normaly. is_empty does not recive
        if self.seek_rx.try_recv().is_ok() {
            'frame_recv_loop: while let Some(frame) = reciver.blocking_recv() {
                if frame.id != 0 {
                    self.frame_index
                        .store(self.frame_index.load(Relaxed) + 1, Relaxed);
                    continue 'frame_recv_loop;
                };
                self.frame_index.store(1, Relaxed);
                return Ok(frame.data);
            }
        } else {
            self.frame_index
                .store(self.frame_index.load(Relaxed) + 1, Relaxed);
            return match reciver.blocking_recv() {
                Some(frame) => Ok(frame.data),
                None => Err(anyhow!("JVM frame reciver closed!")),
            };
        }

        Err(anyhow!("Unable to recive JVM rame"))
    }

    fn video_data(&self) -> anyhow::Result<VideoData> {
        //let width = self.width.expect("Couldn't get multi video width");
        Ok(VideoData {
            width: self.width,
            height: self.height,
            fps: self.fps,
        })
    }

    fn destroy(self: Box<Self>) -> anyhow::Result<()> {
        match self.stop_tx.send(true) {
            Ok(_) => {}
            Err(_) => {
                return Err(anyhow!(
                    "Couldn't send stop tx signal! Unable to destroy native resources"
                ))
            }
        }

        let runtime = match Arc::try_unwrap(self.runtime) {
            Ok(val) => val,
            Err(_) => return Err(anyhow!("Unable to get ownership of async runtime")),
        };

        runtime.shutdown_background();
        Ok(())
    }

    fn handle_jvm_msg(&self, msg: NativeCommunication) -> anyhow::Result<()> {
        match &self.map_server {
            Some(server) => {
                let server = server.clone();
                server.send_message(msg)?;
            }
            None => match msg {
                NativeCommunication::VideoSeek { second } => {
                    self.seek_tx.send(second)?;
                }
                _ => return Err(anyhow!("Map server is not enabled!")),
            },
        }
        Ok(())
    }
}
