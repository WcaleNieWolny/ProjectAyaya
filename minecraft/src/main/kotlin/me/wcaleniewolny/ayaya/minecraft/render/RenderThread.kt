package me.wcaleniewolny.ayaya.minecraft.render

import java.util.concurrent.atomic.AtomicBoolean

abstract class RenderThread: Thread() {
    abstract fun renderFrames(): AtomicBoolean
    abstract fun ptr(): Long
    abstract override fun run();
}