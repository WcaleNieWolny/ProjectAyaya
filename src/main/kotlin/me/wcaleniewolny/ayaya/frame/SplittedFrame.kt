package me.wcaleniewolny.ayaya.frame

data class SplittedFrame(val startX: Int, val startY: Int, val width: Int, val height: Int, val data: ByteArray) {

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as SplittedFrame

        if (startX != other.startX) return false
        if (startY != other.startY) return false
        if (width != other.width) return false
        if (height != other.height) return false
        if (!data.contentEquals(other.data)) return false

        return true
    }

    override fun hashCode(): Int {
        var result = startX
        result = 31 * result + startY
        result = 31 * result + width
        result = 31 * result + height
        result = 31 * result + data.contentHashCode()
        return result
    }
}
