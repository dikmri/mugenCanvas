import { AppState } from '../app/app-state.ts';

export function buildStatusBar(el: HTMLElement, appState: AppState): void {
  el.innerHTML = '';
  const left = document.createElement('span');
  const right = document.createElement('span');
  right.className = 'statusbar-right';
  el.append(left, right);

  const update = () => {
    const s = appState.get();
    const { settings } = s.project;
    left.textContent = `${settings.width} x ${settings.height}px  ${settings.fps}fps`;
    right.textContent = `フレーム: ${s.currentFrame}/${settings.totalFrames}  表示倍率: ${Math.round(s.viewport.zoom * 100)}%`;
  };
  update();
  appState.subscribe(update);
}
