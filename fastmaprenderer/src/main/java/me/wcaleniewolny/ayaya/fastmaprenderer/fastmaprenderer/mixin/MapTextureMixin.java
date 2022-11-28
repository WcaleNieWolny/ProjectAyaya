package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.mixin;

import net.minecraft.client.render.MapRenderer;
import net.minecraft.client.render.VertexConsumerProvider;
import net.minecraft.client.util.math.MatrixStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(MapRenderer.MapTexture.class)
public class MapTextureMixin {

    @Inject(method = "draw", at = @At(value = "INVOKE", target = "Lnet/minecraft/item/map/MapState;getIcons()Ljava/lang/Iterable;"), cancellable = true)
    private void injected3(MatrixStack matrices, VertexConsumerProvider vertexConsumers, boolean hidePlayerIcons, int light, CallbackInfo ci) {
        ci.cancel();
    }
}