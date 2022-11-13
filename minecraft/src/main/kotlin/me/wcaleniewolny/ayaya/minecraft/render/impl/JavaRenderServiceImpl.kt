package me.wcaleniewolny.ayaya.minecraft.render.impl

import me.wcaleniewolny.ayaya.minecraft.render.RenderService
import me.wcaleniewolny.ayaya.minecraft.render.RenderThread
import org.bukkit.plugin.java.JavaPlugin

class JavaRenderServiceImpl(
    private val renderThread: RenderThread
) : RenderService {

    private var initialized = false
    override fun init(plugin: JavaPlugin) {
        TODO("Not yet implemented")
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
        renderThread.renderFrames.set(false)
    }

    override fun killRendering() {

    }

}