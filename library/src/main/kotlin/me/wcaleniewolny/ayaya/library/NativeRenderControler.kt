package me.wcaleniewolny.ayaya.library

class NativeRenderControler {
    /**
     * @return Byte array of transformed frame (color index)
     */
    external fun loadFrame(): ByteArray

    /**
     * Initialize C library. Required to call [.loadFrame]
     */
    external fun init(fileName: String?): Int

    /**
     * Tell C library to free any native memory. After that calling [.loadFrame] is an illegal operation.
     */
    external fun destroy()

    val width: Int
        external get
    val height: Int
        external get

    companion object {
        init {
            System.loadLibrary("wolnyjni")
        }
    }
}