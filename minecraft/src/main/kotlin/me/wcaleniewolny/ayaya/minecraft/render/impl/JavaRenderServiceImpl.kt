package me.wcaleniewolny.ayaya.minecraft.render.impl

import me.wcaleniewolny.ayaya.library.NativeLibCommunication
import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.minecraft.render.RenderService
import me.wcaleniewolny.ayaya.minecraft.render.RenderThread
import org.bukkit.plugin.java.JavaPlugin

open class JavaRenderServiceImpl(
    private val renderThread: RenderThread,
    private val useDiscord: Boolean
) : RenderService {

    private var initialized = false
    override fun init(plugin: JavaPlugin) {
        //Do nothing
    }

    override fun startRendering() {
        if (!initialized) {
            renderThread.renderFrames().set(true)
        }
        renderThread.name = "ProjectAyaya Render Thread"
        renderThread.priority = Thread.MAX_PRIORITY
        renderThread.start()
        initialized = true
    }

    override fun pauseRendering() {
        if (!initialized) {
            throw IllegalStateException("Cannot pause rendering due to render thread being not initialized")
        }

        val isRunning = renderThread.renderFrames()
        if (isRunning.get()) {
            isRunning.set(false)
            if (useDiscord) {
                NativeRenderControler.communicate(renderThread.ptr(), NativeLibCommunication.STOP_RENDERING, "1")
            }
        } else {
            isRunning.set(true)
            if (useDiscord) {
                NativeRenderControler.communicate(renderThread.ptr(), NativeLibCommunication.START_RENDERING, "1")
            }
        }
    }

    override fun killRendering() {
        renderThread.renderFrames().set(false)
        NativeRenderControler.destroy(renderThread.ptr())
    }

    override fun seekSecond(second: Int) {
        NativeRenderControler.communicate(renderThread.ptr(), NativeLibCommunication.VIDEO_SEEK, second.toString())
    }
}