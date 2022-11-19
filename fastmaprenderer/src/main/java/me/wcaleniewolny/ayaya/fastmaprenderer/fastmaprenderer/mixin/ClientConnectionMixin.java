package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.mixin;

import io.netty.channel.ChannelHandler;
import io.netty.channel.ChannelHandlerContext;
import io.netty.channel.ChannelPipeline;
import io.netty.util.concurrent.DefaultEventExecutorGroup;
import io.netty.util.concurrent.EventExecutorGroup;
import me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.ClientDisconnectCallback;
import net.minecraft.network.ClientConnection;
import net.minecraft.util.ActionResult;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ClientConnection.class)
public class ClientConnectionMixin {

//    @Mixin(targets="net.minecraft.network.ClientConnection$1")
//    @Redirect(method = "initChannel(Lio/netty/channel/Channel;)V", at = @At(value = "INVOKE", target = "Lio/netty/channel/ChannelPipeline;addLast(Ljava/lang/String;Lio/netty/channel/ChannelHandler;)Lio/netty/channel/ChannelPipeline;"))
//    private ChannelPipeline injected(ChannelPipeline instance, String s, ChannelHandler channelHandler) {
//        System.out.println("ADDING " + channelHandler.getClass().getName());
//        instance.addLast(s, channelHandler);
//        return instance;
//    }

    @Inject(method = "channelInactive", at = @At("TAIL"), cancellable = true)
    private void onDisconnect(ChannelHandlerContext context, CallbackInfo info){
        ActionResult result = ClientDisconnectCallback.EVENT.invoker().interact();

        if(result == ActionResult.FAIL) {
            info.cancel();
        }
    }

}
