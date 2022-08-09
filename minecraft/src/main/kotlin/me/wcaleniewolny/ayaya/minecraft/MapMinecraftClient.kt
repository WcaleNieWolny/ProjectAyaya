package me.wcaleniewolny.ayaya.minecraft

import com.comphenix.protocol.PacketType
import com.comphenix.protocol.ProtocolLibrary
import com.comphenix.protocol.events.PacketContainer
import com.comphenix.protocol.utility.MinecraftReflection
import me.wcaleniewolny.ayaya.library.FrameSplitter
import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.library.SplittedFrame
import me.wcaleniewolny.ayaya.minecraft.render.RenderServiceFactory
import org.bukkit.command.Command
import org.bukkit.command.CommandExecutor
import org.bukkit.command.CommandSender
import org.bukkit.entity.Player
import org.bukkit.plugin.java.JavaPlugin

class MapMinecraftClient : CommandExecutor, JavaPlugin() {

    private val fileName =  "/home/wolny/Downloads/test.mp4"
    //private val fileName = "/home/wolny/IdeaProjects/ProjectAyaya/library/src/test/resources/test.webm"
    private val renderService = RenderServiceFactory.create(fileName)

    override fun onEnable() {
        getCommand("test")!!.setExecutor(this)

    }

    override fun onCommand(sender: CommandSender, command: Command, label: String, args: Array<out String>): Boolean {
        renderService.startRendering()

        return true
    }
}