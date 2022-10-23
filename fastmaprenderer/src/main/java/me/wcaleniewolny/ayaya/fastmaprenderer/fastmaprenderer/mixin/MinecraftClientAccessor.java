package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.mixin;

import java.util.Queue;
import net.minecraft.client.MinecraftClient;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(MinecraftClient.class)
public interface MinecraftClientAccessor {

    @Accessor
    Queue<Runnable> getRenderTaskQueue();
}
