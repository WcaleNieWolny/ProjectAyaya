package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.netty;

import io.netty.buffer.ByteBuf;
import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.ByteToMessageDecoder;
import java.nio.ByteBuffer;
import java.util.Arrays;
import java.util.List;
import java.util.zip.Inflater;

public class CompressionDecoder extends ByteToMessageDecoder {

    private final int frameLength;
    private byte[] buffer;

    public CompressionDecoder(int frameLength) {
        this.frameLength = frameLength;
        this.buffer = new byte[frameLength];
    }

    @Override
    protected void decode(ChannelHandlerContext ctx, ByteBuf in, List<Object> out) throws Exception {

        System.out.println("PRE COMPRESSION: " + in.readableBytes());

        if(in.readableBytes() == 0){
            return;
        }

        Inflater inflater = new Inflater();
        byte[] input;

        if(in.hasArray()){
            input = in.array();
        }else {
            input = new byte[in.readableBytes()];
            in.readBytes(input,0, in.readableBytes());
        }

        inflater.setInput(input);
        inflater.inflate(buffer);

        out.add(buffer);
    }
}
