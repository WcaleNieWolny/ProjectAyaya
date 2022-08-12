package me.wcaleniewolny.ayaya.minecraft.map

import org.bukkit.Bukkit
import org.bukkit.Material
import org.bukkit.inventory.ItemStack
import org.bukkit.inventory.meta.MapMeta
import org.bukkit.map.MapView

object MapCleanerService {
    fun cleanMaps(startID: Int, len: Int){
        for (i in startID until len){
            val map = getBukkitMapView(i)
            map.isLocked = true
            map.renderers.forEach{ map.removeRenderer(it) }
            map.isTrackingPosition = false
            map.isUnlimitedTracking = false
        }
    }

    private fun getBukkitMapView(id: Int): MapView{
        var map = Bukkit.getMap(id)

        if(map == null){
            var depth = 0;
            while (Bukkit.createMap(Bukkit.getWorlds()[0]).id != id){
                depth++

                if(depth == 100){
                    throw RuntimeException("Project Ayaya may or may not just created 100 useless map ids. Something went wrong, we are sorry")
                }
            }
            map = Bukkit.getMap(id)!!
        }

        return map
    }

    fun generateMapItem(id: Int): ItemStack{
        val item = ItemStack(Material.FILLED_MAP)
        val meta = item.itemMeta as MapMeta
        meta.mapView = getBukkitMapView(id)
        item.itemMeta = meta
        return item
    }
}