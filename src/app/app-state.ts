import type {
  AppStateData, AnimationLayer, BrushSettings, CameraTrack,
  OnionSkinSettings, ProjectFile, ProjectSettings, ToolType, Viewport,
} from '../types/model.ts';

function makeDefaultProject(): ProjectFile {
  const settings: ProjectSettings = { width: 1920, height: 1080, fps: 24, totalFrames: 60 };
  const layer1: AnimationLayer = {
    id: 'layer-bg',
    name: '背景',
    visible: true,
    locked: false,
    frames: Array.from({ length: 60 }, (_, i) => ({ frame: i + 1, drawingId: null, holdSource: null, isKeyframe: i === 0 })),
  };
  const layer2: AnimationLayer = {
    id: 'layer-1',
    name: 'レイヤー 1',
    visible: true,
    locked: false,
    frames: Array.from({ length: 60 }, (_, i) => ({ frame: i + 1, drawingId: null, holdSource: null, isKeyframe: i === 0 })),
  };
  const cameraTrack: CameraTrack = {
    id: 'camera',
    name: 'カメラ',
    visible: true,
    locked: false,
    keyframes: [{ frame: 1, x: 0, y: 0, scale: 1, rotation: 0, width: 1920, height: 1080 }],
  };
  const viewport: Viewport = { offsetX: 0, offsetY: 0, zoom: 1, rotation: 0 };
  return { app: 'mugenCanvas', version: '0.1.0', settings, viewport, cameraTrack, layers: [layer1, layer2] };
}

function makeDefaultBrush(): BrushSettings {
  return { size: 15, opacity: 1, color: '#222222', antiAlias: true, minSize: 0.2 };
}

function makeDefaultEraser(): BrushSettings {
  return { size: 30, opacity: 1, color: '#000000', antiAlias: true, minSize: 1 };
}

function makeDefaultOnionSkin(): OnionSkinSettings {
  return { prevColor: '#4a90e2', prevAlpha: 0.5, nextColor: '#ff4b4b', nextAlpha: 0.4 };
}

type Listener = (state: AppStateData) => void;

export class AppState {
  private static instance: AppState;
  private _state: AppStateData;
  private listeners = new Set<Listener>();

  private constructor() {
    const project = makeDefaultProject();
    this._state = {
      project,
      currentFrame: 1,
      selectedLayerId: project.layers[1].id,
      selectedTool: 'brush',
      viewport: { ...project.viewport },
      brush: makeDefaultBrush(),
      eraser: makeDefaultEraser(),
      isPlaying: false,
      onionSkin: false,
      onionSkinSettings: makeDefaultOnionSkin(),
      theme: 'system',
    };
  }

  static getInstance(): AppState {
    if (!AppState.instance) AppState.instance = new AppState();
    return AppState.instance;
  }

  get(): AppStateData { return this._state; }

  update(partial: Partial<AppStateData>): void {
    this._state = { ...this._state, ...partial };
    this.notify();
  }

  setTool(tool: ToolType): void { this.update({ selectedTool: tool }); }

  setBrush(partial: Partial<BrushSettings>): void {
    this.update({ brush: { ...this._state.brush, ...partial } });
  }

  setEraser(partial: Partial<BrushSettings>): void {
    this.update({ eraser: { ...this._state.eraser, ...partial } });
  }

  setFrame(frame: number): void {
    const total = this._state.project.settings.totalFrames;
    this.update({ currentFrame: Math.max(1, Math.min(total, frame)) });
  }

  addLayer(): void {
    const { project } = this._state;
    const id = `layer-${Date.now()}`;
    const count = project.layers.length + 1;
    const newLayer: AnimationLayer = {
      id, name: `レイヤー ${count}`, visible: true, locked: false,
      frames: Array.from({ length: project.settings.totalFrames }, (_, i) => ({
        frame: i + 1, drawingId: null, holdSource: null, isKeyframe: i === 0,
      })),
    };
    this.update({ project: { ...project, layers: [...project.layers, newLayer] }, selectedLayerId: id });
  }

  removeLayer(id: string): void {
    const { project } = this._state;
    if (project.layers.length <= 1) return;
    const layers = project.layers.filter(l => l.id !== id);
    const selectedId = this._state.selectedLayerId === id ? layers[layers.length - 1].id : this._state.selectedLayerId;
    this.update({ project: { ...project, layers }, selectedLayerId: selectedId });
  }

  markFrameDrawn(layerId: string, frame: number): void {
    const { project } = this._state;
    const layer = project.layers.find(l => l.id === layerId);
    if (!layer) return;
    const f = layer.frames.find(fr => fr.frame === frame);
    if (!f || f.drawingId !== null) return;
    const layers = project.layers.map(l =>
      l.id !== layerId ? l : {
        ...l,
        frames: l.frames.map(fr => fr.frame === frame ? { ...fr, drawingId: 'drawn' } : fr),
      }
    );
    this._state = { ...this._state, project: { ...project, layers } };
    this.notify();
  }

