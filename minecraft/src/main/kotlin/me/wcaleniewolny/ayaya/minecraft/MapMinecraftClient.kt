package me.wcaleniewolny.ayaya.minecraft

import co.aikar.commands.PaperCommandManager
import me.wcaleniewolny.ayaya.minecraft.command.VideoCommand
import me.wcaleniewolny.ayaya.minecraft.command.VideoCommandCompletion
import me.wcaleniewolny.ayaya.minecraft.map.MapScreen
import me.wcaleniewolny.ayaya.minecraft.playback.PlaybackControllerFactory
import me.wcaleniewolny.ayaya.minecraft.playback.RenderServiceType
import me.wcaleniewolny.ayaya.minecraft.screen.ScreenController
import net.kyori.adventure.text.minimessage.MiniMessage
import net.kyori.adventure.text.serializer.legacy.LegacyComponentSerializer
import org.bukkit.Bukkit
import org.bukkit.block.BlockFace
import org.bukkit.command.Command
import org.bukkit.command.CommandExecutor
import org.bukkit.command.CommandSender
import org.bukkit.plugin.java.JavaPlugin
import org.bukkit.util.Vector


class MapMinecraftClient : CommandExecutor, JavaPlugin() {

    private val fileName = "/home/wolny/Downloads/4k_test.mp4"
    private val playbackController = PlaybackControllerFactory.create(this, fileName, RenderServiceType.NATIVE)

    override fun onEnable() {
        this.saveDefaultConfig()

        val screenController = ScreenController(this);
        screenController.init()

        val manager = PaperCommandManager(this)
        val videoCommandCompletion = VideoCommandCompletion(screenController)

        videoCommandCompletion.init(this, manager)
        manager.registerCommand(VideoCommand(
            screenController,
            this.config
        ))

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
                    Vector(-13, 74, 18),
                    Vector(-13, 66, 32),
                    BlockFace.WEST,
                    Bukkit.getWorld("world")!!
                )

                screen.buildScreen()
            }
        }

        return true
    }
}

fun CommandSender.sendColoredMessage(msg: String){
    sendMessage(MiniMessage.miniMessage().deserialize(msg))
}