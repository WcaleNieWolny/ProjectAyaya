package me.wcaleniewolny.ayaya


fun main(args: Array<String>) {
    val nativeRenderControler = NativeRenderControler()
    nativeRenderControler.init("/home/wolny/Downloads/vid_c.mp4")

    val start = System.currentTimeMillis();
    val byteArray = nativeRenderControler.loadFrame()
    println("took: ${System.currentTimeMillis() - start}")

    if (byteArray != null) {
        println(byteArray.size)
        println(byteArray.contentHashCode())
    }else{
        println("!!! NULL")
    }

    AwtGui(byteArray, nativeRenderControler.width, nativeRenderControler.height)
    println(":)")
}