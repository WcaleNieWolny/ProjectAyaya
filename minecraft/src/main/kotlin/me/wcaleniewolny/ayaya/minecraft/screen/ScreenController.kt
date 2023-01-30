package me.wcaleniewolny.ayaya.minecraft.screen

import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.library.VideoRequestCapablyResponse
import me.wcaleniewolny.ayaya.minecraft.command.VideoPlayType
import me.wcaleniewolny.ayaya.minecraft.display.broadcaster.impl.MinecraftNativeBroadcaster
import me.wcaleniewolny.ayaya.minecraft.extenstion.forEachIn
import me.wcaleniewolny.ayaya.minecraft.game.NativeGameController
import me.wcaleniewolny.ayaya.minecraft.render.RenderServiceFactory
import me.wcaleniewolny.ayaya.minecraft.render.RenderServiceType
import me.wcaleniewolny.ayaya.minecraft.sendColoredMessage
import org.bukkit.Bukkit
import org.bukkit.Location
import org.bukkit.Material
import org.bukkit.World
import org.bukkit.block.BlockFace
import org.bukkit.command.CommandSender
import org.bukkit.configuration.file.YamlConfiguration
import org.bukkit.craftbukkit.v1_18_R2.entity.CraftPlayer
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
    private val random = Random()

    fun init() {

        dir.listFiles()
            ?.filterNot { it.name.contains(" ") }
            ?.forEach { file ->
                val screenYaml = YamlConfiguration.loadConfiguration(file)

                val world = Bukkit.getWorld(UUID.fromString(screenYaml.getString("world"))) ?: return

                val startID = screenYaml.getInt("id")
                val facing = ScreenFacing.valueOf(screenYaml.getString("facing")!!)
                val x1 = screenYaml.getInt("x1")
                val y1 = screenYaml.getInt("y1")
                val z1 = screenYaml.getInt("z1")
                val x2 = screenYaml.getInt("x2")
                val y2 = screenYaml.getInt("y2")
                val z2 = screenYaml.getInt("z2")
                val gX = screenYaml.getInt("gx")
                val gY = screenYaml.getInt("gy")
                val gZ = screenYaml.getInt("gz")
                val useGame = screenYaml.getBoolean("useGame")

                screens.add(
                    Screen(
                        startID,
                        file.nameWithoutExtension,
                        facing.toBlockFace(),
                        x1,
                        y1,
                        z1,
                        x2,
                        y2,
                        z2,
                        gX,
                        gY,
                        gZ,
                        useGame,
                        world
                    )
                )
            }
    }

    fun createScreen(
        name: String,
        facing: ScreenFacing,
        x1: Int,
        y1: Int,
        z1: Int,
        x2: Int,
        y2: Int,
        z2: Int,
        gameX: Int,
        gameY: Int,
        gameZ: Int,
        useGame: Boolean,
        world: World
    ) {
        val screenFile = File(dir, "${name}.yml")

        if (screenFile.exists()) {
            throw IllegalStateException("Screen with name $name exists")
        }

        val face = facing.toBlockFace()
        val startID = buildScreen(world, Vector(x1, y1, z1), Vector(x2, y2, z2), face, facing)

        val screen = Screen(startID, name, face, x1, y1, z1, x2, y2, z2, gameX, gameY, gameZ, useGame, world)

        val screenYaml = YamlConfiguration.loadConfiguration(screenFile)

        screenYaml.set("id", startID)
        screenYaml.set("facing", facing.toString())
        screenYaml.set("x1", x1)
        screenYaml.set("y1", y1)
        screenYaml.set("z1", z1)
        screenYaml.set("x2", x2)
        screenYaml.set("y2", y2)
        screenYaml.set("z2", z2)
        screenYaml.set("gx", gameX)
        screenYaml.set("gy", gameY)
        screenYaml.set("gz", gameZ)
        screenYaml.set("useGame", useGame)
        screenYaml.set("world", world.uid.toString())

        screenYaml.save(screenFile)

        screens.add(screen)
    }

    fun startPlayback(
        videoPlayType: VideoPlayType,
        file: File,
        sender: CommandSender,
        screen: Screen,
        useDiscord: Boolean
    ) {
        Bukkit.getScheduler().runTaskAsynchronously(plugin, Runnable {
            val verify = NativeRenderControler.verifyScreenCapabilities(
                file.absolutePath,
                screen.width,
                screen.height,
                useDiscord
            )
            when (verify) {
                VideoRequestCapablyResponse.OK -> {}
                VideoRequestCapablyResponse.INVALID_DIMENSIONS -> {
                    Bukkit.getScheduler().runTask(plugin, Runnable {
                        sender.sendColoredMessage(plugin.config.getString("videoVerificationInvalidDimensions")!!)
                    })
                    return@Runnable
                }

                VideoRequestCapablyResponse.TO_SMALL -> {
                    Bukkit.getScheduler().runTask(plugin, Runnable {
                        sender.sendColoredMessage(plugin.config.getString("videoVerificationToSmall")!!)
                    })
                    return@Runnable
                }

                VideoRequestCapablyResponse.TO_LARGE -> {
                    Bukkit.getScheduler().runTask(plugin, Runnable {
                        sender.sendColoredMessage(plugin.config.getString("videoVerificationToLarge")!!)
                    })
                    return@Runnable
                }

                VideoRequestCapablyResponse.DISCORD_IN_USE -> {
                    Bukkit.getScheduler().runTask(plugin, Runnable {
                        sender.sendColoredMessage(plugin.config.getString("videoVerificationDiscordInUse")!!)
                    })
                    return@Runnable
                }
            }

            val useMapServer = videoPlayType == VideoPlayType.MAP_SERVER
            val renderService = RenderServiceFactory.create(
                plugin,
                file.absolutePath,
                screen.name,
                screen.startID,
                useMapServer,
                if (useMapServer) RenderServiceType.NATIVE else RenderServiceType.JAVA,
                videoPlayType,
                useDiscord = useDiscord
            )

            screen.renderService = Optional.of(renderService)
            renderService.startRendering()
            Bukkit.getScheduler().runTask(plugin, Runnable {
                sender.sendColoredMessage(plugin.config.getString("success")!!)
            })
        })
    }

    fun startGame(
        game: String,
        screen: Screen,
        player: Player
    ) {
        val renderService = RenderServiceFactory.create(
            plugin,
            game, //Note: in game mode this will load selected game
            screen.name,
            screen.startID,
            false,
            RenderServiceType.JAVA,
            VideoPlayType.GAME,
            nativeGameController::renderCallback
        )

        nativeGameController.registerGamer(player, screen)

        screen.renderService = Optional.of(renderService)
        renderService.startRendering()
    }

    fun startX11(
        screen: Screen,
        useMapServer: Boolean,
        screenDetails: String
    ) {
        //"$" is the splitting sign of the X11 player
        val renderService = RenderServiceFactory.create(
            plugin,
            screenDetails,
            screen.name,
            screen.startID,
            useMapServer,
            if (!useMapServer) RenderServiceType.JAVA else RenderServiceType.NATIVE,
            VideoPlayType.X11,
        )

        screen.renderService = Optional.of(renderService)
        renderService.startRendering()
    }

    fun killPlayback(screen: Screen) {
        val renderServiceOptional = screen.renderService
        if (renderServiceOptional.isEmpty) {
            return
        }

        val renderService = renderServiceOptional.get()

        //This is a NOOP when a screen is a non gaming screen
        //This needs to be called before killing the service
        //Doing it after the kill method could go VERY wrong (potential SEGFAULT)
        nativeGameController.unregisterScreen(screen)

        renderService.killRendering()
        screen.renderService = Optional.empty()

        restartVideoScreen(screen)
    }

    fun getScreens(): List<Screen> {
        //Make immutable
        return screens
    }

    fun restartVideoScreen(screen: Screen) {
        for (i in screen.startID until screen.startID + (screen.width * screen.height) / 16384) {
            val mapPacket = MinecraftNativeBroadcaster.makeMapPacket(
                i,
                0,
                0,
                128,
                128,
                ByteArray(16384) { 0 }
            )

            Bukkit.getOnlinePlayers().forEach {
                (it as CraftPlayer).handle.connection.send(mapPacket)
            }
        }
    }

    private fun buildScreen(
        world: World,
        loc1: Vector,
        loc2: Vector,
        blockFace: BlockFace,
        screenFacing: ScreenFacing
    ): Int {
        world.forEachIn(loc1, loc2, screenFacing) {
            it.type = Material.SEA_LANTERN
        }

        val cloneLoc1 = loc1.clone()
        val cloneLoc2 = loc2.clone()

        cloneLoc1.add(blockFace.direction)
        cloneLoc2.add(blockFace.direction)

        //It will have fake item init - no need to have realists ids
        val preMap = random.nextInt(2_000_000 - 1_000_000) + 1_000_000

        var i = preMap

        world.forEachIn(cloneLoc1, cloneLoc2, screenFacing) {
            it.type = Material.AIR

            val location = it.location
            getFrameAt(location).orElseGet {
                val newFrame = world.spawnEntity(location, EntityType.ITEM_FRAME) as ItemFrame
                newFrame.isInvulnerable = true
                newFrame.setFacingDirection(blockFace, true)
                newFrame.setItem(MapCleanerService.generateMapItem(i, world))
                newFrame
            }
            i++
        }

        return preMap
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