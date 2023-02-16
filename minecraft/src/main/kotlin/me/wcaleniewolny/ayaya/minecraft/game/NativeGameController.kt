package me.wcaleniewolny.ayaya.minecraft.game

import me.wcaleniewolny.ayaya.library.NativeLibCommunication
import me.wcaleniewolny.ayaya.library.NativeRenderControler
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
import org.bukkit.event.player.PlayerQuitEvent
import org.bukkit.plugin.java.JavaPlugin
import org.bukkit.potion.PotionEffect
import org.bukkit.potion.PotionEffectType
import org.bukkit.util.Vector
import java.util.*
import java.util.concurrent.ConcurrentLinkedQueue

class NativeGameController(private val plugin: JavaPlugin) : Listener {

    private val games = mutableListOf<NativeGame>()

    private fun verifyMove(
        xDelta: Double,
        yDelta: Double,
        zDelta: Double,
        direction1: MoveDirection,
        direction2: MoveDirection,
        direction3: MoveDirection,
        direction4: MoveDirection
    ): MoveDirection? {
        return if (yDelta > 0) {
            MoveDirection.UP
        } else if (zDelta < 0) {
            direction1
        } else if (zDelta > 0) {
            direction2
        } else if (xDelta < 0) {
            direction3
        } else if (xDelta > 0) {
            direction4
        } else {
            return null
        }
    }

    @EventHandler(priority = EventPriority.HIGH)
    private fun onMoveEvent(event: PlayerMoveEvent) {
        val game = games.firstOrNull { it.player == event.player } ?: return
        event.isCancelled = true

        val player = event.player
        val from = event.from
        val to = event.to

        if (from.x != to.x || from.y != to.y || from.z != to.z) {
            val playerDirection = player.facing
            val xDelta = to.x - from.x
            val yDelta = to.y - from.y
            val zDelta = to.z - from.z

            val direction = when (playerDirection) {
                BlockFace.NORTH -> {
                    verifyMove(
                        xDelta,
                        yDelta,
                        zDelta,
                        MoveDirection.FORWARD,
                        MoveDirection.BACKWARDS,
                        MoveDirection.LEFT,
                        MoveDirection.RIGHT
                    ) ?: return
                }

                BlockFace.SOUTH -> {
                    verifyMove(
                        xDelta,
                        yDelta,
                        zDelta,
                        MoveDirection.BACKWARDS,
                        MoveDirection.FORWARD,
                        MoveDirection.RIGHT,
                        MoveDirection.LEFT
                    ) ?: return
                }

                BlockFace.WEST -> {
                    verifyMove(
                        xDelta,
                        yDelta,
                        zDelta,
                        MoveDirection.RIGHT,
                        MoveDirection.LEFT,
                        MoveDirection.FORWARD,
                        MoveDirection.BACKWARDS
                    ) ?: return
                }

                BlockFace.EAST -> {
                    verifyMove(
                        xDelta,
                        yDelta,
                        zDelta,
                        MoveDirection.LEFT,
                        MoveDirection.RIGHT,
                        MoveDirection.BACKWARDS,
                        MoveDirection.FORWARD
                    ) ?: return
                }

                else -> return
            }

            game.moveEventQueue.add(direction)
        }
    }

    @EventHandler(priority = EventPriority.HIGH)
    private fun onDropQuit(event: PlayerQuitEvent) {
        val gameIndex = games.indexOfFirst { it.player == event.player }
        if (gameIndex == -1) {
            return
        }

        val game = games[gameIndex]
        game.player.removePotionEffect(PotionEffectType.SLOW)
        game.screen.renderService.get().killRendering()
        game.screen.renderService = Optional.empty()

        games.removeAt(gameIndex)
    }

    @EventHandler(priority = EventPriority.HIGH)
    private fun onDropEvent(event: PlayerDropItemEvent) {
        games.firstOrNull { it.player == event.player } ?: return

        event.isCancelled = true
    }

    @EventHandler(priority = EventPriority.HIGH)
    private fun onDestroyEvent(event: BlockBreakEvent) {
        games.firstOrNull { it.player == event.player } ?: return

        event.isCancelled = true
    }

    @EventHandler(priority = EventPriority.HIGH)
    private fun onPlayerDamage(event: EntityDamageEvent) {
        val player = if (event.entity is Player) event.entity as Player else return
        games.firstOrNull { it.player == player } ?: return

        event.isCancelled = true
    }

    @EventHandler(priority = EventPriority.HIGH)
    private fun onPlayerAttack(event: EntityDamageByEntityEvent) {
        val player = if (event.damager is Player) event.damager as Player else return
        games.firstOrNull { it.player == player } ?: return

        event.isCancelled = true
    }

    fun init() {
        Bukkit.getServer().pluginManager.registerEvents(this, plugin)
    }

    fun stopCleanup() {
        games.forEach { game -> game.player.removePotionEffect(PotionEffectType.SLOW) }
    }

    fun registerGamer(player: Player, screen: Screen) {
        games.add(NativeGame(screen, player, ConcurrentLinkedQueue()))

        val facing = when (screen.mapFace) {
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

        player.addPotionEffect(PotionEffect(PotionEffectType.SLOW, Int.MAX_VALUE, 0, true, false))
        player.teleport(location)
    }

    fun unregisterScreen(screen: Screen) {
        val gameIndex = games.indexOfFirst { it.screen == screen }
        if (gameIndex == -1) {
            return
        }
        val game = games.get(gameIndex)
        game.player.removePotionEffect(PotionEffectType.SLOW)

        games.removeAt(gameIndex)
    }

    fun renderCallback(ptr: Long, screenName: String) {
        val game = games.firstOrNull { it.screen.name == screenName } ?: return
        val stringBuilder = StringBuilder()

        while (true) {
            val element = game.moveEventQueue.poll() ?: break
            stringBuilder.append("_${element.shortName}")
        }

        if (stringBuilder.isEmpty()) {
            return
        }

        NativeRenderControler.communicate(ptr, NativeLibCommunication.GAME_INPUT, stringBuilder.toString())
    }
}
