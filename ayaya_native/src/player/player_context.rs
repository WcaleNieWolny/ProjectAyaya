use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use ffmpeg::frame::Video;
use ffmpeg::software::scaling::Context;
use ffmpeg::{Error, Packet};

use crate::map_server::ServerOptions;

macro_rules! get_dyn_mutex {
    (
        $MUTEX: ident
    ) => {
        match $MUTEX.lock() {
            Ok(val) => val,
            Err(_) => return Err(anyhow!("Cannot lock mutex!")),
        }
    };
}

pub struct PlayerContext {
    player: Arc<Mutex<Box<dyn VideoPlayer>>>, //Pointer
}

pub struct VideoData {
    pub width: i32,
    pub height: i32,
    pub fps: i32,
}

#[derive(Debug, PartialEq)]
pub enum NativeCommunication {
    StartRendering { fps: i32 },
    StopRendering,
}

impl PlayerContext {
    pub fn from_player<T>(player: T) -> Self
    where
        T: VideoPlayer + 'static,
    {
        Self {
            player: Arc::new(Mutex::new(Box::new(player))),
        }
    }

    pub fn wrap_to_ptr(self) -> i64 {
        Box::into_raw(Box::new(self)) as i64
    }

    pub fn load_frame(ptr: i64) -> anyhow::Result<Vec<i8>> {
        let player_context = unsafe { ManuallyDrop::new(Box::from_raw(ptr as *mut PlayerContext)) };

        let player_context = player_context.player.clone();
        let mut player_context = get_dyn_mutex!(player_context);
        player_context.load_frame()
    }

    pub fn video_data(ptr: i64) -> anyhow::Result<VideoData> {
        let player_context = unsafe { ManuallyDrop::new(Box::from_raw(ptr as *mut PlayerContext)) };

        let player_context = player_context.player.clone();
        let player_context = get_dyn_mutex!(player_context);
        player_context.video_data()
    }

    pub fn pass_jvm_msg(ptr: i64, msg: NativeCommunication) -> anyhow::Result<()> {
        let player_context = unsafe { ManuallyDrop::new(Box::from_raw(ptr as *mut PlayerContext)) };

        let player_context = player_context.player.clone();
        let player_context = get_dyn_mutex!(player_context);
        player_context.handle_jvm_msg(msg)
    }

    pub fn destroy(ptr: i64) -> anyhow::Result<()> {
        let player_context = unsafe { ManuallyDrop::new(Box::from_raw(ptr as *mut PlayerContext)) };
        let player_context = ManuallyDrop::into_inner(player_context);

        let player_context = match Arc::try_unwrap(player_context.player) {
            Ok(val) => val,
            Err(_) => {
                return Err(anyhow!(
                    "Unable to get the inner value of arc! Not dropping!"
                ))
            }
        };

        let player_context = match player_context.into_inner() {
            Ok(val) => val,
            Err(_) => return Err(anyhow!("Coudln't get inner value of mutex! Not dropping!")),
        };

        VideoPlayer::destroy(player_context)
    }
}

pub fn receive_and_process_decoded_frames(
    decoder: &mut ffmpeg::decoder::Video,
    scaler: &mut Context,
    packet: &Packet,
) -> anyhow::Result<Video> {
    let mut decoded = Video::empty();
    let mut rgb_frame = Video::empty();

    let mut out = decoder.receive_frame(&mut decoded);

    while !out.is_ok() {
        let err = out.unwrap_err();

        if err == Error::from(-11) {
            decoder
                .send_packet(packet)
                .expect("Couldn't send packet to decoder");
            out = decoder.receive_frame(&mut decoded);
        } else {
            return Err(anyhow::Error::from(err));
        }
    }

    scaler
        .run(&decoded, &mut rgb_frame)
        .expect("Scaler run failed");
    return Ok(rgb_frame);
}

pub trait VideoPlayer {
    fn create(file_name: String, server_options: ServerOptions) -> anyhow::Result<PlayerContext>
    where
        Self: Sized;
    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>>;
    fn video_data(&self) -> anyhow::Result<VideoData>;
    fn handle_jvm_msg(&self, msg: NativeCommunication) -> anyhow::Result<()>;
    fn destroy(self: Box<Self>) -> anyhow::Result<()>;
    //Note: This should free any resources of the implementation. Also self is being moved to the destroy fn so it will be dropped without drop call
}
