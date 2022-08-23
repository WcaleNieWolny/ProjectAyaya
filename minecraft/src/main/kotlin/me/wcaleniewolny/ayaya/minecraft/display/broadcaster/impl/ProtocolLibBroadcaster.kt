package me.wcaleniewolny.ayaya.minecraft.display.broadcaster.impl

import com.comphenix.protocol.PacketType
import com.comphenix.protocol.ProtocolLibrary
import com.comphenix.protocol.events.PacketContainer
import com.comphenix.protocol.utility.MinecraftReflection
import me.wcaleniewolny.ayaya.library.SplittedFrame
import me.wcaleniewolny.ayaya.minecraft.display.broadcaster.Broadcaster
import org.bukkit.entity.Player

class ProtocolLibBroadcaster : Broadcaster {

    override fun sendPackets(data: MutableList<SplittedFrame>, players: List<Player>) {

        for (i in 0 until data.size) {
            val frame = data[i]

            //TODO: Do not have static map ID
            val mapPacket = makeMapPacket(
                i,
                frame.startX,
                frame.startY,
                frame.width,
                frame.height,
                frame.data
            )

            players.forEach {
                ProtocolLibrary.getProtocolManager().sendServerPacket(it, mapPacket)
            }

        }
    }

    override fun blackoutFrames(data: MutableList<SplittedFrame>, players: List<Player>) {
        for (i in 0 until data.size) {
            val mapPacket = makeMapPacket(
                i,
                0,
                0,
                128,
                128,
                ByteArray(16384) { 119 }
                //119 is probably the blackest you can get. -49 is technically closer to black but is kind of red and 119 is black but a little grayish
            )

            players.forEach {
                ProtocolLibrary.getProtocolManager().sendServerPacket(it, mapPacket)
            }
        }
    }

    private fun makeMapPacket(
        id: Int,
        startX: Int,
        startY: Int,
        width: Int,
        height: Int,
        data: ByteArray
    ): PacketContainer {
        val mapPacket = PacketContainer(PacketType.Play.Server.MAP)
        mapPacket.integers.write(0, id) //Map ID
        mapPacket.bytes.write(0, 0) //Scale, do not change
        mapPacket.booleans.write(0, false) //lock the map
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
            startX,
            startY,
            width,
            height,
            data
        )

        mapPacket.modifier.write(4, mapPatchObject)

        return mapPacket
    }
}