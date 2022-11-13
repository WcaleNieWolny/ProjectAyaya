package me.wcaleniewolny.ayaya.minecraft.playback

import me.wcaleniewolny.ayaya.minecraft.render.RenderService
import me.wcaleniewolny.ayaya.minecraft.render.impl.JavaRenderServiceImpl

class PlaybackController(
    private val renderService: RenderService
) {

    fun startPlayback() {
        renderService.startRendering()
    }

    /*
    Serves both as pause and unpause function
     */
    fun pausePlayback() {
        renderService.pauseRendering()
    }
}