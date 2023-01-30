package me.wcaleniewolny.ayaya.minecraft.screen

import me.wcaleniewolny.ayaya.minecraft.render.RenderService
import org.bukkit.World
import org.bukkit.block.BlockFace
import java.util.*

data class Screen(
    val startID: Int,
    val name: String,
    val mapFace: BlockFace,
    val x1: Int,
    val y1: Int,
    val z1: Int,
    val x2: Int,
    val y2: Int,
    val z2: Int,
    val gameX: Int,
    val gameY: Int,
    val gameZ: Int,
    val useGame: Boolean,
    val world: World,
    val width: Int = if ((mapFace == BlockFace.SOUTH) || (mapFace == BlockFace.WEST)) (if (x1 == x2) (z2 - z1 + 1) * 128 else (x2 - x1 + 1) * 128) else (if (x1 == x2) (z1 - z2 + 1) * 128 else (x1 - x2 + 1) * 128),
    val height: Int = (y1 - y2 + 1) * 128,
    var renderService: Optional<RenderService> = Optional.empty(),
) {
    override fun equals(other: Any?): Boolean {
        val otherScreen = if (other is Screen) other else return false
        return otherScreen.name == this.name
    }
}

enum class ScreenFacing {
    NORTH,
    EAST,
    SOUTH,
    WEST;

    fun toBlockFace(): BlockFace {
        return when (this) {
            NORTH -> BlockFace.NORTH
            EAST -> BlockFace.EAST
            SOUTH -> BlockFace.SOUTH
            WEST -> BlockFace.WEST
        }
    }
}