use std::collections::HashMap;
use std::time::{Duration, Instant};

use egui::{Color32, Context, Key, Pos2, Rect, Sense};

use crate::canvas::viewport::{fit_viewport, pointer_to_world, unrotate_point, world_to_screen_rotated, zoom_around};
use crate::canvas::{camera::get_camera_at_frame, CanvasState};
use crate::io;
use crate::model::Tool;
use crate::state::AppState;
use crate::ui::topbar::TopbarAction;
use crate::undo::UndoManager;

pub struct MugenCanvasApp {
    state: AppState,
    canvas: CanvasState,
    undo: UndoManager,

    // Rendering
    canvas_texture: Option<egui::TextureHandle>,
    canvas_panel_size: (usize, usize),
    dirty: bool,
    viewport_fitted: bool,

    // Drawing input state
    is_painting: bool,
    is_panning: bool,
    is_mmb_panning: bool,
    is_space_down: bool,
    is_r_down: bool,
    last_paint_pos: Option<(f32, f32)>,
    pan_start_pointer: Pos2,
    pan_start_offset: (f32, f32),

    // Playback
    last_frame_time: Option<Instant>,

    // Export / status
    status_msg: Option<(String, Instant)>,
    gif_exporting: bool,

    // Frame clipboard (Ctrl+C / Ctrl+V)
    frame_clipboard: Option<HashMap<(i32, i32), Vec<u8>>>,

    // Camera tool drag state
    is_camera_dragging: bool,
    camera_drag_world_start: (f32, f32),
    camera_drag_kf_start: Option<crate::model::CameraKeyframe>,
}

