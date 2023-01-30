package me.wcaleniewolny.ayaya.minecraft.screen

import org.bukkit.Material
import org.bukkit.World
import org.bukkit.inventory.ItemStack
import org.bukkit.inventory.meta.MapMeta


object MapCleanerService {

    fun generateMapItem(id: Int, world: World): ItemStack {
        val item = ItemStack(Material.FILLED_MAP)
        val meta = item.itemMeta as MapMeta
        meta.mapView = FakeBukkitMapView(id)
        item.itemMeta = meta
        return item
    }
}