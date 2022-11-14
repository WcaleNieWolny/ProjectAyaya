use std::{sync::Arc, io::Write};

use flate2::{write::GzEncoder, Compression};
use tokio::{net::TcpListener, io::{AsyncReadExt, AsyncWriteExt}};

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub use_server: bool,
    pub bind_ip: String,
    pub port: i32
}

pub struct MapServer{
    options: ServerOptions
}

pub type MapServerData = Option<Arc<MapServer>>;

impl MapServer {
    pub fn new(options: &ServerOptions) -> MapServerData {
        return if options.use_server{
            Some(Arc::new(MapServer{
                options: options.clone()
            }))
        }else {
            None
        }
    }

    pub async fn init(&self) -> anyhow::Result<()>{
        let bind = format!("{}:{}", &self.options.bind_ip, &self.options.port.to_string());
        println!("Binding map server on: {}", bind);
        let listener = TcpListener::bind(bind).await?;

        //Note
        //1. encode packet data (https://crates.io/crates/libflate or https://github.com/rust-lang/flate2-rs#Backends) USE ZLIB
        //2. write size of encoded data to the packet buf (buf) <-- use i16 or u16
        //3. write encoded data to buf
        //
        //Java:
        //1. https://netty.io/4.0/api/io/netty/handler/codec/LengthFieldBasedFrameDecoder.html (Short.MAX_VALUE, 0, 2, 0, 2)
        //2. https://netty.io/4.0/api/io/netty/handler/codec/compression/ZlibDecoder.html

        loop {
            let (mut socket, addr) = listener.accept().await?;
            socket.set_nodelay(true).unwrap();
            println!("GOT CONNECTION FROM: {:?}", addr);
            tokio::spawn(async move {
                loop {
                    let mut buffer = [0u8; 1024];
                    let data = "Hello Map Server".as_bytes();
                    buffer[..data.len()].copy_from_slice(data);

                    //Main data
                    let mut encoder_vec = Vec::with_capacity(2048);
                    encoder_vec.write_u32(0).await.unwrap(); //Write len index to the vec
                    let mut encoder = GzEncoder::new(encoder_vec,  Compression::default());
                    encoder.write_all(data).expect("Compression failed");
                    let mut buffer = encoder.finish().expect("Finishing compression failed");

                    //Write len
                    let mut len_vec: Vec<u8> = Vec::with_capacity(4);
                    len_vec.write_u32(buffer.len() as u32 - 4 as u32).await.unwrap();
                    buffer[0..4].copy_from_slice(&len_vec);
                    
                    socket.write_all(&buffer).await.expect("Couldn't send data!'");
                    break;
                }
            });
        }
    }
}
