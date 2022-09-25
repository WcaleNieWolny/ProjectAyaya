package me.wcaleniewolny.ayaya

import me.wcaleniewolny.ayaya.library.FrameSplitter
import me.wcaleniewolny.ayaya.library.NativeRenderControler
import me.wcaleniewolny.ayaya.library.NativeRenderType
import org.junit.jupiter.api.Test
import java.util.concurrent.CompletableFuture
import kotlin.test.assertTrue

//fun main(){
//    val future = CompletableFuture<Boolean>()
//
//    //val file = javaClass.classLoader.getResource("test.webm")!!.path
//    val ptr = NativeRenderControler.init("/home/wolny/rick.webm", true);
//
//    NativeRenderControler.startMultithreading(ptr)
//
//    println("try w!")
//
//    val w = NativeRenderControler.width(ptr)
//
//    println("get w! ($w)")
//
//    val h = NativeRenderControler.height(ptr)
//
//    println("get h ($h)!")
//
//    val splitted = FrameSplitter.initializeFrames(w, h)
//    FrameSplitter.splitFrames(NativeRenderControler.loadFrame(ptr), splitted, w, h)
//    FrameAwtGui(splitted, w, h)
//    //FullAwtGui(w, h, future, ptr)
//
//    assertTrue(future.get(), "User decided that app is not working")
//    //assertTrue(true, "User has confirmed, that application is working")
//}

internal class MainAppTest {

    @Test
    fun splitTest() {
        val ptr = NativeRenderControler.init("/home/wolny/Downloads/vid_c.mp4", NativeRenderType.MULTI_THREADED)
        val data = NativeRenderControler.getVideoData(ptr)

        val w = data.width
        println("get w! ($w)")

        val h = data.height
        println("get h ($h)!")

        val frame = NativeRenderControler.loadFrame(ptr)

        println("s: ${frame.size}")

        val nativeSplit = NativeRenderControler.test(frame, ptr)

        val split = FrameSplitter.initializeFrames(w, h)
        val legacySplit = FrameSplitter.initializeFrames(w, h)

        FrameSplitter.splitFrames(nativeSplit, split, w);
        FrameSplitter.legacy_splitFrames(frame, legacySplit, w, h)

        val i = 10

        println("a: ${split[i].data.contentHashCode()}")
        println("b: ${legacySplit[i].data.contentHashCode()}")

        FrameAwtGui(split, w, h)
        //FrameAwtGui(legacySplit, w, h)

        while (true) {

        }
    }

    @Test
    fun guiTest() {
        val future = CompletableFuture<Boolean>()

        //val file = javaClass.classLoader.getResource("test.webm")!!.path
        val ptr = NativeRenderControler.init("/home/wolny/rick-hd.webm", NativeRenderType.GPU);

        println("try w!")

        val data = NativeRenderControler.getVideoData(ptr)

        println(data)

        val w = data.width

        println("get w! ($w)")

        val h = data.height

        println("get h ($h)!")

        val splitted = FrameSplitter.initializeFrames(w, h)
        //FrameSplitter.splitFrames(NativeRenderControler.loadFrame(ptr), splitted, w, h)
        FrameAwtGui(splitted, w, h)
        FullAwtGui(w, h, future, ptr)

        assertTrue(future.get(), "User decided that app is not working")
        //assertTrue(true, "User has confirmed, that application is working")
    }
}