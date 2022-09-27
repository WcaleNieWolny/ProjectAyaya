use std::mem::ManuallyDrop;

use ffmpeg::{Error, Packet};
use ffmpeg::frame::Video;
use ffmpeg::software::scaling::Context;

use crate::player::multi_video_player::MultiVideoPlayer;
use crate::player::player_context::PlayerType::{Gpu, MultiThreaded, SingleThreaded};
use crate::player::single_video_player::SingleVideoPlayer;
use crate::player::gpu_player::GpuVideoPlayer;

pub enum PlayerType {
    SingleThreaded,
    MultiThreaded,
    Gpu
}

pub struct PlayerContext {
    player_type: PlayerType,
    ptr: i64, //Pointer
}

pub struct VideoData {
    pub width: i32,
    pub height: i32,
    pub fps: i32,
}

impl PlayerContext {
    pub fn from_single_video_player(single_video_player: SingleVideoPlayer) -> Self {
        Self {
            player_type: SingleThreaded,
            ptr: Box::into_raw(Box::new(single_video_player)) as i64,
        }
    }

    pub fn from_multi_video_player(multi_video_player: MultiVideoPlayer) -> Self {
        Self {
            player_type: MultiThreaded,
            ptr: Box::into_raw(Box::new(multi_video_player)) as i64,
        }
    }

    pub fn from_gpu_video_player(gpu_video_player: GpuVideoPlayer) -> Self {
        Self {
            player_type: Gpu,
            ptr: Box::into_raw(Box::new(gpu_video_player)) as i64,
        }
    }

    // pub fn post_creation(&mut self) -> anyhow::Result<()>{
    //     let video_data = PlayerContext::video_data(self.ptr)?;
    //     
    //     let initialized_data = SplittedFrame::initialize_frames(video_data.width, video_data.height)?;
    //     self.splitter_frames.extend_from_slice(initialized_data.as_slice());
    //     
    //     Ok(())
    // }

    pub fn wrap_to_ptr(self) -> i64 {
        Box::into_raw(Box::new(self)) as i64
    }

    pub fn load_frame(ptr: i64) -> anyhow::Result<Vec<i8>> {
        let player_context = unsafe {
            ManuallyDrop::new(Box::from_raw(ptr as *mut PlayerContext))
        };

        match &player_context.player_type {
            SingleThreaded => {
                let mut single_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut SingleVideoPlayer))
                };

                single_video_player.load_frame()
            }
            MultiThreaded => {
                let mut multi_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut MultiVideoPlayer))
                };

                multi_video_player.load_frame()
            }
            Gpu => {
                let mut gpu_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut GpuVideoPlayer))
                };
                gpu_video_player.load_frame()
            }
        }
    }

    pub fn video_data(ptr: i64) -> anyhow::Result<VideoData> {
        let player_context = unsafe {
            ManuallyDrop::new(Box::from_raw(ptr as *mut PlayerContext))
        };
        match &player_context.player_type {
            SingleThreaded => {
                let single_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut SingleVideoPlayer))
                };
                return single_video_player.video_data();
            }
            MultiThreaded => {
                let multi_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut MultiVideoPlayer))
                };

                multi_video_player.video_data()
            }
            Gpu => {
                let gpu_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut GpuVideoPlayer))
                };
                
                gpu_video_player.video_data()
            }
        }
    }

    pub fn destroy(ptr: i64) -> anyhow::Result<()> {
        let player_context = unsafe {
            ManuallyDrop::new(Box::from_raw(ptr as *mut PlayerContext))
        };
        match &player_context.player_type {
            SingleThreaded => {
                let single_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut SingleVideoPlayer))
                };
                let single_video_player = ManuallyDrop::into_inner(single_video_player);
                single_video_player.destroy()?;
            }
            MultiThreaded => {
                let single_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut MultiVideoPlayer))
                };
                let single_video_player = ManuallyDrop::into_inner(single_video_player);
                single_video_player.destroy()?;
            }
            Gpu => {
                let gpu_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut GpuVideoPlayer))
                };
                let gpu_video_player = ManuallyDrop::into_inner(gpu_video_player);
                gpu_video_player.destroy()?;
            }
        }
        Ok(())
    }
}

pub fn receive_and_process_decoded_frames(decoder: &mut ffmpeg::decoder::Video, scaler: &mut Context, packet: &Packet) -> anyhow::Result<Video> {
    let mut decoded = Video::empty();
    let mut rgb_frame = Video::empty();

    let mut out = decoder.receive_frame(&mut decoded);

    while !out.is_ok() {
        let err = out.unwrap_err();

        if err == Error::from(-11) {
            decoder.send_packet(packet).expect("Couldn't send packet to decoder");
            out = decoder.receive_frame(&mut decoded);
        } else {
            return Err(anyhow::Error::from(err));
        }
    }

    scaler.run(&decoded, &mut rgb_frame).expect("Scaler run failed");
    return Ok(rgb_frame);
}

pub trait VideoPlayer {
    fn create(file_name: String) -> anyhow::Result<PlayerContext>;
    fn init(&mut self) -> anyhow::Result<()>;
    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>>;
    fn video_data(&self) -> anyhow::Result<VideoData>;
    fn destroy(self) -> anyhow::Result<()>; //Note: This should free any resources of the implementation. Also self is being moved to the destroy fn so it will be dropped without drop call
}