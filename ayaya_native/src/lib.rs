#![feature(const_fn_floating_point_arithmetic)]
#![feature(mem_copy_fn)]
#![feature(strict_provenance)]
#![feature(pointer_is_aligned)]
#![feature(new_uninit)]

extern crate core;
extern crate ffmpeg_next as ffmpeg;
extern crate lazy_static;

use anyhow::anyhow;
use ffmpeg::codec::Capabilities;
use ffmpeg::decoder::Decoder;
use ffmpeg::format::input;
use ffmpeg::media::Type;
use ffmpeg::threading::Config;
use ffmpeg::threading::Type::{Frame, Slice};
use ffmpeg::Error;
use jni::objects::*;
use jni::sys::{jboolean, jbyteArray, jint, jlong, jobject, jsize};
use jni::JNIEnv;
use map_server::ServerOptions;
use player::game_player::GamePlayer;
use player::player_context::NativeCommunication;

use crate::player::multi_video_player::MultiVideoPlayer;
use crate::player::player_context::{PlayerContext, VideoPlayer};
use crate::player::single_video_player::SingleVideoPlayer;
use crate::splitting::SplittedFrame;

mod colorlib;
mod map_server;
mod player;
mod splitting;

fn ffmpeg_set_multithreading(target_decoder: &mut Decoder, file_name: String) {
    let copy_input = input(&file_name).unwrap();
    let copy_input = copy_input
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)
        .expect("Couldn't find video stream");

    let copy_context_decoder =
        ffmpeg::codec::context::Context::from_parameters(copy_input.parameters()).unwrap();

    let copy_decoder = copy_context_decoder.decoder();

    let mut copy_video = copy_decoder
        .video()
        .expect("Couldn't enable multithreading due to creating decoder error");

    let copy_codec = copy_video.codec().unwrap();
    let copy_capabilities = copy_codec.capabilities();

    if copy_capabilities.contains(Capabilities::FRAME_THREADS) {
        let config = Config {
            kind: Frame,
            count: 2,
            safe: false,
        };

        target_decoder.set_threading(config);
    } else if copy_capabilities.contains(Capabilities::SLICE_THREADS) {
        let config = Config {
            kind: Slice,
            count: 2,
            safe: false,
        };

        target_decoder.set_threading(config);
    }

    copy_video.send_eof().expect("Couldn't close cloned codec");
}

fn verify_capabilities(
    env: JNIEnv,
    file_name: JString,
    width: jint,
    height: jint,
) -> anyhow::Result<jboolean> {
    let file_name: String = env.get_string(file_name)?.into();

    ffmpeg::init()?;
    if let Ok(ictx) = input(&file_name) {
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(Error::StreamNotFound)?;

        let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;

        let decoder = context_decoder.decoder();
        let decoder = decoder.video()?;

        let v_width = decoder.width();
        let v_height = decoder.height();

        if v_width % 2 != 0 || v_height % 2 != 0 {
            return Ok(false.into());
        }

        if v_width > width as u32 || v_height > height as u32 {
            return Ok(false.into());
        }
        return Ok(true.into());
    };
    Err(anyhow!("Coudln't create ffmpeg decoder!"))
}

//Init function
fn init(
    env: JNIEnv,
    file_name: JString,
    render_type: JObject,
    server_options: JObject,
) -> anyhow::Result<jlong> {
    let file_name: String = env.get_string(file_name)?.into();

    let use_server = env.call_method(server_options, "getUseServer", "()Z", &[])?;
    let use_server = use_server.z()?;

    let bind_ip = env.call_method(server_options, "getBindIp", "()Ljava/lang/String;", &[])?;
    let bind_ip = bind_ip.l()?;
    let bind_ip = env.get_string(bind_ip.into())?;
    let bind_ip: String = bind_ip.into();

    let port = env.call_method(server_options, "getPort", "()I", &[])?;
    let port = port.i()?;

    let server_options = ServerOptions {
        use_server,
        bind_ip,
        port,
    };

    let render_type = env.call_method(render_type, "ordinal", "()I", &[])?;
    let render_type = render_type.i()?;

    return match render_type {
        0 => {
            let player_context = SingleVideoPlayer::create(file_name, server_options)?;
            Ok(PlayerContext::wrap_to_ptr(player_context))
        }
        1 => {
            let player_context = MultiVideoPlayer::create(file_name, server_options)?;
            Ok(PlayerContext::wrap_to_ptr(player_context))
        }
        2 => {
           let player_context = GamePlayer::create(file_name, server_options)?;
           Ok(PlayerContext::wrap_to_ptr(player_context))
        }
        _ => Err(anyhow::Error::msg(format!("Invalid id ({})", render_type))),
    };
}

//According to kotlin "@return Byte array of transformed frame (color index)"
fn load_frame(env: JNIEnv, ptr: jlong) -> anyhow::Result<jbyteArray> {
    let data = PlayerContext::load_frame(ptr)?;
    let output = env.new_byte_array(data.len() as jsize)?; //Can't fail to create array unless system is out of memory
    env.set_byte_array_region(output, 0, &data.as_slice())?;

    Ok(output)
}

fn recive_jvm_msg(
    env: JNIEnv,
    ptr: jlong,
    native_lib_communication: JObject,
    info: JString,
) -> anyhow::Result<()> {
    let msg_type = env.call_method(native_lib_communication, "ordinal", "()I", &[])?;
    let msg_type = msg_type.i()?;

    let info_string: String = env.get_string(info)?.into();

    let msg_type = match msg_type {
        0 => {
            let fps = info_string.parse::<i32>()?;
            NativeCommunication::StartRendering { fps }
        }
        1 => NativeCommunication::StopRendering,
        _ => return Err(anyhow!("Invalid msg enum")),
    };

    PlayerContext::pass_jvm_msg(ptr, msg_type)?;
    Ok(())
}

//Destroy function must be called to drop video_player struct
fn destroy(_env: JNIEnv, ptr: jlong) -> anyhow::Result<()> {
    PlayerContext::destroy(ptr)?;
    Ok(())
}

fn get_video_data(env: JNIEnv, ptr: jlong) -> anyhow::Result<jobject> {
    // let jclass = env.find_class("me/wcaleniewolny/ayaya/library/VideoData")?;
    // let jconstructor = env.get_method_id(jclass, "<init>", "(III)V")?;

    let video_data = PlayerContext::video_data(ptr)?;

    let jobject = env.new_object(
        "me/wcaleniewolny/ayaya/library/VideoData",
        "(III)V",
        &[
            JValue::Int(video_data.width),
            JValue::Int(video_data.height),
            JValue::Int(video_data.fps),
        ],
    )?;
    Ok(jobject.into_inner())
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

jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_getVideoData, get_video_data, jobject, {
    ptr: jlong,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_destroy, destroy,
{
    ptr: jlong,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_init, init, jlong, {
    filename: JString,
    render_type: JObject,
    server_options: JObject,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_loadFrame, load_frame, jbyteArray, {
    ptr: jlong,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_communicate, recive_jvm_msg, {
    ptr: jlong,
    native_lib_communication: JObject,
    info: JString,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_verifyScreenCapabilities, verify_capabilities, jboolean, {
    file_name: JString,
    width: jint,
    height: jint,
});
