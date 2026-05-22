use crate::model::Viewport;

pub fn screen_to_world(sx: f32, sy: f32, vp: &Viewport) -> (f32, f32) {
    ((sx - vp.offset_x) / vp.zoom, (sy - vp.offset_y) / vp.zoom)
}

pub fn world_to_screen(wx: f32, wy: f32, vp: &Viewport) -> (f32, f32) {
    (wx * vp.zoom + vp.offset_x, wy * vp.zoom + vp.offset_y)
}

pub fn zoom_around(vp: &Viewport, pivot_x: f32, pivot_y: f32, factor: f32) -> Viewport {
    let new_zoom = (vp.zoom * factor).clamp(0.05, 32.0);
    Viewport {
        zoom: new_zoom,
        offset_x: pivot_x - (pivot_x - vp.offset_x) * (new_zoom / vp.zoom),
        offset_y: pivot_y - (pivot_y - vp.offset_y) * (new_zoom / vp.zoom),
        rotation: vp.rotation,
    }
}

pub fn fit_viewport(panel_w: f32, panel_h: f32, world_w: f32, world_h: f32) -> Viewport {
    let zoom = (panel_w / world_w).min(panel_h / world_h) * 0.85;
    Viewport {
        zoom,
        offset_x: (panel_w - world_w * zoom) / 2.0,
        offset_y: (panel_h - world_h * zoom) / 2.0,
        rotation: 0.0,
    }
}

/// Un-rotate a point around (cx, cy) by -rotation.
pub fn unrotate_point(x: f32, y: f32, cx: f32, cy: f32, rotation: f32) -> (f32, f32) {
    if rotation == 0.0 { return (x, y); }
    let cos = (-rotation).cos();
    let sin = (-rotation).sin();
    let dx = x - cx;
    let dy = y - cy;
    (cx + dx * cos - dy * sin, cy + dx * sin + dy * cos)
}

/// Convert a canvas-panel-local pointer position to world coords, accounting for rotation.
pub fn pointer_to_world(local_x: f32, local_y: f32, panel_w: f32, panel_h: f32, vp: &Viewport) -> (f32, f32) {
    let cx = panel_w / 2.0;
    let cy = panel_h / 2.0;
    let (ux, uy) = unrotate_point(local_x, local_y, cx, cy, vp.rotation);
    screen_to_world(ux, uy, vp)
}
