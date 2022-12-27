package me.wcaleniewolny.ayaya.minecraft.display

import org.bukkit.entity.Player

interface DisplayService {
    fun displayFrame(data: ByteArray)
    fun init()
    fun allPlayers(): List<Player>
}