package me.wcaleniewolny.ayaya.minecraft.display

interface DisplayService {
    fun displayFrame(data: ByteArray)
    fun init()
}