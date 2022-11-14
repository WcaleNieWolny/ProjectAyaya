package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client;

import io.netty.buffer.Unpooled;
import me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.netty.MapNettyClient;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.api.EnvType;
import net.fabricmc.api.Environment;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayNetworking;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.util.Identifier;

@Environment(EnvType.CLIENT)
public class FastMapRendererClient implements ClientModInitializer {

    public static final String NAMESPACE = "fastmap";
    public static final Identifier HANDSHAKE_CHANNEL = new Identifier(NAMESPACE, "handshake");

    @Override
    public void onInitializeClient() {
        ClientPlayNetworking.registerGlobalReceiver(HANDSHAKE_CHANNEL, (client, handler, buf, responseSender) -> {
            String string = buf.readString();
            int port = buf.readVarInt();
            System.out.println("GOT FUNNY MSH: " + string + "PORT: " + port);

            PacketByteBuf outputBuffer = new PacketByteBuf(Unpooled.buffer());
            outputBuffer.writeVarInt(1);
            ClientPlayNetworking.send(HANDSHAKE_CHANNEL, outputBuffer);

            MapNettyClient nettyClient = new MapNettyClient();
            nettyClient.run();
        });
    }
}