impl MugenCanvasApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_fonts(&cc.egui_ctx);
        let state = AppState::default();
        let mut canvas = CanvasState::default();
        canvas.set_canvas_size(state.project.settings.width, state.project.settings.height);
        Self {
            state,
            canvas,
            undo: UndoManager::default(),
            canvas_texture: None,
            canvas_panel_size: (0, 0),
            dirty: true,
            viewport_fitted: false,
            is_painting: false,
            is_panning: false,
            is_mmb_panning: false,
            is_space_down: false,
            is_r_down: false,
            last_paint_pos: None,
            pan_start_pointer: Pos2::ZERO,
            pan_start_offset: (0.0, 0.0),
            last_frame_time: None,
            status_msg: None,
            gif_exporting: false,
            frame_clipboard: None,
            is_camera_dragging: false,
            camera_drag_world_start: (0.0, 0.0),
            camera_drag_kf_start: None,
        }
    }

    fn show_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some((msg.into(), Instant::now()));
    }

    // ─── Keyboard shortcuts ──────────────────────────────────────────────────

    fn handle_shortcuts(&mut self, ctx: &Context) {
        ctx.input(|i| {
            let ctrl = i.modifiers.ctrl || i.modifiers.command;
            let shift = i.modifiers.shift;

            if ctrl && !shift && i.key_pressed(Key::Z) { self.do_undo(); }
            if ctrl && shift && i.key_pressed(Key::Z) { self.do_redo(); }
            if ctrl && i.key_pressed(Key::Y) { self.do_redo(); }
            if ctrl && !shift && i.key_pressed(Key::S) { self.do_save(); }
            if ctrl && !shift && i.key_pressed(Key::O) { self.do_open(); }
            if ctrl && !shift && i.key_pressed(Key::N) { self.do_new(); }
            if ctrl && !shift && i.key_pressed(Key::R) { self.state.viewport.rotation = 0.0; self.dirty = true; }
            if ctrl && !shift && i.key_pressed(Key::E) { self.state.viewport.rotation += 20.0_f32.to_radians(); self.dirty = true; }
            if ctrl && !shift && i.key_pressed(Key::Q) { self.state.viewport.rotation -= 20.0_f32.to_radians(); self.dirty = true; }
            if ctrl && !shift && i.key_pressed(Key::C) { self.do_copy_frame(); }
            if ctrl && !shift && i.key_pressed(Key::V) { self.do_paste_frame(); }

            // Tool shortcuts (no modifier)
            if !ctrl && !shift {
                if i.key_pressed(Key::B) { self.state.selected_tool = Tool::Brush; }
                if i.key_pressed(Key::E) { self.state.selected_tool = Tool::Eraser; }
                if i.key_pressed(Key::G) { self.state.selected_tool = Tool::Fill; }
                if i.key_pressed(Key::H) { self.state.selected_tool = Tool::Hand; }
                if i.key_pressed(Key::Z) { self.state.selected_tool = Tool::Zoom; }
                if i.key_pressed(Key::C) { self.state.selected_tool = Tool::Camera; }
                if i.key_pressed(Key::Enter) { self.state.is_playing = !self.state.is_playing; }

                // Frame navigation
                if i.key_pressed(Key::ArrowLeft) {
                    let f = self.state.current_frame.saturating_sub(1).max(1);
                    self.state.set_frame(f); self.dirty = true;
                }
                if i.key_pressed(Key::ArrowRight) {
                    let f = (self.state.current_frame + 1).min(self.state.project.settings.total_frames);
                    self.state.set_frame(f); self.dirty = true;
                }

                // . key: toggle koma hold on current frame of selected layer
                if i.key_pressed(Key::Period) {
                    let frame = self.state.current_frame;
                    let layer_id = self.state.selected_layer_id.clone();
                    let is_hold = self.state.project.layers.iter()
                        .find(|l| l.id == layer_id)
                        .and_then(|l| l.frames.iter().find(|f| f.frame == frame))
                        .map(|f| f.hold_source.is_some())
                        .unwrap_or(false);
                    if is_hold {
                        self.state.release_koma_hold(frame, &layer_id);
                    } else {
                        self.state.set_koma_hold(frame, &layer_id);
                    }
                    self.dirty = true;
                }
            }

            // Space for temporary hand tool
            self.is_space_down = i.key_down(Key::Space);
            // R key for canvas rotation (actual rotation applied via pointer_delta in handle_canvas_input)
            self.is_r_down = i.key_down(Key::R) && !ctrl;
        });
    }

    // ─── Playback ────────────────────────────────────────────────────────────

    fn handle_playback(&mut self, ctx: &Context) {
        if !self.state.is_playing { self.last_frame_time = None; return; }
        let fps = self.state.project.settings.fps as f32;
        let frame_dur = Duration::from_secs_f32(1.0 / fps);
        let now = Instant::now();
        if let Some(last) = self.last_frame_time {
            if now.duration_since(last) >= frame_dur {
                self.state.advance_frame();
                self.last_frame_time = Some(last + frame_dur);
                self.dirty = true;
            }
        } else {
            self.last_frame_time = Some(now);
        }
        ctx.request_repaint();
    }

    // ─── Canvas rendering ─────────────────────────────────────────────────────

    fn update_canvas_texture(&mut self, ctx: &Context) {
        let (w, h) = self.canvas_panel_size;
        if w == 0 || h == 0 || !self.dirty { return; }

        let pixels = self.canvas.composite(
            &self.state.project.layers,
            &self.state.viewport,
            self.state.current_frame,
            self.state.onion_skin,
            &self.state.onion_skin_settings,
            w, h,
        );

        // Draw camera frame outline directly into pixel buffer
        draw_camera_overlay(&mut pixels.clone(), &mut self.canvas, &self.state, w, h);
        // (overlay drawn into separate pass below to avoid borrow issues)

        let mut final_pixels = pixels;
        draw_camera_overlay_onto(&mut final_pixels, &self.state, w, h);

        let image = egui::ColorImage::from_rgba_unmultiplied([w, h], &final_pixels);
        if let Some(ref mut handle) = self.canvas_texture {
            handle.set(image, egui::TextureOptions::NEAREST);
        } else {
            self.canvas_texture = Some(ctx.load_texture("canvas", image, egui::TextureOptions::NEAREST));
        }
        self.dirty = false;
    }

    // ─── Canvas input ─────────────────────────────────────────────────────────

    fn handle_canvas_input(&mut self, response: &egui::Response, ctx: &Context) {
        let panel_rect = response.rect;
        let panel_w = panel_rect.width();
        let panel_h = panel_rect.height();

        // Map current pointer to panel-local coords.
        // Use latest_pos() (not hover_pos()) so drag operations like rotation
        // continue to track the pointer after it crosses the decidedly-dragging threshold.
        let local_pos: Option<(f32, f32)> = ctx.input(|i| {
            i.pointer.latest_pos()
                .map(|p| (p.x - panel_rect.min.x, p.y - panel_rect.min.y))
        });

        ctx.input(|i| {
            let primary_pressed = i.pointer.button_pressed(egui::PointerButton::Primary);
            let primary_down = i.pointer.button_down(egui::PointerButton::Primary);
            let primary_released = i.pointer.button_released(egui::PointerButton::Primary);
            let mmb_pressed = i.pointer.button_pressed(egui::PointerButton::Middle);
            let mmb_down = i.pointer.button_down(egui::PointerButton::Middle);
            let mmb_released = i.pointer.button_released(egui::PointerButton::Middle);
            let pointer_delta = i.pointer.delta();

            // ── R held: rotate canvas by horizontal pointer movement ──────────
            // pointer_delta is non-zero on any mouse move, button-press not needed.
            if self.is_r_down && (pointer_delta.x != 0.0 || pointer_delta.y != 0.0) {
                let rot_delta = pointer_delta.x * (std::f32::consts::PI / 300.0);
                self.state.viewport.rotation += rot_delta;
                self.dirty = true;
            }

            // ── Middle mouse pan ──────────────────────────────────────────────
            if mmb_pressed && response.hovered() {
                self.is_mmb_panning = true;
                self.pan_start_pointer = i.pointer.hover_pos().unwrap_or(panel_rect.center());
                self.pan_start_offset = (self.state.viewport.offset_x, self.state.viewport.offset_y);
            }
            if self.is_mmb_panning && mmb_down {
                self.state.viewport.offset_x += pointer_delta.x;
                self.state.viewport.offset_y += pointer_delta.y;
                self.dirty = true;
            }
            if mmb_released { self.is_mmb_panning = false; }

            // ── Scroll zoom ───────────────────────────────────────────────────
            if response.hovered() {
                let scroll = i.raw_scroll_delta;
                if scroll.y.abs() > 0.0 {
                    if let Some((lx, ly)) = local_pos {
                        let cx = panel_w / 2.0;
                        let cy = panel_h / 2.0;
                        let (px, py) = unrotate_point(lx, ly, cx, cy, self.state.viewport.rotation);
                        let factor: f32 = if scroll.y > 0.0 { 1.1 } else { 1.0 / 1.1 };
                        self.state.viewport = zoom_around(&self.state.viewport, px, py, factor);
                        self.dirty = true;
                    }
                }
                // Horizontal scroll → pan X
                if scroll.x.abs() > 0.0 {
                    self.state.viewport.offset_x -= scroll.x;
                    self.dirty = true;
                }
            }

            // ── Primary button ────────────────────────────────────────────────
            let is_hand = self.state.selected_tool == Tool::Hand || self.is_space_down;

            if primary_pressed && response.hovered() {
                if self.is_r_down {
                    // Rotation handled above; block other tools while R is held.
                } else if is_hand || self.is_mmb_panning {
                    // Begin pan
                    self.is_panning = true;
                    self.pan_start_pointer = i.pointer.hover_pos().unwrap_or(panel_rect.center());
                    self.pan_start_offset = (self.state.viewport.offset_x, self.state.viewport.offset_y);
                } else if self.state.selected_tool == Tool::Camera {
                    if let Some((lx, ly)) = local_pos {
                        let (wx, wy) = pointer_to_world(lx, ly, panel_w, panel_h, &self.state.viewport);
                        let kf = get_camera_at_frame(&self.state.project.camera_track.keyframes, self.state.current_frame);
                        self.is_camera_dragging = true;
                        self.camera_drag_world_start = (wx, wy);
                        self.camera_drag_kf_start = Some(kf);
                    }
                } else if self.state.selected_tool == Tool::Zoom {
                    if let Some((lx, ly)) = local_pos {
                        let cx = panel_w / 2.0; let cy = panel_h / 2.0;
                        let (px, py) = unrotate_point(lx, ly, cx, cy, self.state.viewport.rotation);
                        let factor: f32 = if i.modifiers.alt { 1.0 / 1.3 } else { 1.3 };
                        self.state.viewport = zoom_around(&self.state.viewport, px, py, factor);
                        self.dirty = true;
                    }
                } else if self.state.selected_tool == Tool::Fill {
                    if let Some((lx, ly)) = local_pos {
                        let (wx, wy) = pointer_to_world(lx, ly, panel_w, panel_h, &self.state.viewport);
                        if self.state.can_paint() {
                            let layer_id = self.state.selected_layer_id.clone();
                            let frame = self.state.current_frame;
                            let snap = self.canvas.snapshot_tiles(&layer_id, frame);
                            self.undo.snapshot(snap, &layer_id, frame);
                            let [fr, fg, fb] = self.state.brush.color;
                            self.canvas.flood_fill_tiles(&layer_id, frame, wx, wy, fr, fg, fb);
                            self.state.mark_frame_drawn(&layer_id, frame);
                            self.dirty = true;
                        }
                    }
                } else if self.state.selected_tool == Tool::Brush || self.state.selected_tool == Tool::Eraser {
                    if self.state.can_paint() {
                        let layer_id = self.state.selected_layer_id.clone();
                        let frame = self.state.current_frame;
                        let snap = self.canvas.snapshot_tiles(&layer_id, frame);
                        self.undo.snapshot(snap, &layer_id, frame);
                        self.canvas.ensure_base_tiles(&layer_id, frame);
                        self.state.mark_frame_drawn(&layer_id, frame);
                        if let Some((lx, ly)) = local_pos {
                            let (wx, wy) = pointer_to_world(lx, ly, panel_w, panel_h, &self.state.viewport);
                            let settings = self.state.active_brush().clone();
                            let is_eraser = self.state.selected_tool == Tool::Eraser;
                            self.canvas.draw_segment_to_tiles(&layer_id, frame, (wx, wy), (wx, wy), &settings, is_eraser);
                            self.last_paint_pos = Some((wx, wy));
                            self.is_painting = true;
                            self.dirty = true;
                        }
                    }
                }
            }

            if primary_down {
                if self.is_camera_dragging {
                    if let Some((lx, ly)) = local_pos {
                        if let Some(ref kf_start) = self.camera_drag_kf_start.clone() {
                            let (wx, wy) = pointer_to_world(lx, ly, panel_w, panel_h, &self.state.viewport);
                            let dx = (wx - self.camera_drag_world_start.0) as f64;
                            let dy = (wy - self.camera_drag_world_start.1) as f64;
                            let new_x = kf_start.x + dx;
                            let new_y = kf_start.y + dy;
                            let frame = self.state.current_frame;
                            self.upsert_camera_kf_pos(frame, new_x, new_y);
                            self.dirty = true;
                        }
                    }
                } else if self.is_panning {
                    self.state.viewport.offset_x += pointer_delta.x;
                    self.state.viewport.offset_y += pointer_delta.y;
                    self.dirty = true;
                } else if self.is_painting {
                    if let Some((lx, ly)) = local_pos {
                        let (wx, wy) = pointer_to_world(lx, ly, panel_w, panel_h, &self.state.viewport);
                        if let Some(last) = self.last_paint_pos {
                            let layer_id = self.state.selected_layer_id.clone();
                            let frame = self.state.current_frame;
                            let settings = self.state.active_brush().clone();
                            let is_eraser = self.state.selected_tool == Tool::Eraser;
                            self.canvas.draw_segment_to_tiles(&layer_id, frame, last, (wx, wy), &settings, is_eraser);
                            self.last_paint_pos = Some((wx, wy));
                            self.dirty = true;
                        }
                    }
                }
            }

            if primary_released {
                self.is_painting = false;
                self.is_panning = false;
                self.is_camera_dragging = false;
                self.camera_drag_kf_start = None;
                self.last_paint_pos = None;
            }
        });
    }

    // ─── Actions ──────────────────────────────────────────────────────────────

    fn do_undo(&mut self) {
        let snap = self.canvas.snapshot_tiles(&self.state.selected_layer_id, self.state.current_frame);
        if let Some(entry) = self.undo.undo(snap) {
            self.canvas.restore_tiles(&entry.layer_id, entry.frame, entry.tiles);
            self.state.selected_layer_id = entry.layer_id;
            self.state.current_frame = entry.frame;
            self.dirty = true;
        }
    }

    fn do_redo(&mut self) {
        let snap = self.canvas.snapshot_tiles(&self.state.selected_layer_id, self.state.current_frame);
        if let Some(entry) = self.undo.redo(snap) {
            self.canvas.restore_tiles(&entry.layer_id, entry.frame, entry.tiles);
            self.state.selected_layer_id = entry.layer_id;
            self.state.current_frame = entry.frame;
            self.dirty = true;
        }
    }

    fn do_new(&mut self) {
        self.state = AppState::default();
        self.canvas = CanvasState::default();
        self.canvas.set_canvas_size(self.state.project.settings.width, self.state.project.settings.height);
        self.undo.clear();
        self.viewport_fitted = false;
        self.dirty = true;
    }

    fn do_save(&mut self) {
        let path = if let Some(ref p) = self.state.project_path.clone() {
            p.clone()
        } else {
            match rfd::FileDialog::new()
                .add_filter("mugenCanvas", &["mugencanvas"])
                .set_file_name("project.mugencanvas")
                .save_file()
            {
                Some(p) => { let s = p.to_string_lossy().to_string(); self.state.project_path = Some(s.clone()); s }
                None => return,
            }
        };
        match io::save_project(&path, &self.state.project, &self.canvas) {
            Ok(()) => self.show_status("保存しました"),
            Err(e) => self.show_status(format!("保存失敗: {}", e)),
        }
    }

    #[allow(dead_code)]
    fn do_save_as(&mut self) {
        let path = match rfd::FileDialog::new()
            .add_filter("mugenCanvas", &["mugencanvas"])
            .set_file_name("project.mugencanvas")
            .save_file()
        {
            Some(p) => p.to_string_lossy().to_string(),
            None => return,
        };
        self.state.project_path = Some(path.clone());
        match io::save_project(&path, &self.state.project, &self.canvas) {
            Ok(()) => self.show_status("保存しました"),
            Err(e) => self.show_status(format!("保存失敗: {}", e)),
        }
    }

    fn do_open(&mut self) {
        let path = match rfd::FileDialog::new()
            .add_filter("mugenCanvas", &["mugencanvas", "json"])
            .pick_file()
        {
            Some(p) => p.to_string_lossy().to_string(),
            None => return,
        };
        match io::load_project(&path) {
            Ok((project, canvas)) => {
                let sel = project.layers.last().map(|l| l.id.clone()).unwrap_or_default();
                self.state.project = project;
                self.state.selected_layer_id = sel;
                self.state.current_frame = 1;
                self.state.project_path = Some(path);
                self.canvas = canvas;
                self.undo.clear();
                self.viewport_fitted = false;
                self.dirty = true;
                self.show_status("プロジェクトを開きました");
            }
            Err(e) => self.show_status(format!("読み込み失敗: {}", e)),
        }
    }

    fn do_export_png(&mut self) {
        let path = match rfd::FileDialog::new()
            .add_filter("PNG", &["png"])
            .set_file_name(format!("frame_{:04}.png", self.state.current_frame))
            .save_file()
        {
            Some(p) => p.to_string_lossy().to_string(),
            None => return,
        };
        let kf = get_camera_at_frame(&self.state.project.camera_track.keyframes, self.state.current_frame);
        match io::export_png(
            &path, &self.canvas, &self.state.project.layers,
            self.state.current_frame,
            kf.x as f32, kf.y as f32, kf.width as u32, kf.height as u32,
        ) {
            Ok(()) => self.show_status("PNGを書き出しました"),
            Err(e) => self.show_status(format!("PNG書き出し失敗: {}", e)),
        }
    }

    fn upsert_camera_kf_pos(&mut self, frame: u32, x: f64, y: f64) {
        let interp = get_camera_at_frame(&self.state.project.camera_track.keyframes, frame);
        let kfs = &mut self.state.project.camera_track.keyframes;
        if let Some(existing) = kfs.iter_mut().find(|k| k.frame == frame) {
            existing.x = x;
            existing.y = y;
        } else {
            let mut new_kf = interp;
            new_kf.frame = frame;
            new_kf.x = x;
            new_kf.y = y;
            kfs.push(new_kf);
            kfs.sort_by_key(|k| k.frame);
        }
    }

    fn do_copy_frame(&mut self) {
        let layer_id = self.state.selected_layer_id.clone();
        let frame = self.state.current_frame;
        self.frame_clipboard = Some(self.canvas.snapshot_tiles(&layer_id, frame));
        self.show_status(format!("フレーム {} をコピーしました", frame));
    }

    fn do_paste_frame(&mut self) {
        let clipboard = match self.frame_clipboard.clone() {
            Some(c) => c,
            None => { self.show_status("クリップボードが空です"); return; }
        };
        if !self.state.can_paint() {
            self.show_status("このフレームには貼り付けできません（ロック or コマ）");
            return;
        }
        let layer_id = self.state.selected_layer_id.clone();
        let frame = self.state.current_frame;
        let snap = self.canvas.snapshot_tiles(&layer_id, frame);
        self.undo.snapshot(snap, &layer_id, frame);
        self.canvas.restore_tiles(&layer_id, frame, clipboard);
        self.state.mark_frame_drawn(&layer_id, frame);
        self.dirty = true;
        self.show_status(format!("フレーム {} に貼り付けました", frame));
    }

    fn do_export_png_sequence(&mut self) {
        let dir = match rfd::FileDialog::new()
            .set_title("連番PNG書き出し先フォルダを選択")
            .pick_folder()
        {
            Some(p) => p.to_string_lossy().to_string(),
            None => return,
        };
        let kf = get_camera_at_frame(&self.state.project.camera_track.keyframes, 1);
        let layers = self.state.project.layers.clone();
        let total = self.state.project.settings.total_frames;
        let (cam_x, cam_y, cam_w, cam_h) = (kf.x as f32, kf.y as f32, kf.width as u32, kf.height as u32);

        self.show_status("連番PNG書き出し中...");
        match io::export_png_sequence(&dir, &self.canvas, &layers, total, cam_x, cam_y, cam_w, cam_h, |cur, tot| {
            eprintln!("[PNG seq] frame {}/{}", cur, tot);
        }) {
            Ok(()) => self.show_status(format!("連番PNG書き出し完了 ({} フレーム) → {}", total, dir)),
            Err(e) => self.show_status(format!("連番PNG書き出し失敗: {}", e)),
        }
    }

    fn do_export_gif(&mut self) {
        if self.gif_exporting { return; }
        let path = match rfd::FileDialog::new()
            .add_filter("GIF", &["gif"])
            .set_file_name("animation.gif")
            .save_file()
        {
            Some(p) => p.to_string_lossy().to_string(),
            None => return,
        };
        let kf = get_camera_at_frame(&self.state.project.camera_track.keyframes, 1);
        let layers = self.state.project.layers.clone();
        let total = self.state.project.settings.total_frames;
        let fps = self.state.project.settings.fps;
        let (cam_x, cam_y, cam_w, cam_h) = (kf.x as f32, kf.y as f32, kf.width as u32, kf.height as u32);

        self.gif_exporting = true;
        self.show_status("GIF書き出し中...");

        match io::export_gif(&path, &self.canvas, &layers, total, fps, cam_x, cam_y, cam_w, cam_h, |cur, tot| {
            eprintln!("[GIF] frame {}/{}", cur, tot);
        }) {
            Ok(()) => self.show_status("GIFを書き出しました"),
            Err(e) => self.show_status(format!("GIF書き出し失敗: {}", e)),
        }
        self.gif_exporting = false;
    }
}

