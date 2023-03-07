#include <math.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <libavcodec/packet.h>
#include <libavutil/pixfmt.h>
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/imgutils.h>
#include <libswscale/swscale.h>
#include <strings.h>

#include "rust.h"
#include "logger.h"
#include "data_structures.h"

#define WORKER_THREADS 8;

typedef struct {
	AVFrame* p_frame_input;
	AsyncPromise* p_promise;
	pthread_mutex_t lock;
	pthread_cond_t wait_cond;
} WorkerThreadTask;

typedef struct ExternalVideoData {
    size_t width;
    size_t height;
    size_t fps;
} ExternalVideoData;

typedef struct {
	uint8_t* p_colorTransformTable;
	AVPacket* p_av_packet;
	AVCodec* p_codec;
	AVFormatContext* p_format_ctx;
	AVCodecContext *p_codec_ctx;
	AVCodecParameters* p_codec_parm;
	AVFrame* p_frame;
	AVFrame* p_frame_rgb;
	struct SwsContext* p_sws_ctx;
	size_t video_stream_index;
	int num_bytes;
	size_t width;
	size_t height;
	size_t fps;
	struct RustVec* p_mem_ranges; 
	atomic_bool shutdown_bool;
	WorkerThreadTask* p_worker_thread_tasks;
} ExternalPlayer;

bool fast_yuv_frame_transform(
	int8_t* p_output,
	uint8_t* p_y_arr, 
	uint8_t* p_cb_arr, 
	uint8_t* p_cr_arr,
	uint8_t* p_color_transform_table,
	struct MemCopyRange* p_ranges,
	size_t ranges_len,
	uint64_t width,
	uint64_t height
){
	size_t area = (size_t) (width * height);

	int8_t* tmp_buf = malloc(area * sizeof(int8_t));
	if (tmp_buf == NULL) {
		log_error("[C ERR] malloc returned NULL!");
		return false;
	}

	#pragma omp parallel for simd
	for (size_t index = 0; index < area; index++) {
		size_t y = (size_t) *(p_y_arr + index);
		size_t cb = (size_t) *(p_cb_arr + index);
		size_t cr = (size_t) *(p_cr_arr + index);

		size_t offset = (y * 256 * 256) + (cb * 256) + cr;
		int8_t color = (int8_t) p_color_transform_table[offset];

		//size_t output_offset = *(p_fast_lookup_table + index);
		//*(p_output + output_offset) = color;
		*(tmp_buf + index) = color;
	}

	for (size_t i = 0; i < ranges_len; ++i) {
		struct MemCopyRange memCopyRange = *(p_ranges + i);

		memcpy((void*) p_output + memCopyRange.dstOffset, (void*) tmp_buf + memCopyRange.srcOffset, memCopyRange.len);
	}

	free((void*)tmp_buf);

	return true;
}

void free_rust_vec(struct RustVec* vec) {
	(*vec->destructor)(vec);
}

