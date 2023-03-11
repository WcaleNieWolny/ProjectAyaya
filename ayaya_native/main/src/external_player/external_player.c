#include <libavutil/frame.h>
#include <libavutil/mem.h>
#include <math.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <pthread.h>

#include <libavcodec/packet.h>
#include <libavutil/pixfmt.h>
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/imgutils.h>
#include <libswscale/swscale.h>

#include "rust.h"
#include "logger.h"
#include "data_structures.h"

#define WORKER_THREADS 8 
#define PIXFMT AV_PIX_FMT_YUV444P
#define ALIGN 1

typedef struct {
	AVFrame* p_frame_input;
	AsyncPromise* p_promise;
	pthread_mutex_t lock;
	pthread_cond_t wait_cond;
	size_t width;
	size_t height;
	enum AVPixelFormat pixfmt;
	uint8_t* p_color_transform_table;
	struct RustVec* p_ranges;
} WorkerThreadTask;

typedef struct ExternalVideoData {
    size_t width;
    size_t height;
    size_t fps;
} ExternalVideoData;

typedef struct {
	size_t width;
	size_t height;
	size_t fps;
	atomic_bool shutdown_bool;
	pthread_t master_thread;
	CircularBuffer* p_frame_buffer;	
} ExternalPlayer;

typedef struct {
	uint8_t* p_colorTransformTable;
	AVPacket* p_av_packet;
	AVCodec* p_codec;
	AVFormatContext* p_format_ctx;
	AVCodecContext *p_codec_ctx;
	AVCodecParameters* p_codec_parm;
	struct SwsContext* p_sws_ctx;
	size_t video_stream_index;
	struct RustVec* p_mem_ranges;
	WorkerThreadTask* p_worker_thread_tasks;
	CircularBuffer* p_frame_buffer;	
} MasterThreadInput;

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

static inline bool init_worker_task(WorkerThreadTask* p_task, size_t width, size_t height, enum AVPixelFormat pixfmt, uint8_t* p_color_transform_table, struct RustVec* p_ranges) {
	if (pthread_mutex_init(&p_task->lock, NULL) != 0) {
		log_error("Cannot init WorkerThreadTask mutex");
		return false;
	}

	if (pthread_cond_init(&p_task->wait_cond, NULL) != 0) {
		log_error("Cannot init WorkerThreadTask cond");
		pthread_mutex_destroy(&p_task->lock);
		return false;
	}	

	p_task->width = width;
	p_task->height = height;
	p_task->pixfmt = pixfmt;
	p_task->p_color_transform_table = p_color_transform_table;
	p_task->p_ranges = p_ranges;

	return true;
}

void* master_thread_start(void* arg) {
	MasterThreadInput* p_input = arg;
	
	log_info("Hello from worker!");

	size_t i = 0;

	//Block
	for (;;) {
		if (!circular_buffer_lock(p_input->p_frame_buffer)) {
			log_error("Lock frame buffer failed");
			return NULL;
		}
		AsyncPromise* p_promise = circular_buffer_read(p_input->p_frame_buffer);
		if (p_promise == NULL) {
			log_error("Promise read == null");
			circular_buffer_unlock(p_input->p_frame_buffer);
			return NULL;
		};

		if (async_promise_init(p_promise)) {
			circular_buffer_unlock(p_input->p_frame_buffer);
			return NULL;
		}

		AVFrame* p_frame = av_frame_alloc();
		if (p_frame == NULL) {
			log_error("Promise read == null");
			return NULL;
		}

		int ret;
		bool readFrame = true;

		while (readFrame) {
			if ((ret = av_read_frame(p_input->p_format_ctx, p_input->p_av_packet)) < 0){
				av_free(p_frame);
				log_error("AV cannot read frame");
				return NULL;
			}
			if (p_input->p_av_packet->stream_index == (int) p_input->video_stream_index) {
				ret = avcodec_send_packet(p_input->p_codec_ctx, p_input->p_av_packet);
				if (ret < 0) {
					log_error("Error while sending a ren_packet to the decoder");
					return NULL;
				}

				while (ret >= 0) {
					ret = avcodec_receive_frame(p_input->p_codec_ctx, p_frame);
					if (ret == AVERROR(EAGAIN) || ret == AVERROR_EOF) {
						if(ret != AVERROR(EAGAIN)){
							printf("TEST THROW: %s", av_err2str(ret));
							fflush(stdout);
							av_packet_unref(p_input->p_av_packet);
							av_frame_unref(p_frame);
							return NULL;
						}
						break;
					} else if (ret < 0) {
						log_error("Error while receiving a frame from the decoder");
						av_packet_unref(p_input->p_av_packet);
						av_frame_unref(p_frame);
						return NULL;
					}
					readFrame = false;
					av_frame_unref(p_frame);
				}
			} else {
				log_error("Packet stream index != video_stream_index (\?\?\?)");
				av_packet_unref(p_input->p_av_packet);
				return NULL;
			}
			av_packet_unref(p_input->p_av_packet);
		}
		
		i += 1;
		if (i == WORKER_THREADS) {
			i = 0;
		}

		WorkerThreadTask* p_task = (p_input->p_worker_thread_tasks + i);

		if (pthread_mutex_lock(&p_task->lock) != 0) {
			log_error("Cannot lock slave mutex");
			return NULL;
		};

		p_task->p_promise = p_promise;
		p_task->p_frame_input = p_frame;
	}
}

