export interface PanelOptions {
  id: string;
  title: string;
  x: number;
  y: number;
  width: number;
  height?: number;
  extraClass?: string;
}

export class FloatingPanel {
  el: HTMLDivElement;
  private titlebar: HTMLDivElement;
  private bodyEl: HTMLDivElement;
  private pinBtn: HTMLButtonElement;
  private pinned = false;
  private collapsed = false;

  constructor(container: HTMLElement, opts: PanelOptions) {
    this.el = document.createElement('div');
    this.el.className = `floating-panel${opts.extraClass ? ' ' + opts.extraClass : ''}`;
    this.el.style.cssText = `left:${opts.x}px;top:${opts.y}px;width:${opts.width}px;${opts.height ? `height:${opts.height}px;` : ''}`;
    this.el.style.position = 'absolute';

    this.titlebar = document.createElement('div');
    this.titlebar.className = 'panel-titlebar';

    const title = document.createElement('span');
    title.className = 'panel-title';
    title.textContent = opts.title;

    this.pinBtn = document.createElement('button');
    this.pinBtn.className = 'panel-btn';
    this.pinBtn.title = '固定';
    this.pinBtn.innerHTML = pinIcon(false);
    this.pinBtn.addEventListener('click', () => this.togglePin());

    const collapseBtn = document.createElement('button');
    collapseBtn.className = 'panel-btn';
    collapseBtn.title = '折りたたみ';
    collapseBtn.textContent = '−';
    collapseBtn.addEventListener('click', () => this.toggleCollapse());

    const closeBtn = document.createElement('button');
    closeBtn.className = 'panel-btn';
    closeBtn.title = '閉じる';
    closeBtn.textContent = '×';
    closeBtn.addEventListener('click', () => this.hide());

    this.titlebar.append(title, this.pinBtn, collapseBtn, closeBtn);

    this.bodyEl = document.createElement('div');
    this.bodyEl.className = 'panel-body';

    this.el.append(this.titlebar, this.bodyEl);
    container.appendChild(this.el);

    this.makeDraggable();
    this.bringToFront();
    this.el.addEventListener('pointerdown', () => this.bringToFront());
  }

  get body(): HTMLDivElement { return this.bodyEl; }

  private makeDraggable(): void {
    let ox = 0, oy = 0, startX = 0, startY = 0;
    const onMove = (e: PointerEvent) => {
      if (this.pinned) return;
      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      this.el.style.left = (ox + dx) + 'px';
      this.el.style.top = (oy + dy) + 'px';
    };
    const onUp = () => {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
    };
    this.titlebar.addEventListener('pointerdown', (e) => {
      if (this.pinned) return;
      if ((e.target as HTMLElement).tagName === 'BUTTON') return;
      e.preventDefault();
      ox = parseInt(this.el.style.left) || 0;
      oy = parseInt(this.el.style.top) || 0;
      startX = e.clientX;
      startY = e.clientY;
      window.addEventListener('pointermove', onMove);
      window.addEventListener('pointerup', onUp);
    });

    this.titlebar.addEventListener('dblclick', (e) => {
      if ((e.target as HTMLElement).tagName === 'BUTTON') return;
      this.toggleCollapse();
    });
  }

  private bringToFront(): void {
    const panels = document.querySelectorAll<HTMLElement>('.floating-panel');
    let max = 100;
    panels.forEach(p => { const z = parseInt(p.style.zIndex || '100'); if (z > max) max = z; });
    this.el.style.zIndex = String(max + 1);
  }

  private togglePin(): void {
    this.pinned = !this.pinned;
    this.el.classList.toggle('pinned', this.pinned);
    this.pinBtn.innerHTML = pinIcon(this.pinned);
  }

  private toggleCollapse(): void {
    this.collapsed = !this.collapsed;
    this.el.classList.toggle('collapsed', this.collapsed);
  }

  hide(): void { this.el.style.display = 'none'; }
  show(): void { this.el.style.display = ''; }
}

function pinIcon(active: boolean): string {
  const color = active ? '#4a90e2' : 'currentColor';
  return `<svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M8 1L13 6L9.5 7.5L7 12L5.5 8.5L2 7L6.5 4.5L8 1Z" stroke="${color}" stroke-width="1.4" stroke-linejoin="round"/>
    <path d="M2 12L5 9" stroke="${color}" stroke-width="1.4" stroke-linecap="round"/>
  </svg>`;
}
