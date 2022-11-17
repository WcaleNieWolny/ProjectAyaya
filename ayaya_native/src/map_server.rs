use std::{sync::{Arc, atomic::AtomicI64, mpsc}, io::Write, cell::RefCell, fmt::Debug};

use flate2::{write::GzEncoder, Compression};
use tokio::{net::TcpListener, io::{AsyncReadExt, AsyncWriteExt}, sync::{mpsc::{Receiver, Sender}, oneshot}};

use crate::player::player_context::NativeCommunication;

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub use_server: bool,
    pub bind_ip: String,
    pub port: i32
}

#[derive(Debug)]
pub struct MapServer{
    options: ServerOptions,
    frame_index: Arc<AtomicI64>,
    command_sender: Sender<NativeCommunication>,
}

pub type MapServerData = Option<Arc<MapServer>>;

impl MapServer {
    pub async fn create(
        options: &ServerOptions,
        frame_index: Arc<AtomicI64>,
        map_reciver: Arc<Receiver<Vec<i8>>>) -> anyhow::Result<MapServerData> {

        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(8);

        let server = Arc::new(MapServer{
            options: options.clone(),
            frame_index: frame_index,
            command_sender: cmd_tx,
        });

        server.init(map_reciver, cmd_rx).await?;

        Ok(Some(server)) 
    }

    async fn init(&self, map_reciver: Arc<Receiver<Vec<i8>>>, mut cmd_reciver: Receiver<NativeCommunication>) -> anyhow::Result<()>{
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

        tokio::spawn(async move {
            let msg = cmd_reciver.recv().await.expect("Couldn't recive NativeCommunication send_message");
            println!("GOT: {:?}", msg);
        });

        tokio::spawn(async move {
            let map_reciver = map_reciver;

            loop {
                let (mut socket, addr) = listener.accept().await.expect("Couldn't accept server connection!'");
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
        });
        Ok(())
    }

    pub fn send_message(&self, message: NativeCommunication) -> anyhow::Result<()>{
        self.command_sender.blocking_send(message)?;
        Ok(())
    }
}
