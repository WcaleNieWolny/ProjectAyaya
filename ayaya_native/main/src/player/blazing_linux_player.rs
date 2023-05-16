use std::sync::mpsc::{channel, Receiver, Sender, sync_channel};

use anyhow::anyhow;
use ffmpeg::decoder::Video;
use ffmpeg::format::context::Input;
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::Error::Eof;
use ffmpeg::{rescale, Error, Rescale};

use crate::colorlib::{transform_frame_to_mc, fast_frame_to_mc};
use crate::map_server::ServerOptions;
use crate::player::player_context::{receive_and_process_decoded_frames, VideoData, VideoPlayer};
use crate::{ffmpeg_set_multithreading, SplittedFrame, TOKIO_RUNTIME};

use super::player_context;

static MAP_PACKET_ID: u8 = 0x27u8;

fn write_var_int(mut value: i32) -> (usize, [u8; 5]) {
    let mut output = [0u8; 5];
    let mut i = 0usize;

    loop {
        if (value & !0x7F) == 0 {
            output[i] = (value as u8);
            i += 1;
            return (i, output);
        }

        output[i] = (((value & 0x7F) | 0x80) as u8).to_be();
        i += 1;

        // Note: >>> means that the sign bit is shifted with the rest of the number rather than being left alone
        value >>= 7;
    }
}

//This struct skips some fields, as they will always be constant!
struct MinecraftMapPacket {
    map_id: i32,
    columns: u8,
    rows: u8,
    x: u8,
    z: u8,
    data: Vec<i8>
}

struct CompressionRange {
    start: usize,
    end: usize,
    width: usize,
    height: usize,
}

pub struct LinuxBlazingPlayer {
    width: i32,
    height: i32,
    fps: i32,
    frames_reciver: Receiver<Vec<i8>>
}

impl VideoPlayer for LinuxBlazingPlayer {
    fn create(file_name: String, server_options: ServerOptions) -> anyhow::Result<Self> {
        if server_options.use_server {
            return Err(anyhow!("Single video player does not support map server"));
        }

        let (prefix, file_name) = file_name.split_once("$$$").ok_or_else(|| anyhow!("Expected filename format START_MAP_ID$$$FILE_NAME"))?;
        let start_map_id: i32 = prefix.parse()?;
        let file_name = file_name.to_owned();

        ffmpeg::init()?;

        if let Ok(mut ictx) = input(&file_name) {
            let input = ictx
                .streams()
                .best(Type::Video)
                .ok_or(Error::StreamNotFound)?;

            let video_stream_index = input.index();

            let context_decoder =
                ffmpeg::codec::context::Context::from_parameters(input.parameters())?;

            let mut decoder = context_decoder.decoder();
            ffmpeg_set_multithreading(&mut decoder, file_name);

            let mut decoder = decoder.video()?;

            let width = decoder.width();
            let height = decoder.height();

            let fps = input.rate().0 / input.rate().1;

            let (splitted_frames, all_frames_x, all_frames_y) =
                SplittedFrame::initialize_frames(width as usize, height as usize)?;

            let (frame_tx, frame_rx) = sync_channel::<Vec<i8>>(90);

            let mem_cpy_ranges = SplittedFrame::prepare_external_ranges(&splitted_frames, width as usize, height as usize, all_frames_x, all_frames_y)?;
            let compression_ranges = prepare_compression_ranges(&splitted_frames);
            let mut prev_data: Option<Vec<u8>> = None;

            TOKIO_RUNTIME.spawn_blocking(move || {
                let mut scaler = Context::get(
                    decoder.format(),
                    width,
                    height,
                    Pixel::RGB24,
                    width,
                    height,
                    Flags::BILINEAR,
                ).expect("Cannot create scaler");

                while let Some((stream, packet)) = ictx.packets().next() {
                    if stream.index() == video_stream_index {
                        decoder.send_packet(&packet).expect("Cannot send packet!");
                        let frame_data = receive_and_process_decoded_frames(
                            &mut decoder,
                            &mut scaler,
                            &packet,
                        ).expect("Cannot recive and process decoded frame!");

                        let transformed_frame = fast_frame_to_mc(
                            frame_data.data(0),
                            width as usize,
                            height as usize,
                            frame_data.stride(0),
                        );

                        let transformed_frame = SplittedFrame::unsafe_split_frames(
                            &transformed_frame,
                            &mem_cpy_ranges,
                            width as usize,
                            height as usize,
                        ).expect("Cannot perform unsafe frame splitting");

                        let mut prev_frame_i = 0usize;

                        //Final len so we do not have to realloc (27 is a magic val, see MinecraftMapPacket code below) 
                        let mut final_frame = Vec::<u8>::with_capacity((width as usize * height as usize) + (27 * splitted_frames.len()));
                        splitted_frames
                            .iter()
                            .enumerate()
                            .map(|(i, frame)| {
                                let packet = MinecraftMapPacket {
                                    map_id: start_map_id + i as i32,
                                    columns: frame.width as u8,
                                    rows: frame.height as u8,
                                    x: 0, //For now 0
                                    z: 0, //for not 0
                                    data: transformed_frame[prev_frame_i..][..frame.frame_length].to_vec(),
                                };
                                prev_frame_i += frame.frame_length;
                                packet
                            })
                            .try_for_each(|packet| -> anyhow::Result<()> {
                                let vec = packet.serialize_to_mc()?;
                                final_frame.extend_from_slice(&vec);
                                Ok(())
                            }).expect("Cannot perform final packet encoding");

                      
                        if let Some(old_data) = prev_data {
                            compress_final_data(&compression_ranges, &final_frame, &old_data)
                        }
                        prev_data = Some(final_frame.clone());
                        frame_tx.send(bytemuck::cast_vec(final_frame)).expect("Cannot send final frame!");
                    }
                };
            });

            return Ok(Self {
                frames_reciver: frame_rx,
                fps,
                width: width as i32,
                height: height as i32
            });
        };

        Err(anyhow::Error::new(Error::StreamNotFound))
    }

