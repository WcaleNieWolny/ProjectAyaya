package me.wcaleniewolny.ayaya.minecraft.render

import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.minecraft.display.DisplayService
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.math.max

//Note: We assume that ptr is a valid pointer and nativeRenderControler has been initialized
class RenderThread(
    private val displayService: DisplayService,
    private val fps: Int,
    private val ptr: Long
) : Thread() {

    val renderFrames = AtomicBoolean(false)
    private var frame: ByteArray = ByteArray(0)
    private val timeWindow = oneFrameTimeWindow()

    private val debug = false

    override fun run() {
        displayService.init()
        renderLoop()
    }

    private fun renderLoop() {

        while (true) {
            val start = System.nanoTime();
            val frame = if (frame.isNotEmpty()) frame else NativeRenderControler.loadFrame(ptr)

            displayService.displayFrame(frame)

            this.frame = NativeRenderControler.loadFrame(ptr)

            val took = (System.nanoTime() - start)
            val toWait = max(0, timeWindow - took)
            val toWaitMilis = TimeUnit.NANOSECONDS.toMillis(toWait)
            if (toWait > 0) {
                sleep(toWaitMilis, (toWait - (toWaitMilis * 1000000)).toInt())
            }

            if(debug){
                println("DEBUG: toWait: $toWaitMilis ($toWait), took: ${TimeUnit.NANOSECONDS.toMillis(took)}")
            }


            while (!renderFrames.get()) {
                sleep(50)
            }
        }
    }

    private fun oneFrameTimeWindow(): Long {
        return TimeUnit.SECONDS.toNanos(1) / fps
    }
}

//Scheme:
//1. get frame form variable
//2. pass it to PlaybackService
//3. generate next frame and store it in a variable
//4. wait for the next frame
//5. call this scheme again (recursive function)