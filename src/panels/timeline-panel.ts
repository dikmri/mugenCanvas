import { FloatingPanel } from './floating-panel.ts';
import { AppState } from '../app/app-state.ts';
import type { AnimationLayer } from '../types/model.ts';

const FRAME_W = 14;
const ROW_H = 28;

export class TimelinePanel {
  private panel: FloatingPanel;
  private layersList!: HTMLDivElement;
  private framesCanvas!: HTMLCanvasElement;
  private numbersCanvas!: HTMLCanvasElement;
  private framesWrap!: HTMLDivElement;
  private numWrap!: HTMLDivElement;
  private playheadEl!: HTMLDivElement;
  private frameInfo!: HTMLSpanElement;
  private onionBtn!: HTMLButtonElement;
  private appState: AppState;
  private isRenaming = false;

  constructor(container: HTMLElement, appState: AppState) {
    this.appState = appState;
    const ih = window.innerHeight;
    this.panel = new FloatingPanel(container, {
      id: 'timeline',
      title: 'タイムライン',
      x: 240,
      y: ih - 260,
      width: Math.min(960, window.innerWidth - 280),
      height: 220,
      extraClass: 'timeline-panel',
    });
    this.buildUI();
    appState.subscribe(() => this.render());
    this.render();
  }

  private buildUI(): void {
    const body = this.panel.body;
    body.style.display = 'flex';
    body.style.flexDirection = 'column';
    body.style.overflow = 'hidden';

    // Toolbar
    const toolbar = document.createElement('div');
    toolbar.className = 'timeline-toolbar';

    const addLayerBtn = this.makeTlBtn('+', 'レイヤー追加');
    addLayerBtn.addEventListener('click', () => this.appState.addLayer());

    const prevBtn = this.makeTlBtn('◀', '前フレーム (,)');
    prevBtn.addEventListener('click', () => this.appState.setFrame(this.appState.get().currentFrame - 1));

    const playBtn = this.makeTlBtn('▶', '再生 (Enter)');
    playBtn.addEventListener('click', () => {
      const s = this.appState.get();
      this.appState.update({ isPlaying: !s.isPlaying });
      playBtn.textContent = this.appState.get().isPlaying ? '⏸' : '▶';
    });
    this.appState.subscribe(s => { playBtn.textContent = s.isPlaying ? '⏸' : '▶'; });

    const nextBtn = this.makeTlBtn('▶|', '次フレーム (.)');
    nextBtn.addEventListener('click', () => this.appState.setFrame(this.appState.get().currentFrame + 1));

    this.onionBtn = this.makeTlBtn('◎', 'オニオンスキン');
    this.onionBtn.addEventListener('click', () => {
      const s = this.appState.get();
      this.appState.update({ onionSkin: !s.onionSkin });
      this.onionBtn.classList.toggle('active', this.appState.get().onionSkin);
    });

    this.frameInfo = document.createElement('span');
    this.frameInfo.className = 'tl-frame-info';

    toolbar.append(addLayerBtn, prevBtn, playBtn, nextBtn, this.onionBtn, this.frameInfo);
    body.appendChild(toolbar);

    // Main area
    const main = document.createElement('div');
    main.className = 'timeline-main';

    // Layer column
    const layerCol = document.createElement('div');
    layerCol.className = 'tl-layer-col';
    const layerHdr = document.createElement('div');
    layerHdr.className = 'tl-layer-header';
    layerHdr.textContent = 'レイヤー';
    this.layersList = document.createElement('div');
    this.layersList.className = 'tl-layers-list';
    layerCol.append(layerHdr, this.layersList);

    // Frame column — scrollable container holds both numbers and frames
    const frameCol = document.createElement('div');
    frameCol.className = 'tl-frame-col';

    this.numWrap = document.createElement('div');
    this.numWrap.style.cssText = 'height:28px;flex-shrink:0;border-bottom:1px solid #dfe3e7;background:#fafafa;';
    this.numbersCanvas = document.createElement('canvas');
    this.numbersCanvas.height = 28;
    this.numWrap.appendChild(this.numbersCanvas);

    this.framesWrap = document.createElement('div');
    this.framesWrap.style.cssText = 'flex:1;position:relative;';
    this.framesCanvas = document.createElement('canvas');
    this.framesWrap.appendChild(this.framesCanvas);

    this.playheadEl = document.createElement('div');
    this.playheadEl.className = 'tl-playhead';
    this.framesWrap.appendChild(this.playheadEl);

    this.framesCanvas.addEventListener('pointerdown', this.onFramesClick);

    frameCol.append(this.numWrap, this.framesWrap);
    main.append(layerCol, frameCol);
    body.appendChild(main);

    window.addEventListener('keydown', this.onKeyDown);

    // Playback loop
    let lastTime = 0;
    const tick = (t: number) => {
      const s = this.appState.get();
      if (s.isPlaying) {
        const interval = 1000 / s.project.settings.fps;
        if (t - lastTime >= interval) {
          lastTime = t;
          const next = s.currentFrame >= s.project.settings.totalFrames ? 1 : s.currentFrame + 1;
          this.appState.setFrame(next);
        }
      }
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }

  private makeTlBtn(text: string, title: string): HTMLButtonElement {
    const btn = document.createElement('button');
    btn.className = 'tl-btn';
    btn.textContent = text;
    btn.title = title;
    return btn;
  }

  private render(): void {
    const s = this.appState.get();
    const { project, currentFrame, selectedLayerId } = s;
    const totalFrames = project.settings.totalFrames;
    const reversedLayers = [...project.layers].reverse();

    this.frameInfo.textContent = `フレーム: ${currentFrame} / ${totalFrames}`;

    if (!this.isRenaming) {
      this.buildLayersList(reversedLayers, selectedLayerId);
    }

    this.drawFrames(reversedLayers, totalFrames, currentFrame);
    this.updatePlayhead(currentFrame, totalFrames);
  }

  private buildLayersList(reversedLayers: AnimationLayer[], selectedLayerId: string): void {
    this.layersList.innerHTML = '';

    // Camera row
    const camRow = document.createElement('div');
    camRow.className = 'tl-layer-row camera-row';
    camRow.innerHTML = `<span class="tl-layer-name">📹 カメラ</span>`;
    this.layersList.appendChild(camRow);

    reversedLayers.forEach(layer => {
      const row = document.createElement('div');
      row.className = 'tl-layer-row' + (layer.id === selectedLayerId ? ' selected' : '');
      row.addEventListener('click', () => this.appState.update({ selectedLayerId: layer.id }));

      const visBtn = document.createElement('button');
      visBtn.className = 'tl-layer-icon-btn';
      visBtn.title = layer.visible ? '非表示にする' : '表示する';
      visBtn.textContent = layer.visible ? '👁' : '—';
      visBtn.addEventListener('click', (e) => { e.stopPropagation(); this.appState.toggleLayerVisible(layer.id); });

      const lockBtn = document.createElement('button');
      lockBtn.className = 'tl-layer-icon-btn';
      lockBtn.title = layer.locked ? 'ロック解除' : 'ロック';
      lockBtn.textContent = layer.locked ? '🔒' : '🔓';
      lockBtn.style.fontSize = '11px';
      lockBtn.addEventListener('click', (e) => { e.stopPropagation(); this.appState.toggleLayerLock(layer.id); });

      const delBtn = document.createElement('button');
      delBtn.className = 'tl-layer-icon-btn';
      delBtn.title = 'レイヤー削除';
      delBtn.textContent = '×';
      delBtn.addEventListener('click', (e) => { e.stopPropagation(); this.appState.removeLayer(layer.id); });

      const name = document.createElement('span');
      name.className = 'tl-layer-name';
      name.textContent = layer.name;
      name.title = 'ダブルクリックで名前変更';

      name.addEventListener('dblclick', (e) => {
        e.stopPropagation();
        this.isRenaming = true;
        const input = document.createElement('input');
        input.type = 'text';
        input.value = layer.name;
        input.className = 'tl-layer-rename-input';
        name.replaceWith(input);
        input.focus();
        input.select();

        let committed = false;
        const commit = () => {
          if (committed) return;
          committed = true;
          const newName = input.value.trim() || layer.name;
          this.isRenaming = false;
          this.appState.renameLayer(layer.id, newName);
        };
        const cancel = () => {
          if (committed) return;
          committed = true;
          this.isRenaming = false;
          input.replaceWith(name);
        };
        input.addEventListener('blur', commit);
        input.addEventListener('keydown', (ke) => {
          if (ke.key === 'Enter') { ke.preventDefault(); commit(); }
          if (ke.key === 'Escape') { ke.preventDefault(); cancel(); }
        });
      });

      row.append(visBtn, lockBtn, name, delBtn);
      this.layersList.appendChild(row);
    });
  }

  private drawFrames(layers: AnimationLayer[], totalFrames: number, currentFrame: number): void {
    const containerW = this.framesWrap.clientWidth || 500;
    const canvasW = Math.max(totalFrames * FRAME_W + 20, containerW);
    const numRows = layers.length + 1; // +1 for camera
    const canvasH = numRows * ROW_H;

    this.framesCanvas.width = canvasW;
    this.framesCanvas.height = canvasH;
    this.numbersCanvas.width = canvasW;

    // Sync min-width so tl-frame-col scrollbar appears when needed
    this.numWrap.style.minWidth = canvasW + 'px';
    this.framesWrap.style.minWidth = canvasW + 'px';

    const fctx = this.framesCanvas.getContext('2d')!;
    const nctx = this.numbersCanvas.getContext('2d')!;

    fctx.clearRect(0, 0, canvasW, canvasH);
    nctx.clearRect(0, 0, canvasW, 28);

    // Frame numbers
    nctx.fillStyle = '#fafafa';
    nctx.fillRect(0, 0, canvasW, 28);
    nctx.font = '11px Inter, sans-serif';
    nctx.fillStyle = '#888';
    nctx.textAlign = 'center';
    for (let f = 1; f <= totalFrames; f++) {
      const x = (f - 1) * FRAME_W + FRAME_W / 2;
      if (f === 1 || f % 5 === 0) nctx.fillText(String(f), x, 18);
      nctx.fillStyle = '#ccc';
      nctx.fillRect(x - 0.5, 22, 1, 6);
      nctx.fillStyle = '#888';
    }

    // Background rows
    for (let row = 0; row < numRows; row++) {
      fctx.fillStyle = row % 2 === 0 ? '#fafafa' : '#f4f5f6';
      fctx.fillRect(0, row * ROW_H, canvasW, ROW_H);
    }

    // Grid lines
    fctx.strokeStyle = '#eef0f3';
    fctx.lineWidth = 1;
    for (let f = 0; f <= totalFrames; f++) {
      const x = f * FRAME_W + 0.5;
      fctx.beginPath(); fctx.moveTo(x, 0); fctx.lineTo(x, canvasH); fctx.stroke();
    }
    for (let row = 0; row <= numRows; row++) {
      const y = row * ROW_H + 0.5;
      fctx.beginPath(); fctx.moveTo(0, y); fctx.lineTo(canvasW, y); fctx.stroke();
    }

    // Camera row keyframes
    const camKfs = this.appState.get().project.cameraTrack.keyframes;
    camKfs.forEach(kf => {
      const x = (kf.frame - 1) * FRAME_W;
      fctx.fillStyle = '#ff4b4b';
      fctx.beginPath();
      fctx.arc(x + FRAME_W / 2, ROW_H / 2, 4, 0, Math.PI * 2);
      fctx.fill();
    });

    // Layer frames — ◯ for drawn frames, small dot for koma-hold frames
    layers.forEach((layer, i) => {
      const rowY = (i + 1) * ROW_H;
      const cy = rowY + ROW_H / 2;
      layer.frames.forEach(frame => {
        const x = (frame.frame - 1) * FRAME_W + FRAME_W / 2;
        if (frame.drawingId !== null) {
          fctx.strokeStyle = '#4a90e2';
          fctx.lineWidth = 1.5;
          fctx.beginPath();
          fctx.arc(x, cy, 4, 0, Math.PI * 2);
          fctx.stroke();
        } else if (frame.holdSource !== null) {
          fctx.fillStyle = '#4a90e2';
          fctx.beginPath();
          fctx.arc(x, cy, 2, 0, Math.PI * 2);
          fctx.fill();
        }
      });
    });

    // Current frame highlight column
    const fx = (currentFrame - 1) * FRAME_W;
    fctx.fillStyle = 'rgba(255,75,75,0.08)';
    fctx.fillRect(fx, 0, FRAME_W, canvasH);
  }

  private updatePlayhead(currentFrame: number, _totalFrames: number): void {
    const x = (currentFrame - 1) * FRAME_W + FRAME_W / 2;
    this.playheadEl.style.left = x + 'px';
  }

  private onFramesClick = (e: PointerEvent): void => {
    const rect = this.framesCanvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const frame = Math.max(1, Math.min(this.appState.get().project.settings.totalFrames, Math.floor(x / FRAME_W) + 1));
    this.appState.setFrame(frame);

    const move = (ev: PointerEvent) => {
      const x2 = ev.clientX - rect.left;
      const f2 = Math.max(1, Math.min(this.appState.get().project.settings.totalFrames, Math.floor(x2 / FRAME_W) + 1));
      this.appState.setFrame(f2);
    };
    const up = () => { window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', up); };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  };

  private onKeyDown = (e: KeyboardEvent): void => {
    if ((e.target as HTMLElement).tagName === 'INPUT') return;
    // Frame navigation
    if (e.key === ',' || e.key === 'ArrowLeft') {
      e.preventDefault();
      this.appState.setFrame(this.appState.get().currentFrame - 1);
    }
    if (e.key === 'ArrowRight') {
      e.preventDefault();
      this.appState.setFrame(this.appState.get().currentFrame + 1);
    }
    // Koma assignment / release
    if (e.key === '.') {
      e.preventDefault();
      const s = this.appState.get();
      const layer = s.project.layers.find(l => l.id === s.selectedLayerId);
      const frameData = layer?.frames.find(f => f.frame === s.currentFrame);
      if (frameData?.holdSource != null) {
        this.appState.releaseKomaHold(s.currentFrame, s.selectedLayerId);
      } else if (frameData?.drawingId === null) {
        this.appState.setKomaHold(s.currentFrame, s.selectedLayerId);
      }
    }
    // Delete blank range
    if (e.key === 'Delete') {
      e.preventDefault();
      this.appState.deleteBlankRange(this.appState.get().currentFrame);
    }
    if (e.key === 'Enter') {
      const s = this.appState.get();
      this.appState.update({ isPlaying: !s.isPlaying });
    }
  };
}
