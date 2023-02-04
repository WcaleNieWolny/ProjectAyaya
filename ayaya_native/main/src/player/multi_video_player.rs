use std::collections::HashMap;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::TrySendError;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::anyhow;
use ffmpeg::decoder::Video;
use ffmpeg::format::context::Input;
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::Error::Eof;
use ffmpeg::{rescale, Error, Rescale};
use tokio::sync::oneshot::error::TryRecvError;
use tokio::sync::{broadcast, oneshot};

use crate::colorlib::transform_frame_to_mc;
use crate::map_server::{MapServer, MapServerData, ServerOptions};
use crate::player::player_context::{receive_and_process_decoded_frames, VideoData};
use crate::{ffmpeg_set_multithreading, SplittedFrame, VideoPlayer, TOKIO_RUNTIME};

use super::player_context::{FrameWithIdentifier, NativeCommunication};

pub struct MultiVideoPlayer {
    width: i32,
    height: i32,
    fps: i32,
    pub frame_index: Arc<AtomicI64>,
    receiver: Option<Arc<Mutex<tokio::sync::mpsc::Receiver<FrameWithIdentifier>>>>,
    map_server: MapServerData,
    #[allow(unused)]
    stop_tx: oneshot::Sender<bool>,
    seek_tx: broadcast::Sender<i32>,
    seek_rx: broadcast::Receiver<i32>,
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
        let handle = TOKIO_RUNTIME.handle().clone();
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

                let frame_initial_split =
                    SplittedFrame::initialize_frames(width as i32, height as i32)
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

                            match stop_rx.try_recv() {
                                Ok(_) => {}
                                Err(err) => {
                                    if matches!(err, TryRecvError::Closed) {
                                        break 'main;
                                    }
                                }
                            }
                        }
                    }

                    match stop_rx.try_recv() {
                        Ok(_) => {}
                        Err(err) => {
                            if matches!(err, TryRecvError::Closed) {
                                break 'main;
                            }
                        }
                    }

                    if let Ok(position) = seek_rx_clone.try_recv() {
                        let position = position.rescale((1, 1), rescale::TIME_BASE);
                        if ictx.seek(position, ..position).is_err() {
                            println!("Cannot seek in async context! Quiting!");
                            break 'main;
                        }
                        //We do not flush the decoder due to some wierd bug that causes "End of
                        //file" when we do. It SHOULD be fine! (It causes a small lag but it is ok)
                        //We start at zero so -1 is fine
                        frame_id = -1;
                    }

                    let frame = match MultiVideoPlayer::decode_frame(
                        &mut ictx,
                        video_stream_index,
                        &mut decoder,
                        &mut scaler,
                    ) {
                        Ok(val) => val,
                        Err(err) => {
                            if let Some(downcast) = err.downcast_ref::<ffmpeg::Error>() {
                                if matches!(downcast, ffmpeg::Error::Eof) {
                                    println!(
                                        "[ProjectAyaya] End of file (stream). This is normal!"
                                    );
                                    break 'main;
                                }
                            };
                            println!(
                                "[ProjectAyaya] Creating async frame failed! Reason: {:?}",
                                err
                            );
                            break 'main;
                        }
                    };

                    let (splitted_frames, all_frames_x, all_frames_y) = frame_initial_split.clone();

                    let sender = frames_tx.clone();

                    handle.spawn(async move {
                        let vec = transform_frame_to_mc(frame.data(0), width, height, frame.stride(0));
                        let vec = SplittedFrame::split_frames(vec.as_slice(), &splitted_frames, width as i32, all_frames_x, all_frames_y).expect("Couldn't split frames async");

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

            'decode_loop: loop {
                if last_id - frame_index.load(Relaxed) > 80 {
                    if let Err(_) = processing_sleep_tx.send(true) {
                        println!("[ProjectAyaya] Unable to send sleep request!");
                        break 'decode_loop;
                    }
                    while last_id - frame_index.load(Relaxed) > 80 {
                        thread::sleep(Duration::from_millis(50))
                    }
                    if let Err(_) = processing_sleep_tx.send(false) {
                        println!("[ProjectAyaya] Unable to send disable sleep request!");
                        break 'decode_loop;
                    }
                }

                let cached_frame = frame_hash_map.remove(&(last_id as u64 + 1_u64));

                if let Some(cached_frame) = cached_frame {
                    last_id += 1;
                    global_tx
                        .blocking_send(cached_frame)
                        .expect("Couldn't send cached global frame");

                    continue;
                }

                let frame = match frames_rx.recv() {
                    Ok(val) => val,
                    Err(err) => {
                        println!(
                            "[ProjectAyaya] Unable to recive frames with identifier! Cache Size: {:?}  {} Error: {:?}",
                            frame_hash_map.len(),
                            last_id,
                            err
                        );

                        break 'decode_loop;
                    }
                };

                //We have a single dropped frame somewhere here but I don't care enough to fix this
                if frame.id == last_id + 1 || frame.id == -1 {
                    if frame.id == -1 {
                        frame_hash_map.clear();
                        last_id = -1;
                    } else {
                        last_id += 1;
                    }

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
                if frame.id != -1 {
                    self.frame_index
                        .store(self.frame_index.load(Relaxed) + 1, Relaxed);
                    continue 'frame_recv_loop;
                };
                self.frame_index.store(0, Relaxed);
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

    fn destroy(&self) -> anyhow::Result<()> {
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
