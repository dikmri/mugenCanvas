export type PanelState = 'floating' | 'pinned' | 'collapsed' | 'hidden';

export type ToolType =
  | 'select' | 'move' | 'brush' | 'eraser'
  | 'fill' | 'shape' | 'text' | 'hand' | 'zoom' | 'camera';

export interface Vec2 { x: number; y: number; }

export interface Viewport {
  offsetX: number;
  offsetY: number;
  zoom: number;
  rotation: number; // radians
}

export interface OnionSkinSettings {
  prevColor: string;
  prevAlpha: number;
  nextColor: string;
  nextAlpha: number;
}

export interface FloatingPanel {
  id: string;
  title: string;
  x: number;
  y: number;
  width: number;
  height: number;
  state: PanelState;
  zIndex: number;
}

export interface ProjectSettings {
  width: number;
  height: number;
  fps: number;
  totalFrames: number;
}

export interface CameraKeyframe {
  frame: number;
  x: number;
  y: number;
  scale: number;
  rotation: number;
  width: number;
  height: number;
}

export interface CameraTrack {
  id: 'camera';
  name: string;
  visible: boolean;
  locked: boolean;
  keyframes: CameraKeyframe[];
}

export interface AnimationFrame {
  frame: number;
  drawingId: string | null;
  holdSource: number | null; // non-null = this frame holds the drawing from holdSource frame number
  isKeyframe: boolean;
}

export interface AnimationLayer {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  frames: AnimationFrame[];
}

export interface BrushSettings {
  size: number;
  opacity: number;
  color: string;
  antiAlias: boolean;
  minSize: number;
}

export interface ProjectFile {
  app: 'mugenCanvas';
  version: string;
  settings: ProjectSettings;
  viewport: Viewport;
  cameraTrack: CameraTrack;
  layers: AnimationLayer[];
}

export interface AppStateData {
  project: ProjectFile;
  currentFrame: number;
  selectedLayerId: string;
  selectedTool: ToolType;
  viewport: Viewport;
  brush: BrushSettings;
  eraser: BrushSettings;
  isPlaying: boolean;
  onionSkin: boolean;
  onionSkinSettings: OnionSkinSettings;
  theme: 'system' | 'light' | 'dark';
}