void* slave_thread_start(void* args) {
	WorkerThreadTask* p_task = args;

	AVFrame* p_frame_yuv = av_frame_alloc();
	if (p_frame_yuv == NULL) {
		log_error("YUV frame alloc failed");
		return NULL;
	}

	if (av_image_alloc((*p_frame_yuv).data, (*p_frame_yuv).linesize, p_task->width, p_task->height, PIXFMT, ALIGN) < 0) {
		log_error("Array fill error");
		av_free(p_frame_yuv);
		return NULL;
	};

	struct SwsContext* p_sws_ctx = NULL;
    p_sws_ctx = sws_getContext(
		p_task->width,
		p_task->height,
		p_task->pixfmt,
		p_task->width,
		p_task->height,
		PIXFMT,
		SWS_BILINEAR,
		NULL,
		NULL,
		NULL
	);

	if (p_sws_ctx == NULL) {
		log_error("Cannot do sws init");
		return NULL;
	}

	if (pthread_mutex_lock(&p_task->lock) != 0) {
		log_error("Cannot lock slave mutex");
	}

	int len = p_task->width * p_task->height;

	for (;;) {
		//No error checking, as this should never fail
		if (pthread_cond_wait(&p_task->wait_cond, &p_task->lock) != 0) {
			log_error("Cannot wait on slave cond");
			return NULL;
		};


		int8_t* output;
		output = (int8_t*)malloc(len * sizeof(int8_t));

		if (output == NULL) {
			log_error("Cannot malloc output");
			pthread_mutex_unlock(&p_task->lock);
			return NULL;
		}

		sws_scale(
			p_sws_ctx,
			(uint8_t const* const*)p_task->p_frame_input->data,
			p_task->p_frame_input->linesize,
			0,
			p_task->height,
			p_frame_yuv->data,
			p_frame_yuv->linesize
		);
		
		fast_yuv_frame_transform(
			output,
			p_frame_yuv->data[0],
			p_frame_yuv->data[1],
			p_frame_yuv->data[2],
			p_task->p_color_transform_table,
			(struct MemCopyRange*) p_task->p_ranges->ptr, //Super safe pointer cast
			p_task->p_ranges->len,
			p_task->width,
			p_task->height
		);	
		
		if (!async_promise_fufil(p_task->p_promise, output)) {
			log_error("Cannot fufil async slave promise");
			return NULL;
		}

		av_frame_free(&p_task->p_frame_input);
	};
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
	size_t video_stream_index = SIZE_MAX;
	int num_bytes;
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

    // Determine required ren_rgb_buffer size and allocate ren_rgb_buffer
	// What the fuck does this code do?
    num_bytes = av_image_get_buffer_size(
		PIXFMT,
		p_codec_parm->width,
		p_codec_parm->height,
		ALIGN	
	);

	size_t width = (size_t) p_codec_parm->width; 
	size_t height = (size_t) p_codec_parm->height;

	CircularBuffer* p_frame_buffer = malloc(sizeof(CircularBuffer));

	if (p_frame_buffer == NULL) {
		log_error("Cannot init WorkerThreadTask");
		av_packet_free(&p_av_packet);
		avformat_close_input(&p_format_ctx);
		avcodec_free_context(&p_codec_ctx);
		sws_freeContext(p_sws_ctx);
		return NULL;
	}

	circular_buffer_init(64, sizeof(AsyncPromise*));

	log_info("OK!");

	struct RustVec* p_rust_memcpy_range_vec = malloc(sizeof(struct RustVec));
	if (p_rust_memcpy_range_vec->ptr == NULL) {
		log_error("Rust generate_memcpy_ranges callback failed");
		av_packet_free(&p_av_packet);
		avformat_close_input(&p_format_ctx);
		avcodec_free_context(&p_codec_ctx);
		sws_freeContext(p_sws_ctx);
		circular_buffer_free(p_frame_buffer);	
		return NULL;
	};

	memset(p_rust_memcpy_range_vec, 0, sizeof(struct RustVec));
	generate_memcpy_ranges(p_rust_memcpy_range_vec, width, height);

	WorkerThreadTask* worker_tasks = calloc(WORKER_THREADS, sizeof(WorkerThreadTask));

	if (worker_tasks == NULL) {
		log_error("Cannot init worker tasks");
		av_packet_free(&p_av_packet);
		avformat_close_input(&p_format_ctx);
		avcodec_free_context(&p_codec_ctx);
		sws_freeContext(p_sws_ctx);
		circular_buffer_free(p_frame_buffer);	
		return NULL;
	}

	for (size_t i = 0; i < WORKER_THREADS; i++) {
		if (!init_worker_task(worker_tasks + i, width, height, p_codec_parm->format, p_colorTransformTable, p_rust_memcpy_range_vec)) {
			log_error("Cannot init WorkerThreadTask");
			av_packet_free(&p_av_packet);
			avformat_close_input(&p_format_ctx);
			avcodec_free_context(&p_codec_ctx);
			sws_freeContext(p_sws_ctx);
			return NULL;
		}
	}

	ExternalPlayer* player = (ExternalPlayer*) malloc(sizeof(ExternalPlayer));
	MasterThreadInput* master_thread_input = malloc(sizeof(MasterThreadInput));

	if (player == NULL || master_thread_input == NULL) {
		log_error("Player or master_input malloc returned null");
		av_packet_free(&p_av_packet);
		avformat_close_input(&p_format_ctx);
		avcodec_free_context(&p_codec_ctx);
		sws_freeContext(p_sws_ctx);
		free_rust_vec(p_rust_memcpy_range_vec);
		free(worker_tasks);
		circular_buffer_free(p_frame_buffer);	
		return NULL;
	}

	//We init that
	master_thread_input->p_colorTransformTable = p_colorTransformTable;
	player->width = width;
	player->height = height;
	player->fps = (size_t) ((double_t) p_format_ctx->streams[video_stream_index]->r_frame_rate.num / (double_t) p_format_ctx->streams[video_stream_index]->r_frame_rate.den);
	player->p_frame_buffer = p_frame_buffer;

	//FFmpeg does that
	master_thread_input->p_codec_ctx = p_codec_ctx;
	master_thread_input->p_codec_parm = p_codec_parm;
	master_thread_input->p_codec = p_codec;
	master_thread_input->p_sws_ctx = p_sws_ctx;
	master_thread_input->p_av_packet = p_av_packet;
	master_thread_input->p_format_ctx = p_format_ctx;
	master_thread_input->p_mem_ranges = p_rust_memcpy_range_vec;
	master_thread_input->video_stream_index = video_stream_index;
	master_thread_input->p_worker_thread_tasks = worker_tasks;
	master_thread_input->p_frame_buffer = p_frame_buffer;

	player->shutdown_bool = false;

	int pthread_ret = pthread_create(
		&player->master_thread,
		NULL,
		master_thread_start,
		master_thread_input //For now arg = null
	);

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
	return NULL;
}

//TODO: FIX MEM LEAK
void external_player_free(void* self) {
	return;
}
