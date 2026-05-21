import type { AppStateData, BrushSettings, Vec2 } from '../types/model.ts';
import { AppState } from '../app/app-state.ts';
import { screenToWorld, zoomAround, fitViewport, unrotatePoint } from './viewport.ts';
import { Renderer, resolveLayerFrame } from './renderer.ts';
import { BrushEngine } from './brush-engine.ts';
import { UndoManager } from '../app/undo-manager.ts';
import { drawCameraFrame } from './camera-track.ts';

export class CanvasView {
  private ctx: CanvasRenderingContext2D;
  private renderer = new Renderer();
  private brushEngine = new BrushEngine();
  private undoManager: UndoManager;
  private brushCursor: HTMLDivElement;
  private isPainting = false;
  private isPanning = false;
  private isMmbPanning = false;
  private isSpaceDown = false;
  private isRKeyDown = false;
  private isRotating = false;
  private panStart = { x: 0, y: 0 };
  private panStartOffset = { x: 0, y: 0 };
  private rotateStartAngle = 0;
  private rotateStartVPAngle = 0;
  private rafId = 0;
  private dirty = true;
  private appState: AppState;

  constructor(private canvas: HTMLCanvasElement, appState: AppState) {
    this.appState = appState;
    this.ctx = canvas.getContext('2d')!;

    this.undoManager = new UndoManager(
      (layerId, frame) => this.renderer.snapshotTiles(layerId, frame),
      (layerId, frame, tiles) => this.renderer.restoreTiles(layerId, frame, tiles),
    );

    this.brushCursor = this.createBrushCursor();
    this.attachEvents();
    const initSettings = appState.get().project.settings;
    this.renderer.setCanvasSize(initSettings.width, initSettings.height);
    appState.subscribe((s) => {
      this.renderer.setCanvasSize(s.project.settings.width, s.project.settings.height);
      this.dirty = true;
    });

    const s = appState.get();
    const vp = fitViewport(window.innerWidth, window.innerHeight - 84, s.project.settings.width, s.project.settings.height);
    appState.update({ viewport: vp });

    this.resize();
    this.loop();
  }

  private createBrushCursor(): HTMLDivElement {
    const el = document.createElement('div');
    el.style.cssText = [
      'position:absolute', 'border-radius:50%',
      'border:1px solid rgba(255,255,255,0.8)',
      'outline:1px solid rgba(0,0,0,0.6)',
      'pointer-events:none', 'transform:translate(-50%,-50%)',
      'display:none', 'z-index:9999',
    ].join(';');
    const parent = this.canvas.parentElement!;
    parent.style.position = 'relative';
    parent.appendChild(el);
    return el;
  }

  resize(): void {
    const el = this.canvas.parentElement!;
    this.canvas.width = el.clientWidth;
    this.canvas.height = el.clientHeight;
    this.dirty = true;
  }

  markDirty(): void { this.dirty = true; }
  getRenderer(): Renderer { return this.renderer; }
  getUndoManager(): UndoManager { return this.undoManager; }

  private loop(): void {
    if (this.dirty) { this.render(); this.dirty = false; }
    this.rafId = requestAnimationFrame(() => this.loop());
  }

  private render(): void {
    const { ctx } = this;
    const w = this.canvas.width;
    const h = this.canvas.height;
    const s = this.appState.get();
    const vp = s.viewport;

    ctx.clearRect(0, 0, w, h);

    ctx.fillStyle = '#ffffff';
    ctx.fillRect(0, 0, w, h);

    // Apply rotation transform around canvas center
    ctx.save();
    if (vp.rotation !== 0) {
      ctx.translate(w / 2, h / 2);
      ctx.rotate(vp.rotation);
      ctx.translate(-w / 2, -h / 2);
    }

    this.renderer.drawLayers(
      ctx, s.project.layers, vp, s.currentFrame,
      s.onionSkin, s.onionSkinSettings, w, h,
    );
    drawCameraFrame(ctx, s.project.cameraTrack, s.currentFrame, vp);

    // Rotation indicator crosshair (shown while rotating)
    if (this.isRotating || this.isRKeyDown) {
      ctx.strokeStyle = 'rgba(74,144,226,0.5)';
      ctx.lineWidth = 1;
      ctx.setLineDash([4, 4]);
      ctx.beginPath();
      ctx.moveTo(w / 2 - 20, h / 2);
      ctx.lineTo(w / 2 + 20, h / 2);
      ctx.moveTo(w / 2, h / 2 - 20);
      ctx.lineTo(w / 2, h / 2 + 20);
      ctx.stroke();
      ctx.setLineDash([]);
    }

    ctx.restore();
  }

