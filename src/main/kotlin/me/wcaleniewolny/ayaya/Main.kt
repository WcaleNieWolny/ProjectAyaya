package me.wcaleniewolny.ayaya

import me.wcaleniewolny.ayaya.frame.FrameSplitter


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

    val width = nativeRenderControler.width
    val height = nativeRenderControler.height


    val start2 = System.currentTimeMillis();
    val splittedFrames = FrameSplitter.splitFrames(width, height, byteArray)
    println("Splitting took: ${System.currentTimeMillis() - start2}")

    FrameAwtGui(splittedFrames, nativeRenderControler.width, nativeRenderControler.height)
    FullAwtGui(byteArray, width, height)
    println(":)")
}