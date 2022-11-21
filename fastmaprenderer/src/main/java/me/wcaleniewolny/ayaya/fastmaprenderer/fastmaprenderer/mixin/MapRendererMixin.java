package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.mixin;

import com.mojang.blaze3d.systems.RenderSystem;
import java.util.Queue;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutionException;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.MapRenderer;
import net.minecraft.item.map.MapState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(MapRenderer.class)
public class MapRendererMixin {
    @Unique
    private final ConcurrentHashMap<Integer, MapRenderer.MapTexture> syncTextureMap = new ConcurrentHashMap<Integer, MapRenderer.MapTexture>();

    @Inject(method = "getMapTexture", at = @At("HEAD"), cancellable = true)
    private void injected(int id, MapState state, CallbackInfoReturnable<MapRenderer.MapTexture> cir) {
        MapRenderer.MapTexture texture = syncTextureMap.get(id);
        if (texture == null) {
            if (RenderSystem.isOnRenderThread()) {
                texture = MapTextureAccessor.callInit((MapRenderer) (Object) (this), id, state);
                syncTextureMap.put(id, texture);
            } else {
                CompletableFuture<MapRenderer.MapTexture> future = new CompletableFuture<>();

                Queue<Runnable> renderTaskQueue = ((MinecraftClientAccessor) MinecraftClient.getInstance()).getRenderTaskQueue();

                renderTaskQueue.add(() -> future.complete(MapTextureAccessor.callInit((MapRenderer) (Object) (this), id, state)));

                try {
                    texture = future.get();
                    syncTextureMap.put(id, texture);
                } catch (InterruptedException | ExecutionException ex) {
                    throw new RuntimeException(ex);
                }
            }
        }

        cir.setReturnValue(texture);
        cir.cancel();
    }

//    @Inject(method = "draw", at = @At("HEAD"), cancellable = true)
//    void draw(MatrixStack matrices, VertexConsumerProvider vertexConsumers, int id, MapState state, boolean hidePlayerIcons, int light, CallbackInfo ci){
//        ci.cancel();
//    }
}