// ─── Font setup ──────────────────────────────────────────────────────────────

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Japanese fonts (Windows system fonts) — appended as fallback after default ASCII font
    let jp_candidates = [
        "C:/Windows/Fonts/meiryo.ttc",
        "C:/Windows/Fonts/YuGothR.ttc",
        "C:/Windows/Fonts/YuGothM.ttc",
        "C:/Windows/Fonts/msgothic.ttc",
        "C:/Windows/Fonts/msmincho.ttc",
    ];
    for path in &jp_candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert("jp".to_owned(), egui::FontData::from_owned(data));
            for family in fonts.families.values_mut() {
                family.push("jp".to_owned());
            }
            break;
        }
    }

    // Phosphor Icons font — provides all toolbar/UI icons as Unicode glyphs
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    ctx.set_fonts(fonts);
}

// ─── eframe::App impl ─────────────────────────────────────────────────────────

impl eframe::App for MugenCanvasApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Handle playback timer
        self.handle_playback(ctx);

        // Handle keyboard shortcuts (only when no text input has focus)
        if !ctx.wants_keyboard_input() {
            self.handle_shortcuts(ctx);
        }

        // ── Top bar ───────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            let action = crate::ui::topbar::show(
                ui, &mut self.state,
                self.undo.can_undo(), self.undo.can_redo(),
            );
            match action {
                TopbarAction::New => self.do_new(),
                TopbarAction::Open => self.do_open(),
                TopbarAction::Save => self.do_save(),
                TopbarAction::ExportPng => self.do_export_png(),
                TopbarAction::ExportPngSequence => self.do_export_png_sequence(),
                TopbarAction::ExportGif => self.do_export_gif(),
                TopbarAction::Undo => self.do_undo(),
                TopbarAction::Redo => self.do_redo(),
                TopbarAction::None => {}
            }
        });

        // ── Status bar ────────────────────────────────────────────────────────
        egui::TopBottomPanel::bottom("statusbar").show(ctx, |ui| {
            let msg = if let Some((ref s, t)) = self.status_msg {
                if t.elapsed() > Duration::from_secs(4) {
                    self.status_msg = None; ""
                } else { s.as_str() }
            } else { "" };

            ui.horizontal(|ui| {
                ui.label(msg);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let vp = &self.state.viewport;
                    ui.label(format!(
                        "{}x{}  zoom:{:.0}%  rot:{:.0}°",
                        self.state.project.settings.width,
                        self.state.project.settings.height,
                        vp.zoom * 100.0,
                        vp.rotation * 180.0 / std::f32::consts::PI,
                    ));
                });
            });
        });

        // ── Timeline (bottom) ─────────────────────────────────────────────────
        egui::TopBottomPanel::bottom("timeline").min_height(80.0).show(ctx, |ui| {
            if crate::ui::timeline::show(ui, &mut self.state) {
                self.dirty = true;
            }
        });

        // ── Left panel: tool + color ──────────────────────────────────────────
        egui::SidePanel::left("left_panel").default_width(60.0).show(ctx, |ui| {
            crate::ui::toolbar::show(ui, &mut self.state);
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                crate::ui::color_panel::show(ui, &mut self.state);
            });
        });

        // ── Right panel: layers ───────────────────────────────────────────────
        egui::SidePanel::right("layers_panel").default_width(180.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let (dirty, _removed) = crate::ui::layers_panel::show(ui, &mut self.state, &mut self.canvas);
                if dirty { self.dirty = true; }
            });
        });

        // ── Central canvas ────────────────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            let (new_w, new_h) = (available.x as usize, available.y as usize);

            // Fit viewport on first layout
            if !self.viewport_fitted && new_w > 0 && new_h > 0 {
                let s = &self.state.project.settings;
                self.state.viewport = fit_viewport(
                    new_w as f32, new_h as f32,
                    s.width as f32, s.height as f32,
                );
                self.viewport_fitted = true;
                self.dirty = true;
            }

            if self.canvas_panel_size != (new_w, new_h) {
                self.canvas_panel_size = (new_w, new_h);
                self.dirty = true;
            }

            // Upload texture if dirty
            self.update_canvas_texture(ctx);

            // Allocate painter for input + rendering
            let (response, painter) =
                ui.allocate_painter(available, Sense::click_and_drag());

            // Draw canvas texture
            if let Some(ref texture) = self.canvas_texture {
                painter.image(
                    texture.id(),
                    response.rect,
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }

            // Rotation crosshair overlay
            if self.is_r_down {
                let cr = response.rect.center();
                let stroke = egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(74, 144, 226, 128));
                painter.line_segment([egui::pos2(cr.x - 20.0, cr.y), egui::pos2(cr.x + 20.0, cr.y)], stroke);
                painter.line_segment([egui::pos2(cr.x, cr.y - 20.0), egui::pos2(cr.x, cr.y + 20.0)], stroke);
            }

            // Handle input
            self.handle_canvas_input(&response, ctx);

            // Cursor
            let cursor = match self.state.selected_tool {
                _ if self.is_space_down || self.state.selected_tool == Tool::Hand =>
                    egui::CursorIcon::Grabbing,
                Tool::Brush | Tool::Eraser => egui::CursorIcon::Crosshair,
                Tool::Zoom => egui::CursorIcon::ZoomIn,
                Tool::Hand => egui::CursorIcon::Grab,
                _ => egui::CursorIcon::Default,
            };
            ctx.set_cursor_icon(cursor);
        });
    }
}

