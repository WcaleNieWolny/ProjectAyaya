package me.wcaleniewolny.ayaya

import java.awt.Color


fun main(args: Array<String>) {
    val nativeControler = NativeControler()
    val byteArray = nativeControler.loadFrame()

    if (byteArray != null) {
        println(byteArray.size)
    }



    println(":)")
}