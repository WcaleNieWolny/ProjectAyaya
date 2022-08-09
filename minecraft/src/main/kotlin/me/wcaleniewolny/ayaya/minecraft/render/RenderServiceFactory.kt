package me.wcaleniewolny.ayaya.minecraft.render

import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.minecraft.display.broadcaster.impl.ProtocolLibBroadcaster
import me.wcaleniewolny.ayaya.minecraft.display.impl.DisplayServiceImpl

object RenderServiceFactory {

    fun create(
        filename: String,
    ): RenderService{
        val ptr = NativeRenderControler.init(filename)
        val width = NativeRenderControler.width(ptr)
        val height = NativeRenderControler.height(ptr)

        val fps = 30 //TODO: use FFMPEG to extract it later from video

        return RenderService(
            RenderThread(
                DisplayServiceImpl(
                    ProtocolLibBroadcaster(),
                    width, height
                ),
                fps,
                ptr
            )
        )
    }
}