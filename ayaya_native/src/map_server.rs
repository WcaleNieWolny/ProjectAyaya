use std::sync::Arc;

use tokio::{net::TcpListener, io::AsyncReadExt};

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
            let (mut socket, _addr) = listener.accept().await?;
            
            tokio::spawn(async move {
                loop {
                    let mut buffer = [0u8; 1024];

                    let bytes_read = socket.read(&mut buffer).await.expect("Couldn't read");
                    let string = String::from_utf8(buffer[..bytes_read].to_vec()).expect("Not utf8");

                    println!("DATA: {}, {}", bytes_read, string);
                }
            });
        }
    }
}
