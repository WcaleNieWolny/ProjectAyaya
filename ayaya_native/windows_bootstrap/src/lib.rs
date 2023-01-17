use std::{
    ffi::{c_void, OsStr},
    fs::{self, File},
    io::{self, Read},
    iter::once,
    mem::ManuallyDrop,
    os::windows::prelude::OsStrExt,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use jni::{
    objects::{JClass, JString},
    sys::jlong,
    JNIEnv, NativeMethod,
};
use libloading::Library;
use winapi::um::winbase;

static EXTERNAL_METHODS: [(&[u8], &str, &str); 6] = [
    (b"Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_verifyScreenCapabilities", "verifyScreenCapabilities", "(Ljava/lang/String;II)Z"),
    (b"Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_destroy", "destroy", "(J)V"),
    (b"Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_loadFrame", "loadFrame", "(J)[B"),
    (b"Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_communicate", "communicate", "(JLme/wcaleniewolny/ayaya/library/NativeLibCommunication;Ljava/lang/String;)V"),
    (b"Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_init", "init", "(Ljava/lang/String;Lme/wcaleniewolny/ayaya/library/NativeRenderType;Lme/wcaleniewolny/ayaya/library/MapServerOptions;)J"),
    (b"Java_me_wcaleniewolny_ayaya_library_NativeRenderControler_getVideoData", "getVideoData", "(J)Lme/wcaleniewolny/ayaya/library/VideoData;")
];

static FFMPEG_URL: &str = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-lgpl-shared.zip";
static FFMPEG_FOLDER_PATH_PREFIX: &str = "ffmpeg-master-latest-win64-lgpl-shared";

#[no_mangle]
pub extern "system" fn Java_me_wcaleniewolny_ayaya_library_WindowsBootstrap_bootstrap(
    env: JNIEnv,
    _class: JClass,
    lib_path: JString,
    app_folder: JString,
) -> jlong {
    let response = bootstrap(env, lib_path, app_folder);

    match response {
        Ok(some) => some,
        Err(error) => {
            env.throw_new("java/lang/RuntimeException", format!("{:?}", error))
                .expect("Couldn't throw java error");
            return 0;
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_me_wcaleniewolny_ayaya_library_WindowsBootstrap_cleanup(
    env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    let response = cleanup(env, ptr);

    if let Err(error) = response {
        env.throw_new("java/lang/RuntimeException", format!("{:?}", error))
            .expect("Couldn't throw java error");
    }
}

fn bootstrap(env: JNIEnv, lib_path: JString, app_folder: JString) -> anyhow::Result<jlong> {
    let lib_path: String = env.get_string(lib_path)?.into();
    let app_folder: String = env.get_string(app_folder)?.into();

    if !Path::new(&lib_path).exists() {
        return Err(anyhow!("DLL file does not exists!"));
    };

    let mut ffmpeg_folder_path_buf = PathBuf::new();
    ffmpeg_folder_path_buf.push(&app_folder);
    ffmpeg_folder_path_buf.push("ffmpeg_bootstrap");

    let ffmpeg_folder_path_buf_clone = ffmpeg_folder_path_buf.clone();
    let ffmpeg_folder_path = ffmpeg_folder_path_buf_clone.as_path();

    if !ffmpeg_folder_path.exists() {
        fs::create_dir_all(ffmpeg_folder_path)?;
    }

    let mut ffmpeg_zip_path_buf = ffmpeg_folder_path_buf.clone();
    ffmpeg_zip_path_buf.push("ffmpeg-zip.zip");
    let ffmpeg_zip_path = ffmpeg_zip_path_buf.as_path();

    if !ffmpeg_zip_path.exists() {
        println!("[AyayaNative Bootstrap] FFmpeg zip does not exists! Downloading now! (This will block server start for a few seconds)");
        downlaod_ffmpeg(&ffmpeg_zip_path)?;
        extract_zip(&ffmpeg_zip_path)?;
    }

    ffmpeg_folder_path_buf.push(&FFMPEG_FOLDER_PATH_PREFIX);
    ffmpeg_folder_path_buf.push("bin");

    let ffmpeg_folder_path = match ffmpeg_folder_path_buf.to_str() {
        Some(val) => val,
        None => return Err(anyhow!("Cannot get ffmpeg bin folder path to str")),
    };

    let wide: Vec<u16> = OsStr::new(ffmpeg_folder_path)
        .encode_wide()
        .chain(once(0))
        .collect();
    let status_code = unsafe { winbase::SetDllDirectoryW(wide.as_ptr()) };

    if status_code != 1 {
        return Err(anyhow!(format!(
            "SetDllDirectoryW status code ({:?}) is not 1",
            status_code
        )));
    }

    println!("[AyayaNative Bootstrap] Lib path: {}", lib_path);
    unsafe {
        let lib = libloading::Library::new(lib_path)?;
        let jclass = env.find_class("me/wcaleniewolny/ayaya/library/NativeRenderControler")?;

        for (symbol, name, sig) in EXTERNAL_METHODS {
            let symbol: libloading::Symbol<*mut c_void> = lib.get(symbol)?;
            println!(
                "[AyayaNative Bootstrap] Registering {:?} at {:?}",
                name, symbol
            );
            let native_method = NativeMethod {
                name: name.into(),
                sig: sig.into(),
                fn_ptr: *symbol,
            };

            //There is no need to use only one register call - I don' that this is an expensive call
            env.register_native_methods(jclass, &[native_method])?;
        }

        //Leak the lib so it does not unload
        let lib = ManuallyDrop::new(lib);
        let lib_box = Box::new(lib);
        let lib_ptr = Box::into_raw(lib_box) as *const () as i64;
        return Ok(lib_ptr);
    };
}

fn downlaod_ffmpeg(path: &Path) -> anyhow::Result<()> {
    let mut file = File::create(path)?;

    let resp = reqwest::blocking::get(FFMPEG_URL)?;
    let data = resp.bytes()?;
    let mut data = data.take(data.len() as u64);

    io::copy(&mut data, &mut file)?;

    Ok(())
}

//https://github.com/zip-rs/zip/blob/master/examples/extract.rs
fn extract_zip(fname: &Path) -> anyhow::Result<()> {
    let file = fs::File::open(&fname)?;
    let fname = match fname.parent() {
        Some(val) => val,
        None => return Err(anyhow!("Unable to get zip folder parrent!")),
    };

    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        let outpath = fname.join(outpath);

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

fn cleanup(env: JNIEnv, ptr: jlong) -> anyhow::Result<()> {
    let jclass = env.find_class("me/wcaleniewolny/ayaya/library/NativeRenderControler")?;

    env.unregister_native_methods(jclass)?;

    unsafe {
        let lib_box: Box<ManuallyDrop<Library>> =
            Box::from_raw(ptr as *mut () as *mut ManuallyDrop<Library>);
        let lib = ManuallyDrop::into_inner(*lib_box);
        drop(lib);
    };

    println!("Windows bootstrap cleanup ended!");
    Ok(())
}
