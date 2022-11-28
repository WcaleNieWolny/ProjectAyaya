package me.wcaleniewolny.ayaya.minecraft.command

import me.wcaleniewolny.ayaya.library.NativeRenderType

enum class VideoPlayType {
    SINGLE,
    MULTI,
    MAP_SERVER;

    fun toNativeRenderType(): NativeRenderType {
        return when (this) {
            SINGLE -> NativeRenderType.SINGLE_THREADED
            MULTI -> NativeRenderType.MULTI_THREADED
            MAP_SERVER -> NativeRenderType.MULTI_THREADED
        }
    }
}