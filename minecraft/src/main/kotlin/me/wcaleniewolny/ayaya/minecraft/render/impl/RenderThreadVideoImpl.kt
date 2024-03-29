package me.wcaleniewolny.ayaya.minecraft.render.impl

import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.minecraft.display.DisplayService
import me.wcaleniewolny.ayaya.minecraft.render.RenderThread
import org.bukkit.Bukkit
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.math.max

// Note: We assume that ptr is a valid pointer and nativeRenderControler has been initialized
class RenderThreadVideoImpl(
    private val displayService: DisplayService,
    private val renderCallback: ((ptr: Long, screenName: String) -> Unit)?,
    private val fps: Int,
    private val screenName: String,
    private val ptr: Long
) : RenderThread() {

    private val renderFrames = AtomicBoolean(false)
    private var frame: ByteArray = ByteArray(0)
    private val timeWindow = oneFrameTimeWindow()

    private val debug = false

    override fun run() {
        displayService.init()
        renderLoop()
    }

    override fun renderFrames(): AtomicBoolean {
        return renderFrames
    }

    override fun ptr(): Long {
        return ptr
    }

    private fun renderLoop() {
        while (true) {
            val start = System.nanoTime()

            renderCallback?.invoke(ptr, screenName)
            var frame: ByteArray? = null

            try {
                frame = if (this.frame.isNotEmpty()) this.frame else NativeRenderControler.loadFrame(ptr)
            } catch (exception: RuntimeException) {
                if (exception.message?.contains("JVM frame reciver closed!") == true) {
                    Bukkit.getLogger().warning("Unable to receive frame from native code! End of file!")
                    break
                }
                Bukkit.getLogger().warning("Unable to receive frame from native code! Exiting!")
                exception.printStackTrace()
                break
            }

            displayService.displayFrame(frame)

            try {
                this.frame = NativeRenderControler.loadFrame(ptr)
            } catch (exception: RuntimeException) {
                if (exception.message?.contains("JVM frame reciver closed!") == true) {
                    Bukkit.getLogger().warning("Unable to receive frame from native code! End of file!")
                    break
                }
                Bukkit.getLogger().warning("Unable to receive frame from native code! Exiting!")
                exception.printStackTrace()
                break
            }

            val took = (System.nanoTime() - start)
            val toWait = max(0, timeWindow - took)
            val toWaitMilis = TimeUnit.NANOSECONDS.toMillis(toWait)
            if (toWait > 0) {
                sleep(toWaitMilis, (toWait - (toWaitMilis * 1000000)).toInt())
            }

            if (debug) {
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
