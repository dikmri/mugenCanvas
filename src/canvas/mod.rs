pub mod brush;
pub mod camera;
pub mod viewport;

use std::collections::{HashMap, HashSet};
use crate::model::{AnimationLayer, BrushSettings, OnionSkinSettings, Viewport, resolve_layer_frame};
use brush::{TILE_SIZE, TILE_BYTES, draw_segment, flood_fill};
use viewport::world_to_screen;

const TS: usize = TILE_SIZE as usize;

type TileKey = (String, u32, i32, i32);
type LfKey = (String, u32);

pub struct CanvasState {
    tiles: HashMap<TileKey, Vec<u8>>,
    tile_sets: HashMap<LfKey, HashSet<(i32, i32)>>,
    pub project_w: u32,
    pub project_h: u32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self { tiles: HashMap::new(), tile_sets: HashMap::new(), project_w: 1920, project_h: 1080 }
    }
}

impl CanvasState {
    pub fn set_canvas_size(&mut self, w: u32, h: u32) {
        if self.project_w == w && self.project_h == h { return; }
        self.project_w = w;
        self.project_h = h;
        self.tiles.clear();
        self.tile_sets.clear();
    }

    fn get_or_create_tile_mut(&mut self, layer_id: &str, frame: u32, tx: i32, ty: i32) -> &mut Vec<u8> {
        let key: TileKey = (layer_id.to_string(), frame, tx, ty);
        self.tile_sets
            .entry((layer_id.to_string(), frame))
            .or_default()
            .insert((tx, ty));
        self.tiles.entry(key).or_insert_with(|| vec![0u8; TILE_BYTES])
    }

    pub fn ensure_base_tiles(&mut self, layer_id: &str, frame: u32) {
        let tx_max = (self.project_w as i32 + TILE_SIZE - 1) / TILE_SIZE;
        let ty_max = (self.project_h as i32 + TILE_SIZE - 1) / TILE_SIZE;
        for tx in -1..=tx_max {
            for ty in -1..=ty_max {
                self.get_or_create_tile_mut(layer_id, frame, tx, ty);
            }
        }
    }

    pub fn draw_segment_to_tiles(
        &mut self, layer_id: &str, frame: u32,
        from: (f32, f32), to: (f32, f32),
        settings: &BrushSettings, is_eraser: bool,
    ) {
        let r = settings.size / 2.0;
        let tx_min = ((from.0.min(to.0) - r) / TILE_SIZE as f32).floor() as i32;
        let tx_max = ((from.0.max(to.0) + r) / TILE_SIZE as f32).floor() as i32;
        let ty_min = ((from.1.min(to.1) - r) / TILE_SIZE as f32).floor() as i32;
        let ty_max = ((from.1.max(to.1) + r) / TILE_SIZE as f32).floor() as i32;
        let [cr, cg, cb] = settings.color;

        for tx in tx_min..=tx_max {
            for ty in ty_min..=ty_max {
                let tile = self.get_or_create_tile_mut(layer_id, frame, tx, ty);
                let tile_ox = (tx * TILE_SIZE) as f32;
                let tile_oy = (ty * TILE_SIZE) as f32;
                draw_segment(tile, tile_ox, tile_oy, from, to, settings.size, cr, cg, cb, settings.opacity, is_eraser);
            }
        }
    }

