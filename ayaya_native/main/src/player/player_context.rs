use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use anyhow::anyhow;

#[cfg(feature = "ffmpeg")]
use {
    ffmpeg::frame::Video,
    ffmpeg::software::scaling::Context,
    ffmpeg::{Error, Packet},
};

use crate::map_server::ServerOptions;

use super::game_player::GameInputDirection;

macro_rules! get_context {
    (
        $PTR: ident
    ) => {{
        let arc_ptr = $PTR as *const () as *const Arc<Mutex<dyn VideoPlayer>>;
        Arc::clone(unsafe { &*arc_ptr })
    }};
}

macro_rules! lock_mutex {
    (
        $MUTEX: ident
    ) => {
        match $MUTEX.lock() {
            Ok(val) => val,
            Err(_) => return Err(anyhow!("Cannot lock arc!")),
        }
    };
}

pub struct VideoData {
    pub width: i32,
    pub height: i32,
    pub fps: i32,
}

#[derive(Debug)]
pub enum NativeCommunication {
    StartRendering { fps: i32 },
    StopRendering,
    GameInput { input: Vec<GameInputDirection> },
    VideoSeek { second: i32 },
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

//Thanks to https://github.com/alexschrod for helping me with getting this Arc pointer to work
//He made this code much better!
pub fn wrap_to_ptr<T>(to_wrap: T) -> i64
where
    T: VideoPlayer,
{
    let arc = Arc::new(Mutex::new(to_wrap)) as Arc<Mutex<dyn VideoPlayer>>;
    Box::into_raw(Box::new(arc)) as *const () as i64
}

pub fn load_frame(ptr: i64) -> anyhow::Result<Vec<i8>> {
    let player_context = get_context!(ptr);
    let mut player_context = lock_mutex!(player_context);

    player_context.load_frame()
}

pub fn video_data(ptr: i64) -> anyhow::Result<VideoData> {
    let player_context = get_context!(ptr);
    let player_context = lock_mutex!(player_context);

    player_context.video_data()
}

pub fn pass_jvm_msg(ptr: i64, msg: NativeCommunication) -> anyhow::Result<()> {
    let player_context = get_context!(ptr);
    let player_context = lock_mutex!(player_context);

    player_context.handle_jvm_msg(msg)
}

pub fn destroy(ptr: i64) -> anyhow::Result<()> {
    drop(unsafe { Box::from_raw(ptr as *mut Arc<Mutex<dyn VideoPlayer>>) });

    Ok(())
}

#[cfg(feature = "ffmpeg")]
pub fn receive_and_process_decoded_frames(
    decoder: &mut ffmpeg::decoder::Video,
    scaler: &mut Context,
    packet: &Packet,
) -> anyhow::Result<Video> {
    let mut decoded = Video::empty();
    let mut rgb_frame = Video::empty();

    let mut out = decoder.receive_frame(&mut decoded);

    while out.is_err() {
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
    Ok(rgb_frame)
}

pub trait VideoPlayer {
    fn create(file_name: String, server_options: ServerOptions) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>>;
    fn video_data(&self) -> anyhow::Result<VideoData>;
    fn handle_jvm_msg(&self, msg: NativeCommunication) -> anyhow::Result<()>;
    fn destroy(self: Box<Self>) -> anyhow::Result<()>;
    //Note: This should free any resources of the implementation. Also self is being moved to the destroy fn so it will be dropped without drop call
}
