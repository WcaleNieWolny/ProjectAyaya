package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client;

import io.netty.buffer.Unpooled;
import me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client.netty.MapNettyClient;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.api.EnvType;
import net.fabricmc.api.Environment;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayNetworking;
import net.minecraft.client.MinecraftClient;
import net.minecraft.network.MessageType;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.text.LiteralText;
import net.minecraft.text.Style;
import net.minecraft.util.ActionResult;
import net.minecraft.util.Formatting;
import net.minecraft.util.Identifier;
import org.jetbrains.annotations.Nullable;

@Environment(EnvType.CLIENT)
public class FastMapRendererClient implements ClientModInitializer {

    public static final String NAMESPACE = "fastmap";
    public static final Identifier HANDSHAKE_CHANNEL = new Identifier(NAMESPACE, "handshake");
    public static final Identifier ACKNOWLEDGEMENT_CHANNEL = new Identifier(NAMESPACE, "acknowledgement");

    public static final int PROTOCOL_VERSION = 0;

    @Nullable
    private MapNettyClient mapNettyClient;

    @Override
    public void onInitializeClient() {

        ClientPlayNetworking.registerGlobalReceiver(ACKNOWLEDGEMENT_CHANNEL, (client, handler, buf, responseSender) -> {
            int protocolVersion = buf.readVarInt();
            sendColorMessage("[FastMap] Server tried to create map connection!", Formatting.RED, client);
            if (protocolVersion != PROTOCOL_VERSION) {
                if (PROTOCOL_VERSION > protocolVersion) {
                    //client.inGameHud.addChatMessage(MessageType.SYSTEM, Text.of("[FastMap] Client mod is newer than server map protocol! Aborting!"), client.player.getUuid());
                    sendColorMessage("[FastMap] Client mod is newer than server map protocol! Aborting!", Formatting.RED, client);
                    sendStatusPacket(1, ACKNOWLEDGEMENT_CHANNEL);
                } else {
                    //client.inGameHud.addChatMessage(MessageType.SYSTEM, Text.of("[FastMap] Client mod is older than server map protocol! Please upgrade!"), client.player.getUuid());
                    sendColorMessage("[FastMap] Client mod is older than server map protocol! Please upgrade!", Formatting.RED, client);
                    sendStatusPacket(2, ACKNOWLEDGEMENT_CHANNEL);
                }
                return;
            }

            sendStatusPacket(0, ACKNOWLEDGEMENT_CHANNEL);
        });

        ClientPlayNetworking.registerGlobalReceiver(HANDSHAKE_CHANNEL, (client, handler, buf, responseSender) -> {
            String string = buf.readString();
            int port = buf.readVarInt();
            int xMargin = buf.readVarInt();
            int yMargin = buf.readVarInt();
            int allFramesX = buf.readVarInt();
            int allFramesY = buf.readVarInt();
            int finalLength = buf.readVarInt();

            RenderMetadata metadata = new RenderMetadata(xMargin, yMargin, allFramesX, allFramesY, finalLength);
            System.out.println(metadata);

            new Thread(() -> {
                try {
                    MapNettyClient nettyClient = new MapNettyClient(string, port, metadata);
                    nettyClient.run();
                    mapNettyClient = nettyClient;
                    sendStatusPacket(0, HANDSHAKE_CHANNEL);
                }catch (Exception exception){
                    exception.printStackTrace();
                    sendStatusPacket(1, HANDSHAKE_CHANNEL);
                }
            }).start();
        });

        ClientDisconnectCallback.EVENT.register(() -> {
            if(mapNettyClient != null){
                mapNettyClient.close();
            }
            return ActionResult.PASS;
        });
    }

    private void sendStatusPacket(int status, Identifier channel){
        PacketByteBuf outputBuffer = new PacketByteBuf(Unpooled.buffer());
        outputBuffer.writeVarInt(status);
        ClientPlayNetworking.send(channel, outputBuffer);
    }

    private static void sendColorMessage(String msg, Formatting color, MinecraftClient client){
        Style style =  Style.EMPTY.withColor(color);
        LiteralText text = new LiteralText(msg);
        text.setStyle(style);

        if (client.player != null) {
            client.inGameHud.addChatMessage(MessageType.SYSTEM, text, client.player.getUuid());
        }
    }
}
