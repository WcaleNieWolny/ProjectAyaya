package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.netty;

import io.netty.bootstrap.Bootstrap;
import io.netty.channel.Channel;
import io.netty.channel.ChannelInitializer;
import io.netty.channel.ChannelPipeline;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.nio.NioSocketChannel;
import io.netty.handler.codec.LengthFieldBasedFrameDecoder;
import io.netty.handler.codec.compression.JZlibDecoder;
import io.netty.handler.codec.compression.JdkZlibDecoder;
import io.netty.handler.codec.compression.ZlibDecoder;
import io.netty.handler.codec.compression.ZlibWrapper;

public class MapNettyClient {

    private Channel channel;

    public void run(){
        NioEventLoopGroup group = new NioEventLoopGroup(1);

        try {
            Bootstrap bootstrap = new Bootstrap()
                    .group(group)
                    .channel(NioSocketChannel.class)
                    .handler(new ChannelInitializer() {

                        @Override
                        protected void initChannel(Channel socketChannel) throws Exception {
                            System.out.println("TR!");
                            ChannelPipeline pipeline = socketChannel.pipeline();
                            pipeline.addLast("framer", new LengthFieldBasedFrameDecoder(Short.MAX_VALUE, 0, 4, 0, 4));
                            pipeline.addLast("decompression", new JdkZlibDecoder(ZlibWrapper.GZIP));
                            pipeline.addLast("handler", new NettyDataHandler());
                        }
                    });
            this.channel = bootstrap.connect("localhost", 1965).sync().channel(); //TODO constructor
        } catch (InterruptedException e) {
            e.printStackTrace();
        }
    }
}
