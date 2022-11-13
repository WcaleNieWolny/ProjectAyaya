package me.wcaleniewolny.ayaya.minecraft.display.impl

import me.wcaleniewolny.ayaya.library.FrameSplitter
import me.wcaleniewolny.ayaya.library.SplittedFrame
import me.wcaleniewolny.ayaya.minecraft.display.DisplayService
import me.wcaleniewolny.ayaya.minecraft.display.broadcaster.Broadcaster
import me.wcaleniewolny.ayaya.minecraft.map.MapCleanerService
import org.bukkit.Bukkit
import org.bukkit.entity.Player

class DisplayServiceImpl(
    private val broadcaster: Broadcaster,
    private val width: Int,
    private val height: Int
) : DisplayService {


    private var initialized = false
    private val frames = mutableListOf<SplittedFrame>()

    override fun displayFrame(data: ByteArray) {
        if (!initialized) {
            throw IllegalStateException("PlaybackService is not initialized")
        }

        FrameSplitter.splitFrames(data, frames, width)

        broadcaster.sendPackets(frames, allPlayers())

    }

    private fun allPlayers() = Bukkit.getServer().onlinePlayers.map { it as Player }

    override fun init() {
        frames.addAll(FrameSplitter.initializeFrames(width, height)) //Initialize frames

        val players = allPlayers()
        broadcaster.init(players);

        broadcaster.blackoutFrames(frames, allPlayers())
        MapCleanerService.cleanMaps(0, frames.size)
        initialized = true
    }
}