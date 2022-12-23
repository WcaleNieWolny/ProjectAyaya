package me.wcaleniewolny.ayaya.minecraft.game

import me.wcaleniewolny.ayaya.minecraft.screen.Screen
import org.bukkit.entity.Player
import java.util.concurrent.ConcurrentLinkedQueue

data class NativeGame(
    val screen: Screen,
    val player: Player,
    val moveEventQueue: ConcurrentLinkedQueue<MoveDirection>
)

enum class MoveDirection(val shortName: String){
    FORWARD("F"),
    BACKWARDS("B"),
    LEFT("L"),
    RIGHT("R"),
    UP("U")
}