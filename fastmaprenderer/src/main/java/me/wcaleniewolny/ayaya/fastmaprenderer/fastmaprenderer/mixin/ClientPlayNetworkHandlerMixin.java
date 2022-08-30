package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.mixin;

import java.util.Objects;
import java.util.Queue;
import net.minecraft.block.MapColor;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import net.minecraft.client.render.MapRenderer;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.item.FilledMapItem;
import net.minecraft.item.map.MapState;
import net.minecraft.network.packet.s2c.play.MapUpdateS2CPacket;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ClientPlayNetworkHandler.class)
public class ClientPlayNetworkHandlerMixin {

    @Inject(method = "onMapUpdate(Lnet/minecraft/network/packet/s2c/play/MapUpdateS2CPacket;)V", at = @At("HEAD"), cancellable = true)
    private void injected(MapUpdateS2CPacket packet, CallbackInfo ci) {
        MinecraftClient client = ((ClientPlayNetworkHandlerAccessor)this).getClient();
        //NetworkThreadUtils.forceMainThread(packet, ((ClientPlayNetworkHandler)(Object)this), client);

        Queue<Runnable> renderTaskQueue = ((MinecraftClientAccessor)client).getRenderTaskQueue();

        MapRenderer mapRenderer = client.gameRenderer.getMapRenderer();
        int i = packet.getId();
        String string = FilledMapItem.getMapName(i);

        if(client.world == null){
            return;
        }

        MapState mapState = client.world.getMapState(string);
        if (mapState == null) {
            mapState = MapState.of(packet.getScale(), packet.isLocked(), client.world.getRegistryKey());
            client.world.putMapState(string, mapState);
        }
        packet.apply(mapState);

        MapRendererInvoker mapRendererInvoker = ((MapRendererInvoker) mapRenderer);

        MapRenderer.MapTexture texture;


        texture = mapRendererInvoker.invokeGetMapTexture(i, mapState);



        MapTextureAccessor accessor = Objects.requireNonNull((MapTextureAccessor) texture);

        NativeImageBackedTexture nativeTexture = accessor.getTexture();

        for (int o = 0; o < 128; ++o) {
            for (int j = 0; j < 128; ++j) {
                int k = j + o * 128;
                nativeTexture.getImage().setColor(j, o, MapColor.getRenderColor(mapState.colors[k]));
            }
        }

        //client.executeSync(() -> mapRenderer.updateTexture(i, finalMapState));
        renderTaskQueue.add(nativeTexture::upload);

        //mapRenderer.updateTexture(i, finalMapState);

        ci.cancel();
    }
}
