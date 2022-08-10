#![feature(const_fn_floating_point_arithmetic)]
#![feature(strict_provenance)]
#![feature(ptr_to_from_bits)]
#![feature(core_intrinsics)]

extern crate core;
extern crate ffmpeg_next as ffmpeg;
extern crate lazy_static;

use std::ops::Sub;
use std::time::{SystemTime, UNIX_EPOCH};
use ffmpeg::{Error, Packet};
use ffmpeg::codec::Capabilities;
use ffmpeg::format::{input, Pixel};
use ffmpeg::frame::Video;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::threading::Config;
use ffmpeg::threading::Type::{Frame, Slice};
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


        let mut decoder = context_decoder.decoder();

        let codec = decoder.codec().expect("Couldn't get codec");
        let capabilities = codec.capabilities();


        if capabilities.contains(Capabilities::FRAME_THREADS) {
            let config = Config {
                kind: Frame,
                count: 2,
                safe: false
            };

            decoder.set_threading(config);
        }else if capabilities.contains(Capabilities::SLICE_THREADS) {
            let config = Config {
                kind: Slice,
                count: 2,
                safe: false
            };

            decoder.set_threading(config);
        }


        let decoder = decoder
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

                scaler.run(&decoded, &mut rgb_frame).expect("Scaler run failed");
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

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let frame = player.decode_frame().expect("Couldn't decode frame");
    let data = frame.data(0);

    println!("DEBUG RUST: {}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().sub(since_the_epoch).as_micros());

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

    (
        $BOILERPLATE_NAME: ident,
        $IMPLEMENTATION_NAME: ident,
        $RETURN_TYPE: tt,
        {
            $($a:ident: $b:tt,)+
        }
    ) => {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "system" fn $BOILERPLATE_NAME(environment: JNIEnv, _class: JClass, $($a: $b),+) -> $RETURN_TYPE {
            let response = $IMPLEMENTATION_NAME(environment, $($a),+);

            match response {
                Ok(some) => some,
                Err(error) => {
                    environment.throw_new("java/lang/RuntimeException", format!("{:?}", error))
                        .expect("Couldn't throw java error");

                    return 0 as $RETURN_TYPE
                }
            }

        }
    };

        (
        $BOILERPLATE_NAME: ident,
        $IMPLEMENTATION_NAME: ident,
        $RETURN_TYPE: tt,
    ) => {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "system" fn $BOILERPLATE_NAME(environment: JNIEnv, _class: JClass) -> $RETURN_TYPE {
            let response = $IMPLEMENTATION_NAME(environment);

            match response {
                Ok(some) => some,
                Err(error) => {
                    environment.throw_new("java/lang/RuntimeException", format!("{:?}", error))
                        .expect("Couldn't throw java error");

                    return 0 as $RETURN_TYPE
                }
            }
        }
    };
}

jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_height, get_height, jint, {
    ptr: jlong,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_width, get_width, jint, {
    ptr: jlong,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_destroy, destroy,
{
    ptr: jlong,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_init, init, jlong, {
    filename: JString,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_loadFrame, load_frame, jbyteArray, {
    ptr: jlong,
});
