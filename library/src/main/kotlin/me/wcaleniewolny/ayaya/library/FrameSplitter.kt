package me.wcaleniewolny.ayaya.library

import kotlin.math.ceil

//Note: If we want to speed this up we need to use C and JNI to do it.
//We can take advantage of vectorizable loops (https://www.intel.com/content/www/us/en/developer/articles/technical/requirements-for-vectorizable-loops.html)
//This will be required if we want to achieve > 50 FPS (50 fps is for my hardware. If you have an old CPU you will never achieve 60 FPS.)
object FrameSplitter {

    private var allFramesX = 0
    private var allFramesY = 0

    @Throws(IllegalArgumentException::class)
    fun splitFrames(data: ByteArray, frames: List<SplittedFrame>) {

        if (allFramesX * allFramesY != frames.size) {
            throw IllegalArgumentException("Frame list size does not match required lenght (${allFramesX * allFramesY})")
        }

        var i = 0
        var bI = 0 //byte index

        for (x in 0 until allFramesX) {
            for (y in 0 until allFramesY) {
                val frame = frames[i]

                val frameData = frames[i].data

                System.arraycopy(data, bI, frameData, 0, frame.frameLength)

                if (!frames[i].initialized) {
                    frames[i].initialized = true
                }

                bI += frame.frameLength
                i++
            }
        }

        println("FF size: ${frames.size}")
    }

    fun initializeFrames(width: Int, height: Int): List<SplittedFrame> {
        val frames = mutableListOf<SplittedFrame>()

        if (width % 2 != 0) {
            throw IllegalArgumentException("asymmetrical width is not supported")
        }
        if (height % 2 != 0) {
            throw IllegalArgumentException("asymmetrical height is not supported")
        }

        val framesX = width / 128.0
        val framesY = height / 128.0

        val xMargin = width - (framesX.toInt() * 128)
        val yMargin = height - (framesY.toInt() * 128)

        allFramesX = ceil(framesX).toInt()
        allFramesY = ceil(framesY).toInt()

        for (x in 0 until allFramesX) {
            for (y in 0 until allFramesY) {
                val xFrameMargin = if (x == 0) xMargin else 0
                val yFrameMargin = if (y == 0) yMargin else 0

                //startX = xFrameMargin && startY = yFrameMargin
                //This is due to the fact that we start at index 0 and not 1

                val frameWidth = 128 - xFrameMargin
                val frameHeight = 128 - yFrameMargin
                val frameLength = frameHeight * frameWidth

                frames.add(
                    SplittedFrame(
                        xFrameMargin,
                        yFrameMargin,
                        frameWidth,
                        frameHeight,
                        xFrameMargin,
                        yFrameMargin,
                        ByteArray(frameLength)
                    )
                )
            }
        }

        return frames;
    }
}