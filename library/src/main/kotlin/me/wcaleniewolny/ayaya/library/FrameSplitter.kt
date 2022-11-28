package me.wcaleniewolny.ayaya.library

import kotlin.math.ceil

//Note: If we want to speed this up we need to use C and JNI to do it.
//We can take advantage of vectorizable loops (https://www.intel.com/content/www/us/en/developer/articles/technical/requirements-for-vectorizable-loops.html)
//This will be required if we want to achieve > 50 FPS (50 fps is for my hardware. If you have an old CPU you will never achieve 60 FPS.)
object FrameSplitter {

    private var allFramesX = 0
    private var allFramesY = 0

    @Throws(IllegalArgumentException::class)
    fun splitFrames(data: ByteArray, frames: List<SplittedFrame>, width: Int) {

        if (allFramesX * allFramesY != frames.size) {
            throw IllegalArgumentException("Frame list size does not match required lenght (${allFramesX * allFramesY})")
        }

        var frameIndex = 0
        var lenIndex = 0;

        for (y in 0 until allFramesY) {
            for (x in 0 until allFramesX) {
                val frame = frames[frameIndex]

                val frameData = frames[frameIndex].data

                System.arraycopy(data, lenIndex, frameData, 0, frame.frameLength)

                if (!frames[frameIndex].initialized) {
                    frames[frameIndex].initialized = true
                }

                lenIndex += frame.frameLength
                frameIndex++

            }
        }
    }

    fun getRenderData(width: Int, height: Int): IntArray {

        if (width % 2 != 0) {
            throw IllegalArgumentException("asymmetrical width is not supported")
        }
        if (height % 2 != 0) {
            throw IllegalArgumentException("asymmetrical height is not supported")
        }

        val framesX = width / 128.0
        val framesY = height / 128.0

        val xMargin = if (width % 128 == 0) 0 else 128 - (width - (framesX.toInt() * 128))
        val yMargin = if (height % 128 == 0) 0 else 128 - (height - (framesY.toInt() * 128))

        val allFramesX = ceil(framesX).toInt()
        val allFramesY = ceil(framesY).toInt()
        val finalLength = allFramesY * allFramesX * 128 * 128; //TODO: Make it more efficient in rust

        return intArrayOf(xMargin, yMargin, allFramesX, allFramesY, finalLength)
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

        val xMargin = if (width % 128 == 0) 0 else 128 - (width - (framesX.toInt() * 128))
        val yMargin = if (height % 128 == 0) 0 else 128 - (height - (framesY.toInt() * 128))

        allFramesX = ceil(framesX).toInt()
        allFramesY = ceil(framesY).toInt()


        for (y in 0 until allFramesY) {
            for (x in 0 until allFramesX) {
                val xFrameMargin = if (x == 0) (xMargin / 2) else 0
                val yFrameMargin = if (y == 0) (yMargin / 2) else 0

                //startX = xFrameMargin && startY = yFrameMargin
                //This is due to the fact that we start at index 0 and not 1

                val frameWidth = if (x != allFramesX - 1) 128 - xFrameMargin else 128 - (xMargin / 2)
                val frameHeight = if (y != (allFramesY - 1)) 128 - yFrameMargin else 128 - (yMargin / 2)

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

    @Throws(IllegalArgumentException::class)
    fun legacySplitframes(data: ByteArray, frames: List<SplittedFrame>, width: Int, height: Int) {

        if (allFramesX * allFramesY != frames.size) {
            throw IllegalArgumentException("Frame list size does not match required lenght (${allFramesX * allFramesY})")
        }

        var i = 0
        var yI = 0 //Y index

        for (y in 0 until allFramesY) {
            var xI = 0 //X index
            for (x in 0 until allFramesX) {
                val frame = frames[i]

                val frameData = frames[i].data

                for (y1 in 0 until frame.height) {
//                    for (x1 in 0 until frame.width){
//                        frameData[(y1 * frame.width) + x1] = data[((yI * width) + xI) + ((y1 * width) + x1)]
//                    }
                    System.arraycopy(data, (yI * width + xI) + (y1 * width), frameData, y1 * frame.width, frame.width)
                }

                if (!frames[i].initialized) {
                    frames[i].initialized = true
                }


                xI += frame.width
                i++
            }
            yI += frames[y * allFramesX].height
        }
    }
}