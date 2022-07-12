package me.wcaleniewolny.ayaya


fun main(args: Array<String>) {
    val nativeRenderControler = NativeRenderControler()
    nativeRenderControler.init()
    val byteArray = nativeRenderControler.loadFrame()

    if (byteArray != null) {
        println(byteArray.size)
    }

    println(":)")
}