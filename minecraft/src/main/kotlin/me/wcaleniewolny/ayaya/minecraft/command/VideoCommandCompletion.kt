package me.wcaleniewolny.ayaya.minecraft.command

import co.aikar.commands.PaperCommandManager
import me.wcaleniewolny.ayaya.minecraft.screen.ScreenController
import org.bukkit.plugin.java.JavaPlugin
import java.io.File

class VideoCommandCompletion(
    private val screenController: ScreenController
) {

    fun init(plugin: JavaPlugin, manager: PaperCommandManager) {
        val dir = File(plugin.dataFolder, "video")
        if (!dir.exists()) {
            dir.mkdirs()
        }

        manager.commandCompletions.registerAsyncCompletion("video") {
            return@registerAsyncCompletion dir.listFiles().map { it.name }
        }

        manager.commandCompletions.registerAsyncCompletion("lookingAt") {

            val lookingAt = it.player.getTargetBlock(4) ?: return@registerAsyncCompletion mutableListOf();

            return@registerAsyncCompletion mutableListOf("${lookingAt.x} ${lookingAt.y} ${lookingAt.z}")
        }

        manager.commandCompletions.registerAsyncCompletion("screens") {
            return@registerAsyncCompletion screenController.getScreens().map { it.name }
        }

        manager.commandCompletions.registerAsyncCompletion("screenFacing") {
            return@registerAsyncCompletion mutableListOf("north", "east", "south", "west")
        }

        manager.commandCompletions.registerAsyncCompletion("videoPlayType") {
            return@registerAsyncCompletion mutableListOf("single", "multi", "map_server")
        }

        manager.commandCompletions.registerAsyncCompletion("games") {
            return@registerAsyncCompletion mutableListOf("flappy_bird")
        }
    }

}