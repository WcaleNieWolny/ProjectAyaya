package me.wcaleniewolny.ayaya.minecraft.command;

import co.aikar.commands.BaseCommand
import co.aikar.commands.CommandHelp
import co.aikar.commands.annotation.*
import me.wcaleniewolny.ayaya.minecraft.render.impl.JavaRenderServiceImpl
import me.wcaleniewolny.ayaya.minecraft.screen.Screen
import me.wcaleniewolny.ayaya.minecraft.screen.ScreenController
import me.wcaleniewolny.ayaya.minecraft.screen.ScreenFacing
import me.wcaleniewolny.ayaya.minecraft.sendColoredMessage
import org.bukkit.command.CommandSender
import org.bukkit.configuration.file.FileConfiguration
import org.bukkit.entity.Player
import org.bukkit.plugin.java.JavaPlugin
import java.io.File
import java.util.Optional
import kotlin.math.max
import kotlin.math.min

@CommandAlias("video|ayaya")
class VideoCommand(
    private val screenController: ScreenController,
    private val fileConfiguration: FileConfiguration,
    private val plugin: JavaPlugin
) : BaseCommand() {

    @HelpCommand
    fun onHelp(sender: CommandSender, help: CommandHelp) {
        help.showHelp()
    }

    @Subcommand("play")
    @Syntax("[screen_id] [play_type] [video]")
    @CommandCompletion("@screens @videoPlayType @video @nothing")
    @Description("Starts video playback")
    fun onPlay(
        sender: CommandSender,
        @Values("@screens") screenId: String,
        @Values("@videoPlayType") playType: String,
        @Values("@video") video: String
    ) {
        val screenOptional = lookupScreen(sender, screenId)
        if (screenOptional.isEmpty) {
            return
        }
        val screen = screenOptional.get()

        if(screen.renderService.isPresent){
            sender.sendColoredMessage(fileConfiguration.getString("unableToStartPlayback")!!)
            return
        }

        val file = File(File(plugin.dataFolder, "video"), video)

        //Prevent path traversal
        //Thanks CDFN (https://github.com/CDFN) for this idea!
        if (!file.normalize().path.startsWith(File(plugin.dataFolder, "video").normalize().path)) {
            sender.sendColoredMessage(fileConfiguration.getString("pathTraversalAttempt")!!)
            return
        }

        if (!file.exists()) {
            sender.sendColoredMessage(fileConfiguration.getString("fileDoesNotExist")!!)
            return
        }

        val videoPlayType = VideoPlayType.valueOf(playType.uppercase())
        if (videoPlayType == VideoPlayType.MAP_SERVER && !plugin.config.getBoolean("allowMapServer")) {
            sender.sendColoredMessage(fileConfiguration.getString("mapServerPlaybackNotAllowed")!!)
            return
        }
        //val allowMapServer = plugin.config.getBoolean("allowMapServer")

        screenController.startPlayback(videoPlayType, file, sender, screen)
    }

    @Subcommand("game")
    @Description("Starts game on selected screen")
    @Syntax("[screen_id] [game]")
    @CommandCompletion("@screens @games @nothing")
    fun onGame(
        sender: Player,
        @Values("@screens") screenId: String,
        @Values("@games") game: String
    ){
        val screenOptional = lookupScreen(sender, screenId)
        if (screenOptional.isEmpty) {
            return
        }
        val screen = screenOptional.get()

        if(screen.renderService.isPresent){
            sender.sendColoredMessage(fileConfiguration.getString("unableToStartPlayback")!!)
            return
        }

        screenController.startGame(game, screen, sender)
    }

    @Subcommand("seek")
    @Syntax("[screen_id]")
    @CommandCompletion("@screens @nothing")
    fun onSeek(
        sender: Player,
        @Values("@screens") screenId: String,
        second: Int
    ){
        if (0 > second) {
            sender.sendColoredMessage(fileConfiguration.getString("seekToNegativeSecond")!!)
            return
        }
        val screenOptional = lookupScreen(sender, screenId)
        if (screenOptional.isEmpty) {
            return
        }
        val screen = screenOptional.get()
        if(screen.renderService.isPresent){
            //This is safe due to rust mutex
            val renderService = screen.renderService.get()
            renderService.seekSecond(second)
        }
    }

    @Subcommand("pause")
    @Syntax("[screen_id]")
    @CommandCompletion("@screens @nothing")
    @Description("Pause video playback")
    fun onVideoPause(
        sender: CommandSender,
        @Values("@screens") screenId: String,
    ) {
        val screenOptional = lookupScreen(sender, screenId)
        if (screenOptional.isEmpty) {
            return
        }
        val screen = screenOptional.get()

        val renderServiceOptional = screen.renderService
        if (renderServiceOptional.isEmpty) {
            sender.sendColoredMessage(fileConfiguration.getString("unableToPausePlayback")!!)
            return
        }

        val renderService = renderServiceOptional.get()
        renderService.pauseRendering()
        sender.sendColoredMessage(fileConfiguration.getString("success")!!)
    }

    @Subcommand("kill")
    @Syntax("[screen_id]")
    @CommandCompletion("@screens @nothing")
    @Description("Kill video playback")
    fun onVideoKill(
        sender: CommandSender,
        @Values("@screens") screenId: String,
    ) {
        val screenOptional = lookupScreen(sender, screenId)
        if (screenOptional.isEmpty) {
            return
        }

        val screen = screenOptional.get()
        val renderServiceOptional = screen.renderService
        if (renderServiceOptional.isEmpty) {
            sender.sendColoredMessage(fileConfiguration.getString("unableToPausePlayback")!!)
            return
        }

        screenController.killPlayback(screen)
        sender.sendColoredMessage(fileConfiguration.getString("success")!!)
    }


    @Subcommand("screen create")
    @Syntax("[name] [facing] [x1] [y1] [z1] [x2] [y2] [z2] [game_x] [game_y] [game_z]")
    @CommandCompletion("@nothing @screenFacing @lookingAt @lookingAt @lookingAt @lookingAt @lookingAt @lookingAt @lookingAt @lookingAt @lookingAt @nothing")
    @Description("Create video screen")
    fun onScreenCreate(
        sender: Player,
        name: String,
        screenFacing: ScreenFacing,
        x1: Int,
        y1: Int,
        z1: Int,
        x2: Int,
        y2: Int,
        z2: Int,
        gameX: Int,
        gameY: Int,
        gameZ: Int
    ) {

        if (z1 != z2 && x1 != x2) {
            sender.sendColoredMessage(fileConfiguration.getString("screenInvalidCoordinate")!!)
            return
        }


        if (z1 == z2 && screenFacing != ScreenFacing.NORTH && screenFacing != ScreenFacing.SOUTH) {
            sender.sendColoredMessage(fileConfiguration.getString("screenIllegalFacing")!!)
            return
        }

        if (x1 == x2 && screenFacing != ScreenFacing.WEST && screenFacing != ScreenFacing.EAST) {
            sender.sendColoredMessage(fileConfiguration.getString("screenIllegalFacing")!!)
            return
        }

        try {
            if(screenFacing == ScreenFacing.SOUTH || screenFacing == ScreenFacing.WEST){
                screenController.createScreen(
                    name,
                    screenFacing,
                    min(x1, x2),
                    max(y1, y2),
                    min(z1, z2),
                    max(x1, x2),
                    min(y1, y2),
                    max(z1, z2),
                    gameX,
                    gameY,
                    gameZ,
                    sender.world
                )

            }else {
                screenController.createScreen(
                    name,
                    screenFacing,
                    max(x1, x2),
                    max(y1, y2),
                    max(z1, z2),
                    min(x1, x2),
                    min(y1, y2),
                    min(z1, z2),
                    gameX,
                    gameY,
                    gameZ,
                    sender.world
                )
            }

        } catch (e: Exception) {
            sender.sendColoredMessage(fileConfiguration.getString("screenCreationFailed")!!)
            return
        }

        sender.sendColoredMessage(fileConfiguration.getString("screenCreationSuccess")!!)

    }

    @Subcommand("screen info")
    @Syntax("[name]")
    @CommandCompletion("@screens @nothing")
    @Description("Get info about screen")
    fun onScreenInfo(sender: CommandSender, @Values("@screens") screenId: String) {
        val screenOptional = lookupScreen(sender, screenId)
        if (screenOptional.isEmpty) {
            return
        }
        val screen = screenOptional.get()

        sender.sendColoredMessage(fileConfiguration.getString("screenLookupName")!!.replace("$", name))
        sender.sendColoredMessage(
            fileConfiguration.getString("screenLookupWidth")!!.replace("$", screen.width.toString())
        )
        sender.sendColoredMessage(
            fileConfiguration.getString("screenLookupHeight")!!.replace("$", screen.height.toString())
        )
        sender.sendColoredMessage(
            fileConfiguration.getString("screenLookupFacing")!!.replace("$", screen.mapFace.toString())
        )
    }

    private fun lookupScreen(sender: CommandSender, id: String): java.util.Optional<Screen> {
        val screens = screenController.getScreens().filter { it.name == id }
        if (screens.isEmpty()) {
            sender.sendColoredMessage(fileConfiguration.getString("screenLookupEmpty")!!)
            return Optional.empty()
        }

        return Optional.of(screens[0])
    }

}
