package me.wcaleniewolny.ayaya.minecraft.screen

import org.bukkit.block.BlockFace

data class Screen(val name: String, val x1: Int, val y1: Int, val z1: Int, val x2: Int, val y2: Int, val z2: Int, val mapFace: BlockFace)