package me.wcaleniewolny.ayaya

import me.wcaleniewolny.ayaya.library.FrameSplitter
import me.wcaleniewolny.ayaya.library.NativeRenderControler
import org.junit.jupiter.api.Test
import java.util.concurrent.CompletableFuture
import kotlin.test.assertTrue


internal class MainAppTest {

    @Test
    fun guiTest() {
        val future = CompletableFuture<Boolean>()

        //val file = javaClass.classLoader.getResource("test.webm")!!.path
        val ptr = NativeRenderControler.init("/home/wolny/Downloads/test.mp4", true);

        NativeRenderControler.startMultithreading(ptr)

        println("try w!")

        val w = NativeRenderControler.width(ptr)

        println("get w! ($w)")

        val h = NativeRenderControler.height(ptr)

        println("get h ($h)!")

        val splitted = FrameSplitter.initializeFrames(w, h)
        FrameSplitter.splitFrames(NativeRenderControler.loadFrame(ptr), splitted, w, h)
        FrameAwtGui(splitted, w, h)
        //FullAwtGui(w, h, future, ptr)

        assertTrue(future.get(), "User decided that app is not working")
        //assertTrue(true, "User has confirmed, that application is working")
    }
}