void* external_player_init(
	uint8_t* p_colorTransformTable,
	char* filename
) {
	AVPacket* p_av_packet;
	AVCodec* p_codec = NULL;
	AVFormatContext* p_format_ctx = NULL;
	AVCodecContext *p_codec_ctx = NULL;
	AVCodecParameters* p_codec_parm = NULL;
	AVFrame* p_frame = NULL;
	AVFrame* p_frame_rgb = NULL;
	size_t video_stream_index = SIZE_MAX;
	int num_bytes;
	uint8_t* p_rgb_buffer = NULL;
	struct SwsContext* p_sws_ctx = NULL;
	p_av_packet = av_packet_alloc();

	if (!p_av_packet) {
		log_error("av_packet_alloc failed");
		return NULL;
	}

    // Open video file
    if (avformat_open_input(&p_format_ctx, filename, NULL, NULL) != 0) {
        log_error("Cannot open input file");
		av_packet_free(&p_av_packet);
		return NULL;
    }

    if (avformat_find_stream_info(p_format_ctx, NULL) < 0) {
		log_error("Cannot find stream info!");
		av_packet_free(&p_av_packet);
		avformat_close_input(&p_format_ctx);
		return NULL;
	}

	av_dump_format(p_format_ctx, 0, filename, 0);

	for (size_t i = 0; i < p_format_ctx->nb_streams; i++) {
        if (p_format_ctx->streams[i]->codecpar->codec_type == AVMEDIA_TYPE_VIDEO) {
            video_stream_index = i;
            break;
        }
	}

	if (video_stream_index == SIZE_MAX) {
		log_error("Cannot find video stream index!");
		av_packet_free(&p_av_packet);
		avformat_close_input(&p_format_ctx);
		return NULL;
	}

	//fill codec parameters
	p_codec_parm = p_format_ctx->streams[video_stream_index]->codecpar;

    // Find the decoder for the video stream
    p_codec = avcodec_find_decoder(p_codec_parm->codec_id);

    //Get a pointer to the codec context for the video stream
    p_codec_ctx = avcodec_alloc_context3(p_codec);

    if (p_codec == NULL) {
		log_error("Unsupported coded!");	
		av_packet_free(&p_av_packet);
		avformat_close_input(&p_format_ctx);
		avcodec_free_context(&p_codec_ctx);
        return NULL; // Codec not found
    }

	//Fix codec contex - required to use mp4 files
	//Who said we need this? What is this about?
    avcodec_parameters_to_context(p_codec_ctx, p_codec_parm);

    // Open codec
    if (avcodec_open2(p_codec_ctx, p_codec, NULL) != 0){
		avformat_close_input(&p_format_ctx);
		avcodec_free_context(&p_codec_ctx);
		av_packet_free(&p_av_packet);
        log_error("Could not open codec");
        return NULL;
    }

    p_frame = av_frame_alloc();

    if(p_frame == NULL){
        log_error("Could allocate frame");
		av_packet_free(&p_av_packet);
		avformat_close_input(&p_format_ctx);
		avcodec_free_context(&p_codec_ctx);
        return NULL;
    }

	p_frame_rgb = av_frame_alloc();
    if(p_frame_rgb == NULL){
        log_error("Could allocate RGB frame");
		av_packet_free(&p_av_packet);
		avformat_close_input(&p_format_ctx);
		avcodec_free_context(&p_codec_ctx);
		av_frame_free(&p_frame);
        return NULL;
    }

	int pixfmt = AV_PIX_FMT_YUV444P;
	int align = 1;

    // Determine required ren_rgb_buffer size and allocate ren_rgb_buffer
	// What the fuck does this code do?
    num_bytes = av_image_get_buffer_size(
		pixfmt,
		p_codec_parm->width,
		p_codec_parm->height,
		align
	);

    p_sws_ctx = sws_getContext(
		p_codec_parm->width,
		p_codec_parm->height,
		p_codec_parm->format,
		p_codec_parm->width,
		p_codec_parm->height,
		pixfmt,
		SWS_BILINEAR,
		NULL,
		NULL,
		NULL
	);

    // Assign appropriate parts of ren_rgb_buffer to image planes in ren_pFrameRGB
    // Note that ren_pFrameRGB is an AVFrame, but AVFrame is a superset
    // of AVPicture
	//
	// Again: What the fuck does this code do?
    if (av_image_alloc((*p_frame_rgb).data, (*p_frame_rgb).linesize, p_codec_parm->width, p_codec_parm->height, pixfmt, align) < 0) {
		log_error("Array fill error");
		av_packet_free(&p_av_packet);
		avformat_close_input(&p_format_ctx);
		avcodec_free_context(&p_codec_ctx);
		av_frame_free(&p_frame);
		av_frame_free(&p_frame_rgb);
		sws_freeContext(p_sws_ctx);
		return NULL;
	};

	log_info("OK!");

	size_t width = (size_t) p_codec_parm->width; 
	size_t height = (size_t) p_codec_parm->height;

	struct RustVec* p_rust_memcpy_range_vec = malloc(sizeof(struct RustVec));
	memset(p_rust_memcpy_range_vec, 0, sizeof(struct RustVec));

	generate_memcpy_ranges(p_rust_memcpy_range_vec, width, height);

	if (p_rust_memcpy_range_vec->ptr == NULL) {
		log_error("Rust generate_memcpy_ranges callback failed");
		av_packet_free(&p_av_packet);
		avformat_close_input(&p_format_ctx);
		avcodec_free_context(&p_codec_ctx);
		av_frame_free(&p_frame);
		av_frame_free(&p_frame_rgb);
		sws_freeContext(p_sws_ctx);
		return NULL;
	};

	ExternalPlayer* player = (ExternalPlayer*) malloc(sizeof(ExternalPlayer));

	//We init that
	player->p_colorTransformTable = p_colorTransformTable;
	player->width = width;
	player->height = height;

	player->fps = (size_t) ((double_t) p_format_ctx->streams[video_stream_index]->r_frame_rate.num / (double_t) p_format_ctx->streams[video_stream_index]->r_frame_rate.den);

	//FFmpeg does that
	player->p_codec_ctx = p_codec_ctx;
	player->p_codec_parm = p_codec_parm;
	player->p_codec = p_codec;
	player->p_frame = p_frame;
	player->p_sws_ctx = p_sws_ctx;
	player->p_frame_rgb = p_frame_rgb;
	player->p_av_packet = p_av_packet;
	player->p_format_ctx = p_format_ctx;
	
	player->p_mem_ranges = p_rust_memcpy_range_vec;
	player->video_stream_index = video_stream_index;
	player->num_bytes = num_bytes;
	player->shutdown_bool = false;

	//Mem leak above: We do not clear previously allocated data on error (fixed)
	//Mem leak in the rust callback (also fixed)
	return (void*) player;
}

