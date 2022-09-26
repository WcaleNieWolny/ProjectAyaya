use core::time;
use std::cell::{RefCell, Cell};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, mpsc, Mutex};
use std::thread;

use ffmpeg::decoder::Video;
use ffmpeg::Error;
use ffmpeg::Error::Eof;
use ffmpeg::format::{input, Pixel};
use ffmpeg::format::context::Input;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage, DeviceLocalBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::{DeviceCreateInfo, QueueCreateInfo, Device};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::instance::Instance;
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::sync::GpuFuture;
use vulkano::{VulkanLibrary, DeviceSize, sync};

use crate::{ffmpeg_set_multithreading, SplittedFrame, colorlib};
use crate::colorlib::transform_frame_to_mc;
use crate::player::player_context::{PlayerContext, receive_and_process_decoded_frames, VideoData, VideoPlayer};

use super::multi_video_player::{MultiVideoPlayer};

pub struct GpuVideoPlayer {
    width: i32,
    height: i32,
    fps: i32,
    jvm_receiver: Arc<Mutex<tokio::sync::mpsc::Receiver<Vec<i8>>>>,
    jvm_sender: Arc<Mutex<tokio::sync::mpsc::Sender<Vec<i8>>>>,
    gpu_reciver: Arc<Mutex<tokio::sync::mpsc::Receiver<GpuFrameWithIdentifier>>>,
    splitted_frames: Vec<SplittedFrame>,
}

struct GpuFrameWithIdentifier {
    id: i64,
    data: ffmpeg::frame::Video,
}

unsafe impl Sync for GpuFrameWithIdentifier {}
unsafe impl Send for GpuFrameWithIdentifier {}

impl VideoPlayer for GpuVideoPlayer {
    fn create(file_name: String) -> anyhow::Result<PlayerContext> {
        ffmpeg::init()?;

        if let Ok(mut ictx) = input(&file_name) {

            let (global_tx, global_rx) = tokio::sync::mpsc::channel::<Vec<i8>>(100);
            let (gpu_tx,gpu_rx) = tokio::sync::mpsc::channel::<GpuFrameWithIdentifier>(100);
            let (data_tx, data_rx) = mpsc::sync_channel::<i32>(3);

            //ffmpeg setup
            {
                thread::spawn(move || {
                    let input = ictx
                    .streams()
                    .best(Type::Video)
                    .ok_or(Error::StreamNotFound).expect("Couldn't find stream");
    
                    let video_stream_index = input.index();
    
                    let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters()).expect("Couldn't create context_decoder");
    
                    let mut decoder = context_decoder.decoder();
                    ffmpeg_set_multithreading(&mut decoder, file_name);
    
                    let mut decoder = decoder.video().expect("Couldn't get decoder");
    
                    let width = decoder.width();
                    let height = decoder.height();
    
                    let fps = input.rate().0 / input.rate().1;
    
                    let mut scaler = Context::get(
                        decoder.format(),
                        width,
                        height,
                        Pixel::RGB24,
                        width,
                        height,
                        Flags::BILINEAR,
                    ).expect("Couldn't create scaler");
    
                    data_tx.send(fps).unwrap();
                    data_tx.send(width as i32).unwrap();
                    data_tx.send(height as i32).unwrap();
    
                    let mut id = 0;

                    loop {    
                        let frame = MultiVideoPlayer::decode_frame(&mut ictx, video_stream_index, &mut decoder, &mut scaler).expect("Couldn't create async frame");

                        let frame_with_id = GpuFrameWithIdentifier{
                            id: id,
                            data: frame,
                        };
                        gpu_tx.blocking_send(frame_with_id);
                    }
                });
            };

            let fps = data_rx.recv().unwrap();
            let width = data_rx.recv().unwrap();
            let height = data_rx.recv().unwrap();

            if width % 8 != 0 || height % 8 != 0 {
                return Err(anyhow::Error::msg(format!("The width or height is not divisible by 8! The GPU does NOT support that ({}, {})", width, height)))
            }

            let mut gpu_video_player = Self {
                width,
                height,
                fps,
                jvm_sender: Arc::new(Mutex::new(global_tx)),
                jvm_receiver: Arc::new(Mutex::new(global_rx)),
                gpu_reciver: Arc::new(Mutex::new(gpu_rx)),
                splitted_frames: SplittedFrame::initialize_frames(width as i32, height as i32)?
            };

            GpuVideoPlayer::init(&mut gpu_video_player)?;

            return Ok(PlayerContext::from_gpu_video_player(gpu_video_player));
        };

