package me.wcaleniewolny.ayaya.minecraft.render.impl

import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.minecraft.render.RenderService
import me.wcaleniewolny.ayaya.minecraft.render.RenderThread
import org.bukkit.plugin.java.JavaPlugin

class JavaRenderServiceImpl(
    private val renderThread: RenderThread
) : RenderService {

    private var initialized = false
    override fun init(plugin: JavaPlugin) {
        //Do nothing
    }

    override fun startRendering() {
        if (!initialized) {
            renderThread.renderFrames.set(true)
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

        val isRunning = renderThread.renderFrames
        if (isRunning.get()) {
            isRunning.set(false)
        } else {
            isRunning.set(true)
        }
    }

    override fun killRendering() {
        //Potential race condition
        //If we destroy native resources in the time when render thread is  getting a frame we have a SEGFAULT
        renderThread.renderFrames.set(false)
        NativeRenderControler.destroy(renderThread.ptr)
    }

}