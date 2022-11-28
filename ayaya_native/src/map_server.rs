use std::{
    fmt::Debug,
    io::Write,
    mem,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::Duration,
};

use flate2::{write::ZlibEncoder, Compression};
use tokio::{
    io::AsyncWriteExt,
    net::TcpListener,
    sync::{
        broadcast,
        mpsc::{Receiver, Sender},
    },
    time,
};

use crate::player::player_context::NativeCommunication;

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub use_server: bool,
    pub bind_ip: String,
    pub port: i32,
}

#[derive(Debug)]
pub struct MapServer {
    options: ServerOptions,
    frame_index: Arc<AtomicI64>,
    command_sender: Sender<NativeCommunication>,
}

pub type MapServerData = Option<Arc<MapServer>>;

impl MapServer {
    pub async fn create(
        options: &ServerOptions,
        frame_index: Arc<AtomicI64>,
        map_reciver: Receiver<Vec<i8>>,
    ) -> anyhow::Result<MapServerData> {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(8);

        let server = Arc::new(MapServer {
            options: options.clone(),
            frame_index: frame_index,
            command_sender: cmd_tx,
        });

        server.init(map_reciver, cmd_rx).await?;

        Ok(Some(server))
    }

    async fn init(
        &self,
        mut map_reciver: Receiver<Vec<i8>>,
        mut cmd_reciver: Receiver<NativeCommunication>,
    ) -> anyhow::Result<()> {
        let bind = format!(
            "{}:{}",
            &self.options.bind_ip,
            &self.options.port.to_string()
        );
        println!("Binding map server on: {}", bind);
        let listener = TcpListener::bind(bind).await?;
        let frame_index = self.frame_index.clone();

        //Note
        //1. encode packet data (https://crates.io/crates/libflate or https://github.com/rust-lang/flate2-rs#Backends) USE ZLIB
        //2. write size of encoded data to the packet buf (buf) <-- use i16 or u16
        //3. write encoded data to buf
        //
        //Java:
        //1. https://netty.io/4.0/api/io/netty/handler/codec/LengthFieldBasedFrameDecoder.html (Short.MAX_VALUE, 0, 2, 0, 2)
        //2. https://netty.io/4.0/api/io/netty/handler/codec/compression/ZlibDecoder.html

        let (tcp_frame_tx, tcp_frame_rx) = broadcast::channel::<Arc<Vec<u8>>>(512);

        tokio::spawn(async move {
            let msg = cmd_reciver
                .recv()
                .await
                .expect("Couldn't recive NativeCommunication send_message");
            println!("GOT: {:?}", msg);

            match msg {
                NativeCommunication::StartRendering { fps } => {
                    let mut data_size = 0;
                    let data = map_reciver
                        .recv()
                        .await
                        .expect("Couldn't recive first frame");

                    let mut data = Self::prepare_frame(data, &mut data_size)
                        .await
                        .expect("Couldn't preprare tcp frame");

                    //1000000000 nanoseconds = 1 second. Rust feature for that is unsable
                    let dur = Duration::from_millis(1000 as u64 / (fps as u64));
                    let mut interval = time::interval(dur);
                    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

                    loop {
                        interval.tick().await;
                        tcp_frame_tx
                            .send(Arc::new(data))
                            .expect("Couldn't send tcp frame");

                        let temp_data = map_reciver
                            .recv()
                            .await
                            .expect("Couldn't recive frame from main map server loop");
                        frame_index.fetch_add(1, Ordering::Relaxed);

                        data = Self::prepare_frame(temp_data, &mut data_size)
                            .await
                            .expect("Couldn't prepare tcp frame");

                        match cmd_reciver.try_recv() {
                            Ok(msg) => {
                                if let NativeCommunication::StopRendering = msg {
                                    match cmd_reciver.recv().await {
                                        Some(msg) => match msg {
                                            NativeCommunication::StartRendering { fps } => {
                                                let dur = Duration::from_millis(
                                                    1000 as u64 / (fps as u64),
                                                );
                                                interval = time::interval(dur);
                                                continue;
                                            }
                                            NativeCommunication::StopRendering => {
                                                panic!("Invalid message! (Expected StartRendering, got: {:?})", msg);
                                            }
                                        },
                                        None => {
                                            println!("Couldn't recive JVM msg");
                                        }
                                    }
                                } else {
                                    panic!(
                                        "Invalid message! (Expected StopRendering, got: {:?})",
                                        msg
                                    );
                                }
                                break;
                            }
                            Err(_) => {}
                        };
                    }
                }
                _ => {
                    println!("[FastMapServer] You cannot stop a non working render thread! MapServer will exit!");
                }
            };
        });

        tokio::spawn(async move {
            loop {
                let (mut socket, addr) = listener
                    .accept()
                    .await
                    .expect("Couldn't accept server connection!'");
                socket.set_nodelay(true).unwrap();
                println!("GOT CONNECTION FROM: {:?}", addr);

                let mut frame_rx = tcp_frame_rx.resubscribe();

                'tcp: loop {
                    let data = match frame_rx.recv().await {
                        Ok(data) => data,
                        Err(_) => {
                            break 'tcp;
                        }
                    };

                    match socket.write_all(&data).await {
                        Ok(_) => {}
                        Err(_) => {
                            break 'tcp;
                        }
                    }
                }
            }
        });
        Ok(())
    }

    async fn prepare_frame(data: Vec<i8>, data_size: &mut usize) -> anyhow::Result<Vec<u8>> {
        let encoder_capacity: usize = match data_size {
            0 => 2048,
            _ => *data_size,
        };

        //println!("Pre compression: {}", data.len());
        let mut encoder_vec = Vec::<u8>::with_capacity(encoder_capacity);
        encoder_vec.write_u32(0).await.unwrap(); //Future TCP frame length
        let mut encoder = ZlibEncoder::new(encoder_vec, Compression::new(1));

        let mut data = mem::ManuallyDrop::new(data);
        let data = unsafe {
            let data_ptr = data.as_mut_ptr() as *mut u8;
            let data_len = data.len();
            let data_cap = data.capacity();
            Vec::from_raw_parts(data_ptr, data_len, data_cap)
        };

        encoder.write_all(&data)?;
        let mut buffer = encoder.finish()?;

        //Write len
        let mut len_vec: Vec<u8> = Vec::with_capacity(4);
        len_vec
            .write_u32(buffer.len() as u32 - 4 as u32)
            .await
            .unwrap();
        buffer[0..4].copy_from_slice(&len_vec);

        let len = buffer.len();
        *data_size = len;
        //println!("Post compression : {}", len);
        Ok(buffer)
    }

    pub fn send_message(&self, message: NativeCommunication) -> anyhow::Result<()> {
        self.command_sender.blocking_send(message)?;
        Ok(())
    }
}
