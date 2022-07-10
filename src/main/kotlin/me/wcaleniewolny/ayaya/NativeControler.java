package me.wcaleniewolny.ayaya;

import org.jetbrains.annotations.Nullable;

public class NativeControler {

    static {
        System.loadLibrary("wolnyjni");
    }

    /**
     * @return Byte array of transformed frame (colour index)
     */
    public @Nullable
    native byte[] loadFrame();
}
