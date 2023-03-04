#![feature(linked_list_cursors)]
#![feature(array_chunks)]
#![feature(iter_array_chunks)]
#![feature(test)]

extern crate core;
extern crate test;

#[cfg(feature = "ffmpeg")]
extern crate ffmpeg_next as ffmpeg;

use std::num::NonZeroU64;

use anyhow::anyhow;

#[cfg(feature = "ffmpeg")]
use {
    ffmpeg::codec::Capabilities,
    ffmpeg::decoder::Decoder,
    ffmpeg::format::input,
    ffmpeg::media::Type,
    ffmpeg::threading::Config,
    ffmpeg::threading::Type::{Frame, Slice},
    ffmpeg::Error,
    player::multi_video_player::MultiVideoPlayer,
    player::single_video_player::SingleVideoPlayer,
    player::x11_player::X11Player,
    splitting::SplittedFrame,
};

use jni::objects::*;
use jni::sys::{jbyteArray, jint, jlong, jobject, jsize};
use jni::JNIEnv;

use map_server::ServerOptions;

use crate::discord_audio::DiscordPlayer;
use once_cell::sync::Lazy;
use player::{
    discord_audio::DiscordOptions,
    game_player::{GameInputDirection, GamePlayer},
};
use player::{
    discord_audio::{self, DiscordClient},
    player_context::VideoPlayer,
};
use player::{
    external_player::ExternalPlayer,
    player_context::{self, NativeCommunication},
};
use tokio::runtime::{Builder, Runtime};

mod colorlib;
mod map_server;

mod apps;
mod player;
mod splitting;

static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread()
        .worker_threads(8_usize)
        .thread_name("ProjectAyaya native worker thread")
        .thread_stack_size(3840_usize * 2160_usize * 4) //Big stack due to memory heavy operations (4k is max resolution for now)
        .enable_io()
        .enable_time()
        .build()
        .expect("Couldn't create tokio runtime")
});

#[cfg(feature = "ffmpeg")]
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

#[allow(unused_variables)]
fn verify_capabilities(
    env: &mut JNIEnv,
    file_name: JString,
    width: jint,
    height: jint,
    use_discord: bool,
) -> anyhow::Result<jobject> {
    if width % 128 != 0 || height % 128 != 0 {
        return Err(anyhow!(
            "Width or height of the request not divisble by 128"
        ));
    }

    if use_discord && DiscordClient::is_used()? {
        let discord_in_use = env
            .call_static_method(
                "me/wcaleniewolny/ayaya/library/VideoRequestCapablyResponse",
                "valueOf",
                "(Ljava/lang/String;)Lme/wcaleniewolny/ayaya/library/VideoRequestCapablyResponse;",
                &[(&env.new_string("DISCORD_IN_USE")?).into()],
            )?
            .l()?;
        return Ok(discord_in_use.into_raw());
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "ffmpeg")] {
            let file_name: String = env.get_string(&file_name)?.into();

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
                    let invalid_dimenstions = env.call_static_method("me/wcaleniewolny/ayaya/library/VideoRequestCapablyResponse", "valueOf", "(Ljava/lang/String;)Lme/wcaleniewolny/ayaya/library/VideoRequestCapablyResponse;", &[(&env.new_string("INVALID_DIMENSIONS")?).into()])?.l()?;
                    return Ok(invalid_dimenstions.into_raw());
                }

                if v_width > width as u32 || v_height > height as u32 {
                    let to_small = env.call_static_method("me/wcaleniewolny/ayaya/library/VideoRequestCapablyResponse", "valueOf", "(Ljava/lang/String;)Lme/wcaleniewolny/ayaya/library/VideoRequestCapablyResponse;", &[(&env.new_string("TO_SMALL")?).into()])?.l()?;
                    return Ok(to_small.into_raw());
                }

                let required_frames_x: i32 = (v_width as f32 / 128.0).ceil() as i32;
                let required_frames_y: i32 = (v_height as f32 / 128.0).ceil() as i32;

                let frames_x  = width / 128;
                let frames_y = height / 128;


                if required_frames_x != frames_x || required_frames_y != frames_y {
                    let to_large = env.call_static_method("me/wcaleniewolny/ayaya/library/VideoRequestCapablyResponse", "valueOf", "(Ljava/lang/String;)Lme/wcaleniewolny/ayaya/library/VideoRequestCapablyResponse;", &[(&env.new_string("TO_LARGE")?).into()])?.l()?;
                    return Ok(to_large.into_raw());
                }

                let ok = env.call_static_method("me/wcaleniewolny/ayaya/library/VideoRequestCapablyResponse", "valueOf", "(Ljava/lang/String;)Lme/wcaleniewolny/ayaya/library/VideoRequestCapablyResponse;", &[(&env.new_string("OK")?).into()])?.l()?;
                return Ok(ok.into_raw());
            };
            return Err(anyhow!("Coudln't create ffmpeg decoder!"));
        } else {
            return Err(anyhow!("FFmpeg feature not compiled!"))
        }
    }
}