  /** Mark current frame as a koma-hold extending the nearest previous drawn frame. */
  setKomaHold(frame: number, layerId: string): void {
    const { project } = this._state;
    const layer = project.layers.find(l => l.id === layerId);
    if (!layer) return;
    // Find nearest drawn frame before 'frame'
    let sourceFrame: number | null = null;
    for (let f = frame - 1; f >= 1; f--) {
      const fr = layer.frames.find(fr => fr.frame === f);
      if (!fr) continue;
      if (fr.drawingId !== null) { sourceFrame = f; break; }
      if (fr.holdSource !== null) { sourceFrame = fr.holdSource; break; }
    }
    if (sourceFrame === null) return;
    // Set all blank frames from sourceFrame+1 to frame as holds of sourceFrame
    const frames = layer.frames.map(fr =>
      fr.frame > sourceFrame! && fr.frame <= frame && fr.drawingId === null
        ? { ...fr, holdSource: sourceFrame! }
        : fr
    );
    const layers = project.layers.map(l => l.id === layerId ? { ...l, frames } : l);
    this._state = { ...this._state, project: { ...project, layers } };
    this.notify();
  }

  /** Release a koma-hold on frame and cascade to all subsequent holds of the same source. */
  releaseKomaHold(frame: number, layerId: string): void {
    const { project } = this._state;
    const layer = project.layers.find(l => l.id === layerId);
    if (!layer) return;
    const target = layer.frames.find(f => f.frame === frame);
    if (!target || target.holdSource === null) return;
    const src = target.holdSource;
    const frames = layer.frames.map(fr =>
      fr.frame >= frame && fr.holdSource === src ? { ...fr, holdSource: null } : fr
    );
    const layers = project.layers.map(l => l.id === layerId ? { ...l, frames } : l);
    this._state = { ...this._state, project: { ...project, layers } };
    this.notify();
  }

  /** Delete the contiguous globally-blank frame range that contains 'frame', shifting later frames. */
  deleteBlankRange(frame: number): void {
    const { project } = this._state;
    const isGloballyBlank = (f: number): boolean =>
      project.layers.every(layer => {
        const fr = layer.frames.find(fr => fr.frame === f);
        return !fr || (fr.drawingId === null && fr.holdSource === null);
      });
    if (!isGloballyBlank(frame)) return;
    let start = frame;
    while (start > 1 && isGloballyBlank(start - 1)) start--;
    let end = frame;
    const total = project.settings.totalFrames;
    while (end < total && isGloballyBlank(end + 1)) end++;
    const deleteCount = end - start + 1;
    const layers = project.layers.map(layer => ({
      ...layer,
      frames: layer.frames
        .filter(f => f.frame < start || f.frame > end)
        .map(f => {
          if (f.frame < start) return f;
          const newFrame = f.frame - deleteCount;
          const holdSource = f.holdSource === null ? null
            : f.holdSource < start ? f.holdSource
            : f.holdSource > end ? f.holdSource - deleteCount
            : null;
          return { ...f, frame: newFrame, holdSource };
        }),
    }));
    const cameraKeyframes = project.cameraTrack.keyframes
      .filter(kf => kf.frame < start || kf.frame > end)
      .map(kf => kf.frame > end ? { ...kf, frame: kf.frame - deleteCount } : kf);
    const newTotal = total - deleteCount;
    let currentFrame = this._state.currentFrame;
    if (currentFrame >= start && currentFrame <= end) currentFrame = Math.max(1, start - 1);
    else if (currentFrame > end) currentFrame -= deleteCount;
    this._state = {
      ...this._state,
      currentFrame: Math.max(1, Math.min(newTotal, currentFrame)),
      project: {
        ...project,
        settings: { ...project.settings, totalFrames: newTotal },
        cameraTrack: { ...project.cameraTrack, keyframes: cameraKeyframes },
        layers,
      },
    };
    this.notify();
  }

  renameLayer(id: string, name: string): void {
    const { project } = this._state;
    const layers = project.layers.map(l => l.id === id ? { ...l, name } : l);
    this.update({ project: { ...project, layers } });
  }

  toggleLayerVisible(id: string): void {
    const { project } = this._state;
    const layers = project.layers.map(l => l.id === id ? { ...l, visible: !l.visible } : l);
    this.update({ project: { ...project, layers } });
  }

  toggleLayerLock(id: string): void {
    const { project } = this._state;
    const layers = project.layers.map(l => l.id === id ? { ...l, locked: !l.locked } : l);
    this.update({ project: { ...project, layers } });
  }

  subscribe(listener: Listener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notify(): void {
    this.listeners.forEach(l => l(this._state));
  }
}
