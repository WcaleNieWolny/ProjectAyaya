package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.mixin;

import net.minecraft.item.map.MapState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(MapState.UpdateData.class)
public class MapStateMixin {

    @Inject(method = "setColorsTo", at = @At("HEAD"), cancellable = true)
    void injected(MapState mapState, CallbackInfo ci) {
        MapState.UpdateData data = ((MapState.UpdateData) (Object) this);
        System.arraycopy(data.colors, 0, mapState.colors, data.startZ * data.width + data.startX, data.width * data.height);


        //mapState.setColor(data.startX + i, data.startZ + j, data.colors[i + j * data.width]);
        ci.cancel();
    }
}
