package me.wcaleniewolny.ayaya.minecraft.render

class RenderService(
    private val renderThread: RenderThread
) {

    private var initialized = false

    fun startRendering() {
        if (!initialized) {
            renderThread.renderFrames.set(true)
        }
        renderThread.name = "ProjectAyaya Render Thread"
        renderThread.priority = Thread.MAX_PRIORITY
        renderThread.start()
        initialized = true
    }

    fun pauseRendering() {
        if (!initialized) {
            throw IllegalStateException("Cannot pause rendering due to render thread being not initialized")
        }
        renderThread.renderFrames.set(false)
    }

    fun killRendering() {

    }

}