    pub fn flood_fill_tiles(
        &mut self, layer_id: &str, frame: u32,
        world_x: f32, world_y: f32,
        fill_r: u8, fill_g: u8, fill_b: u8,
    ) {
        let tx0 = (world_x / TILE_SIZE as f32).floor() as i32;
        let ty0 = (world_y / TILE_SIZE as f32).floor() as i32;
        self.get_or_create_tile_mut(layer_id, frame, tx0, ty0);

        let lf: LfKey = (layer_id.to_string(), frame);
        let coords: Vec<(i32, i32)> = self.tile_sets.get(&lf).map(|s| s.iter().cloned().collect()).unwrap_or_default();
        if coords.is_empty() { return; }

        let tx_min = coords.iter().map(|&(tx, _)| tx).min().unwrap();
        let tx_max = coords.iter().map(|&(tx, _)| tx).max().unwrap();
        let ty_min = coords.iter().map(|&(_, ty)| ty).min().unwrap();
        let ty_max = coords.iter().map(|&(_, ty)| ty).max().unwrap();

        let cols = (tx_max - tx_min + 1) as usize;
        let rows = (ty_max - ty_min + 1) as usize;
        let comp_w = cols * TS;
        let comp_h = rows * TS;
        let mut composite = vec![0u8; comp_w * comp_h * 4];

        for &(tx, ty) in &coords {
            if let Some(tile) = self.tiles.get(&(layer_id.to_string(), frame, tx, ty)) {
                let dst_x = (tx - tx_min) as usize * TS;
                let dst_y = (ty - ty_min) as usize * TS;
                for row in 0..TS {
                    let src = row * TS * 4;
                    let dst = ((dst_y + row) * comp_w + dst_x) * 4;
                    composite[dst..dst + TS * 4].copy_from_slice(&tile[src..src + TS * 4]);
                }
            }
        }

        let fx = ((world_x - tx_min as f32 * TILE_SIZE as f32) as usize).min(comp_w.saturating_sub(1));
        let fy = ((world_y - ty_min as f32 * TILE_SIZE as f32) as usize).min(comp_h.saturating_sub(1));
        flood_fill(&mut composite, comp_w, comp_h, fx, fy, fill_r, fill_g, fill_b, 255);

        for &(tx, ty) in &coords {
            let src_x = (tx - tx_min) as usize * TS;
            let src_y = (ty - ty_min) as usize * TS;
            let tile = self.tiles.entry((layer_id.to_string(), frame, tx, ty)).or_insert_with(|| vec![0u8; TILE_BYTES]);
            for row in 0..TS {
                let src = ((src_y + row) * comp_w + src_x) * 4;
                let dst = row * TS * 4;
                tile[dst..dst + TS * 4].copy_from_slice(&composite[src..src + TS * 4]);
            }
        }
    }

    pub fn remove_layer(&mut self, layer_id: &str) {
        self.tiles.retain(|(lid, _, _, _), _| lid != layer_id);
        self.tile_sets.retain(|(lid, _), _| lid != layer_id);
    }

    pub fn snapshot_tiles(&self, layer_id: &str, frame: u32) -> HashMap<(i32, i32), Vec<u8>> {
        let lf: LfKey = (layer_id.to_string(), frame);
        let mut result = HashMap::new();
        if let Some(coords) = self.tile_sets.get(&lf) {
            for &(tx, ty) in coords {
                if let Some(data) = self.tiles.get(&(layer_id.to_string(), frame, tx, ty)) {
                    result.insert((tx, ty), data.clone());
                }
            }
        }
        result
    }

    pub fn restore_tiles(&mut self, layer_id: &str, frame: u32, snapshot: HashMap<(i32, i32), Vec<u8>>) {
        self.tiles.retain(|(lid, f, _, _), _| !(lid == layer_id && *f == frame));
        let lf: LfKey = (layer_id.to_string(), frame);
        self.tile_sets.remove(&lf);
        for ((tx, ty), data) in snapshot {
            self.tiles.insert((layer_id.to_string(), frame, tx, ty), data);
            self.tile_sets.entry(lf.clone()).or_default().insert((tx, ty));
        }
    }

    /// Load a tile from raw RGBA bytes (used during project load).
    pub fn load_tile(&mut self, layer_id: &str, frame: u32, tx: i32, ty: i32, data: Vec<u8>) {
        debug_assert_eq!(data.len(), TILE_BYTES);
        self.tile_sets.entry((layer_id.to_string(), frame)).or_default().insert((tx, ty));
        self.tiles.insert((layer_id.to_string(), frame, tx, ty), data);
    }