        return Err(anyhow::Error::new(Error::StreamNotFound))
    }

    //Note: GPU init!!
    fn init(&mut self) -> anyhow::Result<()> {

        let gpu_reciver = self.gpu_reciver.clone();
        let gpu_sender = self.jvm_sender.clone();
        let len = self.width * self.height;

        let width = self.width;
        let height = self.height;
        let mut splited_frames = self.splitted_frames.clone();

        thread::spawn(move || {
            let library = VulkanLibrary::new().unwrap();
            let instance = Instance::new(library, Default::default()).expect("failed to create instance");

            let device_extensions = vulkano::device::DeviceExtensions {
                khr_storage_buffer_storage_class: true,
                ..vulkano::device::DeviceExtensions::empty()
            };

            let (physical_device, queue_family_index) = instance
                .enumerate_physical_devices()
                .unwrap()
                .filter(|p| p.supported_extensions().contains(&device_extensions))
                .filter_map(|p| {
                    p.queue_family_properties()
                        .iter()
                        .position(|q| q.queue_flags.compute)
                        .map(|i| (p, i as u32))
                })
                .min_by_key(|(p, _)| match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                    _ => 5,
                }).unwrap();

            println!(
                "[GPU SCREEN RENDER] Using device: {} (type: {:?}), mem: {:?}",
                physical_device.properties().device_name,
                physical_device.properties().device_type,
                physical_device.memory_properties()
            );

            let (device, mut queues) = Device::new(
                physical_device,
                DeviceCreateInfo {
                    queue_create_infos: vec![QueueCreateInfo {
                        queue_family_index,
                        ..Default::default()
                    }],
                    enabled_extensions: device_extensions,
                    ..Default::default()
                },
            ).expect("Couldn't create decive");

            let queue = queues.next().unwrap();

            let cache_slice: &[i8] = bytemuck::cast_slice(colorlib::CONVERSION_TABLE);
            let size = cache_slice.len();

            let cache_temp_buffer = unsafe {
                CpuAccessibleBuffer::<[i8]>::uninitialized_array(
                    device.clone(),
                    (size as u64) as vulkano::DeviceSize,
                    BufferUsage {
                        transfer_src: true,
                        ..Default::default()
                    },
                    false
                ).expect("Couldn't alloc temp cache buffer")
            };

            let cache_buffer = DeviceLocalBuffer::<[i8]>::array(
                device.clone(),
                (size as u64) as vulkano::DeviceSize,
                BufferUsage {
                    storage_buffer: true,
                    transfer_dst: true,
                    ..BufferUsage::empty()
                }, // Specify use as a storage buffer and transfer destination.
                device.active_queue_family_indices().iter().copied(),
            ).expect("Couldn't alloc cache buffer");

            // let data_content: Vec<i32> = (0..size).map(|_| 0).collect();
            // let data_buffer =
            // CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage {
            //     storage_buffer: true,
            //     ..Default::default()
            // }, true, data_content)?;

            //This will be build when submiting a frame

            let mut write = cache_temp_buffer.write().expect("Couldn't do a write lock");
            write.copy_from_slice(cache_slice);
            drop(write);

            let mut size = 0;
            let mut gpu_recive = gpu_reciver.lock().unwrap();
            let mut jvm_send = gpu_sender.lock().unwrap();
            loop {
                
                let frame = gpu_recive.blocking_recv().unwrap();
                let data = frame.data;
                size += 1;

                let frame_buffer = unsafe {
                    CpuAccessibleBuffer::<[u8]>::uninitialized_array(
                        device.clone(),
                        ((len * 3) as u64) as vulkano::DeviceSize,
                        BufferUsage {
                            transfer_src: true,
                            ..Default::default()
                        },
                        false
                    ).expect("Couldn't alloc temp cache buffer")
                };

                let output_data_buffer = unsafe {
                    CpuAccessibleBuffer::<[i8]>::uninitialized_array(
                        device.clone(),
                        (len as u64) as vulkano::DeviceSize,
                        BufferUsage {
                            transfer_src: true,
                            ..Default::default()
                        },
                        false
                    ).expect("Couldn't alloc temp cache buffer")
                };

                frame_buffer.write().unwrap().copy_from_slice(data.data(0));

                let shader = cs::load(device.clone()).expect("failed to create shader module");
                let compute_pipeline = ComputePipeline::new(
                    device.clone(),
                    shader.entry_point("main").unwrap(),
                    &(),
                    None,
                    |_| {},
                )
                .expect("failed to create compute pipeline");

                let layout = compute_pipeline.layout().set_layouts().get(0).unwrap();
                let set = PersistentDescriptorSet::new(
                    layout.clone(),
                    [
                        WriteDescriptorSet::buffer(0, output_data_buffer.clone()),
                        WriteDescriptorSet::buffer(1, frame_buffer.clone()),
                        WriteDescriptorSet::buffer(2, cache_buffer.clone())
                    ], // 0 is the binding
                )
                .unwrap();

                let mut builder = AutoCommandBufferBuilder::primary(
                    device.clone(),
                    queue_family_index,
                    CommandBufferUsage::OneTimeSubmit,
                ).unwrap();

                builder
                    .copy_buffer(CopyBufferInfo::buffers(
                        cache_temp_buffer.clone(),
                        cache_buffer.clone(),
                    )).expect("Couldn't copy cache buffer (2)")
                    .bind_pipeline_compute(compute_pipeline.clone())
                    .bind_descriptor_sets(
                        PipelineBindPoint::Compute,
                        compute_pipeline.layout().clone(),
                        0, // 0 is the index of our set
                        set,
                    )
                    .dispatch([((width / 8) as u32), ((height / 8) as u32), 1])
                    .unwrap();


                let command_buffer = builder.build().unwrap();

                let future = sync::now(device.clone())
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_signal_fence_and_flush()
                    .unwrap();

                future.wait(None).unwrap();

                let data = &*output_data_buffer.read().unwrap();
                let splitting = SplittedFrame::split_frames(data, &mut splited_frames, width).expect("Couldn't split frames async");

                jvm_send.blocking_send(splitting).unwrap();
            };
        });
        Ok(())
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        // while let Some((stream, packet)) = self.input.packets().next() {
        //     if stream.index() == self.video_stream_index as usize {
        //         self.decoder.send_packet(&packet)?;
        //         let frame_data = receive_and_process_decoded_frames(&mut self.decoder, &mut self.scaler, &packet)?;
        //         let transformed_frame = transform_frame_to_mc(frame_data.data(0), self.width, self.height);
        //
        //         let transformed_frame = SplittedFrame::split_frames(transformed_frame, &mut self.splitted_frames, self.width as i32)?;
        //
        //         return Ok(transformed_frame);
        //     }
        // };
        //
        // Err(anyhow::Error::new(Eof))
        return loop {
            let frame = self.jvm_receiver.lock().unwrap().try_recv();
            if frame.is_ok() {
                break Ok(frame.unwrap());
            } else {
                thread::sleep(time::Duration::from_millis(3));
            }
        };
    }

    fn video_data(&self) -> anyhow::Result<VideoData> {
        Ok(VideoData {
            width: self.width as i32,
            height: self.height as i32,
            fps: self.fps,
        })
    }

    fn destroy(self) -> anyhow::Result<()> {
        todo!()
    }
}

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        src: "
#version 450
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
layout(set = 0, binding = 0) buffer Data {
int data[];
} buf;
layout(set = 0, binding = 1) buffer Frame {
    uint data[];
} frame;
layout(set = 0, binding = 2) buffer Cache {
uint data[];
} cache;
void main() {
uint idx = gl_GlobalInvocationID.x;
uint idy = gl_GlobalInvocationID.y;

uint width = gl_NumWorkGroups.x * gl_WorkGroupSize.x;

uint r = frame.data[(idy * width * 3) + (idx * 3)];
uint g = frame.data[(idy * width * 3) + (idx * 3) + 1];
uint b = frame.data[(idy * width * 3) + (idx * 3) + 2];

buf.data[(idy * width) + idx] = int(cache.data[(r * 256 * 256) + (g * 256) + b]);
}"
    }
}