ExternalVideoData external_player_video_data(void* self) {
	ExternalPlayer* p_player = (ExternalPlayer*) self;
	ExternalVideoData data = {p_player->width, p_player->height, p_player->fps};
	return data;
}

int8_t* external_player_load_frame(void* self) {
	ExternalPlayer* p_player = (ExternalPlayer*) self;

    int ret;
    bool readFrame = true;

    int len = p_player->width * p_player->height;

    int8_t* output;
    output = (int8_t*)malloc(len * sizeof(int8_t));

    while (readFrame) {
        if ((ret = av_read_frame(p_player->p_format_ctx, p_player->p_av_packet)) < 0){
            log_error("AV cannot read frame");
            return NULL;
        }
        if (p_player->p_av_packet->stream_index == (int) p_player->video_stream_index) {
            ret = avcodec_send_packet(p_player->p_codec_ctx, p_player->p_av_packet);
            if (ret < 0) {
                log_error("Error while sending a ren_packet to the decoder");
				return NULL;
            }

            while (ret >= 0) {
                ret = avcodec_receive_frame(p_player->p_codec_ctx, p_player->p_frame);
                if (ret == AVERROR(EAGAIN) || ret == AVERROR_EOF) {
                    if(ret != AVERROR(EAGAIN)){
                        printf("TEST THROW: %s", av_err2str(ret));
                        fflush(stdout);
						av_packet_unref(p_player->p_av_packet);
						av_frame_unref(p_player->p_frame);
						return NULL;
                    }
                    break;
                } else if (ret < 0) {
                    log_error("Error while receiving a frame from the decoder");
					av_packet_unref(p_player->p_av_packet);
					av_frame_unref(p_player->p_frame);
					return NULL;
                }
                sws_scale(
					p_player->p_sws_ctx,
					(uint8_t const* const*)p_player->p_frame->data,
					p_player->p_frame->linesize,
					0,
					p_player->p_codec_ctx->height,
					p_player->p_frame_rgb->data,
					p_player->p_frame_rgb->linesize
                );
                readFrame = false;
				av_frame_unref(p_player->p_frame);
            }
        } else {
			log_error("Packet stream index != video_stream_index (\?\?\?)");
			av_packet_unref(p_player->p_av_packet);
			return NULL;
		}
		av_packet_unref(p_player->p_av_packet);
    }

	fast_yuv_frame_transform(
		output,
		p_player->p_frame_rgb->data[0],
		p_player->p_frame_rgb->data[1],
		p_player->p_frame_rgb->data[2],
		p_player->p_colorTransformTable,
		(struct MemCopyRange*) p_player->p_mem_ranges->ptr, //Super safe pointer cast
		p_player->p_mem_ranges->len,
		p_player->width,
		p_player->height
	);	

	return output;
}

void external_player_free(void* self) {
	ExternalPlayer* p_player = (ExternalPlayer*) self;

	av_freep(&p_player->p_frame_rgb->data[0]);
	av_freep(&p_player->p_frame->data[0]);
	avformat_close_input(&p_player->p_format_ctx);
	avcodec_free_context(&p_player->p_codec_ctx);
	av_frame_free(&p_player->p_frame_rgb);
	av_frame_free(&p_player->p_frame);
	av_packet_free(&p_player->p_av_packet);
	sws_freeContext(p_player->p_sws_ctx);
	free_rust_vec(p_player->p_mem_ranges);
	free(p_player);
}
