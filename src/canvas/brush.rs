pub const TILE_SIZE: i32 = 1024;
pub const TILE_BYTES: usize = (TILE_SIZE * TILE_SIZE * 4) as usize;

/// Draw an anti-aliased stroke segment onto a tile's RGBA pixel buffer.
/// `tile_ox`, `tile_oy` are the tile's world-space origin.
pub fn draw_segment(
    pixels: &mut [u8],
    tile_ox: f32, tile_oy: f32,
    from: (f32, f32), to: (f32, f32),
    size: f32, r: u8, g: u8, b: u8, opacity: f32,
    is_eraser: bool,
) {
    let dist = ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt();
    let step = (size * 0.25).max(1.0);
    let steps = ((dist / step).ceil() as i32).max(1);

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let wx = from.0 + (to.0 - from.0) * t;
        let wy = from.1 + (to.1 - from.1) * t;
        draw_circle(pixels, wx - tile_ox, wy - tile_oy, size / 2.0, r, g, b, opacity, is_eraser);
    }
}

fn draw_circle(pixels: &mut [u8], cx: f32, cy: f32, radius: f32, r: u8, g: u8, b: u8, opacity: f32, is_eraser: bool) {
    let x0 = ((cx - radius).floor() as i32).max(0);
    let x1 = ((cx + radius).ceil() as i32).min(TILE_SIZE - 1);
    let y0 = ((cy - radius).floor() as i32).max(0);
    let y1 = ((cy + radius).ceil() as i32).min(TILE_SIZE - 1);

    for py in y0..=y1 {
        for px in x0..=x1 {
            let dx = px as f32 + 0.5 - cx;
            let dy = py as f32 + 0.5 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            if d > radius { continue; }
            // Soft anti-aliased edge
            let circle_alpha = if d > radius - 1.0 { (radius - d).clamp(0.0, 1.0) } else { 1.0 };
            let alpha = circle_alpha * opacity;
            let idx = (py * TILE_SIZE + px) as usize * 4;

            if is_eraser {
                let existing_a = pixels[idx + 3] as f32 / 255.0;
                let new_a = (existing_a - alpha).max(0.0);
                pixels[idx + 3] = (new_a * 255.0) as u8;
                if new_a == 0.0 { pixels[idx] = 0; pixels[idx + 1] = 0; pixels[idx + 2] = 0; }
            } else {
                // Source-over compositing
                let sa = alpha;
                let da = pixels[idx + 3] as f32 / 255.0;
                let oa = sa + da * (1.0 - sa);
                if oa > 0.001 {
                    let inv = 1.0 / oa;
                    pixels[idx]     = ((r as f32 * sa + pixels[idx]     as f32 * da * (1.0 - sa)) * inv) as u8;
                    pixels[idx + 1] = ((g as f32 * sa + pixels[idx + 1] as f32 * da * (1.0 - sa)) * inv) as u8;
                    pixels[idx + 2] = ((b as f32 * sa + pixels[idx + 2] as f32 * da * (1.0 - sa)) * inv) as u8;
                    pixels[idx + 3] = (oa * 255.0) as u8;
                }
            }
        }
    }
}

/// Flood fill a pixel buffer. Alpha-aware: matches full RGBA at start pixel.
pub fn flood_fill(
    pixels: &mut [u8],
    width: usize, height: usize,
    start_x: usize, start_y: usize,
    fill_r: u8, fill_g: u8, fill_b: u8, fill_a: u8,
) {
    if start_x >= width || start_y >= height { return; }
    let i0 = (start_y * width + start_x) * 4;
    let (tr, tg, tb, ta) = (pixels[i0], pixels[i0 + 1], pixels[i0 + 2], pixels[i0 + 3]);
    if tr == fill_r && tg == fill_g && tb == fill_b && ta == fill_a { return; }

    let mut stack: Vec<(i32, i32)> = vec![(start_x as i32, start_y as i32)];
    let mut visited = vec![false; width * height];

    while let Some((x, y)) = stack.pop() {
        if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 { continue; }
        let pos = y as usize * width + x as usize;
        if visited[pos] { continue; }
        let idx = pos * 4;
        if pixels[idx] != tr || pixels[idx+1] != tg || pixels[idx+2] != tb || pixels[idx+3] != ta { continue; }
        visited[pos] = true;
        pixels[idx] = fill_r; pixels[idx+1] = fill_g; pixels[idx+2] = fill_b; pixels[idx+3] = fill_a;
        stack.push((x - 1, y)); stack.push((x + 1, y));
        stack.push((x, y - 1)); stack.push((x, y + 1));
    }
}
