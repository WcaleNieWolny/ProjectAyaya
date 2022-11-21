package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.mixin;

import net.minecraft.client.render.MapRenderer;
import org.spongepowered.asm.mixin.Mixin;

@Mixin(MapRenderer.MapTexture.class)
public class MapTextureMixin {
//    @Inject(method = "setState", at = @At("HEAD"), cancellable = true)
//    private void injected(MapState state, CallbackInfo ci){
//        ci.cancel();
//    }
//
//    @Inject(method = "draw", at = @At("HEAD"), cancellable = true)
//    private void injected2(MatrixStack matrices, VertexConsumerProvider vertexConsumers, boolean hidePlayerIcons, int light, CallbackInfo ci){
//        ci.cancel();
//    }

//    @Inject(method = "draw", at = @At(value = "INVOKE", target = "Lnet/minecraft/item/map/MapState;getIcons()Ljava/lang/Iterable;"), cancellable = true)
//    private void injected3(MatrixStack matrices, VertexConsumerProvider vertexConsumers, boolean hidePlayerIcons, int light, CallbackInfo ci){
//        ci.cancel();
//    }
}
