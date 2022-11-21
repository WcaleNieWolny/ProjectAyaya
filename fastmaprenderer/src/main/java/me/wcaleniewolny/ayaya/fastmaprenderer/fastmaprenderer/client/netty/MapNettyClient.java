package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.netty;

import io.netty.bootstrap.Bootstrap;
import io.netty.channel.Channel;
import io.netty.channel.ChannelException;
import io.netty.channel.ChannelInitializer;
import io.netty.channel.ChannelPipeline;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.nio.NioSocketChannel;
import io.netty.handler.codec.LengthFieldBasedFrameDecoder;
import java.util.ArrayList;
import me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.RenderMetadata;
import net.minecraft.item.map.MapState;

public class MapNettyClient {

    private final ArrayList<MapState> mapStates;
    private Channel channel;
    private String ip;
    private int port;
    private RenderMetadata metadata;

    public MapNettyClient(String ip, int port, RenderMetadata metadata, ArrayList<MapState> mapStates) {
        this.ip = ip;
        this.port = port;
        this.metadata = metadata;
        this.mapStates = mapStates;
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
                        pipeline.addLast("framer", new LengthFieldBasedFrameDecoder(Integer.MAX_VALUE, 0, 4, 0, 4));
                        pipeline.addLast("decompression", new CompressionDecoder(metadata.finalLength()));
                        //pipeline.addLast("decompression", new JdkZlibDecoder(ZlibWrapper.ZLIB));
                        pipeline.addLast("handler", new NettyDataHandler(mapStates, metadata));
                    }
                });
        this.channel = bootstrap.connect(ip, port).sync().channel(); //TODO constructor

        if (!channel.isActive()) {
            throw new ChannelException();
        }
    }

    public void close() {
        try {
            this.channel.close().sync();
        } catch (InterruptedException e) {
            throw new RuntimeException(e);
        }
    }
}
