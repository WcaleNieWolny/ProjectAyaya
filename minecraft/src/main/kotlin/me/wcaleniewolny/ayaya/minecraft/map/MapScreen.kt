package me.wcaleniewolny.ayaya.minecraft.map

import me.wcaleniewolny.ayaya.minecraft.extenstion.forEachIn
import org.bukkit.Location
import org.bukkit.Material
import org.bukkit.World
import org.bukkit.block.BlockFace
import org.bukkit.entity.EntityType
import org.bukkit.entity.ItemFrame
import org.bukkit.util.Vector
import java.util.*
import kotlin.math.max
import kotlin.math.min


class MapScreen(
    private val loc1: Vector,
    private val loc2: Vector,
    preFace: BlockFace,
    private val world: World
) {

    private val topLeftCorner = calculateScreenPosition(loc1, loc2)
    private val blockFace = validateBlockFace(preFace)

    //Source: https://github.com/northpl93/NativeScreen/blob/8471e6701a0da3f5a90f92850d76cdb90d696a56/src/main/java/pl/north93/nativescreen/renderer/impl/BoardFactory.java#L54-L69
    fun buildScreen() {
        world.forEachIn(loc1, loc2) {
            it.type = Material.SEA_LANTERN
        }

        val cloneLoc1 = loc1.clone()
        val cloneLoc2 = loc2.clone()

        cloneLoc1.add(blockFace.direction)
        cloneLoc2.add(blockFace.direction)

        var i = 0

        world.forEachIn(cloneLoc1, cloneLoc2) {
            it.type = Material.AIR

            val location = it.location
            val frame = getFrameAt(location).orElseGet {
                val newFrame = world.spawnEntity(location, EntityType.ITEM_FRAME) as ItemFrame
                newFrame.isInvulnerable = true
                newFrame.setFacingDirection(blockFace, true)
                newFrame.setItem(MapCleanerService.generateMapItem(i))
                i++
                newFrame
            }
        }

    }

    //Source: https://github.com/northpl93/NativeScreen/blob/8471e6701a0da3f5a90f92850d76cdb90d696a56/src/main/java/pl/north93/nativescreen/renderer/impl/BoardFactory.java#L71-L82
    private fun getFrameAt(loc: Location): Optional<ItemFrame> {
        val frameLocation = Location(loc.world, loc.blockX + 0.5, loc.blockY + 0.5, loc.z + 0.5)
        for (entity in frameLocation.world.getNearbyEntities(frameLocation, 0.5, 0.5, 0.5)) {
            if (entity is ItemFrame) {
                return Optional.of(entity)
            }
        }
        return Optional.empty()
    }

    private fun validateBlockFace(preFace: BlockFace): BlockFace {
        if (
            preFace != BlockFace.NORTH &&
            preFace != BlockFace.EAST &&
            preFace != BlockFace.SOUTH &&
            preFace != BlockFace.WEST
        ) {
            throw IllegalArgumentException("BlockFace is neither north, east, south or west")
        }

        return preFace
    }

    private fun calculateScreenPosition(loc1: Vector, loc2: Vector): Vector {
        if (loc1.x == loc2.x) {
            return Vector(
                min(loc1.x, loc2.x),
                max(loc1.y, loc2.y),
                loc1.z
            )
        } else if (loc1.y == loc2.y) {
            return Vector(
                loc1.x,
                max(loc1.y, loc2.y),
                min(loc1.z, loc2.z)
            )
        } else {
            throw java.lang.IllegalArgumentException("Neither X or Z is the same for both coordinates")
        }
    }
}