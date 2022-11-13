package me.wcaleniewolny.ayaya.minecraft.render.impl

import io.netty.buffer.Unpooled
import me.wcaleniewolny.ayaya.minecraft.render.RenderService
import net.minecraft.network.FriendlyByteBuf
import org.bukkit.Bukkit
import org.bukkit.entity.Player
import org.bukkit.plugin.java.JavaPlugin
import org.bukkit.plugin.messaging.PluginMessageListener
import java.lang.Thread.sleep
import java.nio.charset.StandardCharsets
import java.util.UUID

class NativeRenderServiceImpl(
    private val plugin: JavaPlugin
) : RenderService, PluginMessageListener {

    val responders = ArrayList<UUID>()

    override fun init(plugin: JavaPlugin) {
        plugin.server.messenger.registerOutgoingPluginChannel(plugin, "fastmap:handshake")
        plugin.server.messenger.registerIncomingPluginChannel(plugin, "fastmap:handshake", this)
    }

    override fun startRendering() {
        val players = Bukkit.getOnlinePlayers()
        Bukkit.getScheduler().runTaskAsynchronously(plugin, Runnable {
            val ip = plugin.config.getString("mapServerRemoteIp")!!
            val buffer = FriendlyByteBuf(Unpooled.buffer())
            //Write IP adress
            buffer.writeVarInt(ip.length)
            buffer.writeBytes(ip.toByteArray(StandardCharsets.UTF_8))
            //Write port
            buffer.writeVarInt(plugin.config.getInt("mapServerPort"))

            players.forEach {
                it.sendPluginMessage(plugin, "fastmap:handshake", buffer.array())
            }

            sleep(500) //Wait for players to get the packet and respond to it

            players
                .filterNot { responders.contains(it.uniqueId) }
                .forEach { player ->
                    player.sendMessage("You do not have FastMap mod installed! We will not display cinema for you!")
                }
        })
    }

    override fun pauseRendering() {
        TODO("Not yet implemented")
    }

    override fun killRendering() {
        TODO("Not yet implemented")
    }

    override fun onPluginMessageReceived(channel: String, player: Player, message: ByteArray) {
        val buffer = FriendlyByteBuf(Unpooled.wrappedBuffer(message))
        val a = buffer.readVarInt()
        println("DATA: $a")

        if (!responders.contains(player.uniqueId)){
            responders.add(player.uniqueId)
        }
    }
}