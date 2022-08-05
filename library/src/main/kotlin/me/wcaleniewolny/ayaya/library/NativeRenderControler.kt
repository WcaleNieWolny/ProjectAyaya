package me.wcaleniewolny.ayaya.library

class NativeRenderControler {
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
     * @see me.wcaleniewolny.ayaya.library.NativeRenderControler.init
     * @see me.wcaleniewolny.ayaya.library.NativeRenderControler.loadFrame
     */
    external fun destroy()

    external fun width(ptr: Long): Int
    external fun height(ptr: Long): Int

    companion object {
        init {
            System.loadLibrary("ayaya_native")
        }
    }
}