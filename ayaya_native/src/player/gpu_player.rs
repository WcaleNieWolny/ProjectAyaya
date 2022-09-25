use ffmpeg::decoder::Video;
use ffmpeg::Error;
use ffmpeg::Error::Eof;
use ffmpeg::format::{input, Pixel};
use ffmpeg::format::context::Input;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage, DeviceLocalBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo};
use vulkano::device::{DeviceCreateInfo, QueueCreateInfo, Device};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::instance::Instance;
use vulkano::sync::GpuFuture;
use vulkano::{VulkanLibrary, DeviceSize, sync};

use crate::{ffmpeg_set_multithreading, SplittedFrame, colorlib};
use crate::colorlib::transform_frame_to_mc;
use crate::player::player_context::{PlayerContext, receive_and_process_decoded_frames, VideoData, VideoPlayer};

pub struct GpuVideoPlayer {
    video_stream_index: usize,
    scaler: Context,
    input: Input,
    decoder: Video,
    splitted_frames: Vec<SplittedFrame>,
    width: u32,
    height: u32,
    fps: i32,
}

impl VideoPlayer for GpuVideoPlayer {
    fn create(file_name: String) -> anyhow::Result<PlayerContext> {
        ffmpeg::init()?;

        if let Ok(ictx) = input(&file_name) {
            let input = ictx
                .streams()
                .best(Type::Video)
                .ok_or(Error::StreamNotFound)?;

            let video_stream_index = input.index();

            let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;

            let mut decoder = context_decoder.decoder();
            ffmpeg_set_multithreading(&mut decoder, file_name);

            let decoder = decoder.video()?;

            let width = decoder.width();
            let height = decoder.height();

            let fps = input.rate().0 / input.rate().1;

            let scaler = Context::get(
                decoder.format(),
                width,
                height,
                Pixel::RGB24,
                width,
                height,
                Flags::BILINEAR,
            )?;

            let mut gpu_video_player = Self {
                video_stream_index,
                scaler,
                input: ictx,
                decoder,
                splitted_frames: SplittedFrame::initialize_frames(width as i32, height as i32)?,
                width,
                height,
                fps,
            };

            //Go to the GPU side of things
            let (queue, device, cache_buffer, queue_family_index) = {
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
                )?;

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
                    )?
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
                )?;

                // let data_content: Vec<i32> = (0..size).map(|_| 0).collect();
                // let data_buffer =
                // CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage {
                //     storage_buffer: true,
                //     ..Default::default()
                // }, true, data_content)?;

                //This will be build when submiting a frame

                let mut write = cache_temp_buffer.write()?;
                write.copy_from_slice(cache_slice);
                drop(write);

                let mut builder = AutoCommandBufferBuilder::primary(
                    device.clone(),
                    queue_family_index,
                    CommandBufferUsage::OneTimeSubmit,
                )?;

                builder
                    .copy_buffer(CopyBufferInfo::buffers(
                        cache_temp_buffer,
                        cache_buffer.clone(),
                    )).expect("Couldn't copy cache buffer (2)");

                let command_buffer = builder.build().unwrap();

                let future = sync::now(device.clone())
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_signal_fence_and_flush()
                    .unwrap();

                future.wait(None).unwrap();

                (queue, device, cache_buffer, queue_family_index)

            };

            GpuVideoPlayer::init(&mut gpu_video_player)?;

            return Ok(PlayerContext::from_gpu_video_player(gpu_video_player));
        };

        return Err(anyhow::Error::new(Error::StreamNotFound))
    }

    fn init(&mut self) -> anyhow::Result<()> {
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
        todo!()
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