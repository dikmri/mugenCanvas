import type { AnimationLayer, BrushSettings, OnionSkinSettings, Vec2, Viewport } from '../types/model.ts';
import { worldToScreen } from './viewport.ts';

export const TILE_SIZE = 1024;

function hexToRgba(hex: string, alpha: number): string {
  const h = hex.replace('#', '');
  const r = parseInt(h.substring(0, 2), 16);
  const g = parseInt(h.substring(2, 4), 16);
  const b = parseInt(h.substring(4, 6), 16);
  return `rgba(${r},${g},${b},${alpha})`;
}

function drawSegmentToCtx(
  ctx: OffscreenCanvasRenderingContext2D,
  from: Vec2, to: Vec2,
  settings: BrushSettings,
  isEraser: boolean,
): void {
  const dist = Math.hypot(to.x - from.x, to.y - from.y);
  const step = Math.max(settings.size * 0.25, 1);
  const steps = Math.max(1, Math.ceil(dist / step));
  const r = settings.size / 2;
  ctx.save();
  if (isEraser) {
    ctx.globalCompositeOperation = 'destination-out';
    ctx.fillStyle = 'rgba(0,0,0,1)';
  } else {
    ctx.globalCompositeOperation = 'source-over';
    ctx.fillStyle = hexToRgba(settings.color, settings.opacity);
  }
  ctx.beginPath();
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    const x = from.x + (to.x - from.x) * t;
    const y = from.y + (to.y - from.y) * t;
    ctx.moveTo(x + r, y);
    ctx.arc(x, y, r, 0, Math.PI * 2);
  }
  ctx.fill();
  ctx.restore();
}

function floodFillCtx(
  ctx: OffscreenCanvasRenderingContext2D,
  startX: number, startY: number,
  fillHex: string, opacity: number,
): void {
  const { width, height } = ctx.canvas;
  const sx = Math.round(startX), sy = Math.round(startY);
  if (sx < 0 || sy < 0 || sx >= width || sy >= height) return;
  const imageData = ctx.getImageData(0, 0, width, height);
  const data = imageData.data;
  const i0 = (sy * width + sx) * 4;
  const tR = data[i0], tG = data[i0 + 1], tB = data[i0 + 2], tA = data[i0 + 3];
  const h = fillHex.replace('#', '');
  const fR = parseInt(h.slice(0, 2), 16);
  const fG = parseInt(h.slice(2, 4), 16);
  const fB = parseInt(h.slice(4, 6), 16);
  const fA = Math.round(opacity * 255);
  if (tR === fR && tG === fG && tB === fB && tA === fA) return;
  const stack: number[] = [sy * width + sx];
  const visited = new Uint8Array(width * height);
  while (stack.length > 0) {
    const pos = stack.pop()!;
    if (visited[pos]) continue;
    const idx = pos * 4;
    if (data[idx] !== tR || data[idx + 1] !== tG || data[idx + 2] !== tB || data[idx + 3] !== tA) continue;
    visited[pos] = 1;
    data[idx] = fR; data[idx + 1] = fG; data[idx + 2] = fB; data[idx + 3] = fA;
    const x = pos % width, y = (pos - x) / width;
    if (x > 0) stack.push(pos - 1);
    if (x < width - 1) stack.push(pos + 1);
    if (y > 0) stack.push(pos - width);
    if (y < height - 1) stack.push(pos + width);
  }
  ctx.putImageData(imageData, 0, 0);
}

export class Renderer {
  // Primary store: "layerId:frame:tileX:tileY" → OffscreenCanvas
  private tiles = new Map<string, OffscreenCanvas>();
  // Secondary index for fast iteration: "layerId:frame" → Set<"tileX:tileY">
  private tileSets = new Map<string, Set<string>>();
  private tintCanvas: OffscreenCanvas | null = null;
  private tintCtx: OffscreenCanvasRenderingContext2D | null = null;
  private projectW = 1920;
  private projectH = 1080;

  setCanvasSize(w: number, h: number): void {
    if (this.projectW === w && this.projectH === h) return;
    this.projectW = w;
    this.projectH = h;
    this.tiles.clear();
    this.tileSets.clear();
  }

  private coordKey(tx: number, ty: number): string { return `${tx}:${ty}`; }
  private lfKey(layerId: string, frame: number): string { return `${layerId}:${frame}`; }
  private fullKey(layerId: string, frame: number, tx: number, ty: number): string {
    return `${layerId}:${frame}:${tx}:${ty}`;
  }

