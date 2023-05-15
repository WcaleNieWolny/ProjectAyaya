package me.wcaleniewolny.ayaya.minecraft.display.impl

import io.netty.buffer.Unpooled
import me.wcaleniewolny.ayaya.minecraft.display.DisplayService
import net.minecraft.network.FriendlyByteBuf
import org.bukkit.Bukkit
import org.bukkit.craftbukkit.v1_18_R2.entity.CraftPlayer
import org.bukkit.entity.Player

class NettyRawDisplayServiceImpl (
    private val width: Int,
    private val height: Int
) : DisplayService {
    @Suppress("UNUSED_VARIABLE")
    override fun displayFrame(data: ByteArray) {
        for (player in allPlayers()) {
            val buf = Unpooled.copiedBuffer(data)
            val friendly = FriendlyByteBuf(buf)

            println("Sending shit!")
            (player as CraftPlayer).handle.connection.connection.channel.pipeline().firstContext().writeAndFlush(Unpooled.wrappedBuffer(data))
        //writeAndFlush(Unpooled.wrappedBuffer(data))
        }

    }

    override fun init() {}

    override fun allPlayers() = Bukkit.getServer().onlinePlayers.map { it as Player }
}