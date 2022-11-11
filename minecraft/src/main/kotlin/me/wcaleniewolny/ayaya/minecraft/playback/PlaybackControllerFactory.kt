package me.wcaleniewolny.ayaya.minecraft.playback

import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.library.NativeRenderType
import me.wcaleniewolny.ayaya.minecraft.display.broadcaster.impl.NativeMinecraftBroadcaster
import me.wcaleniewolny.ayaya.minecraft.display.impl.DisplayServiceImpl
import me.wcaleniewolny.ayaya.minecraft.render.RenderService
import me.wcaleniewolny.ayaya.minecraft.render.RenderThread

object PlaybackControllerFactory {

    fun create(
        filename: String,
    ): PlaybackController {
        val ptr = NativeRenderControler.init(filename, NativeRenderType.MULTI_THREADED)
        val videoData = NativeRenderControler.getVideoData(ptr)
        println("DATA: $videoData")

        val width = videoData.width
        val height = videoData.height

        //val fps = videoData.fps
        val fps = videoData.fps

        return PlaybackController(
            RenderService(
                RenderThread(
                    DisplayServiceImpl(
                        //ProtocolLibBroadcaster(),
                        NativeMinecraftBroadcaster(),
                        width, height
                    ),
                    fps,
                    ptr
                )
            )
        )
    }
}