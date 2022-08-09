package me.wcaleniewolny.ayaya.minecraft

import com.comphenix.protocol.PacketType
import com.comphenix.protocol.ProtocolLibrary
import com.comphenix.protocol.events.PacketContainer
import com.comphenix.protocol.utility.MinecraftReflection
import me.wcaleniewolny.ayaya.library.FrameSplitter
import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.library.SplittedFrame
import org.bukkit.command.Command
import org.bukkit.command.CommandExecutor
import org.bukkit.command.CommandSender
import org.bukkit.entity.Player
import org.bukkit.plugin.java.JavaPlugin

class MapMinecraftClient : CommandExecutor, JavaPlugin() {

    private val nativeRenderControler = NativeRenderControler()
    private val frames = mutableListOf<SplittedFrame>()
    private val ptr = nativeRenderControler.init("/home/wolny/Downloads/test.mp4")

    override fun onEnable() {
        getCommand("test")!!.setExecutor(this)
        println("PTR: $ptr")
        frames.addAll(FrameSplitter.initializeFrames(nativeRenderControler.width(ptr), nativeRenderControler.height(ptr)))
        println(ptr)
    }

    override fun onCommand(sender: CommandSender, command: Command, label: String, args: Array<out String>): Boolean {
        val frame = nativeRenderControler.loadFrame(ptr)
        FrameSplitter.splitFrames(frame, frames)
        val finalFrame = frames[0]

        val mapPacket = PacketContainer(PacketType.Play.Server.MAP)

        mapPacket.integers.write(0, 0) //Map ID
        mapPacket.bytes.write(0, 0) //Scale, do not change
        mapPacket.booleans.write(0, true) //lock the map
        mapPacket.modifier.write(3, null) //Decoration list

        val mapPatchClass = MinecraftReflection.getMinecraftClass("world.level.saveddata.maps.WorldMap\$b")
        val mapPatchConstructor = mapPatchClass.getConstructor(
            Int::class.javaPrimitiveType,
            Int::class.javaPrimitiveType,
            Int::class.javaPrimitiveType,
            Int::class.javaPrimitiveType,
            ByteArray::class.java
        )

        val mapPatchObject = mapPatchConstructor.newInstance(
            finalFrame.startX,
            finalFrame.startY,
            finalFrame.width,
            finalFrame.height,
            finalFrame.data
        )

        println(finalFrame)

        mapPacket.modifier.write(4, mapPatchObject)

        ProtocolLibrary.getProtocolManager().sendServerPacket(sender as Player, mapPacket)
        return true
    }
}