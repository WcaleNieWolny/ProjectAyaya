package me.wcaleniewolny.ayaya.minecraft

import co.aikar.commands.PaperCommandManager
import me.wcaleniewolny.ayaya.library.DiscordOptions
import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.library.WindowsBootstrap
import me.wcaleniewolny.ayaya.minecraft.command.VideoCommand
import me.wcaleniewolny.ayaya.minecraft.command.VideoCommandCompletion
import me.wcaleniewolny.ayaya.minecraft.game.NativeGameController
import me.wcaleniewolny.ayaya.minecraft.screen.ScreenController
import net.kyori.adventure.text.minimessage.MiniMessage
import org.bukkit.Bukkit
import org.bukkit.command.CommandSender
import org.bukkit.plugin.java.JavaPlugin
import java.util.logging.Level

class MapMinecraftClient : JavaPlugin() {

    private var windowsBootstrapPtr: Long = 0
    private var nativeGameController: NativeGameController? = null

    override fun onEnable() {
        this.saveDefaultConfig()

        if (!loadNativeLib()) {
            return
        }

        if (config.getBoolean("useDiscordBot")) {
            NativeRenderControler.initDiscordBot(
                DiscordOptions(
                    true,
                    config.getString("discordToken")!!,
                    config.getString("discordGuildId")!!,
                    config.getString("discordChannelId")!!
                )
            )
        }
        val nativeGameController = NativeGameController(this)
        val screenController = ScreenController(this, nativeGameController)
        this.nativeGameController = nativeGameController

        screenController.init()
        nativeGameController.init()

        val manager = PaperCommandManager(this)
        val videoCommandCompletion = VideoCommandCompletion(screenController)

        videoCommandCompletion.init(this, manager)
        manager.registerCommand(
            VideoCommand(
                screenController,
                this.config,
                this
            )
        )

        manager.enableUnstableAPI("help")
    }

    fun loadNativeLib(): Boolean {
        val unsafe = System.getProperty("me.wcaleniewolny.ayaya.unsafe") != null
        val windowsBootstrap = config.getBoolean("useWindowsBootstrap")

        val os = System.getProperty("os.name")
        logger.info("Detected os: $os")

        if (unsafe) {
            logger.log(Level.WARNING, "UNSAFE LIB LOADING ENABLED")
            try {
                System.loadLibrary("ayaya_native")
            } catch (exception: UnsatisfiedLinkError) {
                logger.log(Level.SEVERE, "Unable to load native library from unsafe! AyayaNative will now get disabled")
                exception.printStackTrace()
                Bukkit.getPluginManager().disablePlugin(this)
                return false
            }
        } else if (windowsBootstrap && !os.contains("Linux", true)) {
            logger.log(
                Level.WARNING,
                "ProjectAyaya will now try to use windows bootstrap! Please read the wiki so you know what you are doing!!!"
            )

            try {
                System.loadLibrary("windows_bootstrap")
                windowsBootstrapPtr = WindowsBootstrap.bootstrap(NativeUtils.latestPath!!, dataFolder.absolutePath)
            } catch (t: Throwable) {
                Bukkit.getPluginManager().disablePlugin(this)
                logger.log(Level.SEVERE, "Unable to use windows boostrap! Quiting!")
                t.printStackTrace()
                return false
            }
        } else {
            if (os.contains("Linux", true)) {
                try {
                    NativeUtils.loadLibraryFromJar("/libayaya_native.so")
                } catch (throwable: Throwable) {
                    logger.log(Level.SEVERE, "Unable to load native library! AyayaNative will now get disabled")
                    Bukkit.getPluginManager().disablePlugin(this)
                    throwable.printStackTrace()
                    return false
                }
            } else if (os.contains("Windows", true)) {
                try {
                    NativeUtils.loadLibraryFromJar("/ayaya_native.dll")
                } catch (throwable: Throwable) {
                    logger.log(Level.SEVERE, "Unable to load native library! AyayaNative will now get disabled")
                    logger.log(
                        Level.SEVERE,
                        "If you got a \"Can't find dependent libraries\" error and you are on windows you can set \"useWindowsBootstrap: true\" in the config.yml to try to solve this"
                    )
                    Bukkit.getPluginManager().disablePlugin(this)
                    throwable.printStackTrace()
                    return false
                }
            }
        }

        return true
    }

    override fun onDisable() {
        if (windowsBootstrapPtr != 0L) {
            WindowsBootstrap.cleanup(windowsBootstrapPtr)
        }

        this.nativeGameController?.stopCleanup()
    }
}

fun CommandSender.sendColoredMessage(msg: String) {
    sendMessage(MiniMessage.miniMessage().deserialize(msg))
}
