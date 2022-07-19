package me.wcaleniewolny.ayaya.library;

public class NativeRenderControler {

    static {
        System.loadLibrary("wolnyjni");
    }

    /**
     * @return Byte array of transformed frame (colour index)
     */
    public
    native byte[] loadFrame();

    /**
     * Initialize C library. Required to call {@link #loadFrame()}
     */
    public native int init(String fileName);

    /**
     * Tell C library to free any native memory. After that calling {@link #loadFrame()} is an illegal operation.
     */
    public native void destroy();

    public native int getWidth();

    public native int getHeight();
}
