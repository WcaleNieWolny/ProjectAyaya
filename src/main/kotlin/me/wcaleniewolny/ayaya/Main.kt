package me.wcaleniewolny.ayaya


fun main(args: Array<String>) {
    val nativeRenderControler = NativeRenderControler()
    nativeRenderControler.init()

    val start = System.currentTimeMillis();
    val byteArray = nativeRenderControler.loadFrame()
    println("took: ${System.currentTimeMillis() - start}")

    if (byteArray != null) {
        println(byteArray.size)
        println(byteArray.contentHashCode())
    }else{
        println("!!! NULL")
    }

    AwtGui(byteArray)
    println(":)")
}