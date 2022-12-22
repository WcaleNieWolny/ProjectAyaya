package me.wcaleniewolny.ayaya.minecraft.render

import me.wcaleniewolny.ayaya.library.MapServerOptions
import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.minecraft.command.VideoPlayType
import me.wcaleniewolny.ayaya.minecraft.display.broadcaster.impl.MinecraftNativeBroadcaster
import me.wcaleniewolny.ayaya.minecraft.display.impl.DisplayServiceImpl
import me.wcaleniewolny.ayaya.minecraft.render.impl.JavaRenderServiceImpl
import me.wcaleniewolny.ayaya.minecraft.render.impl.NativeRenderServiceImpl
import org.bukkit.plugin.java.JavaPlugin

enum class RenderServiceType {
    NATIVE,
    JAVA
}

object RenderServiceFactory {

    fun create(
        plugin: JavaPlugin,
        filename: String,
        screenName: String,
        startID: Int,
        useServer: Boolean,
        service_type: RenderServiceType,
        videoPlayType: VideoPlayType,
        renderCallback: ((ptr: Long, screenName: String) -> Unit)? = null
    ): RenderService {
        val ptr = NativeRenderControler.init(
            filename,
            videoPlayType.toNativeRenderType(),
            MapServerOptions(
                useServer,
                plugin.config.getString("mapServerLocalIp")!!,
                plugin.config.getInt("mapServerPort")
            )
        )
        val videoData = NativeRenderControler.getVideoData(ptr)

        val width = videoData.width
        val height = videoData.height

        val fps = videoData.fps

        val service = if (service_type == RenderServiceType.JAVA) JavaRenderServiceImpl(
            RenderThread(
                DisplayServiceImpl(
                    //ProtocolLibBroadcaster(),
                    MinecraftNativeBroadcaster(startID),
                    width, height
                ),
                renderCallback,
                videoPlayType != VideoPlayType.GAME,
                fps,
                screenName,
                ptr
            )
        ) else NativeRenderServiceImpl(
            plugin,
            videoData,
            MinecraftNativeBroadcaster(startID),
            startID,
            ptr,
        )

        service.init(plugin)

        return service
    }
}