    // ─── Rendering ────────────────────────────────────────────────────────────

    /// Composite all visible layers for the current frame into a screen-sized RGBA buffer.
    pub fn composite(
        &self,
        layers: &[AnimationLayer],
        vp: &Viewport,
        current_frame: u32,
        onion_skin: bool,
        onion: &OnionSkinSettings,
        screen_w: usize,
        screen_h: usize,
    ) -> Vec<u8> {
        // Start with white background (fully opaque)
        let mut out = vec![255u8; screen_w * screen_h * 4];
        // Make alpha = 255
        for i in (3..out.len()).step_by(4) { out[i] = 255; }

        if onion_skin {
            if current_frame > 1 {
                for layer in layers {
                    if !layer.visible { continue; }
                    if let Some(src) = resolve_layer_frame(layer, current_frame - 1) {
                        self.blit_tinted(&mut out, screen_w, screen_h, &layer.id, src, vp, onion.prev_color, onion.prev_alpha);
                    }
                }
            }
            for layer in layers {
                if !layer.visible { continue; }
                if let Some(src) = resolve_layer_frame(layer, current_frame + 1) {
                    self.blit_tinted(&mut out, screen_w, screen_h, &layer.id, src, vp, onion.next_color, onion.next_alpha);
                }
            }
        }

        for layer in layers {
            if !layer.visible { continue; }
            if let Some(src) = resolve_layer_frame(layer, current_frame) {
                self.blit_layer(&mut out, screen_w, screen_h, &layer.id, src, vp, 1.0, None);
            }
        }

        // Draw camera frame outline
        out
    }

    fn blit_layer(
        &self, out: &mut [u8], ow: usize, oh: usize,
        layer_id: &str, frame: u32, vp: &Viewport, alpha: f32,
        tint: Option<[u8; 3]>,
    ) {
        let lf: LfKey = (layer_id.to_string(), frame);

        if vp.rotation == 0.0 {
            // Fast path: no rotation — tile-based forward blit
            let coords: Vec<(i32, i32)> = match self.tile_sets.get(&lf) {
                Some(s) => s.iter().cloned().collect(),
                None => return,
            };
            let scaled = (TILE_SIZE as f32 * vp.zoom) as i32;
            for (tx, ty) in coords {
                if let Some(tile) = self.tiles.get(&(layer_id.to_string(), frame, tx, ty)) {
                    let (sx, sy) = world_to_screen((tx * TILE_SIZE) as f32, (ty * TILE_SIZE) as f32, vp);
                    blit_scaled(out, ow, oh, tile, TS, sx as i32, sy as i32, scaled, alpha, tint);
                }
            }
        } else {
            // Rotation path: per-pixel inverse transform (sample world from each screen pixel)
            let tile_set: std::collections::HashSet<(i32, i32)> = match self.tile_sets.get(&lf) {
                Some(s) => s.clone(),
                None => return,
            };
            if tile_set.is_empty() { return; }

            let cx = ow as f32 / 2.0;
            let cy = oh as f32 / 2.0;
            // Inverse rotation: rotate by -vp.rotation => cos(-θ)=cos(θ), sin(-θ)=-sin(θ)
            let cos_r = vp.rotation.cos();
            let sin_r = vp.rotation.sin();

            for py in 0..oh {
                for px in 0..ow {
                    let dx = px as f32 - cx;
                    let dy = py as f32 - cy;
                    // Unrotate screen point (same formula as unrotate_point / pointer_to_world)
                    let ux = cx + dx * cos_r + dy * sin_r;
                    let uy = cy - dx * sin_r + dy * cos_r;
                    // Screen → world
                    let wx = (ux - vp.offset_x) / vp.zoom;
                    let wy = (uy - vp.offset_y) / vp.zoom;
                    // Tile index
                    let tx = (wx / TILE_SIZE as f32).floor() as i32;
                    let ty = (wy / TILE_SIZE as f32).floor() as i32;

                    if !tile_set.contains(&(tx, ty)) { continue; }

                    if let Some(tile) = self.tiles.get(&(layer_id.to_string(), frame, tx, ty)) {
                        let lx = ((wx - (tx * TILE_SIZE) as f32).max(0.0) as usize).min(TS - 1);
                        let ly = ((wy - (ty * TILE_SIZE) as f32).max(0.0) as usize).min(TS - 1);
                        let si = (ly * TS + lx) * 4;
                        let src_a_raw = tile[si + 3];
                        if src_a_raw == 0 { continue; }

                        let (sr, sg, sb) = if let Some([tr, tg, tb]) = tint {
                            (tr, tg, tb)
                        } else {
                            (tile[si], tile[si + 1], tile[si + 2])
                        };

                        let sa = src_a_raw as f32 / 255.0 * alpha;
                        let di = (py * ow + px) * 4;
                        let da = out[di + 3] as f32 / 255.0;
                        let oa = sa + da * (1.0 - sa);
                        if oa > 0.001 {
                            let inv = 1.0 / oa;
                            out[di]     = ((sr as f32 * sa + out[di]     as f32 * da * (1.0 - sa)) * inv) as u8;
                            out[di + 1] = ((sg as f32 * sa + out[di + 1] as f32 * da * (1.0 - sa)) * inv) as u8;
                            out[di + 2] = ((sb as f32 * sa + out[di + 2] as f32 * da * (1.0 - sa)) * inv) as u8;
                            out[di + 3] = (oa * 255.0) as u8;
                        }
                    }
                }
            }
        }
    }