  private getOrCreateTile(layerId: string, frame: number, tx: number, ty: number): OffscreenCanvas {
    const fk = this.fullKey(layerId, frame, tx, ty);
    if (!this.tiles.has(fk)) {
      const tile = new OffscreenCanvas(TILE_SIZE, TILE_SIZE);
      this.tiles.set(fk, tile);
      const lf = this.lfKey(layerId, frame);
      if (!this.tileSets.has(lf)) this.tileSets.set(lf, new Set());
      this.tileSets.get(lf)!.add(this.coordKey(tx, ty));
    }
    return this.tiles.get(fk)!;
  }

  // Pre-create camera-frame tiles + 1-tile margin on first stroke of a layer-frame.
  ensureBaseTiles(layerId: string, frame: number): void {
    const txMax = Math.ceil(this.projectW / TILE_SIZE);
    const tyMax = Math.ceil(this.projectH / TILE_SIZE);
    for (let tx = -1; tx <= txMax; tx++) {
      for (let ty = -1; ty <= tyMax; ty++) {
        this.getOrCreateTile(layerId, frame, tx, ty);
      }
    }
  }

  // Draw a stroke segment (world coords) to all tiles it touches.
  drawSegmentToTiles(
    layerId: string, frame: number,
    from: Vec2, to: Vec2,
    settings: BrushSettings, isEraser: boolean,
  ): void {
    const r = settings.size / 2;
    const txMin = Math.floor((Math.min(from.x, to.x) - r) / TILE_SIZE);
    const txMax = Math.floor((Math.max(from.x, to.x) + r) / TILE_SIZE);
    const tyMin = Math.floor((Math.min(from.y, to.y) - r) / TILE_SIZE);
    const tyMax = Math.floor((Math.max(from.y, to.y) + r) / TILE_SIZE);
    for (let tx = txMin; tx <= txMax; tx++) {
      for (let ty = tyMin; ty <= tyMax; ty++) {
        const tile = this.getOrCreateTile(layerId, frame, tx, ty);
        const ctx = tile.getContext('2d') as OffscreenCanvasRenderingContext2D;
        const ox = tx * TILE_SIZE, oy = ty * TILE_SIZE;
        drawSegmentToCtx(ctx, { x: from.x - ox, y: from.y - oy }, { x: to.x - ox, y: to.y - oy }, settings, isEraser);
      }
    }
  }

  // Flood fill across all existing tiles (composites → fills → splits back).
  floodFillTiles(
    layerId: string, frame: number,
    worldX: number, worldY: number,
    fillHex: string, opacity: number,
  ): void {
    const lf = this.lfKey(layerId, frame);
    const tx0 = Math.floor(worldX / TILE_SIZE);
    const ty0 = Math.floor(worldY / TILE_SIZE);
    // Ensure the clicked tile exists
    this.getOrCreateTile(layerId, frame, tx0, ty0);
    const coordSet = this.tileSets.get(lf)!;

    // Find bounding box of all tiles
    let txMin = Infinity, txMax = -Infinity, tyMin = Infinity, tyMax = -Infinity;
    for (const c of coordSet) {
      const sep = c.indexOf(':');
      const tx = parseInt(c.slice(0, sep)), ty = parseInt(c.slice(sep + 1));
      if (tx < txMin) txMin = tx; if (tx > txMax) txMax = tx;
      if (ty < tyMin) tyMin = ty; if (ty > tyMax) tyMax = ty;
    }

    // Composite all tiles into a temporary canvas
    const tempW = (txMax - txMin + 1) * TILE_SIZE;
    const tempH = (tyMax - tyMin + 1) * TILE_SIZE;
    const temp = new OffscreenCanvas(tempW, tempH);
    const tc = temp.getContext('2d')!;
    for (const c of coordSet) {
      const sep = c.indexOf(':');
      const tx = parseInt(c.slice(0, sep)), ty = parseInt(c.slice(sep + 1));
      tc.drawImage(this.tiles.get(`${lf}:${c}`)!, (tx - txMin) * TILE_SIZE, (ty - tyMin) * TILE_SIZE);
    }

    floodFillCtx(tc, worldX - txMin * TILE_SIZE, worldY - tyMin * TILE_SIZE, fillHex, opacity);

    // Write back to individual tiles
    for (const c of coordSet) {
      const sep = c.indexOf(':');
      const tx = parseInt(c.slice(0, sep)), ty = parseInt(c.slice(sep + 1));
      const tileCanvas = this.tiles.get(`${lf}:${c}`)!;
      const tileCtx = tileCanvas.getContext('2d')!;
      tileCtx.putImageData(
        tc.getImageData((tx - txMin) * TILE_SIZE, (ty - tyMin) * TILE_SIZE, TILE_SIZE, TILE_SIZE),
        0, 0,
      );
    }
  }

