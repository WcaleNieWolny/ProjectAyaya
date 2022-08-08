#![feature(const_fn_floating_point_arithmetic)]
extern crate core;
extern crate ffmpeg_next as ffmpeg;
extern crate lazy_static;

use ffmpeg::{Error, Packet};
use ffmpeg::format::{input, Pixel};
use ffmpeg::frame::Video;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use jni::JNIEnv;
use jni::objects::*;
use jni::sys::{jbyteArray, jint, jlong, jsize};

use crate::video_player::VideoPlayer;

mod video_player;
mod colorlib;

//Init function
fn init(
    env: JNIEnv,
    file_name: JString,
) -> Result<jlong, Error> {
    let file_name: String = env
        .get_string(file_name)
        .expect("Couldn't get java string!")
        .into();
    ffmpeg::init().unwrap();

    if let Ok(ictx) = input(&file_name) {
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)
            .expect("Couldn't find video stream");

        let video_stream_index = input.index();

        let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())
            .expect("Couldn't decode context decoder");

        let decoder = context_decoder
            .decoder()
            .video()
            .expect("Couldn't create decoder");

        let scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )?;


        let receive_and_process_decoded_frames =
            |decoder: &mut ffmpeg::decoder::Video, scaler: &mut Context, packet: &Packet| -> Result<Video, ffmpeg::Error> {
                let mut decoded = Video::empty();
                let mut rgb_frame = Video::empty();

                let mut out = decoder.receive_frame(&mut decoded);

                while !out.is_ok() {
                    let err = out.unwrap_err();

                    if err == Error::from(-11) {
                        decoder.send_packet(packet).expect("Couldn't send packet to decoder");
                        out = decoder.receive_frame(&mut decoded);
                    } else {
                        return Err(err);
                    }
                }

                scaler.run(&decoded, &mut rgb_frame)?;
                return Ok(rgb_frame);
            };

        let height = decoder.height();
        let width = decoder.width();

        let player = VideoPlayer::new(
            receive_and_process_decoded_frames,
            video_stream_index as i16,
            scaler,
            ictx,
            decoder,
            height,
            width,
        );

        return Ok(player.wrap_to_java());
    }

    Err(Error::StreamNotFound)
}


//According to kotlin "@return Byte array of transformed frame (color index)"
fn load_frame(
    env: JNIEnv,
    ptr: jlong,
) -> Result<jbyteArray, String> {
    let mut player = video_player::decode_from_java(ptr);

    let frame = player.decode_frame().expect("Couldn't decode frame");
    let data = frame.data(0);

    let transformed_frame = colorlib::transform_frame_to_mc(data, player.width, player.height);

    let output = env.new_byte_array((player.width * player.height) as jsize).unwrap(); //Can't fail to create array unless system is out of memory
    env.set_byte_array_region(output, 0, &transformed_frame.as_slice()).unwrap();

    return Ok(output);
}

//Destroy function must be called to drop video_player struct
fn destroy(
    _env: JNIEnv,
    ptr: jlong,
) -> Result<(), String> {
    let mut player = video_player::decode_from_java(ptr);

    player.destroy();
    drop(player);
    Ok(())
}

fn get_width(
    _env: JNIEnv,
    ptr: jlong,
) -> Result<jint, String> {
    Ok(video_player::decode_from_java(ptr).width as jint)
}

fn get_height(
    _env: JNIEnv,
    ptr: jlong,
) -> Result<jint, String> {
    Ok(video_player::decode_from_java(ptr).height as jint)
}

//Thanks to thatbakamono (https://github.com/thatbakamono) for help with developing this macro
//Also some magic is happening here. This macro should not work due to it not having a return type
//Yet somehow rustc and JNI can figure out everything. Please do not touch this or YOU WILL BREAK IT!!!
//This could be optimized to have ony one bracket but I do not understand macros well
macro_rules! jvm_impl {
    (
        $BOILERPLATE_NAME: ident,
        $IMPLEMENTATION_NAME: ident
    ) => {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "system" fn $BOILERPLATE_NAME(environment: JNIEnv, _class: JClass) {
            let response = $IMPLEMENTATION_NAME(environment);

            if let Err(error) = response {
                environment.throw_new("java/lang/RuntimeException", format!("{:?}", error))
                    .expect("Couldn't throw java error");
            }else{
                response.unwrap(); //magic code
            }
        }
    };

    (
        $BOILERPLATE_NAME: ident,
        $IMPLEMENTATION_NAME: ident,
        {
            $($a:ident: $b:tt,)+
        }
    ) => {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "system" fn $BOILERPLATE_NAME(environment: JNIEnv, _class: JClass, $($a: $b),+) {
            let response = $IMPLEMENTATION_NAME(environment, $($a),+);

            if let Err(error) = response {
                environment.throw_new("java/lang/RuntimeException", format!("{:?}", error))
                    .expect("Couldn't throw java error");
            }else{
                response.unwrap();
            }
        }
    };
}

jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_height, get_height, {
    ptr: jlong,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_width, get_width, {
    ptr: jlong,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_destroy, destroy,
{
    ptr: jlong,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_init, init, {
    filename: JString,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_loadFrame, load_frame, {
    ptr: jlong,
});
