//variables:
// nT = number of threes in the thread pool
// wnT = worker number threads (worker thread = transcoding thread) = nT - 1
// vec_results = vector of transcoded frames oneshot channels (size = wnT)
// mpsc = mpsc::channel(capacity);
//
//functions:
//
//ffmpeg(i: usize):
    //use FFMPEG (video_player) to get raw ffmpeg frame (let raw_frame)
    //create oneshot channel (let oneshot)
    //create task to transcode frame and send it to oneshot channel
    //set vec_results[i] = oneshot
//
//Execution:
//New blocking task:
//
//loop wnT times (i = iteration):
//call ffmpeg function (argument i = i) (see above)
//exit loop
//
//loop forever:
    //loop wnT times (i = iteration):
    //get oneshot channel (let oneshot) from vector (vec_results[i])
    //await this channel (let result)
    //put result into mpsc channel
    //call ffmpeg function (argument i = i)

use std::mem::ManuallyDrop;
use std::sync::{Arc};
use std::thread;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{mpsc, Mutex, oneshot};
use tokio::task::JoinHandle;
use crate::colorlib::transform_frame_to_mc;
use crate::VideoPlayer;

pub struct ThreadedVideoPlayer{
    thread_pool_size: i32,
    width: u32,
    height: u32,
    runtime: Runtime,
    receiver: mpsc::Receiver<Vec<i8>>,
    sender: mpsc::Sender<Vec<i8>>
}

impl ThreadedVideoPlayer {
   pub fn new(
       width: u32,
       height: u32,
       thread_pool_size: i32
   ) -> Self{
        let runtime = Builder::new_multi_thread()
            .worker_threads(thread_pool_size as usize)
            .thread_name("ProjectAyaya native worker thread")
            .thread_stack_size((thread_pool_size as usize * width as usize * height as usize * 8) as usize) //Big stack due to memory heavy operations
            .build()
            .expect("Couldn't create tokio runtime");

       let (tx, rx) = mpsc::channel::<Vec<i8>>(50);

       Self{
           thread_pool_size,
           width,
           height,
           runtime,
           receiver: rx,
           sender: tx
       }
   }

    pub fn start(self, player: ManuallyDrop<Box<VideoPlayer>>) -> anyhow::Result<()>{
        if(true){
            thread::spawn(|| {
               println!("THREADED");

                thread::sleep(Duration::from_secs(30))
            });
        }
        // println!("START");
        // let runtime = &self.runtime;
        // println!("RT");
        // let thread_pool_size = self.thread_pool_size;
        // println!("TP");
        //
        // let player_arc = Arc::new(Mutex::new(player));
        // println!("ARC");
        //
        // let sender = self.sender;
        // println!("SENDER");
        // let sender_arc = Arc::new(Mutex::new(sender));
        // println!("SENDER ARC");
        //
        // let result: JoinHandle<Result<(), anyhow::Error>> = runtime.spawn(async move {
        //
        //     println!("ENTER RE");
        //
        //     let mut frames_channels: Vec<oneshot::Receiver<Vec<i8>>> = Vec::new();
        //
        //     let mut player = player_arc.lock().await;
        //
        //     for _ in 0..thread_pool_size {
        //
        //         println!("loop en");
        //
        //         let frame = player.decode_frame()?;
        //
        //         println!("DEC");
        //
        //         let width = self.width;
        //         let height = self.height;
        //
        //         let (tx, rx) = oneshot::channel::<Vec<i8>>();
        //         frames_channels.push(rx);
        //
        //
        //         tokio::spawn(async move {
        //             let vec = transform_frame_to_mc(frame.data(0), width, height);
        //             tx.send(vec)
        //         });
        //     };
        //
        //     loop {
        //         for i in 0..thread_pool_size {
        //             let rx = &mut frames_channels[i as usize];
        //             let result = rx.await?;
        //
        //             let global_sender = sender_arc.lock().await;
        //             global_sender.send(result).await?;
        //             drop(global_sender);
        //
        //             let frame = player.decode_frame()?;
        //
        //             let width = self.width;
        //             let height = self.height;
        //
        //             let (tx, rx) = oneshot::channel::<Vec<i8>>();
        //             frames_channels.insert(i as usize, rx);
        //
        //
        //             tokio::spawn(async move {
        //                 let vec = transform_frame_to_mc(frame.data(0), width, height);
        //                 tx.send(vec)
        //             });
        //         }
        //
        //     }
        // });
        //
        // println!("AFT");

        return Ok(())
    }

    pub fn get_frame(mut self) -> Vec<i8>{
        self.receiver.blocking_recv().unwrap()
    }
}