  private attachEvents(): void {
    const c = this.canvas;
    c.addEventListener('pointerdown', this.onPointerDown);
    c.addEventListener('pointermove', this.onPointerMove);
    c.addEventListener('pointerup', this.onPointerUp);
    c.addEventListener('pointerleave', this.onPointerLeave);
    c.addEventListener('wheel', this.onWheel, { passive: false });
    c.addEventListener('auxclick', (e) => e.preventDefault());
    window.addEventListener('keydown', this.onKeyDown);
    window.addEventListener('keyup', this.onKeyUp);
  }

  /** Convert pointer screen coords → world coords, accounting for canvas rotation. */
  private pointerToWorld(offsetX: number, offsetY: number): ReturnType<typeof screenToWorld> {
    const vp = this.appState.get().viewport;
    const cx = this.canvas.width / 2;
    const cy = this.canvas.height / 2;
    const unrotated = unrotatePoint(offsetX, offsetY, cx, cy, vp.rotation);
    return screenToWorld(unrotated, vp);
  }

  private getActiveSettings(s: AppStateData): BrushSettings {
    return s.selectedTool === 'eraser' ? s.eraser : s.brush;
  }

  private getLayerForPainting() {
    const s = this.appState.get();
    const layer = s.project.layers.find(l => l.id === s.selectedLayerId);
    if (!layer || layer.locked) return null;
    const frameData = layer.frames.find(f => f.frame === s.currentFrame);
    if (frameData != null && frameData.holdSource !== null) return null; // hold frame — release koma first
    return layer;
  }

  private startPan(clientX: number, clientY: number): void {
    const s = this.appState.get();
    this.panStart = { x: clientX, y: clientY };
    this.panStartOffset = { x: s.viewport.offsetX, y: s.viewport.offsetY };
    this.canvas.style.cursor = 'grabbing';
    this.brushCursor.style.display = 'none';
  }

  private onPointerDown = (e: PointerEvent): void => {
    const s = this.appState.get();
    const tool = s.selectedTool;

    // Middle mouse button → pan
    if (e.button === 1) {
      e.preventDefault();
      this.canvas.setPointerCapture(e.pointerId);
      this.isMmbPanning = true;
      this.startPan(e.clientX, e.clientY);
      return;
    }

    this.canvas.setPointerCapture(e.pointerId);

    // R key held → rotate
    if (this.isRKeyDown) {
      this.isRotating = true;
      const cx = this.canvas.width / 2;
      const cy = this.canvas.height / 2;
      this.rotateStartAngle = Math.atan2(e.offsetY - cy, e.offsetX - cx);
      this.rotateStartVPAngle = s.viewport.rotation;
      this.dirty = true;
      return;
    }

    // Space / hand tool → pan
    if (this.isSpaceDown || tool === 'hand') {
      this.isPanning = true;
      this.startPan(e.clientX, e.clientY);
      return;
    }

    if (tool === 'zoom') {
      const cx = this.canvas.width / 2;
      const cy = this.canvas.height / 2;
      const unrotated = unrotatePoint(e.offsetX, e.offsetY, cx, cy, s.viewport.rotation);
      const factor = e.altKey ? 1 / 1.3 : 1.3;
      this.appState.update({ viewport: zoomAround(s.viewport, unrotated, factor) });
      return;
    }

    if (tool === 'fill') {
      const layer = this.getLayerForPainting();
      if (!layer) return;
      const world = this.pointerToWorld(e.offsetX, e.offsetY);
      this.undoManager.snapshot(layer.id, s.currentFrame);
      this.renderer.floodFillTiles(layer.id, s.currentFrame, world.x, world.y, s.brush.color, s.brush.opacity);
      this.appState.markFrameDrawn(layer.id, s.currentFrame);
      this.dirty = true;
      return;
    }

    if (tool === 'brush' || tool === 'eraser') {
      const layer = this.getLayerForPainting();
      if (!layer) return;
      this.isPainting = true;
      const world = this.pointerToWorld(e.offsetX, e.offsetY);
      const settings = this.getActiveSettings(s);
      const isEraser = tool === 'eraser';
      this.undoManager.snapshot(layer.id, s.currentFrame);
      this.renderer.ensureBaseTiles(layer.id, s.currentFrame);
      const lId = layer.id, frm = s.currentFrame;
      const drawFn = (from: Vec2, to: Vec2) =>
        this.renderer.drawSegmentToTiles(lId, frm, from, to, settings, isEraser);
      this.brushEngine.beginStroke(world, settings, isEraser, drawFn);
      this.appState.markFrameDrawn(layer.id, s.currentFrame);
      this.dirty = true;
      return;
    }

    if (tool === 'camera') {
      this.startCameraDrag(e);
    }
  };

