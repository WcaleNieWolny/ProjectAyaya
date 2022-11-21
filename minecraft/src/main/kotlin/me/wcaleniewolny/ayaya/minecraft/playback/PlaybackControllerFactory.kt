package me.wcaleniewolny.ayaya.minecraft.playback

import me.wcaleniewolny.ayaya.library.MapServerOptions
import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.library.NativeRenderType
import me.wcaleniewolny.ayaya.minecraft.display.broadcaster.impl.MinecraftNativeBroadcaster
import me.wcaleniewolny.ayaya.minecraft.display.impl.DisplayServiceImpl
import me.wcaleniewolny.ayaya.minecraft.render.RenderThread
import me.wcaleniewolny.ayaya.minecraft.render.impl.JavaRenderServiceImpl
import me.wcaleniewolny.ayaya.minecraft.render.impl.NativeRenderServiceImpl
import org.bukkit.plugin.java.JavaPlugin

enum class RenderServiceType {
    NATIVE,
    JAVA
}

object PlaybackControllerFactory {

    fun create(
        plugin: JavaPlugin,
        filename: String,
        type: RenderServiceType
    ): PlaybackController {
        val useServer = type == RenderServiceType.NATIVE
        val ptr = NativeRenderControler.init(
            filename, NativeRenderType.MULTI_THREADED, MapServerOptions(
                useServer,
                plugin.config.getString("mapServerLocalIp")!!,
                plugin.config.getInt("mapServerPort")
            )
        )
        val videoData = NativeRenderControler.getVideoData(ptr)
        println("DATA: $videoData")

        val width = videoData.width
        val height = videoData.height

        //val fps = videoData.fps
        val fps = videoData.fps

        val service = if (type == RenderServiceType.JAVA) JavaRenderServiceImpl(
            RenderThread(
                DisplayServiceImpl(
                    //ProtocolLibBroadcaster(),
                    MinecraftNativeBroadcaster(),
                    width, height
                ),
                fps,
                ptr
            )
        ) else NativeRenderServiceImpl(
            plugin,
            videoData,
            MinecraftNativeBroadcaster(),
            ptr,
        )

        service.init(plugin)

        return PlaybackController(
            service
        )
    }
}