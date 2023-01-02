package me.wcaleniewolny.ayaya.library

object NativeRenderControler {
    /**
     * @return Byte array of transformed frame (color index)
     * @param ptr Pointer acquired by calling init function
     * @see me.wcaleniewolny.ayaya.library.NativeRenderControler.init
     * @throws java.lang.RuntimeException if rust panics during native call
     */
    external fun loadFrame(ptr: Long): ByteArray

    /**
     * Make sure that the target screen has enough resolution to handle output
     * @param fileName absolute path to file to be checked
     * @param width width of target screen
     * @param height height of target screen
     * @return true if the target screen can handle the resolution and file is valid
     * @throws java.lang.RuntimeException if rust panics during native call
     */
    external fun verifyScreenCapabilities(fileName: String, width: Int, height: Int): Boolean

    /**
     * Initialize native library. Required to call [NativeRenderControler.loadFrame]
     * @param multithreading If true underlying library will use multithreading
     * @return returns pointer to native memory. WARNING!! CHANGING THAT POINTER WILL CORRUPT MEMORY!
     * @throws java.lang.RuntimeException if rust panics during native call
     */
        external fun init(fileName: String, type: NativeRenderType, serverOptions: MapServerOptions): Long

    /**
     * Tell native library to free any native memory. After that calling [NativeRenderControler.loadFrame] is an illegal operation.
     * @param ptr Pointer acquired by calling init function
     * @see me.wcaleniewolny.ayaya.library.NativeRenderControler.init
     * @throws java.lang.RuntimeException if rust panics during native call
     */
    external fun destroy(ptr: Long)

    /**
     * @param ptr Pointer acquired by calling init function
     * @throws java.lang.RuntimeException if rust panics during native call
     */
    external fun getVideoData(ptr: Long): VideoData

    /**
     * @param ptr Pointer acquired by calling init function
     * @param message Message to send
     * @throws java.lang.RuntimeException if rust panics during native call
     */
    external fun communicate(ptr: Long, message: NativeLibCommunication, additionalInfo: String)
}