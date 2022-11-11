use core::time;
use std::sync::Arc;
use std::{ptr, thread};
use std::sync::atomic::Ordering::Relaxed;

use bytemuck::{Zeroable, Pod};
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::Error;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, channel};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{Device, DeviceCreateInfo, Features, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::shader::{SpecializationConstants, SpecializationMapEntry};
use vulkano::sync::GpuFuture;
use vulkano::{sync, VulkanLibrary};

use crate::player::player_context::{PlayerContext, VideoData, VideoPlayer};
use crate::splitting::{FRAME_SPLITTER_ALL_FRAMES_X, FRAME_SPLITTER_ALL_FRAMES_Y, self};
use crate::{colorlib, ffmpeg_set_multithreading, SplittedFrame};

use super::multi_video_player::MultiVideoPlayer;

pub struct GpuVideoPlayer {
    width: i32,
    height: i32,
    fps: i32,
    jvm_receiver: Option<Arc<Mutex<Receiver<Vec<i8>>>>>, //Receiver<Vec<i8>
    gpu_receiver: Arc<Mutex<Receiver<GpuFrameWithIdentifier>>>,
    splitted_frames: Vec<SplittedFrame>,
    runtime: Arc<Runtime>
}

struct GpuFrameWithIdentifier {
    id: i64,
    data: ffmpeg::frame::Video
}

struct GpuProcessedFrame {
    id: i64,
    data: Arc<CpuAccessibleBuffer<[i8]>>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod, PartialEq, Eq)]
pub struct GpuSplittedFrameInfo {
    pub width_start: u32,
    pub height_start: u32,
    pub width_end: u32,
    pub height_end: u32
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod, PartialEq, Eq)]
pub struct GpuShaderMetadata {
    pub all_frames_x: u32,
    pub all_frames_y: u32
}

impl GpuSplittedFrameInfo {
    pub fn from_splitted_frames(data: &Vec<SplittedFrame>) -> Vec<Self>{
        let mut out = Vec::<GpuSplittedFrameInfo>::with_capacity(data.len());

        let mut offset_x = 0;
        let mut offset_y = 0;
        let mut i = 0;

        let all_frames_x = FRAME_SPLITTER_ALL_FRAMES_X.load(Relaxed) as u32;
        let all_frames_y = FRAME_SPLITTER_ALL_FRAMES_Y.load(Relaxed) as u32;

        for y in 0..all_frames_y {
            for _ in 0..all_frames_x {
                let frame = &data[i];
                
                out.push(GpuSplittedFrameInfo{
                    width_start: offset_x,
                    height_start: offset_y,
                    width_end: offset_x + (frame.width as u32),
                    height_end: offset_y + (frame.height as u32),
                    
                });

                offset_x += frame.width as u32;
                i += 1;
            }
            offset_y += data[(y * all_frames_x) as usize].height as u32;
            offset_x = 0;
        }

        out
    }
}

    unsafe impl SpecializationConstants for GpuSplittedFrameInfo {
    fn descriptors() -> &'static [SpecializationMapEntry] {
        static DESCRIPTORS: [SpecializationMapEntry; 4] = [
            SpecializationMapEntry {
                constant_id: 0,
                offset: 0,
                size: 4,
            },
            SpecializationMapEntry {
                constant_id: 1,
                offset: 4,
                size: 4,
            },
            SpecializationMapEntry {
                constant_id: 2,
                offset: 8,
                size: 4,
            },
            SpecializationMapEntry {
                constant_id: 3,
                offset: 12,
                size: 4,
            },
        ];

        &DESCRIPTORS
    }
}

unsafe impl SpecializationConstants for GpuShaderMetadata {
    fn descriptors() -> &'static [SpecializationMapEntry] {
        static DESCRIPTORS: [SpecializationMapEntry; 2] = [
            SpecializationMapEntry {
                constant_id: 0,
                offset: 0,
                size: 4,
            },
            SpecializationMapEntry {
                constant_id: 1,
                offset: 4,
                size: 4,
            }, 
        ];

