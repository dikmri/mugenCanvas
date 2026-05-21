import { FloatingPanel } from './floating-panel.ts';
import { AppState } from '../app/app-state.ts';
import type { BrushSettings } from '../types/model.ts';

export class BrushColorPanel {
  private panel: FloatingPanel;
  private hue = 0;
  private sat = 0.7;
  private val = 0.8;
  private sqCanvas!: HTMLCanvasElement;
  private hueCanvas!: HTMLCanvasElement;
  private sqCursor!: HTMLDivElement;
  private hueCursor!: HTMLDivElement;
  private hexInput!: HTMLInputElement;
  private hexPreview!: HTMLDivElement;
  private colorSection!: HTMLDivElement;
  private sizeSlider!: HTMLInputElement;
  private sizeVal!: HTMLSpanElement;
  private opacitySlider!: HTMLInputElement;
  private opacityVal!: HTMLSpanElement;
  private sectionLabel!: HTMLDivElement;
  private appState: AppState;
  private activeMode: 'brush' | 'eraser' = 'brush';

  constructor(container: HTMLElement, appState: AppState) {
    this.appState = appState;
    const iw = window.innerWidth;
    this.panel = new FloatingPanel(container, {
      id: 'brush-color',
      title: 'ブラシ・色',
      x: iw - 320,
      y: 120,
      width: 280,
      height: 460,
      extraClass: 'brush-panel',
    });
    this.buildUI();
    this.syncFromState();
    appState.subscribe(() => this.syncFromState());
  }

  private buildUI(): void {
    const body = this.panel.body;

    // ── Color section (brush only) ──
    this.colorSection = document.createElement('div');

    const sqLabel = document.createElement('div');
    sqLabel.className = 'bp-section-label';
    sqLabel.textContent = 'カラー';

    const pickerArea = document.createElement('div');
    pickerArea.className = 'color-picker-area';

    const sqWrap = document.createElement('div');
    sqWrap.className = 'color-sq';
    this.sqCanvas = document.createElement('canvas');
    this.sqCanvas.width = 200; this.sqCanvas.height = 160;
    this.sqCursor = document.createElement('div');
    this.sqCursor.className = 'color-sq-cursor';
    sqWrap.append(this.sqCanvas, this.sqCursor);
    this.sqCanvas.addEventListener('pointerdown', this.onSqDown);

    const hueWrap = document.createElement('div');
    hueWrap.className = 'hue-strip';
    this.hueCanvas = document.createElement('canvas');
    this.hueCanvas.width = 20; this.hueCanvas.height = 160;
    this.hueCursor = document.createElement('div');
    this.hueCursor.className = 'hue-cursor';
    hueWrap.append(this.hueCanvas, this.hueCursor);
    this.hueCanvas.addEventListener('pointerdown', this.onHueDown);

    pickerArea.append(sqWrap, hueWrap);

    const hexRow = document.createElement('div');
    hexRow.className = 'hex-row';
    this.hexPreview = document.createElement('div');
    this.hexPreview.className = 'hex-preview';
    this.hexInput = document.createElement('input');
    this.hexInput.className = 'hex-input';
    this.hexInput.maxLength = 7;
    this.hexInput.addEventListener('change', () => {
      const v = this.hexInput.value;
      if (/^#[0-9a-fA-F]{6}$/.test(v)) this.appState.setBrush({ color: v });
    });
    hexRow.append(this.hexPreview, this.hexInput);

    this.colorSection.append(sqLabel, pickerArea, hexRow);

    // ── Size + opacity ──
    const brushSection = document.createElement('div');
    this.sectionLabel = document.createElement('div');
    this.sectionLabel.className = 'bp-section-label';

    this.sizeSlider = this.makeSlider('サイズ', 1, 200, 1);
    this.sizeVal = this.sizeSlider.parentElement!.querySelector('.slider-value')!;
    this.sizeSlider.addEventListener('input', () => {
      const v = parseInt(this.sizeSlider.value);
      this.sizeVal.textContent = v + ' px';
      if (this.activeMode === 'eraser') this.appState.setEraser({ size: v });
      else this.appState.setBrush({ size: v });
    });

    this.opacitySlider = this.makeSlider('不透明度', 0, 100, 1);
    this.opacityVal = this.opacitySlider.parentElement!.querySelector('.slider-value')!;
    this.opacitySlider.addEventListener('input', () => {
      const v = parseInt(this.opacitySlider.value);
      this.opacityVal.textContent = v + '%';
      if (this.activeMode === 'eraser') this.appState.setEraser({ opacity: v / 100 });
      else this.appState.setBrush({ opacity: v / 100 });
    });

    brushSection.append(this.sectionLabel, this.sizeSlider.parentElement!, this.opacitySlider.parentElement!);
    body.append(this.colorSection, brushSection);

    this.drawHue();
    this.drawSq();
  }

  private makeSlider(label: string, min: number, max: number, step: number): HTMLInputElement {
    const wrap = document.createElement('div');
    wrap.className = 'brush-slider-row';
    const row = document.createElement('div');
    row.className = 'slider-label-row';
    const lbl = document.createElement('span');
    lbl.className = 'slider-label';
    lbl.textContent = label;
    const val = document.createElement('span');
    val.className = 'slider-value';
    row.append(lbl, val);
    const slider = document.createElement('input');
    slider.type = 'range';
    slider.className = 'brush-slider';
    slider.min = String(min); slider.max = String(max); slider.step = String(step);
    wrap.append(row, slider);
    return slider;
  }

  private syncFromState(): void {
    const s = this.appState.get();
    const isEraser = s.selectedTool === 'eraser';
    this.activeMode = isEraser ? 'eraser' : 'brush';

    // Toggle color section visibility
    this.colorSection.style.display = isEraser ? 'none' : '';
    this.sectionLabel.textContent = isEraser ? '消しゴム設定' : 'ブラシ設定';

    const settings: BrushSettings = isEraser ? s.eraser : s.brush;

    if (!isEraser) {
      const { h, s: sv, v } = hexToHsv(settings.color);
      this.hue = h; this.sat = sv; this.val = v;
      this.drawSq();
      this.updateCursors();
      this.hexInput.value = settings.color;
      this.hexPreview.style.background = settings.color;
    }

    this.sizeSlider.value = String(settings.size);
    this.sizeVal.textContent = settings.size + ' px';
    this.opacitySlider.value = String(Math.round(settings.opacity * 100));
    this.opacityVal.textContent = Math.round(settings.opacity * 100) + '%';
  }

  private drawHue(): void {
    const ctx = this.hueCanvas.getContext('2d')!;
    const h = this.hueCanvas.height;
    const grad = ctx.createLinearGradient(0, 0, 0, h);
    for (let i = 0; i <= 12; i++) grad.addColorStop(i / 12, `hsl(${i / 12 * 360},100%,50%)`);
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, this.hueCanvas.width, h);
  }