  private onPointerMove = (e: PointerEvent): void => {
    const s = this.appState.get();

    this.updateBrushCursor(e, s);

    if (this.isMmbPanning || this.isPanning) {
      const dx = e.clientX - this.panStart.x;
      const dy = e.clientY - this.panStart.y;
      this.appState.update({
        viewport: { ...s.viewport, offsetX: this.panStartOffset.x + dx, offsetY: this.panStartOffset.y + dy },
      });
      return;
    }

    if (this.isRotating) {
      const cx = this.canvas.width / 2;
      const cy = this.canvas.height / 2;
      const angle = Math.atan2(e.offsetY - cy, e.offsetX - cx);
      const delta = angle - this.rotateStartAngle;
      this.appState.update({ viewport: { ...s.viewport, rotation: this.rotateStartVPAngle + delta } });
      return;
    }

    if (this.isPainting) {
      if (!this.getLayerForPainting()) return;
      const world = this.pointerToWorld(e.offsetX, e.offsetY);
      this.brushEngine.continueStroke(world);
      this.dirty = true;
    }
  };

  private onPointerUp = (e: PointerEvent): void => {
    if (e.button === 1 || this.isMmbPanning) {
      this.isMmbPanning = false;
      this.canvas.style.cursor = '';
      this.updateCursor();
      return;
    }
    if (this.isRotating) {
      this.isRotating = false;
      this.dirty = true;
      this.updateCursor();
      return;
    }
    if (this.isPainting) { this.brushEngine.endStroke(); this.isPainting = false; }
    if (this.isPanning) { this.isPanning = false; this.canvas.style.cursor = ''; this.updateCursor(); }
  };

  private onPointerLeave = (_e: PointerEvent): void => {
    this.brushCursor.style.display = 'none';
    if (this.isPainting) { this.brushEngine.endStroke(); this.isPainting = false; }
    if (this.isPanning) { this.isPanning = false; this.canvas.style.cursor = ''; this.updateCursor(); }
    if (this.isMmbPanning) { this.isMmbPanning = false; this.canvas.style.cursor = ''; this.updateCursor(); }
  };

  private onWheel = (e: WheelEvent): void => {
    e.preventDefault();
    const s = this.appState.get();
    const cx = this.canvas.width / 2;
    const cy = this.canvas.height / 2;
    const pivot = unrotatePoint(e.offsetX, e.offsetY, cx, cy, s.viewport.rotation);

    if (e.ctrlKey || e.metaKey) {
      const factor = e.deltaY < 0 ? 1.1 : 1 / 1.1;
      this.appState.update({ viewport: zoomAround(s.viewport, pivot, factor) });
    } else if (e.shiftKey) {
      this.appState.update({ viewport: { ...s.viewport, offsetX: s.viewport.offsetX - e.deltaY } });
    } else {
      this.appState.update({
        viewport: { ...s.viewport, offsetX: s.viewport.offsetX - e.deltaX, offsetY: s.viewport.offsetY - e.deltaY },
      });
    }
  };

  private onKeyDown = (e: KeyboardEvent): void => {
    if ((e.target as HTMLElement).tagName === 'INPUT') return;
    if (e.code === 'Space') {
      e.preventDefault();
      this.isSpaceDown = true;
      if (!this.isPainting && !this.isRKeyDown) {
        this.canvas.style.cursor = 'grab';
        this.brushCursor.style.display = 'none';
      }
    }
    if ((e.key === 'r' || e.key === 'R') && !e.ctrlKey && !e.metaKey) {
      e.preventDefault();
      this.isRKeyDown = true;
      this.canvas.style.cursor = 'crosshair';
      this.brushCursor.style.display = 'none';
      this.dirty = true;
    }
    if (e.ctrlKey && e.key === '0') {
      const s = this.appState.get();
      const vp = fitViewport(this.canvas.width, this.canvas.height, s.project.settings.width, s.project.settings.height);
      this.appState.update({ viewport: vp });
    }
    if (e.ctrlKey && e.key === '1') {
      const s = this.appState.get();
      this.appState.update({ viewport: { ...s.viewport, zoom: 1 } });
    }
    // Reset rotation
    if (e.ctrlKey && e.key === 'r') {
      e.preventDefault();
      const s = this.appState.get();
      this.appState.update({ viewport: { ...s.viewport, rotation: 0 } });
    }
  };

