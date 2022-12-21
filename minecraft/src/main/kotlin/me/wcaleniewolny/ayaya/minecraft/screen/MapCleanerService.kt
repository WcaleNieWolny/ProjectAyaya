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

    fun generateMapItem(id: Int, world: World): ItemStack {
        val item = ItemStack(Material.FILLED_MAP)
        val meta = item.itemMeta as MapMeta
        meta.mapView = FakeBukkitMapView(id)
        item.itemMeta = meta
        return item
    }
}