  private drawSq(): void {
    const ctx = this.sqCanvas.getContext('2d')!;
    const w = this.sqCanvas.width, h = this.sqCanvas.height;
    ctx.fillStyle = `hsl(${this.hue * 360},100%,50%)`;
    ctx.fillRect(0, 0, w, h);
    const wGrad = ctx.createLinearGradient(0, 0, w, 0);
    wGrad.addColorStop(0, 'rgba(255,255,255,1)');
    wGrad.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.fillStyle = wGrad; ctx.fillRect(0, 0, w, h);
    const bGrad = ctx.createLinearGradient(0, 0, 0, h);
    bGrad.addColorStop(0, 'rgba(0,0,0,0)');
    bGrad.addColorStop(1, 'rgba(0,0,0,1)');
    ctx.fillStyle = bGrad; ctx.fillRect(0, 0, w, h);
  }

  private updateCursors(): void {
    const sqW = this.sqCanvas.offsetWidth || this.sqCanvas.width;
    const sqH = this.sqCanvas.offsetHeight || this.sqCanvas.height;
    this.sqCursor.style.left = (this.sat * sqW) + 'px';
    this.sqCursor.style.top = ((1 - this.val) * sqH) + 'px';
    const hueH = this.hueCanvas.offsetHeight || this.hueCanvas.height;
    this.hueCursor.style.top = (this.hue * hueH - 2) + 'px';
  }

  private pickSq(e: PointerEvent): void {
    const rect = this.sqCanvas.getBoundingClientRect();
    const sx = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    const sy = Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height));
    this.sat = sx; this.val = 1 - sy;
    const color = hsvToHex(this.hue, this.sat, this.val);
    this.appState.setBrush({ color });
    this.sqCursor.style.left = (sx * rect.width) + 'px';
    this.sqCursor.style.top = (sy * rect.height) + 'px';
  }

  private pickHue(e: PointerEvent): void {
    const rect = this.hueCanvas.getBoundingClientRect();
    this.hue = Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height));
    this.drawSq();
    const color = hsvToHex(this.hue, this.sat, this.val);
    this.appState.setBrush({ color });
    this.hueCursor.style.top = (this.hue * rect.height - 2) + 'px';
  }

  private onSqDown = (e: PointerEvent): void => {
    this.sqCanvas.setPointerCapture(e.pointerId);
    this.pickSq(e);
    const move = (ev: PointerEvent) => this.pickSq(ev);
    const up = () => { window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', up); };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  };

  private onHueDown = (e: PointerEvent): void => {
    this.hueCanvas.setPointerCapture(e.pointerId);
    this.pickHue(e);
    const move = (ev: PointerEvent) => this.pickHue(ev);
    const up = () => { window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', up); };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  };
}

function hsvToHex(h: number, s: number, v: number): string {
  const i = Math.floor(h * 6);
  const f = h * 6 - i;
  const p = v * (1 - s), q = v * (1 - f * s), t = v * (1 - (1 - f) * s);
  let r = 0, g = 0, b = 0;
  switch (i % 6) {
    case 0: r = v; g = t; b = p; break; case 1: r = q; g = v; b = p; break;
    case 2: r = p; g = v; b = t; break; case 3: r = p; g = q; b = v; break;
    case 4: r = t; g = p; b = v; break; case 5: r = v; g = p; b = q; break;
  }
  return '#' + [r, g, b].map(c => Math.round(c * 255).toString(16).padStart(2, '0')).join('');
}

function hexToHsv(hex: string): { h: number; s: number; v: number } {
  const h = hex.replace('#', '');
  const r = parseInt(h.substring(0, 2), 16) / 255;
  const g = parseInt(h.substring(2, 4), 16) / 255;
  const b = parseInt(h.substring(4, 6), 16) / 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  const d = max - min;
  let hue = 0;
  if (d !== 0) {
    if (max === r) hue = ((g - b) / d + 6) % 6;
    else if (max === g) hue = (b - r) / d + 2;
    else hue = (r - g) / d + 4;
    hue /= 6;
  }
  return { h: hue, s: max === 0 ? 0 : d / max, v: max };
}
