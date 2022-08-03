package me.wcaleniewolny.ayaya

import org.junit.jupiter.api.Test
import java.util.concurrent.CompletableFuture
import kotlin.test.assertTrue


internal class MainAppTest{

    @Test
    fun guiTest(){
        val future = CompletableFuture<Boolean>()

        val file = javaClass.classLoader.getResource("test.webm")!!.path
        val nativeRenderControler = me.wcaleniewolny.ayaya.library.NativeRenderControler()
        nativeRenderControler.init(file)
        FullAwtGui(nativeRenderControler, nativeRenderControler.width, nativeRenderControler.height, future)

        assertTrue(future.get(), "User decided that app is not working")
        //assertTrue(true, "User has confirmed, that application is working")
    }
}