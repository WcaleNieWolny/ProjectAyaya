package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.mixin;

import net.minecraft.client.render.MapRenderer;
import net.minecraft.client.render.RenderLayer;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.item.map.MapState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;
import org.spongepowered.asm.mixin.gen.Invoker;

@Mixin(targets = "net.minecraft.client.render.MapRenderer$MapTexture")
public interface MapTextureAccessor {

    @Accessor
    NativeImageBackedTexture getTexture();

    @Accessor("texture")
    public void setTexture(NativeImageBackedTexture texture);

    @Accessor("renderLayer")
    RenderLayer getRenderLayer();

    @Invoker("<init>")
    public static MapRenderer.MapTexture callInit(MapRenderer renderer, int i, MapState state){

        throw new AssertionError();
    };
}
