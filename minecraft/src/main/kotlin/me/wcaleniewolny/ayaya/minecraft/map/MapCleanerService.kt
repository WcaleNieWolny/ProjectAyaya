package me.wcaleniewolny.ayaya.minecraft.map

import net.minecraft.resources.ResourceKey
import net.minecraft.world.level.saveddata.maps.MapItemSavedData
import org.bukkit.Material
import org.bukkit.World
import org.bukkit.craftbukkit.v1_18_R2.CraftWorld
import org.bukkit.inventory.ItemStack
import org.bukkit.inventory.meta.MapMeta
import org.bukkit.map.MapView


object MapCleanerService {
    fun cleanMaps(startID: Int, len: Int) {
        for (i in startID until len) {
            val map = getBukkitMapView(i)
            map.isLocked = true
            map.renderers.forEach { map.removeRenderer(it) }
            map.isTrackingPosition = false
            map.isUnlimitedTracking = false
        }
    }

    private fun getBukkitMapView(world: World, id: Int): MapView {
//        val ws = (world as CraftWorld).handle
//        ws.setMapData("map_$id", MapItemSavedData.createFresh(
//            0.0,
//            0.0,
//            0,
//            false,
//            false,
//            ResourceKey.create(ws.world.resou)
//        ))
    }

    fun generateMapItem(id: Int): ItemStack {
        val item = ItemStack(Material.FILLED_MAP)
        val meta = item.itemMeta as MapMeta
        meta.mapView = getBukkitMapView(id)
        item.itemMeta = meta
        return item
    }
}