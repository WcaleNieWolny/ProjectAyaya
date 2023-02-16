package me.wcaleniewolny.ayaya.minecraft.display.broadcaster

import me.wcaleniewolny.ayaya.library.SplittedFrame
import org.bukkit.entity.Player

interface Broadcaster {
    fun init(players: List<Player>)

    fun sendPackets(data: MutableList<SplittedFrame>, players: List<Player>)

    fun blackoutFrames(data: MutableList<SplittedFrame>, players: List<Player>)
}
