#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/imgutils.h>
#include <libswscale/swscale.h>

struct MemCopyRange {
    size_t srcOffset;
    size_t dstOffset;
    size_t len;
};

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
	uint8_t* p_rgb_buffer;
	size_t video_stream_index;
	int num_bytes;
	size_t width;
	size_t height;
	size_t fps;
} ExternalPlayer;

typedef struct ExternalVideoData {
    size_t width;
    size_t height;
    size_t fps;
} ExternalVideoData;

inline void log_info(char* str) {
	fprintf(stdout, "[C INFO] %s\n", str);
	fflush(stdout);
}

inline void log_error(char* str) {
	fprintf(stderr, "[C ERR] %s\n", str);
	fflush(stderr);
}

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
		size_t y = (size_t)p_y_arr[index];
		size_t cb = (size_t)p_cb_arr[index / 4];
		size_t cr = (size_t)p_cr_arr[index / 4];

		size_t offset = (y * 256 * 256) + (cb * 256) + cr;
		int8_t color = (int8_t)p_color_transform_table[offset];

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

void* external_player_init(
	uint8_t* p_colorTransformTable,
	char* filename
) {
	printf("[C INFO] Welcome from native C code\n");
	printf("[C INFO] Filename: %s\n", filename);
	fflush(stdout);

	AVPacket* p_av_packet;
	AVCodec* p_codec = NULL;
	AVFormatContext* p_format_ctx = NULL;
	AVCodecContext *p_codec_ctx = NULL;
	AVCodecParameters* p_codec_parm = NULL;
	AVFrame* p_frame = NULL;
	AVFrame* p_frame_rgb = NULL;
	size_t video_stream_index = -1;
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
		return NULL;
    }

    if (avformat_find_stream_info(p_format_ctx, NULL) < 0) {
		log_error("Cannot find stream info!");
		return NULL;
	}

	av_dump_format(p_format_ctx, 0, filename, 0);

	for (size_t i = 0; i < p_format_ctx->nb_streams; i++) {
        if (p_format_ctx->streams[i]->codecpar->codec_type == AVMEDIA_TYPE_VIDEO) {
            video_stream_index = i;
            break;
        }
	}

	if (video_stream_index == -1) {
		log_error("Cannot find video stream index!");
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
        return NULL; // Codec not found
    }

	//Fix codec contex - required to use mp4 files
	//Who said we need this? What is this about?
    avcodec_parameters_to_context(p_codec_ctx, p_codec_parm);

    // Open codec
    if (avcodec_open2(p_codec_ctx, p_codec, NULL) != 0){
        log_error("Could not open codec");
        return NULL;
    }

    p_frame = av_frame_alloc();

    if(p_frame == NULL){
        log_error("Could allocate frame");
        return NULL;
    }

	p_frame_rgb = av_frame_alloc();
    if(p_frame_rgb == NULL){
        log_error("Could allocate RGB frame");
        return NULL;
    }

    // Determine required ren_rgb_buffer size and allocate ren_rgb_buffer
	// What the fuck does this code do?
    num_bytes = av_image_get_buffer_size(
		AV_PIX_FMT_RGB24,
		p_codec_parm->width,
		p_codec_parm->height,
		16
	);

    p_rgb_buffer = (uint8_t *)av_malloc(num_bytes * sizeof(uint8_t));
    p_sws_ctx = sws_getContext(
		p_codec_parm->width,
		p_codec_parm->height,
		p_codec_parm->format,
		p_codec_parm->width,
		p_codec_parm->height,
		AV_PIX_FMT_RGB24,
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
    if (av_image_fill_arrays((*p_frame_rgb).data, (*p_frame_rgb).linesize, p_rgb_buffer, AV_PIX_FMT_RGB24, p_codec_parm->width, p_codec_parm->height, 16) < 0) {
		log_error("Array fill error");
		return NULL;
	};

	log_info("OK!");

	ExternalPlayer* player = (ExternalPlayer*) malloc(sizeof(ExternalPlayer));

	//We init that
	player->p_colorTransformTable = p_colorTransformTable;
	player->width = (size_t) p_codec_parm->width;
	player->height = (size_t) p_codec_parm->width;
	player->fps = (size_t) (p_codec_ctx->framerate.num / p_codec_ctx->framerate.den);
	//FFmpeg does that
	player->p_codec_ctx = p_codec_ctx;
	player->p_codec_parm = p_codec_parm;
	player->p_codec = p_codec;
	player->p_frame = p_frame;
	player->p_sws_ctx = p_sws_ctx;
	player->p_frame_rgb = p_frame_rgb;
	player->p_av_packet = p_av_packet;
	player->p_rgb_buffer = p_rgb_buffer;
	player->p_format_ctx = p_format_ctx;
	
	player->video_stream_index = video_stream_index;
	player->num_bytes = num_bytes;

	//Mem leak above: We do not clear previously allocated data on error
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

    int8_t* byteArray;
    byteArray=(int8_t*)malloc(len * sizeof(int8_t));

    while (readFrame) {
        if ((ret = av_read_frame(p_player->p_format_ctx, p_player->p_av_packet)) < 0){
            log_error("AV cannot read frame");
            return NULL;
            //NOTE: If there is an another error it is up to the java application to detect this.
            // It is marked as EndOfFileException as it will be likely the error that happened
        }
        if (p_player->p_av_packet->stream_index == p_player->video_stream_index) {
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
						return NULL;
                    }
                    break;
                } else if (ret < 0) {
                    log_error("Error while receiving a frame from the decoder");
					return NULL;
                }
                sws_scale
                        (
                                p_player->p_sws_ctx,
                                (uint8_t const * const *)p_player->p_frame->data,
                                p_player->p_frame->linesize,
                                0,
                                p_player->p_codec_ctx->height,
                                p_player->p_frame_rgb->data,
                                p_player->p_frame_rgb->linesize
                        );
                readFrame = false;
            }
        }
    }

	return NULL;
}

void external_player_free(void* self) {

}
