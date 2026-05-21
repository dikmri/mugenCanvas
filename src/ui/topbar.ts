import { AppState } from '../app/app-state.ts';
import type { CanvasView } from '../canvas/canvas-view.ts';

export function buildTopBar(el: HTMLElement, appState: AppState, canvasView: CanvasView): void {
  el.innerHTML = '';

  const brand = document.createElement('span');
  brand.className = 'topbar-brand';
  brand.textContent = 'mugenCanvas';

  const sep1 = document.createElement('div'); sep1.className = 'topbar-sep';

  // File buttons
  const newBtn  = makeBtn(fileNewSvg(), '新規 (Ctrl+N)', () => { if (confirm('新規プロジェクトを作成しますか？')) location.reload(); });
  const openBtn = makeBtn(openSvg(),    '開く (Ctrl+O)',  () => triggerKey('o', true));
  const saveBtn = makeBtn(saveSvg(),    '保存 (Ctrl+S)',  () => triggerKey('s', true));

  const sep2 = document.createElement('div'); sep2.className = 'topbar-sep';

  // Undo/Redo
  const onRestored = (layerId: string, frame: number) => {
    appState.update({ selectedLayerId: layerId, currentFrame: frame });
    canvasView.markDirty();
  };
  const undoBtn = makeBtn(undoSvg(), 'Undo (Ctrl+Z)',         () => canvasView.getUndoManager().undo(onRestored));
  const redoBtn = makeBtn(redoSvg(), 'Redo (Ctrl+Shift+Z)',   () => canvasView.getUndoManager().redo(onRestored));

  const sep3 = document.createElement('div'); sep3.className = 'topbar-sep';

  // Playback
  const playBtn = makeBtn(playSvg(), '再生 (Enter)', () => {
    appState.update({ isPlaying: !appState.get().isPlaying });
  });
  appState.subscribe(s => { playBtn.innerHTML = s.isPlaying ? stopSvg() : playSvg(); });

  // Center info
  const center = document.createElement('div');
  center.className = 'topbar-center';
  const projName = document.createElement('div'); projName.className = 'proj-name';
  const projInfo = document.createElement('div'); projInfo.className = 'proj-info';
  center.append(projName, projInfo);
  const updateCenter = () => {
    const s = appState.get().project.settings;
    projName.textContent = '新規プロジェクト';
    projInfo.textContent = `${s.width} x ${s.height}px / ${s.fps}fps`;
  };
  updateCenter();
  appState.subscribe(updateCenter);

  // Right controls
  const right = document.createElement('div');
  right.className = 'topbar-right';

  // Zoom select
  const zoomSelect = document.createElement('select');
  zoomSelect.className = 'zoom-select';
  [12.5, 25, 50, 75, 100, 150, 200, 400].forEach(z => {
    const opt = document.createElement('option');
    opt.value = String(z / 100);
    opt.textContent = z + '%';
    if (z === 100) opt.selected = true;
    zoomSelect.appendChild(opt);
  });
  zoomSelect.addEventListener('change', () => {
    const s = appState.get();
    appState.update({ viewport: { ...s.viewport, zoom: parseFloat(zoomSelect.value) } });
  });
  appState.subscribe(s => {
    const pct = Math.round(s.viewport.zoom * 100);
    const existing = zoomSelect.querySelector<HTMLOptionElement>(`option[value="${s.viewport.zoom}"]`);
    if (!existing) {
      let custom = zoomSelect.querySelector<HTMLOptionElement>('.custom-zoom');
      if (!custom) {
        custom = document.createElement('option');
        custom.className = 'custom-zoom';
        zoomSelect.appendChild(custom);
      }
      custom.value = String(s.viewport.zoom);
      custom.textContent = pct + '%';
      custom.selected = true;
    }
  });

  // Rotation indicator / reset button
  const rotBtn = document.createElement('button');
  rotBtn.className = 'topbar-btn topbar-rot-btn';
  rotBtn.title = 'キャンバス回転をリセット (Ctrl+R)';
  const updateRot = () => {
    const deg = Math.round(appState.get().viewport.rotation * 180 / Math.PI);
    rotBtn.textContent = `${deg >= 0 ? '' : ''}${deg}°`;
    rotBtn.style.opacity = deg === 0 ? '0.4' : '1';
    rotBtn.style.fontWeight = deg !== 0 ? '700' : '400';
  };
  updateRot();
  appState.subscribe(updateRot);
  rotBtn.addEventListener('click', () => {
    const s = appState.get();
    appState.update({ viewport: { ...s.viewport, rotation: 0 } });
  });

  // Export
  const exportBtn = makeBtn(exportSvg(), '現在フレームを書き出し', () => exportCurrentFrame(appState, canvasView));

  // Dark mode toggle
  const themeBtn = makeBtn(sunSvg(), 'テーマ切替', () => cycleTheme(appState));
  appState.subscribe(s => {
    themeBtn.innerHTML = s.theme === 'dark' ? moonSvg() : (s.theme === 'light' ? sunSvg() : autoSvg());
    themeBtn.title = s.theme === 'system' ? 'テーマ: システム' : s.theme === 'light' ? 'テーマ: ライト' : 'テーマ: ダーク';
  });

  // Settings button → opens onion skin settings modal
  const settingsBtn = makeBtn(settingsSvg(), '設定', () => openSettingsModal(appState));

  right.append(zoomSelect, rotBtn, exportBtn, themeBtn, settingsBtn);
  el.append(brand, sep1, newBtn, openBtn, saveBtn, sep2, undoBtn, redoBtn, sep3, playBtn, center, right);
}