        &DESCRIPTORS
    }
}

unsafe impl Sync for GpuFrameWithIdentifier {}
unsafe impl Send for GpuFrameWithIdentifier {}

impl VideoPlayer for GpuVideoPlayer {
    fn create(file_name: String) -> anyhow::Result<PlayerContext> {
            let runtime = Builder::new_multi_thread()
                .worker_threads(6 as usize)
                .thread_name("ProjectAyaya native worker thread")
                .thread_stack_size(3840 as usize * 2160 as usize * 4) //Big stack due to memory heavy operations (4k is max resolution for now)
                .build()
                .expect("Couldn't create tokio runtime");

            let (gpu_tx, gpu_rx) = channel::<GpuFrameWithIdentifier>(100);
            let (data_tx, mut data_rx) = channel::<i32>(3);

            //ffmpeg setup
            {
                thread::spawn(move || {
                        ffmpeg::init().unwrap();

                        if let Ok(mut ictx) = input(&file_name) {
                            let input = ictx
                                .streams()
                                .best(Type::Video)
                                .ok_or(Error::StreamNotFound)
                                .expect("Couldn't find stream");

                            let video_stream_index = input.index();
                            let context_decoder =
                                ffmpeg::codec::context::Context::from_parameters(input.parameters())
                                    .expect("Couldn't create context_decoder");

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
                            )
                            .expect("Couldn't create scaler");

                            data_tx.blocking_send(fps).unwrap();
                            data_tx.blocking_send(width as i32).unwrap();
                            data_tx.blocking_send(height as i32).unwrap();

                            let mut id = 0;

                            loop {
                                let frame = MultiVideoPlayer::decode_frame(
                                    &mut ictx,
                                    video_stream_index,
                                    &mut decoder,
                                    &mut scaler,
                                )
                                .expect("Couldn't create async frame");

                                let frame_with_id = GpuFrameWithIdentifier {
                                    id: id,
                                    data: frame,
                                };

                                gpu_tx.blocking_send(frame_with_id);
                                id += 1;
                        };
                    };
                });
            };

            let fps = data_rx.blocking_recv().unwrap();
            let width = data_rx.blocking_recv().unwrap();
            let height = data_rx.blocking_recv().unwrap();

            println!("{}, {}, {}", fps, width, height);

            if width % 8 != 0 || height % 8 != 0 {
                return Err(anyhow::Error::msg(format!("The width or height is not divisible by 8! The GPU does NOT support that ({}, {})", width, height)));
            }

            let mut gpu_video_player = Self {
                width,
                height,
                fps,
                jvm_receiver: None,
                gpu_receiver: Arc::new(Mutex::new(gpu_rx)),
                splitted_frames: SplittedFrame::initialize_frames(width as i32, height as i32)?,
                runtime: Arc::new(runtime)
            };

            GpuVideoPlayer::init(&mut gpu_video_player)?;

