package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client;

public class RenderMetadata {
    private final int xMargin;
    private final int yMargin;
    private final int allFramesX;
    private final int allFramesY;
    private final int finalLength;
    private final int startMapId;

    public RenderMetadata(int xMargin, int yMargin, int allFramesX, int allFramesY, int finalLength, int startMapId) {
        this.xMargin = xMargin;
        this.yMargin = yMargin;
        this.allFramesX = allFramesX;
        this.allFramesY = allFramesY;
        this.finalLength = finalLength;
        this.startMapId = startMapId;
    }

    public int xMargin() {
        return xMargin;
    }

    public int yMargin() {
        return yMargin;
    }

    public int allFramesX() {
        return allFramesX;
    }

    public int allFramesY() {
        return allFramesY;
    }

    public int finalLength() {
        return finalLength;
    }

    public int startMapId() {
        return startMapId;
    }

    @Override
    public String toString() {
        return "RenderMetadata{" +
                "xMargin=" + xMargin +
                ", yMargin=" + yMargin +
                ", allFramesX=" + allFramesX +
                ", allFramesY=" + allFramesY +
                ", finalLength=" + finalLength +
                ", startMapId=" + startMapId +
                '}';
    }
}
