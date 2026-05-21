import './styles/base.css';
import './styles/layout.css';
import './styles/panels.css';

import { AppState } from './app/app-state.ts';
import { CanvasView } from './canvas/canvas-view.ts';
import { ToolsPanel } from './panels/tools-panel.ts';
import { BrushColorPanel } from './panels/brush-color-panel.ts';
import { TimelinePanel } from './panels/timeline-panel.ts';
import { buildTopBar, applyTheme } from './ui/topbar.ts';
import { buildStatusBar } from './ui/statusbar.ts';

// ── Theme initialization ──────────────────────────────────────────────────
const savedTheme = (localStorage.getItem('theme') as 'system' | 'light' | 'dark' | null) ?? 'system';
applyTheme(savedTheme);

// Follow system preference changes when theme is set to 'system'
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
  const appTheme = AppState.getInstance().get().theme;
  if (appTheme === 'system') applyTheme('system');
});

// ── App bootstrap ─────────────────────────────────────────────────────────
const appState = AppState.getInstance();
appState.update({ theme: savedTheme });

// Canvas (created first so it can be passed to topbar)
const canvasEl = document.getElementById('main-canvas') as HTMLCanvasElement;
const canvasView = new CanvasView(canvasEl, appState);

// Top bar
const topbarEl = document.getElementById('topbar')!;
buildTopBar(topbarEl, appState, canvasView);

// Status bar
const statusbarEl = document.getElementById('statusbar')!;
buildStatusBar(statusbarEl, appState);

// Panels
const panelsContainer = document.getElementById('panels-container')!;
new ToolsPanel(panelsContainer, appState);
new BrushColorPanel(panelsContainer, appState);
new TimelinePanel(panelsContainer, appState);

// Resize
window.addEventListener('resize', () => canvasView.resize());

// Global shortcuts
window.addEventListener('keydown', (e) => {
  if ((e.target as HTMLElement).tagName === 'INPUT') return;
  if (e.ctrlKey && !e.shiftKey && e.key === 'z') {
    e.preventDefault();
    canvasView.getUndoManager().undo((layerId, frame) => {
      appState.update({ selectedLayerId: layerId, currentFrame: frame });
      canvasView.markDirty();
    });
  }
  if (e.ctrlKey && (e.shiftKey && e.key === 'Z' || e.key === 'y')) {
    e.preventDefault();
    canvasView.getUndoManager().redo((layerId, frame) => {
      appState.update({ selectedLayerId: layerId, currentFrame: frame });
      canvasView.markDirty();
    });
  }
  if (e.ctrlKey && e.key === 's') {
    e.preventDefault();
    saveProject(appState, canvasView);
  }
  if (e.ctrlKey && e.key === 'o') {
    e.preventDefault();
    openProject(appState, canvasView);
  }
  if (e.ctrlKey && e.key === 'n') {
    e.preventDefault();
    if (confirm('新規プロジェクトを作成しますか？')) location.reload();
  }
});

async function saveProject(state: AppState, view: CanvasView): Promise<void> {
  const s = state.get();
  const renderer = view.getRenderer();

  // Export all drawn frames as base64 PNG tiles
  const drawings: Record<string, string> = {};
  for (const layer of s.project.layers) {
    for (const frame of layer.frames) {
      if (frame.drawingId !== null) {
        const tiles = await renderer.exportFrameTiles(layer.id, frame.frame);
        for (const [coords, dataUrl] of tiles) {
          drawings[`${layer.id}:${frame.frame}:${coords}`] = dataUrl;
        }
      }
    }
  }

  const payload = JSON.stringify({ ...s.project, drawings });

  try {
    const { invoke, dialog } = await import('./tauri-bridge.ts');
    const path = await dialog.save({ filters: [{ name: 'mugenCanvas', extensions: ['mugencanvas'] }] });
    if (!path) return;
    await invoke('save_project', { path, data: payload });
  } catch {
    const blob = new Blob([payload], { type: 'application/json' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = 'project.mugencanvas';
    a.click();
  }
}

async function openProject(state: AppState, view: CanvasView): Promise<void> {
  let jsonText: string | null = null;

  try {
    const { invoke, dialog } = await import('./tauri-bridge.ts');
    const path = await dialog.open({ filters: [{ name: 'mugenCanvas', extensions: ['mugencanvas', 'json'] }] });
    if (!path || Array.isArray(path)) return;
    jsonText = await invoke<string>('load_project', { path });
  } catch {
    jsonText = await new Promise<string | null>((resolve) => {
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = '.mugencanvas,.json';
      input.addEventListener('change', async () => {
        const file = input.files?.[0];
        resolve(file ? await file.text() : null);
      });
      input.click();
    });
  }

  if (!jsonText) return;

  const raw = JSON.parse(jsonText);
  const { drawings, ...projectData } = raw as { drawings?: Record<string, string> } & Record<string, unknown>;

  // Update state first so setCanvasSize runs with the new project dimensions
  state.update({ project: projectData as unknown as ReturnType<typeof state.get>['project'] });

  // Restore pixel data into renderer canvases
  if (drawings && typeof drawings === 'object') {
    const renderer = view.getRenderer();
    await Promise.all(
      Object.entries(drawings).map(([key, dataUrl]) => {
        // New format: "layerId:frame:tileX:tileY"
        // Parse from the right to handle layerIds that contain ':'
        const sep3 = key.lastIndexOf(':');
        const sep2 = key.lastIndexOf(':', sep3 - 1);
        const sep1 = key.lastIndexOf(':', sep2 - 1);
        const ty = parseInt(key.slice(sep3 + 1));
        const tx = parseInt(key.slice(sep2 + 1, sep3));
        const frame = parseInt(key.slice(sep1 + 1, sep2));
        const layerId = key.slice(0, sep1);
        if (!isNaN(tx) && !isNaN(ty) && !isNaN(frame) && sep1 >= 0) {
          return renderer.loadTile(layerId, frame, tx, ty, dataUrl);
        }
        // Legacy format fallback: "layerId:frame" → load into tile (0,0)
        const legacySep = key.lastIndexOf(':');
        const legacyLayerId = key.slice(0, legacySep);
        const legacyFrame = parseInt(key.slice(legacySep + 1));
        return renderer.loadTile(legacyLayerId, legacyFrame, 0, 0, dataUrl);
      })
    );
    view.markDirty();
  }
}
