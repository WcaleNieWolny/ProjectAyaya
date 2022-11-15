package me.wcaleniewolny.ayaya.minecraft.render.impl

import io.netty.buffer.Unpooled
import me.wcaleniewolny.ayaya.library.FrameSplitter
import me.wcaleniewolny.ayaya.library.NativeLibCommunication
import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.library.VideoData
import me.wcaleniewolny.ayaya.minecraft.render.RenderService
import net.minecraft.network.FriendlyByteBuf
import org.bukkit.Bukkit
import org.bukkit.entity.Player
import org.bukkit.plugin.java.JavaPlugin
import org.bukkit.plugin.messaging.PluginMessageListener
import java.lang.Thread.sleep
import java.nio.charset.StandardCharsets
import java.util.*

class NativeRenderServiceImpl(
    private val plugin: JavaPlugin,
    private val videoData: VideoData,
    private val ptr: Long
) : RenderService, PluginMessageListener {

    companion object {
        const val PROTOCOL_VERSION = 0
    }

    val responders = ArrayList<UUID>()

    override fun init(plugin: JavaPlugin) {

        plugin.server.messenger.registerOutgoingPluginChannel(plugin, "fastmap:handshake")
        plugin.server.messenger.registerIncomingPluginChannel(plugin, "fastmap:handshake", this)
        plugin.server.messenger.registerOutgoingPluginChannel(plugin, "fastmap:acknowledgement")
        plugin.server.messenger.registerIncomingPluginChannel(plugin, "fastmap:acknowledgement", this)
    }

    override fun startRendering() {
        val players = Bukkit.getOnlinePlayers()
        Bukkit.getScheduler().runTaskAsynchronously(plugin, Runnable {
            sendAcknowledgementPackets()

            sleep(500) //Wait for players to get the packet and respond to it

            players
                .filterNot { responders.contains(it.uniqueId) }
                .forEach { player ->
                    player.sendMessage("You do not have FastMap mod installed! We will not display cinema for you!")
                }

            sendHandshakePackets(
                players
                    .filter { responders.contains(it.uniqueId) }
                    .map { it as Player }
            )

            NativeRenderControler.communicate(ptr, NativeLibCommunication.START_RENDER)
        })
    }

    override fun pauseRendering() {
        TODO("Not yet implemented")
    }

    override fun killRendering() {
        TODO("Not yet implemented")
    }

    private fun sendAcknowledgementPackets(){
        val players = Bukkit.getOnlinePlayers()

        val buffer = FriendlyByteBuf(Unpooled.buffer())
        buffer.writeVarInt(PROTOCOL_VERSION)

        players.forEach {
            it.sendPluginMessage(plugin, "fastmap:acknowledgement", buffer.array())
        }
    }

    private fun sendHandshakePackets(players: List<Player>){

        val data = FrameSplitter.getRederData(videoData.width, videoData.height)

        val ip = plugin.config.getString("mapServerRemoteIp")!!
        val buffer = FriendlyByteBuf(Unpooled.buffer())

        //Write IP adress
        buffer.writeVarInt(ip.length)
        buffer.writeBytes(ip.toByteArray(StandardCharsets.UTF_8))

        //Write port
        buffer.writeVarInt(plugin.config.getInt("mapServerPort"))

        //Write render data
        data.forEach {
            buffer.writeVarInt(it)
        }

        players.forEach {
            it.sendPluginMessage(plugin, "fastmap:handshake", buffer.array())
        }
    }

    override fun onPluginMessageReceived(channel: String, player: Player, message: ByteArray) {
        val buffer = FriendlyByteBuf(Unpooled.wrappedBuffer(message))
        val status = buffer.readVarInt()

        when (channel) {
            "fastmap:acknowledgement" -> {
                if (!responders.contains(player.uniqueId)){
                    responders.add(player.uniqueId)
                }
            }
            "fastmap:handshake" -> {
                if(status != 0){
                    Bukkit.getLogger().warning("User ${player.name} couldn't connect to map server. If this message scours multiple time it means the server is configured in a wrong way!")
                }
            }
        }
    }

}