/* DO NOT EDIT THIS FILE - it is machine generated */
#include <jni.h>
#include <libavformat/avformat.h>
/* Header for class me_wcaleniewolny_ayaya_NativeControler */

#ifndef _Included_me_wcaleniewolny_ayaya_NativeControler
#define _Included_me_wcaleniewolny_ayaya_NativeControler
#ifdef __cplusplus
extern "C" {
#endif
/*
 * Class:     me_wcaleniewolny_ayaya_NativeControler
 * Method:    loadFrame
 * Signature: ()[B
 */
JNIEXPORT jbyteArray JNICALL Java_me_wcaleniewolny_ayaya_NativeControler_loadFrame
        (JNIEnv *, jobject);

void SaveFrame(AVFrame *pFrame, int width, int height, int iFrame);

void throwException(JNIEnv *, char* class, char* value);

#ifdef __cplusplus
}
#endif
#endif
