import type { Viewport, Vec2 } from '../types/model.ts';

export function screenToWorld(screen: Vec2, vp: Viewport): Vec2 {
  return {
    x: (screen.x - vp.offsetX) / vp.zoom,
    y: (screen.y - vp.offsetY) / vp.zoom,
  };
}

export function worldToScreen(world: Vec2, vp: Viewport): Vec2 {
  return {
    x: world.x * vp.zoom + vp.offsetX,
    y: world.y * vp.zoom + vp.offsetY,
  };
}

export function zoomAround(vp: Viewport, pivot: Vec2, factor: number): Viewport {
  const newZoom = Math.min(32, Math.max(0.05, vp.zoom * factor));
  return {
    zoom: newZoom,
    offsetX: pivot.x - (pivot.x - vp.offsetX) * (newZoom / vp.zoom),
    offsetY: pivot.y - (pivot.y - vp.offsetY) * (newZoom / vp.zoom),
    rotation: vp.rotation,
  };
}

export function fitViewport(vpWidth: number, vpHeight: number, worldW: number, worldH: number): Viewport {
  const zoom = Math.min(vpWidth / worldW, vpHeight / worldH) * 0.85;
  return {
    zoom,
    offsetX: (vpWidth - worldW * zoom) / 2,
    offsetY: (vpHeight - worldH * zoom) / 2,
    rotation: 0,
  };
}

/** Un-rotate a screen point around the canvas center to get the "pre-rotation" screen coords. */
export function unrotatePoint(x: number, y: number, cx: number, cy: number, rotation: number): Vec2 {
  if (rotation === 0) return { x, y };
  const cos = Math.cos(-rotation);
  const sin = Math.sin(-rotation);
  const dx = x - cx;
  const dy = y - cy;
  return { x: cx + dx * cos - dy * sin, y: cy + dx * sin + dy * cos };
}
