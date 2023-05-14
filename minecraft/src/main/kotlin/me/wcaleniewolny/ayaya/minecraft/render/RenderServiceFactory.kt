package me.wcaleniewolny.ayaya.minecraft.render

import me.wcaleniewolny.ayaya.library.MapServerOptions
import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.minecraft.command.VideoPlayType
import me.wcaleniewolny.ayaya.minecraft.display.DisplayService
import me.wcaleniewolny.ayaya.minecraft.display.broadcaster.impl.MinecraftNativeBroadcaster
import me.wcaleniewolny.ayaya.minecraft.display.impl.DisplayServiceImpl
import me.wcaleniewolny.ayaya.minecraft.display.impl.NettyRawDisplayServiceImpl
import me.wcaleniewolny.ayaya.minecraft.render.impl.JavaRenderServiceImpl
import me.wcaleniewolny.ayaya.minecraft.render.impl.NativeRenderServiceImpl
import me.wcaleniewolny.ayaya.minecraft.render.impl.RenderThreadGameImpl
import me.wcaleniewolny.ayaya.minecraft.render.impl.RenderThreadVideoImpl
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
        serviceType: RenderServiceType,
        videoPlayType: VideoPlayType,
        renderCallback: ((ptr: Long, screenName: String) -> Unit)? = null,
        useDiscord: Boolean = false
    ): RenderService {
        val nativeFilename = if (videoPlayType != VideoPlayType.BLAZING) filename else "${startID}$$$${filename}"
        val ptr = NativeRenderControler.init(
            nativeFilename,
            videoPlayType.toNativeRenderType(),
            MapServerOptions(
                useServer,
                plugin.config.getString("mapServerLocalIp")!!,
                plugin.config.getInt("mapServerPort")
            ),
            useDiscord
        )

        val videoData = NativeRenderControler.getVideoData(ptr)

        val width = videoData.width
        val height = videoData.height

        val fps = videoData.fps

        val displayService = if (videoPlayType != VideoPlayType.BLAZING) {
            DisplayServiceImpl(
                MinecraftNativeBroadcaster(startID),
                width,
                height
            )
        } else {
            NettyRawDisplayServiceImpl(width, height)
        }

        val thread =
            if (videoPlayType != VideoPlayType.GAME && videoPlayType != VideoPlayType.X11) {
                RenderThreadVideoImpl(
                    displayService,
                    renderCallback,
                    fps,
                    screenName,
                    ptr
                )
            } else {
                RenderThreadGameImpl(
                    displayService,
                    startID,
                    renderCallback,
                    fps,
                    screenName,
                    ptr
                )
            }

        val service = if (serviceType == RenderServiceType.JAVA) {
            JavaRenderServiceImpl(
                thread,
                useDiscord
            )
        } else {
            NativeRenderServiceImpl(
                plugin,
                videoData,
                MinecraftNativeBroadcaster(startID),
                startID,
                ptr
            )
        }

        service.init(plugin)

        return service
    }
}
