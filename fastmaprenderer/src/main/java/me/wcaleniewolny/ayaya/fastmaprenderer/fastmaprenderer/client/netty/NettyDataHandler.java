package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.netty;

import io.netty.channel.ChannelHandlerContext;
import io.netty.channel.SimpleChannelInboundHandler;
import java.util.ArrayList;
import me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.RenderMetadata;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.MapRenderer;
import net.minecraft.item.map.MapState;

public class NettyDataHandler extends SimpleChannelInboundHandler<byte[]> {

    private final ArrayList<MapState> mapStates;
    private final RenderMetadata metadata;
    private final MapRenderer mapRenderer;

    public NettyDataHandler(ArrayList<MapState> mapStates, RenderMetadata metadata) {
        this.mapStates = mapStates;
        this.metadata = metadata;
        this.mapRenderer = MinecraftClient.getInstance().gameRenderer.getMapRenderer();
    }

    @Override
    protected void channelRead0(ChannelHandlerContext ctx, byte[] msg) {
        //System.arraycopy(data.colors, 0, mapState.colors, data.startZ * data.width + data.startX, data.width * data.height);

        //TODO: FIX MSG STOPPING FROM GETING TO THE CLIENT (FIX = insecure atomic i32 in rust)
        //What the fuck is this comment?
        //Potetnial mem leak:
        //reuse client when new server

        int i = 0;
        int offset = 0;
        for (int y = 0; y < metadata.allFramesY(); y++) {
            for (int x = 0; x < metadata.allFramesX(); x++) {
                MapState state = mapStates.get(i);

                int xFrameMargin = (x == 0) ? (metadata.xMargin() / 2) : 0;
                int yFrameMargin = (y == 0) ? (metadata.yMargin() / 2) : 0;

                int frameWidth = (x != metadata.allFramesX() - 1) ? 128 - xFrameMargin : 128 - (metadata.xMargin() / 2);
                int frameHeight = (y != (metadata.allFramesY() - 1)) ? 128 - yFrameMargin : 128 - (metadata.yMargin() / 2);

                int len = frameWidth * frameHeight;

                if (frameWidth == 128) {
                    System.arraycopy(msg, offset, state.colors, yFrameMargin * 128 + xFrameMargin, len);
                }else {
                    int loopI = 0;
                    for (int loopY = yFrameMargin; loopY < (frameHeight + yFrameMargin); loopY++) {
                        System.arraycopy(msg, offset + (loopI * frameWidth), state.colors, loopY * 128 + xFrameMargin, frameWidth);
                        loopI++;
                    }
                }

                mapRenderer.updateTexture(metadata.startMapId() + i, state);

                offset += len;
                i++;
            }
        }
        mapRenderer.clearStateTextures();
    }

    @Override
    public void exceptionCaught(ChannelHandlerContext ctx, Throwable cause) {
        System.out.println("NEW ERROR -> " + cause.toString());
        cause.printStackTrace();
    }

    @Override
    public void channelInactive(ChannelHandlerContext ctx) throws Exception {
        System.out.println("[MapServer] map server connection closed!");
    }
}
