use std::thread;

use anyhow::anyhow;
use ffmpeg::format::Pixel;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::{Dictionary, Error, Format};
use tokio::sync::mpsc::Receiver;

use crate::colorlib::{get_cached_index, Color};
use crate::map_server::ServerOptions;
use crate::player::player_context::{receive_and_process_decoded_frames, VideoData, VideoPlayer};
use crate::SplittedFrame;

use super::player_context::NativeCommunication;

pub struct X11Player {
    width: u32,
    height: u32,
    fps: i32,
    jvm_rx: Option<Receiver<Vec<i8>>>,
}

impl VideoPlayer for X11Player {
    fn create(_file_name: String, server_options: ServerOptions) -> anyhow::Result<Self> {
        if server_options.use_server {
            return Err(anyhow!("Single video player does not support map server"));
        }

        //https://docs.rs/ffmpeg-next/latest/ffmpeg_next/format/fn.register.html
        //We propably should call this however this breaks windows compilation!
        ffmpeg::init()?;

        if cfg!(not(target_os = "linux")) {
            return Err(anyhow!(
                "You are not running linux! X11 does not work outside of linux!"
            ));
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
            None => {
                return Err(anyhow!(
                    "Unable to find x11grab format! Reffer to the wiki for help"
                ))
            }
        };

        let mut dictionary = Dictionary::new();
        dictionary.set("framerate", "10"); //TODO: dynamic
        dictionary.set("video_size", "3440x1440");
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
            let (jvm_tx, jvm_rx) = tokio::sync::mpsc::channel::<Vec<i8>>(50);

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

                                let transformed_frame = Self::transform_frame_to_mc(
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

                                if let Err(err) = jvm_tx.blocking_send(transformed_frame) {
                                    println!("Unable to send JVM frame (X11) ({err:?})");
                                    break 'decoder_loop;
                                }
                            }
                        }
                    }
                })?;

            let single_video_player = Self {
                width,
                height,
                fps,
                jvm_rx: Some(jvm_rx),
            };

            return Ok(single_video_player);
        }

        Err(anyhow::Error::new(Error::StreamNotFound))
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        if let Some(jvm_rx) = &mut self.jvm_rx {
            match jvm_rx.blocking_recv() {
                Some(val) => Ok(val),
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

    fn destroy(self: Box<Self>) -> anyhow::Result<()> {
        Ok(()) //Nothing to do
    }

    fn handle_jvm_msg(&self, _msg: NativeCommunication) -> anyhow::Result<()> {
        return Err(anyhow!("X11 player does not support native messages!"));
    }
}

impl X11Player {
    //We need a custom implemenetaion due to ffmpeg not using the same linewidth as the width. It
    //makes the whole calculation wrong. That is why we have "add_width"
    pub fn transform_frame_to_mc(
        data: &[u8],
        width: u32,
        height: u32,
        add_width: usize,
    ) -> Vec<i8> {
        let mut buffer = Vec::<i8>::with_capacity((width * height) as usize);

        for y in 0..height as usize {
            for x in 0..width as usize {
                buffer.push(get_cached_index(&Color::new(
                    data[((y * add_width) + (x * 3))],
                    data[((y * add_width) + (x * 3) + 1)],
                    data[((y * add_width) + (x * 3) + 2)],
                )));
            }
        }

        buffer
    }
}
