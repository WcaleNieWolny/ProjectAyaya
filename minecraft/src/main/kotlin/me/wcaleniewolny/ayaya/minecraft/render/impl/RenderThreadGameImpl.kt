package me.wcaleniewolny.ayaya.minecraft.render.impl

import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.minecraft.display.DisplayService
import me.wcaleniewolny.ayaya.minecraft.display.broadcaster.impl.MinecraftNativeBroadcaster
import me.wcaleniewolny.ayaya.minecraft.render.RenderThread
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.math.max

class RenderThreadGameImpl(
    private val displayService: DisplayService,
    private val startID: Int,
    private val renderCallback: ((ptr: Long, screenName: String) -> Unit)?,
    fps: Int,
    private val screenName: String,
    private val ptr: Long
) : RenderThread() {

    private val timeWindow = TimeUnit.SECONDS.toNanos(1) / fps
    private val debug = false
    private val renderFrames = AtomicBoolean(false)

    override fun renderFrames(): AtomicBoolean {
        return renderFrames
    }

    override fun ptr(): Long {
        return ptr
    }

    override fun run() {
        displayService.init()
        renderLoop()
    }

    private fun renderLoop() {
        while (true) {
            val start = System.nanoTime()
            val frame = NativeRenderControler.loadFrame(ptr)
            renderCallback?.invoke(ptr, screenName)

            // We use a magic value - 0 is a transparent color, and it is not possible to get when using normal video splitting
            // It is safe to assume that if we have a full frame 0 will never be present
            if (frame[0].toInt() != 0) {
                if (frame.size != 1) {
                    displayService.displayFrame(frame)
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
            } else {
                if (frame.size != 1) {
                    // https://stackoverflow.com/questions/2840190/java-convert-4-bytes-to-int
                    val dataStringLen = 0xFF and frame[1].toInt() shl 24 or (0xFF and frame[2].toInt() shl 16) or
                        (0xFF and frame[3].toInt() shl 8) or (0xFF and frame[4].toInt())

                    val dataStringArr = ByteArray(dataStringLen)
                    System.arraycopy(frame, 5, dataStringArr, 0, dataStringLen)
                    val dataString = String(dataStringArr, Charsets.UTF_8)
                    val dataSplit = dataString.split("$")

                    var offset = 0
                    for (split in dataSplit) {
                        val splitArr = split.split("_")
                        // Format: {frame_inxex}_{width}_{height}_{x1}_{y1}$
                        val frameIndex = splitArr[0].toInt()
                        val width = splitArr[1].toInt()
                        val height = splitArr[2].toInt()
                        val x1 = splitArr[3].toInt()
                        val y1 = splitArr[4].toInt()

                        val length = width * height
                        val data = ByteArray(length)

                        System.arraycopy(frame, 5 + offset + dataStringLen, data, 0, length)

                        val packet = MinecraftNativeBroadcaster.makeMapPacket(
                            startID + frameIndex,
                            x1,
                            y1,
                            width,
                            height,
                            data
                        )
                        displayService.allPlayers().forEach { player ->
                            MinecraftNativeBroadcaster.sendPacket(player, packet)
                        }

                        offset += length
                    }
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
            }

            while (!renderFrames.get()) {
                sleep(50)
            }
        }
    }
}
