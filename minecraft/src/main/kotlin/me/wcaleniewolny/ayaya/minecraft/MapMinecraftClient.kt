package me.wcaleniewolny.ayaya.minecraft

import co.aikar.commands.PaperCommandManager
import me.wcaleniewolny.ayaya.minecraft.command.VideoCommand
import me.wcaleniewolny.ayaya.minecraft.command.VideoCommandCompletion
import me.wcaleniewolny.ayaya.minecraft.screen.ScreenController
import net.kyori.adventure.text.minimessage.MiniMessage
import org.bukkit.command.CommandSender
import org.bukkit.plugin.java.JavaPlugin


class MapMinecraftClient : JavaPlugin() {

    override fun onEnable() {
        this.saveDefaultConfig()

        val screenController = ScreenController(this);
        screenController.init()

        val manager = PaperCommandManager(this)
        val videoCommandCompletion = VideoCommandCompletion(screenController)

        videoCommandCompletion.init(this, manager)
        manager.registerCommand(
            VideoCommand(
                screenController,
                this.config,
                this
            )
        )

    }
}

fun CommandSender.sendColoredMessage(msg: String) {
    sendMessage(MiniMessage.miniMessage().deserialize(msg))
}