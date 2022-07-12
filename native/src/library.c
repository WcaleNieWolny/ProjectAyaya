//This library is inspired by https://github.com/mpenkov/ffmpeg-tutorial/blob/master/tutorial01.c
#include <stdio.h>
#include "library.h"

#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/imgutils.h>

#include <libswscale/swscale.h>
#include <stdbool.h>

typedef signed char signedByte;

//"ren" is global prefix for variables used in the global scope by this JNI library
AVCodecContext *ren_pCodecCtx = NULL;
AVCodec *ren_pCodec = NULL;
AVFrame *ren_pFrame = NULL;
AVPacket *ren_packet;
struct SwsContext *ren_sws_ctx = NULL;

int ren_videoStream;
AVFormatContext *ren_pFormatCtx = NULL;
AVFrame *ren_pFrameRGB = NULL;
int ret_image_index=0;
uint8_t *ren_rgb_buffer = NULL;

bool ren_initialized = false;

const char ren_test_finalFileName[] = "/home/wolny/Downloads/test.mp4";

jbyteArray JNICALL Java_me_wcaleniewolny_ayaya_NativeRenderControler_loadFrame
(JNIEnv* env, jobject thisObject){
    printf("Hello from native!");

    if(!ren_initialized){
        throwException((JNIEnv *) env, "java/lang/IllegalStateException", "Native controller is not initialized!");
        return NULL;
    }

    // Read frames and save first five frames to disk
    int ret;
    bool readFrame = true;

    while (readFrame) {
        if ((ret = av_read_frame(ren_pFormatCtx, ren_packet)) < 0)
            break;
        if (ren_packet->stream_index == ren_videoStream) {
            ret = avcodec_send_packet(ren_pCodecCtx, ren_packet);
            if (ret < 0) {
                av_log(NULL, AV_LOG_ERROR, "Error while sending a ren_packet to the decoder\n");
                break;
            }

            while (ret >= 0) {
                ret = avcodec_receive_frame(ren_pCodecCtx, ren_pFrame);
                if (ret == AVERROR(EAGAIN) || ret == AVERROR_EOF) {
                    break;
                } else if (ret < 0) {
                    av_log(NULL, AV_LOG_ERROR, "Error while receiving a frame from the decoder\n");
                }
                sws_scale
                        (
                                ren_sws_ctx,
                                (uint8_t const * const *)ren_pFrame->data,
                                ren_pFrame->linesize,
                                0,
                                ren_pCodecCtx->height,
                                ren_pFrameRGB->data,
                                ren_pFrameRGB->linesize
                        );
                SaveFrame(ren_pFrameRGB, ren_pCodecCtx->width, ren_pCodecCtx->height, ret_image_index);
                ret_image_index++;
                readFrame = false;
            }
        }
    }

    int len = 3;
    signedByte byteArray[len];
    jbyteArray jbyteArray = (*env)->NewByteArray(env, len);

    (*env)->SetByteArrayRegion(env, jbyteArray, 0, 3, byteArray);
    return jbyteArray;
}

