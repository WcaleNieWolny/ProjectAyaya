package me.wcaleniewolny.ayaya.fastmaprenderer.fastmaprenderer.client;

public class RenderMetadata {
    private final int xMargin;
    private final int yMargin;
    private final int allFramesX;
    private final int allFramesY;

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

    @Override
    public String toString() {
        return "RenderMetadata{" +
                "xMargin=" + xMargin +
                ", yMargin=" + yMargin +
                ", allFramesX=" + allFramesX +
                ", allFramesY=" + allFramesY +
                '}';
    }

    public RenderMetadata(int xMargin, int yMargin, int allFramesX, int allFramesY) {
        this.xMargin = xMargin;
        this.yMargin = yMargin;
        this.allFramesX = allFramesX;
        this.allFramesY = allFramesY;
    }
}
