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

use tokio::runtime::{Builder, Runtime};
use tokio::sync::{mpsc, oneshot};
use crate::VideoPlayer;

pub struct ThreadedVideoPlayer{
    thread_pool_size: i32,
    width: i32,
    height: i32,
    runtime: Runtime,
    receiver: mpsc::Receiver<Vec<i8>>,
    sender: mpsc::Sender<Vec<i8>>
}

impl ThreadedVideoPlayer {
   pub fn new(
       width: i32,
       height: i32,
       thread_pool_size: i32
   ) -> Self{
        let runtime = Builder::new_multi_thread()
            .worker_threads(thread_pool_size as usize)
            .thread_name("ProjectAyaya native worker thread")
            .thread_stack_size((thread_pool_size as usize * width as usize * height as usize * 8) as usize) //Big stack due to memory heavy operations
            .build()
            .expect("Couldn't create tokio runtime");

       let (tx, rx) = mpsc::channel::<Vec<i8>>(1);

       Self{
           thread_pool_size,
           width,
           height,
           runtime,
           receiver: rx,
           sender: tx
       }
   }

    pub fn start(self, player: &mut VideoPlayer){
        let runtime = self.runtime;
        let (tx, rx) = oneshot::channel::<Runtime>();

        runtime.spawn(async move {
            let runtime = rx.await.expect("Couldn't recive runtime from parrent thread");


        });

        tx.send(runtime).expect("Couldn't send runtime to manager task");
    }
}