JNIEXPORT jint JNICALL Java_me_wcaleniewolny_ayaya_NativeRenderControler_init
(JNIEnv *env, jobject thisObject) {
    AVCodecParameters *pCodecParm = NULL;
    int numBytes;
    AVDictionary *optionsDict = NULL;

    ren_packet = av_packet_alloc();
    if (!ren_packet)
        exit(1);

    // Open video file
    if (avformat_open_input(&ren_pFormatCtx, ren_test_finalFileName, NULL, NULL) != 0) {
        throwException(env, "java/lang/RuntimeException", "Couldn't open file");
        return -1; // Couldn't open file
    }

    // Retrieve stream information
    if (avformat_find_stream_info(ren_pFormatCtx, NULL) < 0) {
        throwException(env, "java/lang/RuntimeException", "Couldn't find stream information");
        return -1; // Couldn't find stream information
    }

    // Dump information about file onto standard error
    av_dump_format(ren_pFormatCtx, 0, ren_test_finalFileName, 0);

    // Find the first video stream
    ren_videoStream = -1;

    int i;
    for (i = 0; i < ren_pFormatCtx->nb_streams; i++)
        if (ren_pFormatCtx->streams[i]->codecpar->codec_type == AVMEDIA_TYPE_VIDEO) {
            ren_videoStream = i;
            break;
        }

    if (ren_videoStream == -1) {
        throwException(env, "java/lang/RuntimeException", "Didn't find a video stream");
        return -1; // Didn't find a video stream
    }

    //fill codec parameters
    pCodecParm = ren_pFormatCtx->streams[ren_videoStream]->codecpar;

    // Find the decoder for the video stream
    ren_pCodec = avcodec_find_decoder(pCodecParm->codec_id);

    //Get a pointer to the codec context for the video stream
    ren_pCodecCtx = avcodec_alloc_context3(ren_pCodec);

    if (ren_pCodec == NULL) {
        fprintf(stderr, "Unsupported codec!\n");
        fflush(stdout);
        return -1; // Codec not found
    }

    //Fix codec contex - required to use mp4 files
    avcodec_parameters_to_context(ren_pCodecCtx, pCodecParm);

    // Open codec
    if (avcodec_open2(ren_pCodecCtx, ren_pCodec, &optionsDict) < 0){
        throwException(env, "java/lang/RuntimeException", "Could not open codec");
        return -1; // Could not open codec
    }

    // Allocate video frame
    ren_pFrame=av_frame_alloc();

    if(ren_pFrame == NULL){
        throwException(env, "java/lang/RuntimeException", "Could allocate frame");
        return -1;
    }

    //Allocate ren_packet
    ren_packet = av_packet_alloc();

    // Allocate an AVFrame structure
    ren_pFrameRGB=av_frame_alloc();
    if(ren_pFrameRGB == NULL){
        throwException(env, "java/lang/RuntimeException", "Could allocate RGB frame");
        return -1;
    }


    // Determine required ren_rgb_buffer size and allocate ren_rgb_buffer
    numBytes=av_image_get_buffer_size(AV_PIX_FMT_RGB24, pCodecParm->width,
                                      pCodecParm->height, 16);
    ren_rgb_buffer=(uint8_t *)av_malloc(numBytes * sizeof(uint8_t));
    ren_sws_ctx =
            sws_getContext
                    (
                            pCodecParm->width,
                            pCodecParm->height,
                            pCodecParm->format,
                            pCodecParm->width,
                            pCodecParm->height,
                            AV_PIX_FMT_RGB24,
                            SWS_BILINEAR,
                            NULL,
                            NULL,
                            NULL
                    );
    // Assign appropriate parts of ren_rgb_buffer to image planes in ren_pFrameRGB
    // Note that ren_pFrameRGB is an AVFrame, but AVFrame is a superset
    // of AVPicture
    av_image_fill_arrays((*ren_pFrameRGB).data, (*ren_pFrameRGB).linesize, ren_rgb_buffer, AV_PIX_FMT_RGB24, pCodecParm->width, pCodecParm->height, 16);

    return 0;
}

JNIEXPORT void JNICALL Java_me_wcaleniewolny_ayaya_NativeRenderControler_destroy
(JNIEnv *env, jobject thisObject){
    if(!ren_initialized){
        throwException((JNIEnv *) env, "java/lang/IllegalStateException", "Cannot free non existing memory!");
        return;
    }
    // Free the RGB image
    av_free(ren_rgb_buffer);
    av_free(ren_pFrameRGB);
    // Free the YUV frame
    av_free(ren_pFrame);
    // Free ren_packet
    av_packet_free(&ren_packet);

    //Close the codec
    avcodec_free_context(&ren_pCodecCtx);

    // Close the video file
    avformat_close_input(&ren_pFormatCtx);
}

void SaveFrame(AVFrame *pFrame, int width, int height, int iFrame) {
    printf("save\n");
    fflush(stdout);
    FILE *pFile;
    char szFilename[32];
    int  y;

    // Open file
    sprintf(szFilename, "out/frame%d.ppm", iFrame);
    pFile=fopen(szFilename, "wb");
    if(pFile==NULL)
        return;

    // Write header
    fprintf(pFile, "P6\n%d %d\n255\n", width, height);

    // Write pixel data
    for(y=0; y<height; y++){
        unsigned char* pFrameData = pFrame->data[0]+y*pFrame->linesize[0];

        printf("%d\n", pFrameData[0]);
        printf("%d\n", pFrameData[1]);
        printf("%d\n", pFrameData[2]);

        fwrite(pFrameData, 1, width*3, pFile);
    }

    // Close file
    fclose(pFile);
    exit(1);
}

void throwException(JNIEnv *env, char* class, char* value) {
    jclass illegalArgumentExceptionClass = (*env)->FindClass(env, class);
    jmethodID errorConstructor = (*env)->GetMethodID(env, illegalArgumentExceptionClass, "<init>", "(Ljava/lang/String;)V"); //(Ljava/lang/String;)V"\

    jstring errorString = (*env)->NewStringUTF(env, value);
    jobject illegalArgumentExceptionObject = (*env)->NewObject(env, illegalArgumentExceptionClass, errorConstructor, errorString);
    (*env)->Throw(env, illegalArgumentExceptionObject);

    (*env)->DeleteLocalRef(env, errorString);
}