package me.wcaleniewolny.ayaya.minecraft.screen

import org.bukkit.World
import org.bukkit.map.MapRenderer
import org.bukkit.map.MapView
import org.bukkit.map.MapView.Scale

class FakeBukkitMapView(private val id: Int) : MapView {
    override fun getId(): Int {
        return id
    }

    override fun isVirtual(): Boolean {
        return true
    }

    override fun getScale(): Scale {
        return Scale.valueOf("0")
    }

    override fun setScale(scale: Scale) {}

    override fun getCenterX(): Int {
        return 0
    }

    override fun getCenterZ(): Int {
        return 0
    }

    override fun setCenterX(x: Int) {}

    override fun setCenterZ(z: Int) {}

    override fun getWorld(): World? {
        return null
    }

    override fun setWorld(world: World) {}

    override fun getRenderers(): MutableList<MapRenderer> {
        return mutableListOf()
    }

    override fun addRenderer(renderer: MapRenderer) {}

    override fun removeRenderer(renderer: MapRenderer?): Boolean {
        return true
    }

    override fun isTrackingPosition(): Boolean {
        return false
    }

    override fun setTrackingPosition(trackingPosition: Boolean) {}

    override fun isUnlimitedTracking(): Boolean {
        return false
    }

    override fun setUnlimitedTracking(unlimited: Boolean) {}

    override fun isLocked(): Boolean {
        return true
    }

    override fun setLocked(locked: Boolean) {}
}