  private onKeyUp = (e: KeyboardEvent): void => {
    if (e.code === 'Space') {
      this.isSpaceDown = false;
      this.canvas.style.cursor = '';
      this.updateCursor();
    }
    if (e.key === 'r' || e.key === 'R') {
      this.isRKeyDown = false;
      this.isRotating = false;
      this.dirty = true;
      this.updateCursor();
    }
  };

  private updateBrushCursor(e: PointerEvent, s: AppStateData): void {
    const tool = s.selectedTool;
    if ((tool === 'brush' || tool === 'eraser') && !this.isSpaceDown && !this.isPanning && !this.isMmbPanning && !this.isRKeyDown) {
      const settings = this.getActiveSettings(s);
      const size = settings.size * s.viewport.zoom;
      this.brushCursor.style.display = 'block';
      this.brushCursor.style.left = e.offsetX + 'px';
      this.brushCursor.style.top = e.offsetY + 'px';
      this.brushCursor.style.width = size + 'px';
      this.brushCursor.style.height = size + 'px';
    } else {
      this.brushCursor.style.display = 'none';
    }
  }

  private updateCursor(): void {
    const tool = this.appState.get().selectedTool;
    if (this.isRKeyDown) { this.canvas.style.cursor = 'crosshair'; return; }
    if (tool === 'brush' || tool === 'eraser') this.canvas.style.cursor = 'none';
    else if (tool === 'hand') { this.canvas.style.cursor = 'grab'; this.brushCursor.style.display = 'none'; }
    else if (tool === 'zoom') { this.canvas.style.cursor = 'zoom-in'; this.brushCursor.style.display = 'none'; }
    else { this.canvas.style.cursor = 'crosshair'; this.brushCursor.style.display = 'none'; }
  }

  exportCurrentFrame(): ImageData | null {
    const s = this.appState.get();
    const cam = s.project.cameraTrack.keyframes[0];
    if (!cam) return null;
    const oc = new OffscreenCanvas(cam.width, cam.height);
    const octx = oc.getContext('2d')!;
    octx.fillStyle = '#ffffff';
    octx.fillRect(0, 0, cam.width, cam.height);
    for (const layer of s.project.layers) {
      if (!layer.visible) continue;
      const src = resolveLayerFrame(layer, s.currentFrame);
      if (src === null) continue;
      this.renderer.drawRegionToCanvas(layer.id, src, octx, cam.x, cam.y, cam.width, cam.height);
    }
    return octx.getImageData(0, 0, cam.width, cam.height);
  }

  private cameraDragStart: { mx: number; my: number; cx: number; cy: number } | null = null;

  private startCameraDrag(e: PointerEvent): void {
    const s = this.appState.get();
    const kf = s.project.cameraTrack.keyframes[0];
    if (!kf) return;
    this.cameraDragStart = { mx: e.clientX, my: e.clientY, cx: kf.x, cy: kf.y };
    const up = () => {
      this.cameraDragStart = null;
      window.removeEventListener('pointerup', up);
      window.removeEventListener('pointermove', move);
    };
    const move = (ev: PointerEvent) => {
      if (!this.cameraDragStart) return;
      const s2 = this.appState.get();
      const rot = -(s2.viewport.rotation);
      const rawDx = ev.clientX - this.cameraDragStart.mx;
      const rawDy = ev.clientY - this.cameraDragStart.my;
      const cos = Math.cos(rot), sin = Math.sin(rot);
      const dx = (rawDx * cos - rawDy * sin) / s2.viewport.zoom;
      const dy = (rawDx * sin + rawDy * cos) / s2.viewport.zoom;
      const kfs = s2.project.cameraTrack.keyframes.map((k, i) =>
        i === 0 ? { ...k, x: this.cameraDragStart!.cx + dx, y: this.cameraDragStart!.cy + dy } : k
      );
      this.appState.update({ project: { ...s2.project, cameraTrack: { ...s2.project.cameraTrack, keyframes: kfs } } });
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  }
}

