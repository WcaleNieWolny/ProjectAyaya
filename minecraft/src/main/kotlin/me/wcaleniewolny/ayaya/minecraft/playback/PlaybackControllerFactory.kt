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
        val ptr = NativeRenderControler.init(filename, false)

        //NativeRenderControler.startMultithreading(ptr)

        val width = NativeRenderControler.width(ptr)
        val height = NativeRenderControler.height(ptr)

        val fps = 30 //TODO: use FFMPEG to extract it later from video

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