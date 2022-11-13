package me.wcaleniewolny.ayaya.minecraft.render

import org.bukkit.plugin.java.JavaPlugin

interface RenderService {

    fun init(plugin: JavaPlugin)
    fun startRendering()
    fun pauseRendering()
    fun killRendering()

}