            return Ok(PlayerContext::from_gpu_video_player(gpu_video_player));    
    }

    //Note: GPU init!!
    fn init(&mut self) -> anyhow::Result<()> {
        let (global_tx, global_rx) = channel::<Vec<i8>>(100);
        let global_rx = Arc::new(Mutex::new(global_rx));
        self.jvm_receiver = Some(global_rx);

        let gpu_receiver = self.gpu_receiver.clone();

        let len = self.width * self.height;

        let width = self.width;
        let height = self.height;
        let mut splitted_frames = self.splitted_frames.clone();

        thread::spawn(move || {
            let jvm_sender = global_tx.clone();

            let library = VulkanLibrary::new().unwrap();
            let layers = library.layer_properties().unwrap();
            for l in layers {
                println!("\t{}", l.name());
            }
            let instance = Instance::new(
                library,
                InstanceCreateInfo {
                    enabled_extensions: InstanceExtensions {
                        ext_debug_utils: true,
                        //ext_validation_features: true, $VALID
                        ..InstanceExtensions::empty()
                    },
                    // Enable enumerating devices that use non-conformant vulkan implementations. (ex. MoltenVK)
                    enumerate_portability: true,
                    //enabled_validation_features: vec![ValidationFeatureEnable::DebugPrintf], $VALID
                    //enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_string()], $VALID
                    ..Default::default()
                },
            )
            .expect("failed to create Vulkan instance");

            let device_extensions = vulkano::device::DeviceExtensions {
                khr_storage_buffer_storage_class: true,
                khr_shader_float16_int8: true,
                khr_8bit_storage: true,
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
                })
                .unwrap();

            let (device, mut queues) = Device::new(
                physical_device,
                DeviceCreateInfo {
                    queue_create_infos: vec![QueueCreateInfo {
                        queue_family_index,
                        ..Default::default()
                    }],
                    enabled_extensions: device_extensions,
                    enabled_features: Features {
                        shader_int8: true,
                        uniform_and_storage_buffer8_bit_access: true,
                        ..Features::empty()
                    },
                    ..Default::default()
                },
            )
            .expect("Couldn't create deceive");

            let queue = queues.next().unwrap();

            let cache_slice = colorlib::CONVERSION_TABLE;
            let cache_slice: &[i8] = bytemuck::cast_slice(cache_slice);
            let size = cache_slice.len();

            let cache_temp_buffer = unsafe {
                CpuAccessibleBuffer::<[i8]>::uninitialized_array(
                    device.clone(),
                    (size as u64) as vulkano::DeviceSize,
                    BufferUsage {
                        transfer_src: true,
                        ..Default::default()
                    },
                    false,
                )
                .expect("Couldn't alloc temp cache buffer")
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
            )
            .expect("Couldn't alloc cache buffer");


            let all_frames_x = FRAME_SPLITTER_ALL_FRAMES_X.load(Relaxed) as u32;
            let all_frames_y = FRAME_SPLITTER_ALL_FRAMES_Y.load(Relaxed) as u32;

            let index_data = splitting::generate_index_cache(width as u32, height as u32, splitted_frames);
            let index_len = index_data.len();
            let index_temp_cache_buffer = CpuAccessibleBuffer::<[u32]>::from_iter(
                device.clone(),
                BufferUsage {
                    transfer_src: true,
                    ..BufferUsage::empty()
                },
                false,
                index_data 
            ).expect("Couldn't create gpu info buffer!");

            let index_buffer = DeviceLocalBuffer::<[u32]>::array(
                device.clone(),
                (index_len as u64) as vulkano::DeviceSize,
                BufferUsage {
                    storage_buffer: true,
                    transfer_dst: true,
                    ..BufferUsage::empty()
                }, // Specify use as a storage buffer and transfer destination.
                device.active_queue_family_indices().iter().copied(),
            )
            .expect("Couldn't alloc cache buffer");

            let mut write = cache_temp_buffer.write().expect("Couldn't do a write lock");
            write.copy_from_slice(cache_slice);
            drop(write);

            let command_allocator = StandardCommandBufferAllocator::new(device.clone());

            let mut builder = AutoCommandBufferBuilder::primary(
                &command_allocator,
                queue_family_index,
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            builder
                .copy_buffer(CopyBufferInfo::buffers(
                    cache_temp_buffer.clone(),
                    cache_buffer.clone(),
                ))
                .expect("Couldn't copy cache buffer (2)")
                .copy_buffer(CopyBufferInfo::buffers(
                    index_temp_cache_buffer.clone(),
                    index_buffer.clone(),
                ))
                .expect("Couldn't copy cache buffer (3)");


            let command_buffer = builder.build().unwrap();

            let future = sync::now(device.clone())
                .then_execute(queue.clone(), command_buffer)
                .unwrap()
                .then_signal_fence_and_flush()
                .unwrap();

            future.wait(None).unwrap();

            let mut builder = AutoCommandBufferBuilder::primary(
                &command_allocator,
                queue_family_index,
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            //let gpu_receive = gpu_receiver;
            let mut gpu_process_vec = Vec::<GpuProcessedFrame>::with_capacity(128);
            let mut frame_id = 0;

            let mut gpu_receiver = gpu_receiver.blocking_lock();

            loop {
                if frame_id == 64 {
                    println!("G!");
                    // let command_buffer = builder.build().unwrap();

                    // let future = sync::now(device.clone())
                    //     .then_execute(queue.clone(), command_buffer)
                    //     .unwrap()
                    //     .then_signal_fence_and_flush()
                    //     .unwrap();

                    // future.wait(None).unwrap();

                    // let data = &*output_data_buffer.read().unwrap();
                    // let splitting = SplittedFrame::split_frames(data, &mut splitted_frames, width)
                    //     .expect("Couldn't split frames async");

                    // jvm_sender.send(splitting).expect("JVM SEND ERROR");

                    let command_buffer = builder.build().unwrap();

                    let now = std::time::Instant::now();

                    let future = sync::now(device.clone())
                        .then_execute(queue.clone(), command_buffer)
                        .unwrap()
                        .then_signal_fence_and_flush()
                        .unwrap();

                    future.wait(None).unwrap();

                    let elapsed = now.elapsed();
                    println!("Elapsed: {:.2?}", elapsed);

                    for frame in gpu_process_vec.drain(..) {
                        let data = &*frame.data.read().unwrap();
                        jvm_sender.blocking_send(Vec::from(data)).expect("JVM SEND ERROR");
                    }

                    builder = AutoCommandBufferBuilder::primary(
                        &command_allocator,
                        queue_family_index,
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .unwrap();
                    //New buffer due to move of previous one above

                    frame_id = 0;
                    continue;
                };

                let frame = gpu_receiver.blocking_recv().unwrap();

                let frame_buffer = unsafe {
                    CpuAccessibleBuffer::<[u8]>::uninitialized_array(
                        device.clone(),
                        ((len * 3) as u64) as vulkano::DeviceSize,
                        BufferUsage {
                            storage_buffer: true,
                            ..Default::default()
                        },
                        false,
                    )
                    .expect("Couldn't alloc temp cache buffer")
                };

                let output_data_buffer = unsafe {
                    CpuAccessibleBuffer::<[i8]>::uninitialized_array(
                        device.clone(),
                        (len as u64) as vulkano::DeviceSize,
                        BufferUsage {
                            storage_buffer: true,
                            transfer_src: true,
                            ..Default::default()
                        },
                        false,
                    )
                    .expect("Couldn't alloc temp cache buffer")
                };

                let mut write = frame_buffer.write().expect("Couldn't do a write lock");
                unsafe {
                    let dst_ptr: *mut u8 = write.as_mut_ptr();
                    let src_ptr = frame.data.data(0).as_ptr();
                    ptr::copy(src_ptr, dst_ptr, (len * 3) as usize);
                    //The "safe rust way" caused a segfault - mem safety is not desireable
                }
                drop(write);

                let shader = cs::load(device.clone()).expect("failed to create shader module");
                let compute_pipeline = ComputePipeline::new(
                    device.clone(),
                    shader.entry_point("main").unwrap(),
                    &(),
                    None,
                    |_| {},
                )
                .expect("failed to create compute pipeline");

                let descriptor_allocator = StandardDescriptorSetAllocator::new(device.clone());
                
                let layout = compute_pipeline.layout().set_layouts().get(0).unwrap();
                let set = PersistentDescriptorSet::new(
                    &descriptor_allocator,
                    layout.clone(),
                    [
                        WriteDescriptorSet::buffer(0, output_data_buffer.clone()),
                        WriteDescriptorSet::buffer(1, frame_buffer.clone()),
                        WriteDescriptorSet::buffer(2, cache_buffer.clone()),
                        WriteDescriptorSet::buffer(3, index_buffer.clone()),
                    ], // 0 is the binding
                )
                .unwrap();

                builder
                    .bind_pipeline_compute(compute_pipeline.clone())
                    .bind_descriptor_sets(
                        PipelineBindPoint::Compute,
                        compute_pipeline.layout().clone(),
                        0, // 0 is the index of our set
                        set,
                    )
                    .dispatch([((width / 16) as u32), ((height / 16) as u32), 1])
                    .unwrap();

                gpu_process_vec.push(GpuProcessedFrame {
                    id: frame.id,
                    data: output_data_buffer.clone(),
                });

                frame_id += 1;
            }
        });

        println!("ABOUT TO QUIT!");
        Ok(())
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        let mut reciver = self.jvm_receiver.as_ref().unwrap();
        let mut reciver = reciver.blocking_lock();

        return Ok(reciver.blocking_recv().unwrap()); 
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

////1. output_data_buffer 2.frame_buffer 3.cache_buffer
mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        src: r#"
#version 450
//#extension GL_EXT_debug_printf : enable
#extension GL_EXT_shader_8bit_storage : enable
#extension GL_EXT_shader_explicit_arithmetic_types_int8 : enable

layout(local_size_x = 16, local_size_y = 16, local_size_z = 1) in;
layout(set = 0, binding = 0) buffer Data {
        int8_t data[];
    } buf;
layout(set = 0, binding = 1) buffer Frame {
    uint8_t data[];
} frame;
layout(set = 0, binding = 2) buffer Cache {
    uint8_t data[];
} cache;
layout(set = 0, binding = 3) buffer IndexCache {
    uint data[];
} index;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    uint idy = gl_GlobalInvocationID.y;

    uint width = gl_NumWorkGroups.x * gl_WorkGroupSize.x;
    uint height = gl_NumWorkGroups.y * gl_WorkGroupSize.y;

    uint8_t r = frame.data[(idy * width * 3) + (idx * 3)];
    uint8_t g = frame.data[(idy * width * 3) + (idx * 3) + 1];
    uint8_t b = frame.data[(idy * width * 3) + (idx * 3) + 2];

    buf.data[index.data[(idy * width) + idx]] = int8_t(cache.data[(r * 256 * 256) + (g * 256) + b]);
    //buf.data[(idy * width) + idx] = int8_t(cache.data[(r * 256 * 256) + (g * 256) + b]);
}

"#
    }
}

