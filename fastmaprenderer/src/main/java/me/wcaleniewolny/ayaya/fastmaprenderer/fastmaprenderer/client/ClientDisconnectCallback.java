package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client;

import net.fabricmc.fabric.api.event.Event;
import net.fabricmc.fabric.api.event.EventFactory;
import net.minecraft.util.ActionResult;

public interface ClientDisconnectCallback {
    Event<ClientDisconnectCallback> EVENT = EventFactory.createArrayBacked(ClientDisconnectCallback.class,
            (listeners) -> () -> {
                for (ClientDisconnectCallback listener : listeners) {
                    ActionResult result = listener.interact();

                    if (result != ActionResult.PASS) {
                        return result;
                    }
                }

                return ActionResult.PASS;
            });

    ActionResult interact();
}
