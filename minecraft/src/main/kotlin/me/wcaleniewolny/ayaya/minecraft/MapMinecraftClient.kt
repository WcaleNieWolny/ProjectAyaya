package me.wcaleniewolny.ayaya.minecraft

import me.wcaleniewolny.ayaya.minecraft.playback.PlaybackControllerFactory
import org.bukkit.command.Command
import org.bukkit.command.CommandExecutor
import org.bukkit.command.CommandSender
import org.bukkit.plugin.java.JavaPlugin

class MapMinecraftClient : CommandExecutor, JavaPlugin() {

    //private val fileName =  "/home/wolny/Downloads/test.mp4"
    private val fileName = "/home/wolny/IdeaProjects/ProjectAyaya/library/src/test/resources/test.mp4"
    private val playbackController = PlaybackControllerFactory.create(fileName)

    override fun onEnable() {
        getCommand("test")!!.setExecutor(this)

    }

    override fun onCommand(sender: CommandSender, command: Command, label: String, args: Array<out String>): Boolean {
        when (args.size){
            0 -> playbackController.startPlayback()
            1 -> {
                sender.sendMessage("PAUSE!")
                playbackController.pausePlayback()
            }
        }

        return true
    }
}