    fn load_frame(&mut self) -> anyhow::Result<Box<dyn super::player_context::VideoFrame>> {
        Ok(player_context::wrap_frame(self.frames_reciver.recv()?))
    }

    fn video_data(&self) -> anyhow::Result<super::player_context::VideoData> {
        Ok(VideoData {
            width: self.width,
            height: self.height,
            fps: self.fps,
        }) 
    }

    fn handle_jvm_msg(&self, msg: super::player_context::NativeCommunication) -> anyhow::Result<()> {
        Ok(()) 
    }

    fn destroy(&self) -> anyhow::Result<()> {
        Ok(()) //Do nothing for now 
    }
}

fn prepare_compression_ranges(splitted_frames: &Vec<SplittedFrame>) -> Vec<CompressionRange> {
    splitted_frames
        .iter()
        .fold((0usize, Vec::<CompressionRange>::new()), |mut acc, element| {
            acc.1.push(CompressionRange {
                start: acc.0,
                end: acc.0 + element.frame_length,
                width: element.width,
                height: element.height,
            });
            acc.0 += element.frame_length;
            acc
        }).1
}

fn compress_final_data(compression_ranges: &Vec<CompressionRange>, new_data: &[u8], old_data: &[u8]) {
    let final_data = compression_ranges
        .iter()
        .map(|e| (&new_data[e.start..e.end], &old_data[e.start..e.end], e.width, e.height))
        .filter_map(|(new, old, height, width)| {
            let (mut x_start, mut x_end, mut y_start, mut y_end) = (usize::MAX, usize::MAX, usize::MAX, usize::MAX);
            let mut data_changed = 0usize;
            new
                .chunks(width)
                .zip(old.chunks(width))
                .enumerate()
                .for_each(|(y, (new, old))| {
                    new
                        .iter()
                        .zip(old.iter())
                        .enumerate()
                        .filter(|(_, (new, old))| **new != **old)
                        .for_each(|(x, (_, _))| {
                            data_changed += 1;
                            if x_start == usize::MAX {
                                x_start = x;
                            }
                            if y_start == usize::MAX {
                                y_start = y;
                            }

                            if y > y_end || y_end == usize::MAX {
                                y_end = y
                            }

                            if x > x_end || x_end == usize::MAX {
                                x_end = x
                            }
                        });
                });

            if y_start != usize::MAX {
                Some((x_start, y_start, x_end - x_start + 1, y_end - y_start + 1, data_changed))
            } else {
                None
            }
        })
        .map(|(_, _, width, height, changes)| (width * height, changes))
        .fold((0usize, 0usize), |(acc1, acc2), (compress, all)| (acc1 + compress, acc2 + all));

    let (final_data, all) = final_data;
    println!("Compressed frame to {final_data}/{} (max possible: {})", new_data.len(), all);
        
}

impl MinecraftMapPacket {
    fn serialize_to_mc(&self) -> anyhow::Result<Vec<u8>> {
        //27 for the packet wrapping
        let mut output = Vec::<u8>::with_capacity(27 + self.data.len());

        let (map_id_len, map_id) = write_var_int(self.map_id);
        let (data_varint_len, data_varint) = write_var_int(self.data.len() as i32);
        let packet_len = 1 + map_id_len + 7 + data_varint_len + self.data.len();
        let (packet_len_varint_len, packet_len_varint) = write_var_int(packet_len as i32);

        //We DO need to say how long is the packet!
        output.extend_from_slice(&packet_len_varint[..packet_len_varint_len]);
        output.extend_from_slice(&[MAP_PACKET_ID]); //Packet id
        output.extend_from_slice(&map_id[..map_id_len]);
        //scale = 0, locked = true, Has Icons = false, columns, rows, x, z

        output.extend_from_slice(&[0x00i8 as u8, 0x01u8, 0x00u8, self.columns, self.rows, self.x, self.z]);
        output.extend_from_slice(&data_varint[..data_varint_len]);

        let data_slice = unsafe {
            std::slice::from_raw_parts(self.data.as_ptr() as *mut u8, self.data.len())
        };

        output.extend_from_slice(data_slice);

        Ok(output)       
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_write() {
        //Source: https://wiki.vg/Protocol
        let first = write_var_int(0);
        assert!(first.0 == 1);
        assert!(first.1[0] == 0x00);

        let second = write_var_int(2147483647);
        assert!(second.0 == 5);
        assert!(do_vecs_match(&second.1.to_vec(), &vec![0xff, 0xff, 0xff, 0xff, 0x07]));

        let third = write_var_int(2097151);
        assert!(third.0 == 3);
        let third_vec = third.1[..3].to_vec();
        assert!(do_vecs_match(&third_vec, &vec![0xff, 0xff, 0x7f]));

        let (map_data_packet_id_len, map_data_packet_id) = write_var_int(MAP_PACKET_ID as i32);
        assert!(do_vecs_match(&map_data_packet_id[..map_data_packet_id_len].to_vec(), &vec![MAP_PACKET_ID]));
    }

    fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
        let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
        matching == a.len() && matching == b.len()
    }
}
