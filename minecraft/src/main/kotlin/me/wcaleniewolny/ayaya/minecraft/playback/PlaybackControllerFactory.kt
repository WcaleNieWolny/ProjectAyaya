package me.wcaleniewolny.ayaya.minecraft.playback

import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.minecraft.display.broadcaster.impl.ProtocolLibBroadcaster
import me.wcaleniewolny.ayaya.minecraft.display.impl.DisplayServiceImpl
import me.wcaleniewolny.ayaya.minecraft.render.RenderService
import me.wcaleniewolny.ayaya.minecraft.render.RenderThread

object PlaybackControllerFactory {

    fun create(
        filename: String,
    ): PlaybackController {
        val ptr = NativeRenderControler.init(filename, true)
        val videoData = NativeRenderControler.getVideoData(ptr)
        println("DATA: $videoData")

        val width = videoData.width
        val height = videoData.height

        val fps = videoData.fps

        return PlaybackController(
            RenderService(
                RenderThread(
                    DisplayServiceImpl(
                        ProtocolLibBroadcaster(),
                        width, height
                    ),
                    fps,
                    ptr
                )
            )
        )
    }
}