package me.wcaleniewolny.ayaya.minecraft.screen

import me.wcaleniewolny.ayaya.minecraft.extenstion.forEachIn
import me.wcaleniewolny.ayaya.minecraft.map.MapCleanerService
import org.bukkit.Location
import org.bukkit.Material
import org.bukkit.World
import org.bukkit.block.BlockFace
import org.bukkit.configuration.file.YamlConfiguration
import org.bukkit.entity.EntityType
import org.bukkit.entity.ItemFrame
import org.bukkit.plugin.java.JavaPlugin
import org.bukkit.util.Vector
import java.io.File
import java.util.*


class ScreenController(private val plugin: JavaPlugin) {

    private val dir = File(plugin.dataFolder, "screens")
    private val screens = mutableListOf<Screen>()

    fun init(){
        println(dir.listFiles().map { it.name })

        dir.listFiles().forEach {file ->
            val screenYaml = YamlConfiguration.loadConfiguration(file)
            val x1 = screenYaml.getInt("x1")
            val y1 = screenYaml.getInt("y1")
            val z1 = screenYaml.getInt("z1")
            val x2 = screenYaml.getInt("x2")
            val y2 = screenYaml.getInt("y2")
            val z2 = screenYaml.getInt("z2")

            screens.add(Screen(file.nameWithoutExtension, x1, y1, z1, x2, y2, z2, getMapFace(Vector(x1, y1, z1), Vector(x2, y2, z2))))
        }
    }

    fun createScreen(name: String, x1: Int, y1: Int, z1: Int, x2: Int, y2: Int, z2: Int){
        val screenFile = File(dir, "${name}.yml")

        if (screenFile.exists()){
            throw IllegalStateException("Screen with name $name exists")
        }

        val screenYaml = YamlConfiguration.loadConfiguration(screenFile)

        screenYaml.set("x1", x1)
        screenYaml.set("y1", y1)
        screenYaml.set("z1", z1)
        screenYaml.set("x2", x2)
        screenYaml.set("y2", y2)
        screenYaml.set("z2", z2)

        screenYaml.save(screenFile)

        screens.add(Screen(name, x1, y1, z1, x2, y2, z2, getMapFace(Vector(x1, y1, z1), Vector(x2, y2, z2))))
    }

    fun getScreens(): List<Screen>{
        //Make immutable
       return screens
    }

    private fun getMapFace(loc1: Vector, loc2: Vector): BlockFace {
        if (loc1.x > loc2.x) {
            return BlockFace.NORTH
        } else if (loc1.x < loc2.x) {
            return BlockFace.SOUTH
        } else if (loc1.z > loc2.z) {
            return BlockFace.EAST
        } else if (loc1.z < loc2.z) {
            return BlockFace.WEST
        }
        return BlockFace.NORTH
    }

    fun buildScreen(world: World, loc1: Vector, loc2: Vector, blockFace: BlockFace) {
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

    private fun getFrameAt(loc: Location): Optional<ItemFrame> {
        val frameLocation = Location(loc.world, loc.blockX + 0.5, loc.blockY + 0.5, loc.z + 0.5)
        for (entity in frameLocation.world.getNearbyEntities(frameLocation, 0.5, 0.5, 0.5)) {
            if (entity is ItemFrame) {
                return Optional.of(entity)
            }
        }
        return Optional.empty()
    }
}