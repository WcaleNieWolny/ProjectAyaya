use std::mem::ManuallyDrop;
use ffmpeg::{Error, Packet};
use ffmpeg::frame::Video;
use ffmpeg::software::scaling::Context;
use crate::player::multi_video_player::MultiVideoPlayer;
use crate::player::player_context::PlayerType::SingleThreaded;
use crate::player::single_video_player::SingleVideoPlayer;

pub enum PlayerType{
    SingleThreaded,
    MultiThreaded
}
pub struct PlayerContext {
    player_type: PlayerType,
    ptr: i64 //Pointer
}

impl PlayerContext {
    pub fn new(player_type: PlayerType, ptr: i64) -> Self {
        Self {
            player_type,
            ptr
        }
    }

    pub fn from_single_video_player(single_video_player: SingleVideoPlayer) -> Self {
        Self {
            player_type: SingleThreaded,
            ptr: Box::into_raw(Box::new(single_video_player)) as i64
        }
    }

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
            },
            _multi_threaded => {
                panic!("Multithreaded video player is not implemented")
            }
        }
    }

    pub fn width(ptr: i64) -> i32 {
        let player_context = unsafe {
            ManuallyDrop::new(Box::from_raw(ptr as *mut PlayerContext))
        };
        match &player_context.player_type {
            SingleThreaded => {
                let mut single_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut SingleVideoPlayer))
                };
                return single_video_player.width as i32;
            }
            _multi_threaded => {
                panic!("Multithreaded video player is not implemented")
            }
        }
    }

    pub fn height(ptr: i64) -> i32 {
        let player_context = unsafe {
            ManuallyDrop::new(Box::from_raw(ptr as *mut PlayerContext))
        };
        match &player_context.player_type {
            SingleThreaded => {
                let mut single_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut SingleVideoPlayer))
                };
                return single_video_player.height as i32;
            }
            _multi_threaded => {
                panic!("Multithreaded video player is not implemented")
            }
        }
    }

    pub fn destroy(ptr: i64) -> anyhow::Result<()>{
        let player_context = unsafe {
            ManuallyDrop::new(Box::from_raw(ptr as *mut PlayerContext))
        };
        match &player_context.player_type {
            SingleThreaded => {
                let mut single_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut SingleVideoPlayer))
                };
                let single_video_player = ManuallyDrop::into_inner(single_video_player);
                single_video_player.destroy()?;
            }
            MultiThreaded => {
                let mut single_video_player = unsafe {
                    ManuallyDrop::new(Box::from_raw(player_context.ptr as *mut MultiVideoPlayer))
                };
                let single_video_player = ManuallyDrop::into_inner(single_video_player);
                single_video_player.destroy()?;
            }
        }
        Ok(())
    }
}

pub fn receive_and_process_decoded_frames(decoder: &mut ffmpeg::decoder::Video, scaler: &mut Context, packet: &Packet) ->  anyhow::Result<Video> {
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

pub trait VideoPlayer{
    fn create(file_name: String) -> anyhow::Result<PlayerContext>;
    fn init(&mut self) -> anyhow::Result<()>;
    fn load_frame(&mut self)-> anyhow::Result<Vec<i8>>;
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn destroy(self) -> anyhow::Result<()>; //Note: This should free any resources of the implementation. Also self is being moved to the destroy fn so it will be dropped without drop call
}