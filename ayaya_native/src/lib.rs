#![feature(const_fn_floating_point_arithmetic)]
#![feature(mem_copy_fn)]
#![feature(strict_provenance)]
#![feature(pointer_is_aligned)]
#![feature(new_uninit)]

extern crate core;
extern crate ffmpeg_next as ffmpeg;
extern crate lazy_static;

use ffmpeg::{Error};
use ffmpeg::codec::Capabilities;
use ffmpeg::decoder::Decoder;
use ffmpeg::format::{input};
use ffmpeg::media::Type;
use ffmpeg::threading::Config;
use ffmpeg::threading::Type::{Frame, Slice};
use jni::JNIEnv;
use jni::objects::*;
use jni::sys::{jboolean, jbyteArray, jint, jlong, jsize};
use crate::player::multi_video_player::MultiVideoPlayer;
use crate::player::player_context::{PlayerContext, VideoPlayer};
use crate::player::single_video_player::SingleVideoPlayer;

mod colorlib;
mod player;

fn ffmpeg_set_multithreading(
    target_decoder: &mut Decoder,
    file_name: String
){
    let copy_input = input(&file_name).unwrap();

    let copy_input = copy_input
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)
        .expect("Couldn't find video stream");

    let copy_context_decoder = ffmpeg::codec::context::Context::from_parameters(copy_input.parameters())
        .unwrap();

    let copy_decoder = copy_context_decoder.decoder();

    let mut copy_video = copy_decoder.video().expect("Couldn't enable multithreading due to creating decoder error");

    let copy_codec = copy_video.codec().unwrap();
    let copy_capabilities = copy_codec.capabilities();

    if copy_capabilities.contains(Capabilities::FRAME_THREADS) {
        let config = Config {
            kind: Frame,
            count: 2,
            safe: false
        };

        target_decoder.set_threading(config);
    }else if copy_capabilities.contains(Capabilities::SLICE_THREADS) {
        let config = Config {
            kind: Slice,
            count: 2,
            safe: false
        };

        target_decoder.set_threading(config);
    }

    copy_video.send_eof().expect("Couldn't close cloned codec");

}

//Init function
fn init(
    env: JNIEnv,
    file_name: JString,
    multithreading: jboolean,
) -> Result<jlong, Error> {
    let file_name: String = env
        .get_string(file_name)
        .expect("Couldn't get java string!")
        .into();

    let multithreading = multithreading == 1;

    match multithreading {
        false => {
            let player_context = SingleVideoPlayer::create(file_name).expect("Couldn't create single threaded player context");
            return Ok(PlayerContext::wrap_to_ptr(player_context))

        }
        true => {
            let player_context = MultiVideoPlayer::create(file_name).expect("Couldn't create single threaded player context");
            return Ok(PlayerContext::wrap_to_ptr(player_context))
        }
    }
}

fn start_multithreading(
    env: JNIEnv,
    ptr: jlong,
) -> anyhow::Result<()>{
    Ok(())
}


//According to kotlin "@return Byte array of transformed frame (color index)"
fn load_frame(
    env: JNIEnv,
    ptr: jlong,
) -> anyhow::Result<jbyteArray> {
    let data = PlayerContext::load_frame(ptr)?;
    let output = env.new_byte_array(data.len() as jsize)?; //Can't fail to create array unless system is out of memory
    env.set_byte_array_region(output, 0, &data.as_slice())?;

    Ok(output)
}

//Destroy function must be called to drop video_player struct
fn destroy(
    _env: JNIEnv,
    ptr: jlong,
) -> Result<(), String> {
    println!("TODO!");
    Ok(())
}

fn get_width(
    _env: JNIEnv,
    ptr: jlong,
) -> Result<jint, String> {
    Ok(PlayerContext::width(ptr))
}

fn get_height(
    _env: JNIEnv,
    ptr: jlong,
) -> Result<jint, String> {
    Ok(PlayerContext::height(ptr))
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
    multithreading: jboolean,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_startMultithreading, start_multithreading, {
    ptr: jlong,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_loadFrame, load_frame, jbyteArray, {
    ptr: jlong,
});
