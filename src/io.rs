use std::collections::HashMap;
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use serde_json::Value;

use crate::canvas::CanvasState;
use crate::canvas::brush::TILE_BYTES;
use crate::model::ProjectFile;

// ─── Project file format ──────────────────────────────────────────────────────


pub fn save_project(
    path: &str,
    project: &ProjectFile,
    canvas: &CanvasState,
) -> Result<(), String> {
    // Serialize project to JSON Value
    let mut disk: Value = serde_json::to_value(project).map_err(|e| e.to_string())?;

    // Build drawings map: all non-empty tiles as base64 PNG data URLs
    let mut drawings: HashMap<String, String> = HashMap::new();

    for layer in &project.layers {
        for frame in &layer.frames {
            if frame.drawing_id.is_none() { continue; }
            for (tx, ty, pixels) in canvas.iter_tiles(&layer.id, frame.frame) {
                if pixels.iter().all(|&b| b == 0) { continue; } // skip blank tiles
                let png = rgba_to_png(pixels, crate::canvas::brush::TILE_SIZE as u32, crate::canvas::brush::TILE_SIZE as u32)
                    .map_err(|e| format!("PNG encode: {}", e))?;
                let b64 = B64.encode(&png);
                let data_url = format!("data:image/png;base64,{}", b64);
                let key = format!("{}:{}:{}:{}", layer.id, frame.frame, tx, ty);
                drawings.insert(key, data_url);
            }
        }
    }

    disk["drawings"] = serde_json::to_value(&drawings).map_err(|e| e.to_string())?;

    let json = serde_json::to_string_pretty(&disk).map_err(|e| e.to_string())?;
    std::fs::write(path, json.as_bytes()).map_err(|e| e.to_string())
}

pub fn load_project(path: &str) -> Result<(ProjectFile, CanvasState), String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut raw: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    let drawings_val = raw["drawings"].take();
    let project: ProjectFile = serde_json::from_value(raw).map_err(|e| e.to_string())?;

    let mut canvas = CanvasState::default();
    canvas.set_canvas_size(project.settings.width, project.settings.height);

    if let Some(drawings) = drawings_val.as_object() {
        for (key, val) in drawings {
            let data_url = match val.as_str() { Some(s) => s, None => continue };
            // Parse key: "layerId:frame:tx:ty" (layerId may contain colons)
            let sep3 = key.rfind(':').unwrap_or(0);
            let sep2 = key[..sep3].rfind(':').unwrap_or(0);
            let sep1 = key[..sep2].rfind(':').unwrap_or(0);
            let layer_id = &key[..sep1];
            let frame: u32 = key[sep1+1..sep2].parse().unwrap_or(0);
            let tx: i32 = key[sep2+1..sep3].parse().unwrap_or(0);
            let ty: i32 = key[sep3+1..].parse().unwrap_or(0);
            if layer_id.is_empty() || frame == 0 { continue; }

            let b64_data = if data_url.contains(',') {
                &data_url[data_url.find(',').unwrap() + 1..]
            } else {
                data_url
            };
            let png_bytes = B64.decode(b64_data).map_err(|e| format!("base64 decode: {}", e))?;
            let img = image::load_from_memory(&png_bytes).map_err(|e| format!("PNG decode: {}", e))?;
            let img = img.to_rgba8();
            let expected = TILE_BYTES;
            let mut rgba = img.into_raw();
            rgba.resize(expected, 0);
            canvas.load_tile(layer_id, frame, tx, ty, rgba);
        }
    }

    Ok((project, canvas))
}

// ─── PNG export ───────────────────────────────────────────────────────────────

pub fn export_png(
    path: &str,
    canvas: &CanvasState,
    layers: &[crate::model::AnimationLayer],
    frame: u32,
    cam_x: f32, cam_y: f32, cam_w: u32, cam_h: u32,
) -> Result<(), String> {
    let rgba = canvas.export_region(layers, frame, cam_x, cam_y, cam_w, cam_h);
    let png = rgba_to_png(&rgba, cam_w, cam_h).map_err(|e| format!("PNG encode: {}", e))?;
    std::fs::write(path, &png).map_err(|e| e.to_string())
}

// ─── GIF export ───────────────────────────────────────────────────────────────

pub fn export_gif(
    path: &str,
    canvas: &CanvasState,
    layers: &[crate::model::AnimationLayer],
    total_frames: u32,
    fps: u32,
    cam_x: f32, cam_y: f32,
    cam_w: u32, cam_h: u32,
    on_progress: impl Fn(u32, u32) + Send + Sync,
) -> Result<(), String> {
    use gif::{Encoder, Frame, Repeat};
    use rayon::prelude::*;

    const MAX_GIF_W: u32 = 960;
    let scale = (MAX_GIF_W as f32 / cam_w as f32).min(1.0);
    let gif_w = (cam_w as f32 * scale).round() as u32;
    let gif_h = (cam_h as f32 * scale).round() as u32;
    let delay_cs = ((100.0 / fps.max(1) as f64).round() as u16).max(1);

    // Phase 1: render + quantize all frames in parallel
    let frames: Vec<Frame<'static>> = (1..=total_frames as usize)
        .into_par_iter()
        .map(|fi| -> Result<Frame<'static>, String> {
            on_progress(fi as u32, total_frames);
            let rgba = canvas.export_region(layers, fi as u32, cam_x, cam_y, cam_w, cam_h);
            let pixels_w;
            let mut pixels: Vec<u8>;
            if scale < 1.0 {
                // Simple nearest-neighbor downscale
                pixels_w = gif_w as usize;
                pixels = vec![0u8; (gif_w * gif_h * 4) as usize];
                for py in 0..gif_h as usize {
                    for px in 0..gif_w as usize {
                        let sx = (px as f32 / scale) as usize;
                        let sy = (py as f32 / scale) as usize;
                        let sx = sx.min(cam_w as usize - 1);
                        let sy = sy.min(cam_h as usize - 1);
                        let si = (sy * cam_w as usize + sx) * 4;
                        let di = (py * gif_w as usize + px) * 4;
                        pixels[di..di+4].copy_from_slice(&rgba[si..si+4]);
                    }
                }
            } else {
                pixels_w = cam_w as usize;
                pixels = rgba;
            }
            let _ = pixels_w;
            let mut frame = Frame::from_rgba_speed(gif_w as u16, gif_h as u16, &mut pixels, 30);
            frame.delay = delay_cs;
            Ok(frame)
        })
        .collect::<Result<_, _>>()?;

    // Phase 2: write GIF sequentially
    let file = std::fs::File::create(path).map_err(|e| format!("create file: {}", e))?;
    let mut enc = Encoder::new(file, gif_w as u16, gif_h as u16, &[])
        .map_err(|e| format!("encoder init: {}", e))?;
    enc.set_repeat(Repeat::Infinite).map_err(|e| format!("set_repeat: {}", e))?;
    for (i, frame) in frames.into_iter().enumerate() {
        enc.write_frame(&frame).map_err(|e| format!("write frame {}: {}", i, e))?;
    }
    Ok(())
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn rgba_to_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, image::ImageError> {
    use image::{ImageBuffer, Rgba};
    let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, rgba.to_vec())
        .ok_or_else(|| image::ImageError::Parameter(
            image::error::ParameterError::from_kind(image::error::ParameterErrorKind::DimensionMismatch)
        ))?;
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)?;
    Ok(buf.into_inner())
}
