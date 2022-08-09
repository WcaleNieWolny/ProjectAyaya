package me.wcaleniewolny.ayaya.library

object NativeRenderControler {
    /**
     * @return Byte array of transformed frame (color index)
     * @param ptr Pointer acquired by calling init function
     * @see me.wcaleniewolny.ayaya.library.NativeRenderControler.init
     */
    external fun loadFrame(ptr: Long): ByteArray

    /**
     * Initialize native library. Required to call [NativeRenderControler.loadFrame]
     * @return returns pointer to native memory. WARNING!! CHANGING THAT POINTER WILL CORRUPT MEMORY!
     */
    external fun init(fileName: String): Long

    /**
     * Tell native library to free any native memory. After that calling [NativeRenderControler.loadFrame] is an illegal operation.
     * @param ptr Pointer acquired by calling init function
     * @see me.wcaleniewolny.ayaya.library.NativeRenderControler.init
     */
    external fun destroy(ptr: Long)

    /**
     * @param ptr Pointer acquired by calling init function
     */
    external fun width(ptr: Long): Int

    /**
     * @param ptr Pointer acquired by calling init function
     */
    external fun height(ptr: Long): Int

    init {
        System.loadLibrary("ayaya_native")
    }
}