  removeLayer(id: string): void {
    for (const key of [...this.tiles.keys()]) {
      if (key.startsWith(id + ':')) this.tiles.delete(key);
    }
    for (const key of [...this.tileSets.keys()]) {
      if (key.startsWith(id + ':')) this.tileSets.delete(key);
    }
  }

  drawLayers(
    ctx: CanvasRenderingContext2D,
    layers: AnimationLayer[],
    vp: Viewport,
    currentFrame: number,
    onionSkin: boolean,
    onionSettings: OnionSkinSettings,
    screenW: number,
    screenH: number,
  ): void {
    if (onionSkin) {
      if (currentFrame > 1) {
        for (const layer of layers) {
          if (!layer.visible) continue;
          const src = resolveLayerFrame(layer, currentFrame - 1);
          if (src !== null) this.blitTiledTinted(ctx, layer.id, src, vp,
            onionSettings.prevColor, onionSettings.prevAlpha, screenW, screenH);
        }
      }
      for (const layer of layers) {
        if (!layer.visible) continue;
        const src = resolveLayerFrame(layer, currentFrame + 1);
        if (src !== null) this.blitTiledTinted(ctx, layer.id, src, vp,
          onionSettings.nextColor, onionSettings.nextAlpha, screenW, screenH);
      }
    }
    for (const layer of layers) {
      if (!layer.visible) continue;
      const src = resolveLayerFrame(layer, currentFrame);
      if (src !== null) this.blitTiles(ctx, layer.id, src, vp, 1);
    }
  }

  private blitTiles(
    ctx: CanvasRenderingContext2D,
    layerId: string, frame: number,
    vp: Viewport, alpha: number,
  ): void {
    const lf = this.lfKey(layerId, frame);
    const coordSet = this.tileSets.get(lf);
    if (!coordSet) return;
    ctx.save();
    ctx.globalAlpha = alpha;
    for (const c of coordSet) {
      const sep = c.indexOf(':');
      const tx = parseInt(c.slice(0, sep)), ty = parseInt(c.slice(sep + 1));
      const tile = this.tiles.get(`${lf}:${c}`)!;
      const sp = worldToScreen({ x: tx * TILE_SIZE, y: ty * TILE_SIZE }, vp);
      ctx.drawImage(tile, sp.x, sp.y, TILE_SIZE * vp.zoom, TILE_SIZE * vp.zoom);
    }
    ctx.restore();
  }

  private blitTiledTinted(
    ctx: CanvasRenderingContext2D,
    layerId: string, frame: number,
    vp: Viewport,
    tintColor: string, alpha: number,
    screenW: number, screenH: number,
  ): void {
    const lf = this.lfKey(layerId, frame);
    const coordSet = this.tileSets.get(lf);
    if (!coordSet || coordSet.size === 0) return;
    if (!this.tintCanvas || this.tintCanvas.width !== screenW || this.tintCanvas.height !== screenH) {
      this.tintCanvas = new OffscreenCanvas(screenW, screenH);
      this.tintCtx = this.tintCanvas.getContext('2d')!;
    }
    const tc = this.tintCtx!;
    tc.clearRect(0, 0, screenW, screenH);
    for (const c of coordSet) {
      const sep = c.indexOf(':');
      const tx = parseInt(c.slice(0, sep)), ty = parseInt(c.slice(sep + 1));
      const tile = this.tiles.get(`${lf}:${c}`)!;
      const sp = worldToScreen({ x: tx * TILE_SIZE, y: ty * TILE_SIZE }, vp);
      tc.drawImage(tile, sp.x, sp.y, TILE_SIZE * vp.zoom, TILE_SIZE * vp.zoom);
    }
    tc.globalCompositeOperation = 'source-atop';
    tc.fillStyle = tintColor;
    tc.fillRect(0, 0, screenW, screenH);
    tc.globalCompositeOperation = 'source-over';
    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.drawImage(this.tintCanvas, 0, 0);
    ctx.restore();
  }

