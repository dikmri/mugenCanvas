import type { CameraKeyframe, CameraTrack, Viewport } from '../types/model.ts';
import { worldToScreen } from './viewport.ts';

export function getCameraAtFrame(track: CameraTrack, frame: number): CameraKeyframe {
  const kfs = track.keyframes;
  if (kfs.length === 0) {
    return { frame, x: 0, y: 0, scale: 1, rotation: 0, width: 1920, height: 1080 };
  }
  if (kfs.length === 1 || frame <= kfs[0].frame) return { ...kfs[0] };
  if (frame >= kfs[kfs.length - 1].frame) return { ...kfs[kfs.length - 1] };

  let before = kfs[0], after = kfs[kfs.length - 1];
  for (let i = 0; i < kfs.length - 1; i++) {
    if (kfs[i].frame <= frame && kfs[i + 1].frame >= frame) {
      before = kfs[i];
      after = kfs[i + 1];
      break;
    }
  }
  const t = (frame - before.frame) / (after.frame - before.frame);
  return {
    frame,
    x: lerp(before.x, after.x, t),
    y: lerp(before.y, after.y, t),
    scale: lerp(before.scale, after.scale, t),
    rotation: lerp(before.rotation, after.rotation, t),
    width: lerp(before.width, after.width, t),
    height: lerp(before.height, after.height, t),
  };
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

export function drawCameraFrame(
  ctx: CanvasRenderingContext2D,
  track: CameraTrack,
  frame: number,
  vp: Viewport,
): void {
  if (!track.visible) return;
  const cam = getCameraAtFrame(track, frame);

  const tl = worldToScreen({ x: cam.x, y: cam.y }, vp);
  const br = worldToScreen({ x: cam.x + cam.width, y: cam.y + cam.height }, vp);
  const w = br.x - tl.x;
  const h = br.y - tl.y;

  ctx.save();

  // Outer red frame
  ctx.strokeStyle = '#ff4b4b';
  ctx.lineWidth = 2;
  ctx.strokeRect(tl.x, tl.y, w, h);

  // Safe area inner (90%)
  const safeMarginX = w * 0.05;
  const safeMarginY = h * 0.05;
  ctx.strokeStyle = 'rgba(255,75,75,0.4)';
  ctx.lineWidth = 1;
  ctx.setLineDash([4, 4]);
  ctx.strokeRect(tl.x + safeMarginX, tl.y + safeMarginY, w - safeMarginX * 2, h - safeMarginY * 2);
  ctx.setLineDash([]);

  // Center crosshair guides
  const cx = tl.x + w / 2;
  const cy = tl.y + h / 2;
  const guideLen = 16;
  ctx.strokeStyle = 'rgba(255,75,75,0.6)';
  ctx.lineWidth = 1;

  ctx.beginPath();
  ctx.moveTo(cx, tl.y - guideLen);
  ctx.lineTo(cx, tl.y);
  ctx.moveTo(cx, br.y);
  ctx.lineTo(cx, br.y + guideLen);
  ctx.moveTo(tl.x - guideLen, cy);
  ctx.lineTo(tl.x, cy);
  ctx.moveTo(br.x, cy);
  ctx.lineTo(br.x + guideLen, cy);
  ctx.stroke();

  ctx.restore();
}
