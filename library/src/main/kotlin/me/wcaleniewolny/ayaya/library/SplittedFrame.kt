package me.wcaleniewolny.ayaya.library

data class SplittedFrame(
    val startX: Int,
    val startY: Int,
    val width: Int,
    val height: Int,
    val xMargin: Int,
    val yMargin: Int,
    val data: ByteArray,
    var initialized: Boolean = false,
    val frameLength: Int = width * height
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as SplittedFrame

        if (startX != other.startX) return false
        if (startY != other.startY) return false
        if (width != other.width) return false
        if (height != other.height) return false
        if (xMargin != other.xMargin) return false
        if (yMargin != other.yMargin) return false
        if (!data.contentEquals(other.data)) return false
        if (initialized != other.initialized) return false
        if (frameLength != other.frameLength) return false

        return true
    }

    override fun hashCode(): Int {
        var result = startX
        result = 31 * result + startY
        result = 31 * result + width
        result = 31 * result + height
        result = 31 * result + xMargin
        result = 31 * result + yMargin
        result = 31 * result + data.contentHashCode()
        result = 31 * result + initialized.hashCode()
        result = 31 * result + frameLength
        return result
    }
}