  // Draw the world region [srcX, srcX+width) × [srcY, srcY+height) onto destCtx at (0,0).
  drawRegionToCanvas(
    layerId: string, frame: number,
    destCtx: OffscreenCanvasRenderingContext2D,
    srcX: number, srcY: number, width: number, height: number,
  ): void {
    const txMin = Math.floor(srcX / TILE_SIZE);
    const txMax = Math.floor((srcX + width - 1) / TILE_SIZE);
    const tyMin = Math.floor(srcY / TILE_SIZE);
    const tyMax = Math.floor((srcY + height - 1) / TILE_SIZE);
    const lf = this.lfKey(layerId, frame);
    for (let tx = txMin; tx <= txMax; tx++) {
      for (let ty = tyMin; ty <= tyMax; ty++) {
        const tile = this.tiles.get(`${lf}:${this.coordKey(tx, ty)}`);
        if (!tile) continue;
        const tileWorldX = tx * TILE_SIZE, tileWorldY = ty * TILE_SIZE;
        const ox = Math.max(srcX, tileWorldX);
        const oy = Math.max(srcY, tileWorldY);
        const ow = Math.min(srcX + width, tileWorldX + TILE_SIZE) - ox;
        const oh = Math.min(srcY + height, tileWorldY + TILE_SIZE) - oy;
        if (ow <= 0 || oh <= 0) continue;
        destCtx.drawImage(tile, ox - tileWorldX, oy - tileWorldY, ow, oh, ox - srcX, oy - srcY, ow, oh);
      }
    }
  }

  // Capture all tile ImageData for a layer-frame (undo snapshot).
  snapshotTiles(layerId: string, frame: number): Map<string, ImageData> {
    const lf = this.lfKey(layerId, frame);
    const result = new Map<string, ImageData>();
    const coordSet = this.tileSets.get(lf);
    if (!coordSet) return result;
    for (const c of coordSet) {
      const ctx = this.tiles.get(`${lf}:${c}`)!.getContext('2d')!;
      result.set(c, ctx.getImageData(0, 0, TILE_SIZE, TILE_SIZE));
    }
    return result;
  }

  // Restore a layer-frame from a tile snapshot (undo/redo restore).
  restoreTiles(layerId: string, frame: number, snapshot: Map<string, ImageData>): void {
    const lf = this.lfKey(layerId, frame);
    const prefix = lf + ':';
    for (const key of [...this.tiles.keys()]) {
      if (key.startsWith(prefix)) this.tiles.delete(key);
    }
    this.tileSets.delete(lf);
    for (const [c, imgData] of snapshot) {
      const sep = c.indexOf(':');
      const tx = parseInt(c.slice(0, sep)), ty = parseInt(c.slice(sep + 1));
      const tile = new OffscreenCanvas(TILE_SIZE, TILE_SIZE);
      tile.getContext('2d')!.putImageData(imgData, 0, 0);
      this.tiles.set(`${lf}:${c}`, tile);
      if (!this.tileSets.has(lf)) this.tileSets.set(lf, new Set());
      this.tileSets.get(lf)!.add(c);
    }
  }

  // Export all tiles for a layer-frame as base64 PNGs.
  async exportFrameTiles(layerId: string, frame: number): Promise<Map<string, string>> {
    const lf = this.lfKey(layerId, frame);
    const result = new Map<string, string>();
    const coordSet = this.tileSets.get(lf);
    if (!coordSet) return result;
    for (const c of coordSet) {
      const tile = this.tiles.get(`${lf}:${c}`)!;
      const blob = await tile.convertToBlob({ type: 'image/png' });
      const dataUrl = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result as string);
        reader.onerror = reject;
        reader.readAsDataURL(blob);
      });
      result.set(c, dataUrl);
    }
    return result;
  }

  // Load a single tile from a base64 PNG data URL.
  async loadTile(layerId: string, frame: number, tx: number, ty: number, dataUrl: string): Promise<void> {
    const resp = await fetch(dataUrl);
    const blob = await resp.blob();
    const bitmap = await createImageBitmap(blob);
    const tile = this.getOrCreateTile(layerId, frame, tx, ty);
    const ctx = tile.getContext('2d')!;
    ctx.clearRect(0, 0, TILE_SIZE, TILE_SIZE);
    ctx.drawImage(bitmap, 0, 0);
    bitmap.close();
  }
}

export function resolveLayerFrame(layer: AnimationLayer, frame: number): number | null {
  const f = layer.frames.find(fr => fr.frame === frame);
  if (!f) return null;
  if (f.drawingId !== null) return frame;
  if (f.holdSource !== null) return f.holdSource;
  return null;
}
