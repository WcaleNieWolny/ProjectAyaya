package me.wcaleniewolny.ayaya.minecraft.display.broadcaster

import me.wcaleniewolny.ayaya.library.SplittedFrame
import org.bukkit.entity.Player

interface Broadcaster {
    fun sendPackets(data: MutableList<SplittedFrame>, players: List<Player>)
}