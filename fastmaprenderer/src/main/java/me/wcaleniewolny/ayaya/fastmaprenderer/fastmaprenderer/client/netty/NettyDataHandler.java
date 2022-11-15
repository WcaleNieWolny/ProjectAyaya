package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.netty;

import io.netty.buffer.ByteBuf;
import io.netty.channel.ChannelHandlerContext;
import io.netty.channel.SimpleChannelInboundHandler;

public class NettyDataHandler extends SimpleChannelInboundHandler<ByteBuf> {
    @Override
    protected void channelRead0(ChannelHandlerContext ctx, ByteBuf msg) {
        int len = msg.readableBytes();
        byte[] buffer = new byte[len];
        msg.readBytes(buffer);

        System.out.println("LEN!: " + len);
    }
}
