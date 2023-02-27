use std::{ffi::{c_char, c_void, CString}, mem::ManuallyDrop};
use crate::{map_server::ServerOptions, colorlib};
use super::player_context::{VideoPlayer, VideoFrame, VideoData};

use anyhow::anyhow;

pub struct ExternalPlayer {
    ptr: *mut c_void,
    width: usize,
    height: usize,
    fps: usize,
    frame_len: usize
}

pub struct ExternalVideoFrame {
    inner: ManuallyDrop<Vec<i8>>,
    ptr: *mut i8
}

impl ExternalVideoFrame {
    unsafe fn new(ptr: *mut i8, size: usize) -> Self {
        let vec = Vec::<i8>::from_raw_parts(ptr, size, size);

        Self {
            inner: ManuallyDrop::new(vec),
            ptr
        }
    }
}

impl VideoFrame for ExternalVideoFrame {
    fn data(&self) -> &Vec<i8> {
        &self.inner
    }
}

impl Drop for ExternalVideoFrame {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr as *mut c_void)
        }
    }
}

#[repr(C)]
struct ExternalVideoData {
    width: usize,
    height: usize,
    fps: usize
}

extern "C" {
    fn external_player_init(color_transform_table_ptr: *const u8, file_name: *const c_char) -> *mut c_void;
    fn external_player_load_frame(player: *mut c_void) -> *mut i8;
    fn external_player_video_data(player: *mut c_void) -> ExternalVideoData;
    fn external_player_free(player: *mut c_void);
}

impl VideoPlayer for ExternalPlayer {
    fn create(file_name: String, _server_options: ServerOptions) -> anyhow::Result<Self>
    where
        Self: Sized {
        
        unsafe {
            let file_name = CString::new(file_name)?;
            let file_name_ptr = file_name.as_ptr();

            let ptr = external_player_init(colorlib::CONVERSION_TABLE_YUV.as_ptr(), file_name_ptr);

            if ptr.is_null() {
                return Err(anyhow!("Internal ExternalPlayer error, see stderr for more info"));
            }

            let video_data = external_player_video_data(ptr);

            Ok(Self {
                ptr,
                width: video_data.width,
                height: video_data.height,
                fps: video_data.fps,
                frame_len: video_data.width * video_data.height
            })
        }
    }

    fn load_frame(&mut self) -> anyhow::Result<Box<dyn VideoFrame>> {
        unsafe {
            let frame_ptr = external_player_load_frame(self.ptr);

            if frame_ptr.is_null() {
                return Err(anyhow!("Internal ExternalPlayer error, see stderr for more info"));
            }

            let frame = ExternalVideoFrame::new(frame_ptr, self.frame_len);

            return Ok(Box::new(frame));
        }
    }

    fn video_data(&self) -> anyhow::Result<VideoData> {
        Ok(VideoData {
            fps: self.fps as i32,
            width: self.width as i32,
            height: self.height as i32
        })
    }

    fn handle_jvm_msg(&self, _msg: super::player_context::NativeCommunication) -> anyhow::Result<()> {
        todo!()
    }

    fn destroy(&self) -> anyhow::Result<()> {
        unsafe {
            external_player_free(self.ptr)
        }

        Ok(())
    }
}
