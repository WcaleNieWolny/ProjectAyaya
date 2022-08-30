package me.wcaleniewolny.ayaya.minecraft.display.broadcaster.impl

import me.wcaleniewolny.ayaya.library.SplittedFrame
import me.wcaleniewolny.ayaya.minecraft.display.broadcaster.Broadcaster
import net.minecraft.network.protocol.game.ClientboundMapItemDataPacket
import net.minecraft.world.level.saveddata.maps.MapItemSavedData
import org.bukkit.craftbukkit.v1_18_R2.entity.CraftPlayer
import org.bukkit.entity.Player

class NativeMinecraftBroadcaster : Broadcaster {
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
                (it as CraftPlayer).handle.connection.send(mapPacket)
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
                (it as CraftPlayer).handle.connection.send(mapPacket)
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
    ): ClientboundMapItemDataPacket {
        return ClientboundMapItemDataPacket(
            id,
            0,
            false,
            null,
            MapItemSavedData.MapPatch(
                startX,
                startY,
                width,
                height,
                data
            )
        )
    }
}