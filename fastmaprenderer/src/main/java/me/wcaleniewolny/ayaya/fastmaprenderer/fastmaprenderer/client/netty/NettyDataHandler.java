package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.netty;

import io.netty.buffer.ByteBuf;
import io.netty.channel.ChannelHandlerContext;
import io.netty.channel.SimpleChannelInboundHandler;

public class NettyDataHandler extends SimpleChannelInboundHandler<ByteBuf> {
    @Override
    protected void channelRead0(ChannelHandlerContext ctx, ByteBuf msg) throws Exception {
        int len = msg.readableBytes();
        byte[] buffer = new byte[len];
        msg.readBytes(buffer);

        String s = new String(buffer);
        System.out.println("HOPE!: " + s);
    }
}
