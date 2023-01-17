package me.wcaleniewolny.ayaya.library

object WindowsBootstrap {
    external fun bootstrap(dllPath: String, appFolder: String): Long
    external fun cleanup(ptr: Long)
}