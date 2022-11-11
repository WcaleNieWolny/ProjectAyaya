package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.mixin;

import io.netty.channel.ChannelHandler;
import io.netty.channel.ChannelPipeline;
import io.netty.util.concurrent.DefaultEventExecutorGroup;
import io.netty.util.concurrent.EventExecutorGroup;
import net.minecraft.network.ClientConnection;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

@Mixin(targets="net.minecraft.network.ClientConnection$1")
public class ClientConnectionMixin {

    //private static final EventExecutorGroup group = new DefaultEventExecutorGroup(16);

    @Redirect(method = "initChannel(Lio/netty/channel/Channel;)V", at = @At(value = "INVOKE", target = "Lio/netty/channel/ChannelPipeline;addLast(Ljava/lang/String;Lio/netty/channel/ChannelHandler;)Lio/netty/channel/ChannelPipeline;"))
    private ChannelPipeline injected(ChannelPipeline instance, String s, ChannelHandler channelHandler) {
        System.out.println("ADDING " + channelHandler.getClass().getName());
//        if(channelHandler.getClass() == ClientConnection.class){
//            System.out.println("CHANG C");
//            instance.addLast(group, s, channelHandler);
//            return instance;
//        }
        instance.addLast(s, channelHandler);
        return instance;
    }

}
