package me.wcaleniewolny.ayaya.frame

import kotlin.math.ceil

//Note: If we want to speed this up we need to use C and JNI to do it.
//We can take advantage of vectorizable loops (https://www.intel.com/content/www/us/en/developer/articles/technical/requirements-for-vectorizable-loops.html)
//This will be required if we want to achieve > 50 FPS (50 fps is for my hardware. If you have an old CPU you will never achieve 60 FPS.)
object FrameSplitter {

    @Throws(IllegalArgumentException::class)
    fun splitFrames(width: Int, height: Int, data: ByteArray): List<SplittedFrame>{

        if(width % 2 != 0){
            throw IllegalArgumentException("asymmetrical width is not supported")
        }
        if(height % 2 != 0){
            throw IllegalArgumentException("asymmetrical height is not supported")
        }

        val framesX = width / 128.0
        val framesY = height / 128.0

        val xMargin = width - (framesX.toInt() * 128)
        val yMargin = width - (framesY.toInt() * 128)

        val allFramesX = ceil(framesX).toInt()
        val allFramesY = ceil(framesY).toInt()

        val dataStream = data.inputStream()
        val splittedFrames = mutableListOf<SplittedFrame>();

        for (x in 0 until allFramesX){
            for (y in 0 until allFramesY){
                val xFrameMargin = if(x == 0) xMargin else 0
                val yFrameMargin = if(y == 0) yMargin else 0

                //startX = xFrameMargin && startY = yFrameMargin
                //This is due to the fact that we start at index 0 and not 1

                val frameWidth = 128 - xFrameMargin
                val frameHeight = 128 - yFrameMargin

                splittedFrames.add(
                    SplittedFrame(
                        xFrameMargin,
                        yFrameMargin,
                        frameWidth,
                        frameHeight,
                        dataStream.readNBytes(frameWidth * frameHeight)
                    )
                )
            }
        }

        println("FF size: ${splittedFrames.size}")
        return splittedFrames.toList();
    }
}