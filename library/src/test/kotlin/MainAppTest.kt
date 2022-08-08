package me.wcaleniewolny.ayaya

import org.junit.jupiter.api.Test
import java.util.concurrent.CompletableFuture
import kotlin.test.assertTrue


internal class MainAppTest {

    @Test
    fun guiTest() {
        val future = CompletableFuture<Boolean>()

        //val file = javaClass.classLoader.getResource("test.webm")!!.path
        val nativeRenderControler = me.wcaleniewolny.ayaya.library.NativeRenderControler()
        val ptr = nativeRenderControler.init("/home/wolny/Downloads/test.mp4")

        println("try w!")

        val w = nativeRenderControler.width(ptr)

        println("get w! ($w)")

        val h = nativeRenderControler.height(ptr)

        println("get h ($h)!")

        FullAwtGui(nativeRenderControler, w, h, future, ptr)

        assertTrue(future.get(), "User decided that app is not working")
        //assertTrue(true, "User has confirmed, that application is working")
    }
}