//Init function
fn init(
    env: &mut JNIEnv,
    file_name: JString,
    render_type: JObject,
    server_options: JObject,
    use_discord: bool,
) -> anyhow::Result<jlong> {
    let file_name: String = env.get_string(&file_name)?.into();

    let use_server = env.call_method(&server_options, "getUseServer", "()Z", &[])?;
    let use_server = use_server.z()?;

    let bind_ip = env.call_method(&server_options, "getBindIp", "()Ljava/lang/String;", &[])?;
    let bind_ip = bind_ip.l()?;
    let bind_ip: JString = bind_ip.into();
    let bind_ip = env.get_string(&bind_ip)?;
    let bind_ip: String = bind_ip.into();

    let port = env.call_method(&server_options, "getPort", "()I", &[])?;
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
            cfg_if::cfg_if! {
                if #[cfg(feature = "ffmpeg")]
                {
                    let player_context = SingleVideoPlayer::create(file_name.clone(), server_options)?;

                    if use_discord {
                        let player_context = DiscordPlayer::create_with_discord(file_name, Box::new(player_context), false)?;
                        Ok(player_context::wrap_to_ptr(player_context))
                    }else {
                        Ok(player_context::wrap_to_ptr(player_context))
                    }

                }else{
                    return Err(anyhow!("FFmpeg feature not compiled!"))
                }
            }
        }
        1 => {
            cfg_if::cfg_if! {
                if #[cfg(feature = "ffmpeg")]{
                    let player_context = MultiVideoPlayer::create(file_name.clone(), server_options)?;
                    if use_discord {
                        let player_context = DiscordPlayer::create_with_discord(file_name, Box::new(player_context), false)?;
                        Ok(player_context::wrap_to_ptr(player_context))
                    }else {
                        Ok(player_context::wrap_to_ptr(player_context))
                    }
                }else {
                    return Err(anyhow!("FFmpeg feature not compiled!"))
                }
            }
        }
        2 => {
            if use_discord {
                return Err(anyhow!("Game player does not suport discord audio!"));
            }
            let player_context = GamePlayer::create(file_name, server_options)?;
            Ok(player_context::wrap_to_ptr(player_context))
        }
        3 => {
            cfg_if::cfg_if! {
                if #[cfg(feature = "external_player")] {
                    if use_discord {
                        return Err(anyhow!("X11 player does not suport discord audio!"));
                    }
                    let player_context = X11Player::create(file_name, server_options)?;
                    Ok(player_context::wrap_to_ptr(player_context))
                }else {
                    return Err(anyhow!("FFmpeg feature not compiled!"))
                }
            }
        }
        4 => {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "external_player", feature = "ffmpeg"))] {
                    let player_context = ExternalPlayer::create(file_name, server_options)?;
                    Ok(player_context::wrap_to_ptr(player_context))
                }else {
                    return Err(anyhow!("external_player feature not compiled!"))
                }
            }
        }
        _ => Err(anyhow::Error::msg(format!("Invalid id ({render_type})"))),
    };
}

//According to kotlin "@return Byte array of transformed frame (color index)"
fn load_frame(env: &mut JNIEnv, ptr: jlong) -> anyhow::Result<jbyteArray> {
    let data = player_context::load_frame(ptr)?;
    let data_vec = data.data();

    let output = env.new_byte_array(data_vec.len() as jsize)?; //Can't fail to create array unless system is out of memory
    env.set_byte_array_region(&output, 0, data_vec.as_slice())?;
    drop(data);
    Ok(output.into_raw())
}

fn recive_jvm_msg(
    env: &mut JNIEnv,
    ptr: jlong,
    native_lib_communication: JObject,
    info: JString,
) -> anyhow::Result<()> {
    let msg_type_obj = env.call_method(native_lib_communication, "ordinal", "()I", &[])?;
    let msg_type = msg_type_obj.i()?;

    let info_string: String = env.get_string(&info)?.into();

    let msg_type = match msg_type {
        0 => {
            let fps = info_string.parse::<i32>()?;
            NativeCommunication::StartRendering { fps }
        }
        1 => NativeCommunication::StopRendering,
        2 => {
            let info_str_vec = info_string.split('_');
            let mut game_input_vec = Vec::<GameInputDirection>::with_capacity(4);
            for val in info_str_vec {
                let input = match val {
                    "F" => GameInputDirection::Forward,
                    "B" => GameInputDirection::Backwards,
                    "L" => GameInputDirection::Left,
                    "R" => GameInputDirection::Right,
                    "U" => GameInputDirection::Up,
                    "" => continue,
                    _ => return Err(anyhow!("Invalid short game input")),
                };
                game_input_vec.push(input);
            }

            NativeCommunication::GameInput {
                input: game_input_vec,
            }
        }
        3 => {
            let second = info_string.parse::<i32>()?;
            NativeCommunication::VideoSeek { second }
        }
        _ => return Err(anyhow!("Invalid msg enum")),
    };

    player_context::pass_jvm_msg(ptr, msg_type)?;
    Ok(())
}

