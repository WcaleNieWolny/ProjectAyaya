package me.wcaleniewolny.ayaya.minecraft.display.broadcaster.impl

import com.comphenix.protocol.PacketType
import com.comphenix.protocol.ProtocolLibrary
import com.comphenix.protocol.events.PacketContainer
import com.comphenix.protocol.utility.MinecraftReflection
import me.wcaleniewolny.ayaya.library.SplittedFrame
import me.wcaleniewolny.ayaya.minecraft.display.broadcaster.Broadcaster
import org.bukkit.entity.Player

class ProtocolLibBroadcaster: Broadcaster {

    override fun sendPackets(data: MutableList<SplittedFrame>, players: List<Player>) {

        for(i in 0 until data.size){
            val mapPacket = PacketContainer(PacketType.Play.Server.MAP)
            val frame = data[i]

            if(frame.data.isEmpty()){
                continue
            }

            //TODO: Do not have static map ID
            mapPacket.integers.write(0, i) //Map ID
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
                frame.startX,
                frame.startY,
                frame.width,
                frame.height,
                frame.data
            )

            mapPacket.modifier.write(4, mapPatchObject)

            players.forEach{
                ProtocolLibrary.getProtocolManager().sendServerPacket(it, mapPacket)
            }

        }
    }
}