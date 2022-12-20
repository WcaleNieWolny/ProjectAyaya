package me.wcaleniewolny.ayaya.minecraft.screen

import net.minecraft.world.level.saveddata.maps.MapItemSavedData
import org.bukkit.Bukkit
import org.bukkit.Material
import org.bukkit.World
import org.bukkit.craftbukkit.v1_18_R2.CraftWorld
import org.bukkit.inventory.ItemStack
import org.bukkit.inventory.meta.MapMeta
import org.bukkit.map.MapView


object MapCleanerService {

    //Note: We do not need this method, however I will keep it there if I use protocollib in the future
    //NMS does not initialize the map like bukkit does
    fun cleanMaps(world: World, startID: Int, len: Int) {
        for (i in startID until len) {
            val map = getBukkitMapView(world, i)
            map.isLocked = true
            map.isUnlimitedTracking = false
            map.renderers.forEach { map.removeRenderer(it) }
            map.isTrackingPosition = false
            map.isUnlimitedTracking = false
        }
    }

    private fun getBukkitMapView(world: World, id: Int): MapView {
        val map = Bukkit.getMap(id)
        if (map != null) {
            return map
        }

        val ws = (world as CraftWorld).handle

        ws.setMapData(
            "map_$id", MapItemSavedData.createFresh(
                0.0,
                0.0,
                0,
                false,
                false,
                ws.dimension()
            )
        );

        return Bukkit.getMap(id)!!
    }

    fun generateMapItem(id: Int, world: World): ItemStack {
        val item = ItemStack(Material.FILLED_MAP)
        val meta = item.itemMeta as MapMeta
        meta.mapView = getBukkitMapView(world, id)
        item.itemMeta = meta
        return item
    }
}