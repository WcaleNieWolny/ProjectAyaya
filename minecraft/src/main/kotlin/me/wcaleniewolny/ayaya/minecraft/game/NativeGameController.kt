package me.wcaleniewolny.ayaya.minecraft.game

import me.wcaleniewolny.ayaya.minecraft.screen.Screen
import org.bukkit.Bukkit
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
        println("${player.name} become a game!")
        games.add(NativeGame(screen, player))
    }
}