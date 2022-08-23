package me.wcaleniewolny.ayaya.minecraft

import me.wcaleniewolny.ayaya.minecraft.map.MapScreen
import me.wcaleniewolny.ayaya.minecraft.playback.PlaybackControllerFactory
import org.bukkit.Bukkit
import org.bukkit.block.BlockFace
import org.bukkit.command.Command
import org.bukkit.command.CommandExecutor
import org.bukkit.command.CommandSender
import org.bukkit.plugin.java.JavaPlugin
import org.bukkit.util.Vector

class MapMinecraftClient : CommandExecutor, JavaPlugin() {

    //private val fileName =  "/home/wolny/Downloads/test.mp4"
    private val fileName = "/home/wolny/Downloads/4k_test.mp4"
    private val playbackController = PlaybackControllerFactory.create(fileName)

    override fun onEnable() {
        getCommand("test")!!.setExecutor(this)

    }

    override fun onCommand(sender: CommandSender, command: Command, label: String, args: Array<out String>): Boolean {
        when (args.size) {
            0 -> playbackController.startPlayback()
            1 -> {
                sender.sendMessage("PAUSE!")
                playbackController.pausePlayback()
            }
            2 -> {
                val screen = MapScreen(
                    Vector(-13, 82, 36),
                    Vector(-13, 66, 65),
                    BlockFace.WEST,
                    Bukkit.getWorld("world")!!
                )

                screen.buildScreen()
            }
        }

        return true
    }
}