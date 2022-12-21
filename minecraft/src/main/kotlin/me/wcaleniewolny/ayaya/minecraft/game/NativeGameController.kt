package me.wcaleniewolny.ayaya.minecraft.game

import me.wcaleniewolny.ayaya.minecraft.screen.Screen
import org.bukkit.Bukkit
import org.bukkit.Location
import org.bukkit.block.BlockFace
import org.bukkit.entity.Player
import org.bukkit.event.EventHandler
import org.bukkit.event.EventPriority
import org.bukkit.event.Listener
import org.bukkit.event.block.BlockBreakEvent
import org.bukkit.event.entity.EntityDamageByEntityEvent
import org.bukkit.event.entity.EntityDamageEvent
import org.bukkit.event.player.PlayerDropItemEvent
import org.bukkit.event.player.PlayerMoveEvent
import org.bukkit.plugin.java.JavaPlugin
import org.bukkit.util.Vector
import kotlin.math.E

class NativeGameController(private val plugin: JavaPlugin): Listener {

    private val games = mutableListOf<NativeGame>()

    @EventHandler(priority = EventPriority.HIGH)
    private fun onMoveEvent(event: PlayerMoveEvent){
        val game = games.firstOrNull { it.player == event.player } ?: return

        event.isCancelled = true
    }

    @EventHandler(priority = EventPriority.HIGH)
    private fun onDropEvent(event: PlayerDropItemEvent){
        games.firstOrNull { it.player == event.player } ?: return

        event.isCancelled = true
    }

    @EventHandler(priority = EventPriority.HIGH)
    private fun onDestroyEvent(event: BlockBreakEvent){
        games.firstOrNull { it.player == event.player } ?: return

        event.isCancelled = true
    }

    @EventHandler(priority = EventPriority.HIGH)
    private fun onPlayerDamage(event: EntityDamageEvent){
        val player = if (event.entity is Player) event.entity as Player else return;
        games.firstOrNull { it.player == player } ?: return

        event.isCancelled = true
    }

    @EventHandler(priority = EventPriority.HIGH)
    private fun onPlayerAttack(event: EntityDamageByEntityEvent){
        val player = if (event.damager is Player) event.damager as Player else return;
        games.firstOrNull { it.player == player } ?: return

        event.isCancelled = true
    }

    fun init(){
        Bukkit.getServer().pluginManager.registerEvents(this, plugin)
    }

    fun registerGamer(player: Player, screen: Screen){
        games.add(NativeGame(screen, player))

        val facing = when (screen.mapFace){
            BlockFace.NORTH -> BlockFace.SOUTH
            BlockFace.SOUTH -> BlockFace.NORTH
            BlockFace.EAST -> BlockFace.WEST
            BlockFace.WEST -> BlockFace.EAST
            else -> return
        }

        var location = Location(
            screen.world,
            screen.gameX.toDouble(),
            screen.gameY.toDouble(),
            screen.gameZ.toDouble()
        )
        location.direction = facing.direction
        location = location.add(Vector(0.5, 1.0, 0.5))

        player.teleport(location)
    }

    fun unregisterScreen(screen: Screen){
        val gameIndex = games.indexOfFirst { it.screen == screen }
        if(gameIndex == -1) {
            return
        }
        games.removeAt(gameIndex)
    }
}