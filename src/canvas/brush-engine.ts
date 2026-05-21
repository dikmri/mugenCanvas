import type { BrushSettings, Vec2 } from '../types/model.ts';

type DrawSegmentFn = (from: Vec2, to: Vec2) => void;

export class BrushEngine {
  private lastPoint: Vec2 | null = null;
  private drawFn: DrawSegmentFn | null = null;

  beginStroke(point: Vec2, _settings: BrushSettings, _isEraser: boolean, drawFn: DrawSegmentFn): void {
    this.lastPoint = point;
    this.drawFn = drawFn;
    drawFn(point, point);
  }

  continueStroke(point: Vec2): void {
    if (!this.lastPoint || !this.drawFn) return;
    this.drawFn(this.lastPoint, point);
    this.lastPoint = point;
  }

  endStroke(): void {
    this.lastPoint = null;
    this.drawFn = null;
  }
}
