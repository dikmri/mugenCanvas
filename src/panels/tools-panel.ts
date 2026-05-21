import { FloatingPanel } from './floating-panel.ts';
import { AppState } from '../app/app-state.ts';
import type { ToolType } from '../types/model.ts';

const TOOLS: { id: ToolType; title: string; key: string; icon: string }[] = [
  { id: 'select',  title: '選択 (V)',     key: 'V', icon: selectIcon() },
  { id: 'move',    title: '移動 (M)',     key: 'M', icon: moveIcon() },
  { id: 'brush',   title: 'ブラシ (B)',   key: 'B', icon: brushIcon() },
  { id: 'eraser',  title: '消しゴム (E)', key: 'E', icon: eraserIcon() },
  { id: 'fill',    title: '塗りつぶし',   key: 'F', icon: fillIcon() },
  { id: 'shape',   title: '図形',         key: '', icon: shapeIcon() },
  { id: 'text',    title: 'テキスト',     key: 'T', icon: textIcon() },
  { id: 'hand',    title: '手のひら (H)', key: 'H', icon: handIcon() },
  { id: 'zoom',    title: 'ズーム (Z)',   key: 'Z', icon: zoomIcon() },
  { id: 'camera',  title: 'カメラ (C)',   key: 'C', icon: cameraIcon() },
];

export class ToolsPanel {
  private panel: FloatingPanel;
  private buttons = new Map<ToolType, HTMLButtonElement>();
  private appState: AppState;

  constructor(container: HTMLElement, appState: AppState) {
    this.appState = appState;
    this.panel = new FloatingPanel(container, {
      id: 'tools',
      title: 'ツール',
      x: 40,
      y: 120,
      width: 72,
      extraClass: 'tools-panel',
    });

    this.buildUI();
    appState.subscribe(s => this.update(s.selectedTool));

    window.addEventListener('keydown', this.onKeyDown);
  }

  private buildUI(): void {
    const body = this.panel.body;

    TOOLS.forEach(t => {
      const btn = document.createElement('button');
      btn.className = 'tool-btn';
      btn.title = t.title;
      btn.innerHTML = t.icon;
      btn.addEventListener('click', () => this.appState.setTool(t.id));
      this.buttons.set(t.id, btn);
      body.appendChild(btn);
    });

    // Foreground/background color
    const colors = document.createElement('div');
    colors.className = 'tool-colors';
    const bg = document.createElement('div');
    bg.className = 'tool-color-bg';
    const fg = document.createElement('div');
    fg.className = 'tool-color-fg';
    fg.title = '前景色';

    fg.addEventListener('click', () => {
      const input = document.createElement('input');
      input.type = 'color';
      input.value = this.appState.get().brush.color;
      input.style.cssText = 'position:fixed;opacity:0;width:0;height:0;';
      document.body.appendChild(input);
      input.click();
      input.addEventListener('input', () => {
        this.appState.setBrush({ color: input.value });
        fg.style.background = input.value;
      });
      input.addEventListener('change', () => input.remove());
    });

    colors.append(bg, fg);
    body.appendChild(colors);

    this.appState.subscribe(s => { fg.style.background = s.brush.color; });
    this.update(this.appState.get().selectedTool);
  }

  private update(active: ToolType): void {
    this.buttons.forEach((btn, id) => btn.classList.toggle('active', id === active));
  }

  private onKeyDown = (e: KeyboardEvent): void => {
    if ((e.target as HTMLElement).tagName === 'INPUT') return;
    if (e.ctrlKey || e.metaKey) return;
    const map: Record<string, ToolType> = { b: 'brush', e: 'eraser', v: 'select', h: 'hand', z: 'zoom', c: 'camera', t: 'text' };
    const tool = map[e.key.toLowerCase()];
    if (tool) this.appState.setTool(tool);
  };
}

function svgWrap(content: string): string {
  return `<svg width="18" height="18" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="#2f343a" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">${content}</svg>`;
}
function selectIcon(): string { return svgWrap('<path d="M5 3l13 9-7 1-3 7z"/>'); }
function moveIcon(): string { return svgWrap('<path d="M10 2v16M2 10h16M5 5l-3 5 3 5M15 5l3 5-3 5M5 5L10 2l5 3M5 15l5 3 5-3"/>'); }
function brushIcon(): string { return svgWrap('<path d="M3 17l3-7 9-9 4 4-9 9z"/><circle cx="4" cy="16" r="1.5" fill="#2f343a"/>'); }
function eraserIcon(): string { return svgWrap('<path d="M4 16l5-9 7 7-5 2H4z"/><path d="M2 18h16"/>'); }
function fillIcon(): string { return svgWrap('<path d="M4 16c0-4 6-11 6-11s6 7 6 11a6 6 0 01-12 0z"/>'); }
function shapeIcon(): string { return svgWrap('<rect x="3" y="3" width="14" height="14" rx="2"/>'); }
function textIcon(): string { return svgWrap('<path d="M4 5h12M10 5v12M7 17h6"/>'); }
function handIcon(): string { return svgWrap('<path d="M8 7V3M12 6V2M16 8V5M4 11v-3a2 2 0 014 0v2M8 10a2 2 0 014 0v1M12 11a2 2 0 014 0v3c0 3-2 4-4 4H8c-2 0-4-2-4-5v-2"/>'); }
function zoomIcon(): string { return svgWrap('<circle cx="9" cy="9" r="6"/><path d="M15 15l4 4M7 9h4M9 7v4"/>'); }
function cameraIcon(): string { return svgWrap('<rect x="2" y="5" width="16" height="12" rx="2"/><circle cx="10" cy="11" r="3"/>'); }
