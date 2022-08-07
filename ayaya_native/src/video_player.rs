use std::mem::ManuallyDrop;
use ffmpeg::decoder::Video;
use ffmpeg::Error::Eof;
use ffmpeg::format::context::Input;
use ffmpeg::Packet;
use ffmpeg::software::scaling::{Context};

pub struct VideoPlayer {
    decode_function: fn(&mut Video, &mut Context, packet: &Packet) -> Result<ffmpeg::frame::Video, ffmpeg::Error>,
    frame_index: i64, //long
    video_stream_index: i16,
    scaler: Context, //ffmpeg scaling
    input: Input,
    decoder: Video,
    pub height: u32,
    pub width: u32
}

impl VideoPlayer{

    pub fn new(
        decode_function: fn(&mut Video, &mut Context, packet: &Packet) -> Result<ffmpeg::frame::Video, ffmpeg::Error>,
        video_stream_index: i16,
        scaler: Context,
        input: Input,
        decoder: Video,
        height: u32,
        width: u32
    ) -> Self {
        Self {
            decode_function,
            frame_index: 0,
            video_stream_index,
            scaler,
            input,
            decoder,
            width,
            height
        }
    }

    pub fn decode_frame(&mut self) -> Result<ffmpeg_next::util::frame::video::Video, ffmpeg::Error>{
        let decode_function = self.decode_function;
        // let data =  self.input.packets().next();
        //
        // if !data.is_none() {
        //     let data = data.unwrap();
        //     let stream = data.0;
        //     let packet = data.1;
        //
        //     if stream.index() == self.video_stream_index as usize {
        //         self.decoder.send_packet(&packet)?;
        //         decode_function(&mut self.decoder)?;
        //     }
        // }

        while let Some((stream, packet)) = self.input.packets().next() {
            if stream.index() == self.video_stream_index as usize {
                self.decoder.send_packet(&packet)?;
                return Ok(decode_function(&mut self.decoder, &mut self.scaler, &packet).unwrap());
            }
        }



        Err(Eof)
    }

    pub fn wrap_to_java(self) -> i64{
        let b = Box::new(self);
        let c = Box::into_raw(b);

        let d: i64 = c as i64;

        println!("{}", d);

        d
    }

}

impl PartialEq for VideoPlayer {
    fn eq(&self, other: &Self) -> bool {
        self.frame_index == other.frame_index &&
        self.video_stream_index == other.video_stream_index
    }
}

pub fn decode_from_java(ptr: i64) -> ManuallyDrop<Box<VideoPlayer>> {
    unsafe {
        return ManuallyDrop::new(Box::from_raw(ptr as *mut VideoPlayer));

        // let a = Box::from_raw(ptr as *mut VideoPlayer);
        //
        // return a;
    }

}