// ─── Camera frame overlay ──────────────────────────────────────────────────────

fn draw_camera_overlay_onto(pixels: &mut [u8], state: &AppState, w: usize, h: usize) {
    let kf = get_camera_at_frame(&state.project.camera_track.keyframes, state.current_frame);
    if !state.project.camera_track.visible { return; }
    let vp = &state.viewport;
    let scx = w as f32 / 2.0;
    let scy = h as f32 / 2.0;

    // Transform all 4 corners: world → screen (with rotation)
    let world_corners = [
        (kf.x as f32,                       kf.y as f32),
        (kf.x as f32 + kf.width as f32,     kf.y as f32),
        (kf.x as f32 + kf.width as f32,     kf.y as f32 + kf.height as f32),
        (kf.x as f32,                       kf.y as f32 + kf.height as f32),
    ];
    let mut sc = [(0i32, 0i32); 4];
    for (i, &(wx, wy)) in world_corners.iter().enumerate() {
        let (sx, sy) = world_to_screen_rotated(wx, wy, vp, scx, scy);
        sc[i] = (sx as i32, sy as i32);
    }

    let red = [0xffu8, 0x4bu8, 0x4bu8, 0xffu8];
    for i in 0..4 {
        let (x0, y0) = sc[i];
        let (x1, y1) = sc[(i + 1) % 4];
        draw_line(pixels, w, h, x0, y0, x1, y1, red);
    }
}

/// Bresenham's line — clips to buffer bounds.
fn draw_line(pixels: &mut [u8], pw: usize, ph: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1i32 } else { -1i32 };
    let sy = if y0 < y1 { 1i32 } else { -1i32 };
    let mut err = dx - dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        if x >= 0 && x < pw as i32 && y >= 0 && y < ph as i32 {
            let i = (y as usize * pw + x as usize) * 4;
            pixels[i..i + 4].copy_from_slice(&color);
        }
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 > -dy { err -= dy; x += sx; }
        if e2 < dx  { err += dx; y += sy; }
    }
}

fn draw_camera_overlay(_pixels: &mut Vec<u8>, _canvas: &mut CanvasState, _state: &AppState, _w: usize, _h: usize) {
    // stub: actual overlay is done in draw_camera_overlay_onto
}
