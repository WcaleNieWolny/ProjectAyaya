#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_eval_limit)]
#![const_eval_limit = "300000000"]
#[macro_use]
extern crate lazy_static;
extern crate ffmpeg_next as ffmpeg;
extern crate core;

mod video_player;
mod colorlib;

use ffmpeg::{Error, Packet};
use ffmpeg::format::{input, Pixel};
use ffmpeg::frame::Video;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use jni::JNIEnv;
use jni::objects::*;
use jni::sys::{jbyte, jbyteArray, jint, jlong, jsize};

use crate::video_player::VideoPlayer;

//Init function
fn init(
    env: JNIEnv,
    file_name: JString,
) -> Result<jlong, ffmpeg::Error> {

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
                println!("A: DECODE");
                let mut decoded = Video::empty();
                let mut rgb_frame = Video::empty();

                let mut out = decoder.receive_frame(&mut decoded);

                println!("ee: {}", Error::from(-11));
                while !out.is_ok() {
                    let err = out.unwrap_err();

                    if err == Error::from(-11) {
                        decoder.send_packet(packet).expect("Couldn't send packet to decoder");
                        out = decoder.receive_frame(&mut decoded);
                    }else {
                        return Err(err)
                    }
                }

                while out.is_ok() {
                    scaler.run(&decoded, &mut rgb_frame)?;

                    println!("OK, FRAME");
                    return Ok(rgb_frame);

                    //break
                }

                Err(out.unwrap_err())
            };

        let h = decoder.height();
        let w = decoder.width();

        let player = VideoPlayer::new(
                receive_and_process_decoded_frames,
                video_stream_index as i16,
                scaler,
                ictx,
                decoder,
                h,
                w
            );

        return Ok(player.wrap_to_java())
    }

    Err(Error::StreamNotFound)
}


//According to kotlin "@return Byte array of transformed frame (color index)"
fn load_frame(
    env: JNIEnv,
    ptr: jlong
) -> Result<jbyteArray, String> {
    // let buf: [i8; 16384] = [1; 16384];
    //
    // let output = env.new_byte_array(16384).unwrap();
    //
    // env.set_byte_array_region(output, 0, &buf).unwrap();

    let mut player = video_player::decode_from_java(ptr);

    println!("got player!");

    let frame = player.decode_frame().expect("Couldn't decode frame");

    let d = frame.planes();

    println!("planes: {}", d);

    let data = frame.data(0);

    println!("yes");

    let transformed_frame = colorlib::transform_frame_to_mc(data, player.width, player.height);

    println!("no");

    let output = env.new_byte_array((player.width * player.height) as jsize).unwrap(); //Can't fail to create array unless system is out of memory
    env.set_byte_array_region(output, 0, &transformed_frame.as_slice()).unwrap();

    return Ok(output)
}

//Destroy function (might remove later, for now for legacy purpose only)
fn destroy(
    _env: JNIEnv,
) -> Result<(), String> {
    println!("Destroy function called! This is legacy and should not be used!!!");
    Ok(())
}

fn get_width(
    _env: JNIEnv,
    ptr: jlong
) -> Result<jint, String>  {
    println!("R > WID");
    let box_player = video_player::decode_from_java(ptr);
    let w = box_player.width;

    return Ok(w as jint)
}

fn get_height(
    _env: JNIEnv,
    ptr: jlong
) -> Result<jint, String> {
    println!("R > HEI");
    let box_player = video_player::decode_from_java(ptr);
    let h = box_player.height;

    println!("HH: {}", h);

    return Ok(h as jint)
    //Err(String::from("YAU!"))
}

//Thanks to thatbakamono (https://github.com/thatbakamono) for help with developing this macro
//Also some magic is happening here. This macro should not work due to it not having a return type
//Yet somehow rustc and JNI can figure out everything. Please do not touch this or YOU WILL BREAK IT!!!
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
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_destroy, destroy);
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_init, init, {
    filename: JString,
});
jvm_impl!(Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_loadFrame, load_frame, {
    ptr: jlong,
});
