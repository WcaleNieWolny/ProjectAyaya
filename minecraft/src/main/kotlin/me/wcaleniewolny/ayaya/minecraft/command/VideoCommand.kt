package me.wcaleniewolny.ayaya.minecraft.command;

import co.aikar.commands.BaseCommand
import co.aikar.commands.annotation.*
import me.wcaleniewolny.ayaya.minecraft.screen.ScreenController
import me.wcaleniewolny.ayaya.minecraft.sendColoredMessage
import org.bukkit.command.CommandSender
import org.bukkit.configuration.file.FileConfiguration
import kotlin.math.max
import kotlin.math.min

@CommandAlias("video|ayaya")
class VideoCommand(
    private val screenController: ScreenController,
    private val fileConfiguration: FileConfiguration
) : BaseCommand() {

    @Default
    @Subcommand("help")
    @Description("Display help")
    fun onHelp(sender: CommandSender){
        sender.sendColoredMessage(fileConfiguration.getString("videoPlayHelp")!!)
        sender.sendColoredMessage(fileConfiguration.getString("videoPauseHelp")!!)
        sender.sendColoredMessage(fileConfiguration.getString("videoKillHelp")!!)
        sender.sendColoredMessage(fileConfiguration.getString("videoScreenCreateHelp")!!)
    }

    @Subcommand("play")
    @Syntax("[screen_id] [video]")
    @CommandCompletion("@screens @video")
    @Description("Starts video playback")
    fun onPlay(sender: CommandSender, screenId: String, video: String){
        sender.sendMessage("$screenId, $video")
    }

    @Subcommand("screen create")
    @Syntax("[name] [x1] [y1] [z1] [x2] [y2] [z2]")
    @CommandCompletion("@nothing @lookingAt @lookingAt @lookingAt @lookingAt @lookingAt @lookingAt @nothing")
    @Description("Create video screen")
    fun onScreenCreate(sender: CommandSender, name: String, x1: Int, y1: Int, z1: Int, x2: Int, y2: Int, z2: Int){
        sender.sendMessage("N: $name, 1: $x1, $y1, $z1, 2: $x2, $y2, $z2")

        if(z1 != z2 && x1 != x2){
            sender.sendColoredMessage(fileConfiguration.getString("screenInvalidCoordinate")!!)
            return
        }

        try {
            screenController.createScreen(
                name, min(x1, x2), max(y1, y2), min(z1, z2), max(x1, x2), min(y1, y2), max(z1, z2)
            )
            sender.sendColoredMessage(fileConfiguration.getString("screenCreationSuccess")!!)

        }catch (e: Exception){
            sender.sendColoredMessage(fileConfiguration.getString("screenCreationFailed")!!)
        }

    }

    @Subcommand("screen info")
    @Syntax("[name]")
    @CommandCompletion("@screens")
    @Description("Get info about screen")
    fun onScreenInfo(sender: CommandSender, name: String){
        val screens = screenController.getScreens().filter { it.name == name }
        if(screens.isEmpty()){
            sender.sendColoredMessage(fileConfiguration.getString("screenLookupEmpty")!!)
            return
        }

        val screen = screens[0]
        val width = if (screen.x1 == screen.x2) (screen.z2 - screen.z1 + 1) * 128 else (screen.x2 - screen.x1 + 1) * 128
        val height = (screen.y1 - screen.y2 + 1) * 128

        sender.sendColoredMessage(fileConfiguration.getString("screenLookupName")!!.replace("$", name))
        sender.sendColoredMessage(fileConfiguration.getString("screenLookupWidth")!!.replace("$", width.toString()))
        sender.sendColoredMessage(fileConfiguration.getString("screenLookupHeight")!!.replace("$", height.toString()))
        sender.sendColoredMessage(fileConfiguration.getString("screenLookupFacing")!!.replace("$", screen.mapFace.toString()))

        //screenLookupHeight: "<green>Screen height: $"
    }

}
