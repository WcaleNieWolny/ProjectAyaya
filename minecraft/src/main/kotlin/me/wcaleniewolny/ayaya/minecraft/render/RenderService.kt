package me.wcaleniewolny.ayaya.minecraft.render

class RenderService(
    private val renderThread: RenderThread
) {

    fun startRendering(){
        renderThread.name = "ProjectAyaya Render Thread"
        renderThread.priority = Thread.MAX_PRIORITY
        renderThread.start()
    }

    fun pauseRendering(){

    }

    fun killRendering(){

    }

}