// ── Helpers ────────────────────────────────────────────────────────────────

function makeBtn(svg: string, title: string, onClick: () => void): HTMLButtonElement {
  const btn = document.createElement('button');
  btn.className = 'topbar-btn';
  btn.title = title;
  btn.innerHTML = svg;
  btn.addEventListener('click', onClick);
  return btn;
}

function triggerKey(key: string, ctrl: boolean): void {
  window.dispatchEvent(new KeyboardEvent('keydown', { key, ctrlKey: ctrl, bubbles: true }));
}

async function exportCurrentFrame(appState: AppState, canvasView: CanvasView): Promise<void> {
  try {
    const s = appState.get();
    const imageData = canvasView.exportCurrentFrame();
    if (!imageData) { alert('カメラフレームが設定されていません'); return; }
    const oc = new OffscreenCanvas(imageData.width, imageData.height);
    const octx = oc.getContext('2d')!;
    octx.putImageData(imageData, 0, 0);
    const blob = await oc.convertToBlob({ type: 'image/png' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `frame_${String(s.currentFrame).padStart(4, '0')}.png`;
    a.click();
  } catch (e) { console.error('Export failed', e); }
}

function cycleTheme(appState: AppState): void {
  const current = appState.get().theme;
  const next = current === 'system' ? 'light' : current === 'light' ? 'dark' : 'system';
  appState.update({ theme: next });
  applyTheme(next);
}

export function applyTheme(theme: 'system' | 'light' | 'dark'): void {
  localStorage.setItem('theme', theme);
  if (theme === 'system') {
    const dark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    document.documentElement.dataset.theme = dark ? 'dark' : 'light';
  } else {
    document.documentElement.dataset.theme = theme;
  }
}

function openSettingsModal(appState: AppState): void {
  if (document.getElementById('settings-modal')) return;

  const overlay = document.createElement('div');
  overlay.id = 'settings-modal';
  overlay.className = 'settings-overlay';

  const dialog = document.createElement('div');
  dialog.className = 'settings-dialog';

  const title = document.createElement('h2');
  title.className = 'settings-title';
  title.textContent = '設定';

  const closeBtn = document.createElement('button');
  closeBtn.className = 'settings-close';
  closeBtn.textContent = '×';
  closeBtn.addEventListener('click', () => overlay.remove());
  overlay.addEventListener('click', (e) => { if (e.target === overlay) overlay.remove(); });

  // ── Onion skin section ──
  const onionTitle = document.createElement('div');
  onionTitle.className = 'settings-section-title';
  onionTitle.textContent = 'オニオンスキン';

  const s = appState.get();
  const os = s.onionSkinSettings;

  const prevRow = makeColorAlphaRow('前フレームの色', os.prevColor, os.prevAlpha,
    (color) => appState.update({ onionSkinSettings: { ...appState.get().onionSkinSettings, prevColor: color } }),
    (alpha) => appState.update({ onionSkinSettings: { ...appState.get().onionSkinSettings, prevAlpha: alpha } }),
  );
  const nextRow = makeColorAlphaRow('次フレームの色', os.nextColor, os.nextAlpha,
    (color) => appState.update({ onionSkinSettings: { ...appState.get().onionSkinSettings, nextColor: color } }),
    (alpha) => appState.update({ onionSkinSettings: { ...appState.get().onionSkinSettings, nextAlpha: alpha } }),
  );

  dialog.append(title, closeBtn, onionTitle, prevRow, nextRow);
  overlay.appendChild(dialog);
  document.body.appendChild(overlay);
}

function makeColorAlphaRow(
  label: string,
  initColor: string,
  initAlpha: number,
  onColor: (c: string) => void,
  onAlpha: (a: number) => void,
): HTMLDivElement {
  const row = document.createElement('div');
  row.className = 'settings-row';

  const lbl = document.createElement('span');
  lbl.className = 'settings-row-label';
  lbl.textContent = label;

  const colorInput = document.createElement('input');
  colorInput.type = 'color';
  colorInput.value = initColor;
  colorInput.className = 'settings-color-input';
  colorInput.addEventListener('input', () => onColor(colorInput.value));

  const alphaSlider = document.createElement('input');
  alphaSlider.type = 'range';
  alphaSlider.min = '0'; alphaSlider.max = '100'; alphaSlider.step = '1';
  alphaSlider.value = String(Math.round(initAlpha * 100));
  alphaSlider.className = 'settings-alpha-slider';
  const alphaVal = document.createElement('span');
  alphaVal.className = 'settings-alpha-val';
  alphaVal.textContent = Math.round(initAlpha * 100) + '%';
  alphaSlider.addEventListener('input', () => {
    const v = parseInt(alphaSlider.value) / 100;
    alphaVal.textContent = Math.round(v * 100) + '%';
    onAlpha(v);
  });

  row.append(lbl, colorInput, alphaSlider, alphaVal);
  return row;
}

// ── SVG Icons ──────────────────────────────────────────────────────────────

const ICON = 'width="18" height="18" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"';
const svg = (c: string) => `<svg ${ICON}>${c}</svg>`;

function fileNewSvg()  { return svg('<path d="M4 3h8l5 5v10H4V3z"/><path d="M12 3v5h5"/>'); }
function openSvg()     { return svg('<path d="M2 6h5l2-3h9v14H2V6z"/>'); }
function saveSvg()     { return svg('<rect x="3" y="3" width="14" height="14" rx="2"/><path d="M7 3v5h6V3M7 13h6"/><rect x="7" y="13" width="6" height="4"/>'); }
function undoSvg()     { return svg('<path d="M4 9H14a4 4 0 010 8H8"/><path d="M4 9L8 5M4 9L8 13"/>'); }
function redoSvg()     { return svg('<path d="M16 9H6a4 4 0 000 8h6"/><path d="M16 9l-4-4m4 4l-4 4"/>'); }
function playSvg()     { return svg('<path d="M6 4l12 6-12 6z"/>'); }
function stopSvg()     { return svg('<rect x="5" y="5" width="10" height="10" rx="1"/>'); }
function exportSvg()   { return svg('<path d="M10 3v10M6 9l4 4 4-4"/><path d="M3 15v3h14v-3"/>'); }
function settingsSvg() { return svg('<circle cx="10" cy="10" r="3"/><path d="M10 2v2M10 16v2M2 10h2M16 10h2M4.9 4.9l1.4 1.4M13.7 13.7l1.4 1.4M4.9 15.1l1.4-1.4M13.7 6.3l1.4-1.4"/>'); }
function sunSvg()      { return svg('<circle cx="10" cy="10" r="4"/><path d="M10 2v2M10 16v2M2 10h2M16 10h2M4.9 4.9l1.4 1.4M13.7 13.7l1.4 1.4M4.9 15.1l1.4-1.4M13.7 6.3l1.4-1.4"/>'); }
function moonSvg()     { return svg('<path d="M17 12a7 7 0 11-9-9 5 5 0 109 9z"/>'); }
function autoSvg()     { return svg('<circle cx="10" cy="10" r="7" stroke-dasharray="3 3"/><path d="M10 3v14M3 10h14"/>'); }
