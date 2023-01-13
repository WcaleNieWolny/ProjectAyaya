package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.mixin;

import net.minecraft.item.map.MapState;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(MapState.UpdateData.class)
public class MapStateMixin {

    @Shadow @Final public int width;

    @Inject(method = "setColorsTo", at = @At("HEAD"), cancellable = true)
    void injected(MapState mapState, CallbackInfo ci) {
        MapState.UpdateData data = ((MapState.UpdateData) (Object) this);
        //TODO: Implement checking from NettyDataHandler

        if (data.width == 128) {
            System.arraycopy(data.colors, 0, mapState.colors, data.startZ * data.width + data.startX, data.width * data.height);
        }else {
            int loopI = 0;
            for (int loopY = data.startZ; loopY < data.height + data.startZ; loopY++) {
                System.arraycopy(data.colors, loopI * data.width, mapState.colors, loopY * 128 + data.startX, data.width);
                loopI++;
            }
        }




        //mapState.setColor(data.startX + i, data.startZ + j, data.colors[i + j * data.width]);
        ci.cancel();
    }
}