fn init_discord_bot(env: &mut JNIEnv, discord_options: JObject) -> anyhow::Result<()> {
    let discord_options = {
        let use_discord = env
            .call_method(&discord_options, "getUseDiscord", "()Z", &[])?
            .z()?;

        let discord_token = env
            .call_method(
                &discord_options,
                "getDiscordToken",
                "()Ljava/lang/String;",
                &[],
            )?
            .l()?;
        let discord_token: String = env.get_string(&discord_token.into())?.into();

        let guild_id = env
            .call_method(&discord_options, "getGuildId", "()Ljava/lang/String;", &[])?
            .l()?;
        let guild_id: String = env.get_string(&guild_id.into())?.into();

        let channel_id = env
            .call_method(
                &discord_options,
                "getChannelId",
                "()Ljava/lang/String;",
                &[],
            )?
            .l()?;
        let channel_id: String = env.get_string(&channel_id.into())?.into();

        let guild_id: u64 = guild_id.parse()?;
        let channel_id: u64 = channel_id.parse()?;

        let guild_id = match NonZeroU64::new(guild_id) {
            Some(val) => val,
            None => return Err(anyhow!("Guild ID is zero")),
        };

        let channel_id = match NonZeroU64::new(channel_id) {
            Some(val) => val,
            None => return Err(anyhow!("Channel ID is zero")),
        };

        DiscordOptions {
            use_discord,
            discord_token,
            guild_id,
            channel_id,
        }
    };

    if discord_options.use_discord {
        discord_audio::init(&discord_options)?;
    }

    Ok(())
}

//Destroy function must be called to drop video_player struct
fn destroy(_env: &mut JNIEnv, ptr: jlong) -> anyhow::Result<()> {
    player_context::destroy(ptr)?;
    Ok(())
}

fn get_video_data(env: &mut JNIEnv, ptr: jlong) -> anyhow::Result<jobject> {
    // let jclass = env.find_class("me/wcaleniewolny/ayaya/library/VideoData")?;
    // let jconstructor = env.get_method_id(jclass, "<init>", "(III)V")?;

    let video_data = player_context::video_data(ptr)?;

    let jobject = env.new_object(
        "me/wcaleniewolny/ayaya/library/VideoData",
        "(III)V",
        &[
            JValue::Int(video_data.width),
            JValue::Int(video_data.height),
            JValue::Int(video_data.fps),
        ],
    )?;
    Ok(jobject.into_raw())
}

//Thanks to thatbakamono (https://github.com/thatbakamono) for help with developing this macro
//Also some magic is happening here. This macro should not work due to it not having a return type
//Yet somehow rustc and JNI can figure out everything. Please do not touch this or YOU WILL BREAK IT!!!
//This could be optimized to have ony one bracket but I do not understand macros well
macro_rules! jvm_impl {

    (
        $BOILERPLATE_NAME: ident,
        $IMPLEMENTATION_NAME: ident,
        $RETURN_TYPE: tt,
        {
            $($a:ident: $b:tt $(,)?)+
        }
    ) => {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "system" fn $BOILERPLATE_NAME(mut environment: JNIEnv, _class: JClass, $($a: $b),+) -> $RETURN_TYPE {
            let response = $IMPLEMENTATION_NAME(&mut environment, $($a),+);

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
        {
            $($a:ident: $b:tt $(,)?)+
        }
    ) => {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "system" fn $BOILERPLATE_NAME(mut environment: JNIEnv, _class: JClass, $($a: $b),+) {
            let response = $IMPLEMENTATION_NAME(&mut environment, $($a),+);

            if let Err(error) = response {
                environment.throw_new("java/lang/RuntimeException", format!("{:?}", error))
                    .expect("Couldn't throw java error");
            }
        }
    };
}

jvm_impl!(
    Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_getVideoData,
    get_video_data,
    jobject,
    { ptr: jlong }
);
jvm_impl!(
    Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_destroy,
    destroy,
    { ptr: jlong }
);
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_init, init, jlong, {
    filename: JString,
    render_type: JObject,
    server_options: JObject,
    use_discord: bool
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_loadFrame, load_frame, jbyteArray, {
    ptr: jlong,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_communicate, recive_jvm_msg, {
    ptr: jlong,
    native_lib_communication: JObject,
    info: JString
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_verifyScreenCapabilities, verify_capabilities, jobject, {
    file_name: JString,
    width: jint,
    height: jint,
    use_discord: bool
});
jvm_impl!(
    Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_initDiscordBot,
    init_discord_bot,
    { file_name: JObject }
);
