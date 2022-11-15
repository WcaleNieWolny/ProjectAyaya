package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.netty;

import io.netty.bootstrap.Bootstrap;
import io.netty.channel.Channel;
import io.netty.channel.ChannelException;
import io.netty.channel.ChannelInitializer;
import io.netty.channel.ChannelPipeline;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.nio.NioSocketChannel;
import io.netty.handler.codec.LengthFieldBasedFrameDecoder;
import io.netty.handler.codec.compression.JdkZlibDecoder;
import io.netty.handler.codec.compression.ZlibWrapper;
import me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.RenderMetadata;

public class MapNettyClient {

    private Channel channel;
    private String ip;
    private int port;
    private RenderMetadata metadata;

    public MapNettyClient(String ip, int port, RenderMetadata metadata) {
        this.ip = ip;
        this.port = port;
        this.metadata = metadata;
    }

    public void run() throws InterruptedException, ChannelException {
        NioEventLoopGroup group = new NioEventLoopGroup(1);

        Bootstrap bootstrap = new Bootstrap()
                .group(group)
                .channel(NioSocketChannel.class)
                .handler(new ChannelInitializer() {

                    @Override
                    protected void initChannel(Channel socketChannel) {
                        ChannelPipeline pipeline = socketChannel.pipeline();
                        pipeline.addLast("framer", new LengthFieldBasedFrameDecoder(Short.MAX_VALUE, 0, 4, 0, 4));
                        pipeline.addLast("decompression", new JdkZlibDecoder(ZlibWrapper.GZIP));
                        pipeline.addLast("handler", new NettyDataHandler());
                    }
                });
        this.channel = bootstrap.connect(ip, port).sync().channel(); //TODO constructor

        if (!channel.isActive()){
            throw new ChannelException();
        }
    }
}
