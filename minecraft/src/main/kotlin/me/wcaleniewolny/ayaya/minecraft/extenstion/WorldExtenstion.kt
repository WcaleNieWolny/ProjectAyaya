package me.wcaleniewolny.ayaya.minecraft.extenstion

import me.wcaleniewolny.ayaya.minecraft.screen.ScreenFacing
import org.bukkit.World
import org.bukkit.block.Block
import org.bukkit.util.Vector
import kotlin.math.max
import kotlin.math.min

inline fun World.forEachIn(loc1: Vector, loc2: Vector, screenFacing: ScreenFacing, action: (Block) -> Unit) {
    val highestX: Int = loc2.blockX.coerceAtLeast(loc1.blockX)
    val lowestX: Int = loc2.blockX.coerceAtMost(loc1.blockX)

    val highestY: Int = max(loc1.blockY, loc2.blockY)
    val lowestY: Int = min(loc1.blockY, loc2.blockY)

    val highestZ: Int = loc2.blockZ.coerceAtLeast(loc1.blockZ)
    val lowestZ: Int = loc2.blockZ.coerceAtMost(loc1.blockZ)

    for (y in highestY downTo lowestY) {
        if (screenFacing == ScreenFacing.SOUTH || screenFacing == ScreenFacing.WEST) {
            for (x in lowestX..highestX) {
                for (z in lowestZ..highestZ) {
                    action(getBlockAt(x, y, z))
                }
            }
        } else {
            for (x in highestX downTo lowestX) {
                for (z in highestZ downTo lowestZ) {
                    action(getBlockAt(x, y, z))
                }
            }
        }
    }
}