    fn blit_tinted(
        &self, out: &mut [u8], ow: usize, oh: usize,
        layer_id: &str, frame: u32, vp: &Viewport,
        tint: [u8; 3], alpha: f32,
    ) {
        self.blit_layer(out, ow, oh, layer_id, frame, vp, alpha, Some(tint));
    }

    /// Export a world-space region as raw RGBA pixels (for PNG/GIF export).
    pub fn export_region(
        &self,
        layers: &[AnimationLayer],
        frame: u32,
        src_x: f32, src_y: f32,
        width: u32, height: u32,
    ) -> Vec<u8> {
        let (w, h) = (width as usize, height as usize);
        let mut out = vec![255u8; w * h * 4];

        for layer in layers {
            if !layer.visible { continue; }
            let src_frame = match resolve_layer_frame(layer, frame) {
                Some(f) => f,
                None => continue,
            };
            let tx_min = (src_x / TILE_SIZE as f32).floor() as i32;
            let tx_max = ((src_x + width as f32 - 1.0) / TILE_SIZE as f32).floor() as i32;
            let ty_min = (src_y / TILE_SIZE as f32).floor() as i32;
            let ty_max = ((src_y + height as f32 - 1.0) / TILE_SIZE as f32).floor() as i32;
            for tx in tx_min..=tx_max {
                for ty in ty_min..=ty_max {
                    if let Some(tile) = self.tiles.get(&(layer.id.clone(), src_frame, tx, ty)) {
                        let tile_wx = (tx * TILE_SIZE) as f32;
                        let tile_wy = (ty * TILE_SIZE) as f32;
                        let ox = src_x.max(tile_wx) as i32;
                        let oy = src_y.max(tile_wy) as i32;
                        let ow_i = ((src_x + width as f32).min(tile_wx + TILE_SIZE as f32)) as i32 - ox;
                        let oh_i = ((src_y + height as f32).min(tile_wy + TILE_SIZE as f32)) as i32 - oy;
                        if ow_i <= 0 || oh_i <= 0 { continue; }
                        for row in 0..oh_i as usize {
                            for col in 0..ow_i as usize {
                                let tsx = (ox - tile_wx as i32) as usize + col;
                                let tsy = (oy - tile_wy as i32) as usize + row;
                                let si = (tsy * TS + tsx) * 4;
                                let src_a = tile[si + 3];
                                if src_a == 0 { continue; }
                                let dx = (ox - src_x as i32) as usize + col;
                                let dy = (oy - src_y as i32) as usize + row;
                                if dx >= w || dy >= h { continue; }
                                let di = (dy * w + dx) * 4;
                                let sa = src_a as f32 / 255.0;
                                let da = out[di + 3] as f32 / 255.0;
                                let oa = sa + da * (1.0 - sa);
                                if oa > 0.001 {
                                    let inv = 1.0 / oa;
                                    out[di]     = ((tile[si]   as f32 * sa + out[di]   as f32 * da * (1.0 - sa)) * inv) as u8;
                                    out[di + 1] = ((tile[si+1] as f32 * sa + out[di+1] as f32 * da * (1.0 - sa)) * inv) as u8;
                                    out[di + 2] = ((tile[si+2] as f32 * sa + out[di+2] as f32 * da * (1.0 - sa)) * inv) as u8;
                                    out[di + 3] = (oa * 255.0) as u8;
                                }
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// Return all tile data for a layer-frame as (tx, ty, rgba_bytes).
    pub fn iter_tiles(&self, layer_id: &str, frame: u32) -> impl Iterator<Item = (i32, i32, &[u8])> {
        let lf: LfKey = (layer_id.to_string(), frame);
        let keys: Vec<TileKey> = self.tile_sets
            .get(&lf)
            .map(|s| s.iter().map(|&(tx, ty)| (layer_id.to_string(), frame, tx, ty)).collect())
            .unwrap_or_default();
        keys.into_iter().filter_map(move |k| {
            let (_, _, tx, ty) = k.clone();
            self.tiles.get(&k).map(|v| (tx, ty, v.as_slice()))
        })
    }
}

// ─── Utility: scaled blit with optional color tint ────────────────────────────

fn blit_scaled(
    out: &mut [u8], out_w: usize, out_h: usize,
    tile: &[u8], tile_size: usize,
    dst_x: i32, dst_y: i32, dst_size: i32,
    alpha: f32, tint: Option<[u8; 3]>,
) {
    if dst_size <= 0 { return; }
    let x0 = dst_x.max(0) as usize;
    let y0 = dst_y.max(0) as usize;
    let x1 = (dst_x + dst_size).min(out_w as i32) as usize;
    let y1 = (dst_y + dst_size).min(out_h as i32) as usize;
    if x0 >= x1 || y0 >= y1 { return; }

    let scale = tile_size as f32 / dst_size as f32;
    for py in y0..y1 {
        for px in x0..x1 {
            let spx = (((px as i32 - dst_x) as f32 * scale) as usize).min(tile_size - 1);
            let spy = (((py as i32 - dst_y) as f32 * scale) as usize).min(tile_size - 1);
            let si = (spy * tile_size + spx) * 4;
            let src_a_raw = tile[si + 3];
            if src_a_raw == 0 { continue; }

            let (sr, sg, sb) = if let Some([tr, tg, tb]) = tint {
                (tr, tg, tb)
            } else {
                (tile[si], tile[si + 1], tile[si + 2])
            };

            let sa = src_a_raw as f32 / 255.0 * alpha;
            let di = (py * out_w + px) * 4;
            let da = out[di + 3] as f32 / 255.0;
            let oa = sa + da * (1.0 - sa);
            if oa > 0.001 {
                let inv = 1.0 / oa;
                out[di]     = ((sr as f32 * sa + out[di]     as f32 * da * (1.0 - sa)) * inv) as u8;
                out[di + 1] = ((sg as f32 * sa + out[di + 1] as f32 * da * (1.0 - sa)) * inv) as u8;
                out[di + 2] = ((sb as f32 * sa + out[di + 2] as f32 * da * (1.0 - sa)) * inv) as u8;
                out[di + 3] = (oa * 255.0) as u8;
            }
        }
    }
}
