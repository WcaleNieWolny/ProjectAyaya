package me.wcaleniewolny.ayaya.minecraft.screen

import me.wcaleniewolny.ayaya.minecraft.render.RenderService
import org.bukkit.block.BlockFace
import java.util.Optional

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
    val width: Int = if (x1 == x2) (z2 - z1 + 1) * 128 else (x2 - x1 + 1) * 128,
    val height: Int = (y1 - y2 + 1) * 128,
    var renderService: Optional<RenderService> = Optional.empty(),
    )

enum class ScreenFacing{
    NORTH,
    EAST,
    SOUTH,
    WEST;

    fun toBlockFace(): BlockFace{
        return when(this){
            NORTH -> BlockFace.NORTH
            EAST -> BlockFace.EAST
            SOUTH -> BlockFace.SOUTH
            WEST -> BlockFace.WEST
        }
    }
}