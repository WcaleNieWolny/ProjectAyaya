package me.wcaleniewolny.ayaya.minecraft.screen

import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.minecraft.command.VideoPlayType
import me.wcaleniewolny.ayaya.minecraft.extenstion.forEachIn
import me.wcaleniewolny.ayaya.minecraft.game.NativeGameController
import me.wcaleniewolny.ayaya.minecraft.render.RenderServiceFactory
import me.wcaleniewolny.ayaya.minecraft.render.RenderServiceType
import me.wcaleniewolny.ayaya.minecraft.sendColoredMessage
import net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket
import net.minecraft.network.syncher.EntityDataAccessor
import net.minecraft.network.syncher.EntityDataSerializers
import net.minecraft.network.syncher.SynchedEntityData
import net.minecraft.world.entity.Entity
import net.minecraft.world.item.ItemStack
import org.bukkit.Bukkit
import org.bukkit.Location
import org.bukkit.Material
import org.bukkit.World
import org.bukkit.block.BlockFace
import org.bukkit.command.CommandSender
import org.bukkit.configuration.file.YamlConfiguration
import org.bukkit.craftbukkit.v1_18_R2.entity.CraftItemFrame
import org.bukkit.entity.EntityType
import org.bukkit.entity.ItemFrame
import org.bukkit.entity.Player
import org.bukkit.plugin.java.JavaPlugin
import org.bukkit.util.Vector
import java.io.File
import java.util.*


class ScreenController(
    private val plugin: JavaPlugin,
    private val nativeGameController: NativeGameController
) {

    private val dir = File(plugin.dataFolder, "screens")
    private val screens = mutableListOf<Screen>()

    fun init() {

        dir.listFiles()
            ?.filterNot { it.name.contains(" ") }
            ?.forEach { file ->
                val screenYaml = YamlConfiguration.loadConfiguration(file)
                val startID = screenYaml.getInt("id")
                val facing = ScreenFacing.valueOf(screenYaml.getString("facing")!!)
                val x1 = screenYaml.getInt("x1")
                val y1 = screenYaml.getInt("y1")
                val z1 = screenYaml.getInt("z1")
                val x2 = screenYaml.getInt("x2")
                val y2 = screenYaml.getInt("y2")
                val z2 = screenYaml.getInt("z2")

                screens.add(Screen(startID, file.nameWithoutExtension, facing.toBlockFace(), x1, y1, z1, x2, y2, z2))
        }
    }

    fun createScreen(name: String, facing: ScreenFacing, x1: Int, y1: Int, z1: Int, x2: Int, y2: Int, z2: Int) {
        val screenFile = File(dir, "${name}.yml")

        if (screenFile.exists()) {
            throw IllegalStateException("Screen with name $name exists")
        }

        val face = facing.toBlockFace()
        val world = Bukkit.getWorlds()[0] //Hope it is the overworld
        val startID = buildScreen(world, Vector(x1, y1, z1), Vector(x2, y2, z2), face)


        val screen = Screen(startID, name, face, x1, y1, z1, x2, y2, z2)
        MapCleanerService.cleanMaps(world, startID, (screen.width*screen.height) / 16384)

        val screenYaml = YamlConfiguration.loadConfiguration(screenFile)

        screenYaml.set("id", startID)
        screenYaml.set("facing", facing.toString())
        screenYaml.set("x1", x1)
        screenYaml.set("y1", y1)
        screenYaml.set("z1", z1)
        screenYaml.set("x2", x2)
        screenYaml.set("y2", y2)
        screenYaml.set("z2", z2)

        screenYaml.save(screenFile)

        screens.add(screen)
    }

    fun startPlayback(
        videoPlayType: VideoPlayType,
        file: File,
        sender: CommandSender,
        screen: Screen
    ) {
        Bukkit.getScheduler().runTaskAsynchronously(plugin, Runnable {
            val verify = NativeRenderControler.verifyScreenCapabilities(file.absolutePath, screen.width, screen.height)
            if (!verify) {
                sender.sendColoredMessage(plugin.config.getString("videoVerificationFailed")!!)
                return@Runnable
            }

            val useMapServer = videoPlayType == VideoPlayType.MAP_SERVER
            val renderService = RenderServiceFactory.create(
                plugin,
                file.absolutePath,
                screen.startID,
                useMapServer,
                if (useMapServer) RenderServiceType.NATIVE else RenderServiceType.JAVA,
                videoPlayType
            )

            screen.renderService = Optional.of(renderService)
            renderService.startRendering()
            sender.sendColoredMessage(plugin.config.getString("success")!!)
        })
    }

    fun startGame(
        game: String,
        screen: Screen,
        player: Player
    ){
        val renderService = RenderServiceFactory.create(
            plugin,
            game, //Note: in game mode this will load selected game
            screen.startID,
            false,
            RenderServiceType.JAVA,
            VideoPlayType.GAME
        )

        nativeGameController.registerGamer(player, screen)

        screen.renderService = Optional.of(renderService)
        renderService.startRendering()
    }

    fun killPlayback(screen: Screen) {
        val renderServiceOptional = screen.renderService
        if (renderServiceOptional.isEmpty) {
            return
        }

        val renderService = renderServiceOptional.get()
        renderService.killRendering()
        screen.renderService = Optional.empty()

        restartVideoScreen(screen)
    }

    fun getScreens(): List<Screen> {
        //Make immutable
        return screens
    }

    fun restartVideoScreen(screen: Screen){
        for(i in screen.startID until screen.startID + (screen.width * screen.height) / 16384){
            val map = Bukkit.getMap(i)!!
            Bukkit.getOnlinePlayers().forEach {player ->
                player.sendMap(map)
            }
        }
    }

    private fun buildScreen(world: World, loc1: Vector, loc2: Vector, blockFace: BlockFace): Int {
        world.forEachIn(loc1, loc2) {
            it.type = Material.SEA_LANTERN
        }

        val cloneLoc1 = loc1.clone()
        val cloneLoc2 = loc2.clone()

        cloneLoc1.add(blockFace.direction)
        cloneLoc2.add(blockFace.direction)

        val preMap = Bukkit.createMap(world)

        var i = preMap.id + 1

        world.forEachIn(cloneLoc1, cloneLoc2) {
            it.type = Material.AIR

            val location = it.location
            getFrameAt(location).orElseGet {
                val newFrame = world.spawnEntity(location, EntityType.ITEM_FRAME) as ItemFrame
                newFrame.isInvulnerable = true
                newFrame.setFacingDirection(blockFace, true)
                newFrame.setItem(MapCleanerService.generateMapItem(i, world))
                i++
                newFrame
            }
        }

        return preMap.id + 1
    }

    private fun getFrameAt(loc: Location): Optional<ItemFrame> {
        val frameLocation = Location(loc.world, loc.blockX + 0.5, loc.blockY + 0.5, loc.z + 0.5)
        for (entity in frameLocation.world.getNearbyEntities(frameLocation, 0.5, 0.5, 0.5)) {
            if (entity is ItemFrame) {
                //val data = SynchedEntityData(entity as Entity)
                //data.set(EntityDataAccessor(8, EntityDataSerializers.ITEM_STACK), "")
                //val metadataPacket = ClientboundSetEntityDataPacket((entity as CraftItemFrame).entityId, )
                return Optional.of(entity)
            }
        }
        return Optional.empty()
    }

}