// let _debug_callback = unsafe {
//     DebugUtilsMessenger::new(
//         instance.clone(),
//         DebugUtilsMessengerCreateInfo {
//             message_severity: DebugUtilsMessageSeverity {
//                 error: true,
//                 warning: true,
//                 information: true,
//                 verbose: true,
//                 ..DebugUtilsMessageSeverity::empty()
//             },
//             message_type: DebugUtilsMessageType {
//                 general: true,
//                 validation: true,
//                 performance: true,
//                 ..DebugUtilsMessageType::empty()
//             },
//             ..DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|msg| {
//                 let severity = if msg.severity.error {
//                     "error"
//                 } else if msg.severity.warning {
//                     "warning"
//                 } else if msg.severity.information {
//                     "information"
//                 } else if msg.severity.verbose {
//                     "verbose"
//                 } else {
//                     panic!("no-impl");
//                 };

//                 let ty = if msg.ty.general {
//                     "general"
//                 } else if msg.ty.validation {
//                     "validation"
//                 } else if msg.ty.performance {
//                     "performance"
//                 } else {
//                     panic!("no-impl");
//                 };

//                 println!(
//                     "{} {} {}: {}",
//                     msg.layer_prefix.unwrap_or("unknown"),
//                     ty,
//                     severity,
//                     msg.description
//                 );
//             }))
//         },
//     )
//     .ok() };
