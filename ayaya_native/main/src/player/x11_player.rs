#[cfg(feature = "ffmpeg")]
use std::sync::atomic::AtomicI64;
use std::sync::Arc;
use std::thread;

use anyhow::anyhow;
use ffmpeg::format::Pixel;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::{Dictionary, Error, Format};
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot;

use crate::colorlib::{get_cached_index, Color, self};
use crate::map_server::{MapServer, MapServerData, ServerOptions};
use crate::player::player_context::{receive_and_process_decoded_frames, VideoData, VideoPlayer};
use crate::{SplittedFrame, TOKIO_RUNTIME};

use super::player_context::{FrameWithIdentifier, NativeCommunication};

//av_log_set_callback could be used for passing messages (FFmpeg -> JVM)
//We store runtime so it does not get dropped
#[allow(dead_code)]
pub struct X11Player {
    width: u32,
    height: u32,
    fps: i32,
    jvm_rx: Option<Receiver<FrameWithIdentifier>>,
    map_server: MapServerData,
}

impl VideoPlayer for X11Player {
    fn create(input_string: String, map_server_options: ServerOptions) -> anyhow::Result<Self> {
        //https://docs.rs/ffmpeg-next/latest/ffmpeg_next/format/fn.register.html
        //We propably should call this however this breaks windows compilation!
        ffmpeg::init()?;

        if cfg!(not(target_os = "linux")) {
            return Err(anyhow!(
                "You are not running linux! X11 does not work outside of linux!"
            ));
        };

        let input_vec: Vec<&str> = input_string.split('@').collect();

        if input_vec.len() < 2 {
            return Err(anyhow!("Invalid input string for X11 capture"));
        }

        let mut optional_format: Option<Format> = None;
        for format in ffmpeg::device::input::video() {
            if format.name() == "x11grab" {
                optional_format = Some(format);
                break;
            }
        }

        let format = match optional_format {
            Some(val) => val,
            None => {
                return Err(anyhow!(
                    "Unable to find x11grab format! Reffer to the wiki for help"
                ))
            }
        };

        let mut dictionary = Dictionary::new();
        dictionary.set("framerate", input_vec[1]); //TODO: dynamic
        dictionary.set("video_size", input_vec[0]);
        dictionary.set("probesize", "100M");

        if let Ok(conext) = ffmpeg::format::open_with(&":0.0", &format, dictionary) {
            let mut ictx = conext.input();
            let input = ictx
                .streams()
                .best(Type::Video)
                .ok_or(Error::StreamNotFound)?;

            let video_stream_index = input.index();

            let context_decoder =
                ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
            let decoder = context_decoder.decoder();
            let mut decoder = decoder.video()?;

            let width = decoder.width();
            let height = decoder.height();

            let fps = input.rate().0 / input.rate().1;
            let splitted_frames = SplittedFrame::initialize_frames(width as i32, height as i32)?;

            //Small buffer due to fact that we are only decoding UP TO FPS frames per second
            let (jvm_tx, jvm_rx) = tokio::sync::mpsc::channel::<FrameWithIdentifier>(50);

            let (server_tx, server_rx) = oneshot::channel::<anyhow::Result<MapServerData>>();
            let mut jvm_final_reciver: Option<Receiver<FrameWithIdentifier>> = None;

            match map_server_options.use_server {
                true => {
                    let frame_index_clone = Arc::new(AtomicI64::new(0));

                    let handle = TOKIO_RUNTIME.handle().clone();
                    handle.spawn(async move {
                        let result = MapServer::create(
                            &map_server_options.clone(),
                            frame_index_clone,
                            jvm_rx,
                        )
                        .await;
                        server_tx
                            .send(result)
                            .expect("Cannot send map server creation result");
                    });
                }
                false => {
                    jvm_final_reciver = Some(jvm_rx);
                    server_tx.send(Ok(None)).unwrap();
                }
            };

            let map_server = server_rx.blocking_recv()?;
            let map_server = map_server?;

            //Threading is not a speed optymalisation. It is required to have support for map_server
            thread::Builder::new()
                .name("X11 screen graber thread".to_string())
                .spawn(move || {
                    let mut scaler = match Context::get(
                        decoder.format(),
                        width,
                        height,
                        Pixel::RGB24,
                        width,
                        height,
                        Flags::BILINEAR,
                    ) {
                        Ok(val) => val,
                        Err(err) => {
                            println!("Cannot create scaler! (X11) ({err:?})");
                            return;
                        }
                    };

                    'decoder_loop: loop {
                        let mut frame_id = 0;
                        while let Some((stream, packet)) = ictx.packets().next() {
                            if stream.index() == video_stream_index {
                                if let Err(err) = decoder.send_packet(&packet) {
                                    println!("Unable to send packets! (X11) ({err:?})");
                                };
                                let frame_data = match receive_and_process_decoded_frames(
                                    &mut decoder,
                                    &mut scaler,
                                    &packet,
                                ) {
                                    Ok(val) => val,
                                    Err(err) => {
                                        println!("Unable to decode frame (X11) ({err:?})");
                                        break 'decoder_loop;
                                    }
                                };

                                let transformed_frame = colorlib::transform_frame_to_mc(
                                    frame_data.data(0),
                                    width,
                                    height,
                                    frame_data.stride(0),
                                );

                                let transformed_frame = match SplittedFrame::split_frames(
                                    transformed_frame.as_slice(),
                                    &splitted_frames,
                                    width as i32,
                                ) {
                                    Ok(val) => val,
                                    Err(err) => {
                                        println!("Unable to split frames async (X11) ({err:?})");
                                        break 'decoder_loop;
                                    }
                                };

                                if jvm_tx
                                    .blocking_send(FrameWithIdentifier {
                                        id: frame_id,
                                        data: transformed_frame,
                                    })
                                    .is_err()
                                {
                                    //This is designed to fail
                                    println!("Unable to send JVM frame (X11) This is normal");
                                    break 'decoder_loop;
                                }
                                frame_id += 1;
                            }
                        }
                    }
                })?;

            let single_video_player = Self {
                width,
                height,
                fps,
                jvm_rx: jvm_final_reciver,
                map_server,
            };

            return Ok(single_video_player);
        }

        Err(anyhow::Error::new(Error::StreamNotFound))
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        if let Some(jvm_rx) = &mut self.jvm_rx {
            match jvm_rx.blocking_recv() {
                Some(val) => Ok(val.data),
                None => return Err(anyhow!("")),
            }
        } else {
            return Err(anyhow!(
                "Unable to use map_server and jvm reviver at the same time"
            ));
        }
    }

    fn video_data(&self) -> anyhow::Result<VideoData> {
        Ok(VideoData {
            width: self.width as i32,
            height: self.height as i32,
            fps: self.fps,
        })
    }

    fn destroy(&self) -> anyhow::Result<()> {
        Ok(()) //Nothing to do
    }

    fn handle_jvm_msg(&self, msg: NativeCommunication) -> anyhow::Result<()> {
        match &self.map_server {
            Some(server) => {
                let server = server.clone();
                server.send_message(msg)?;
            }
            None => {
                return Err(anyhow!("X11 player does not support native messages!"));
            }
        }
        Ok(())
    }
}
