use egui::{Ui, Color32, RichText, Stroke};
use egui_phosphor::regular as ph;
use crate::canvas::camera::get_camera_at_frame;
use crate::state::AppState;

/// Returns (frame_changed, hold_set, hold_released)
pub fn show(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut dirty = false;
    let total = state.project.settings.total_frames;
    let fps = state.project.settings.fps;

    // Playback controls
    ui.horizontal(|ui| {
        if ui.button("|◀").on_hover_text("先頭へ").clicked() {
            state.set_frame(1); dirty = true;
        }
        if ui.button("◀").on_hover_text("前フレーム").clicked() {
            let f = state.current_frame.saturating_sub(1).max(1);
            state.set_frame(f); dirty = true;
        }
        let play_label = if state.is_playing { "⏸" } else { "▶" };
        if ui.button(play_label).on_hover_text("再生/停止 (Enter)").clicked() {
            state.is_playing = !state.is_playing;
        }
        if ui.button("▶").on_hover_text("次フレーム").clicked() {
            let f = (state.current_frame + 1).min(total);
            state.set_frame(f); dirty = true;
        }
        if ui.button("▶|").on_hover_text("末尾へ").clicked() {
            state.set_frame(total); dirty = true;
        }

        ui.separator();
        ui.label(format!("{} / {}  {}fps", state.current_frame, total, fps));

        ui.separator();
        let onion_icon = if state.onion_skin { ph::EYE } else { ph::EYE_SLASH };
        if ui.button(onion_icon).on_hover_text("オニオンスキン").clicked() {
            state.onion_skin = !state.onion_skin; dirty = true;
        }
    });

    ui.separator();

    // Timeline scroll area
    let row_h = 20.0;
    let frame_w = 16.0;
    let header_w = 80.0;

    egui::ScrollArea::horizontal().id_salt("timeline_scroll").show(ui, |ui| {
        let current = state.current_frame;
        let sel_layer_id = state.selected_layer_id.clone();

        let mut new_frame: Option<u32> = None;
        let mut hold_set: Option<(String, u32)> = None;
        let mut hold_released: Option<(String, u32)> = None;
        let mut cam_kf_add: Option<u32> = None;
        let mut cam_kf_remove: Option<u32> = None;

        // ── Camera track row ──────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.add_sized([header_w - 22.0, row_h], egui::Label::new(
                RichText::new("カメラ").size(11.0)
            ));
            let has_kf_here = state.project.camera_track.keyframes.iter().any(|k| k.frame == current);
            if ui.add_sized([20.0, row_h], egui::Button::new(if has_kf_here { "−" } else { "+" }))
                .on_hover_text(if has_kf_here { "キーフレーム削除" } else { "キーフレーム挿入" })
                .clicked()
            {
                if has_kf_here { cam_kf_remove = Some(current); } else { cam_kf_add = Some(current); }
            }

            for f in 1..=total {
                let is_kf = state.project.camera_track.keyframes.iter().any(|k| k.frame == f);
                let is_current = f == current;
                let fill = if is_current {
                    Color32::from_rgb(60, 140, 220)
                } else if is_kf {
                    Color32::from_rgb(220, 140, 30)
                } else {
                    Color32::from_gray(25)
                };
                let resp = draw_frame_cell(ui, frame_w, row_h, fill, Color32::TRANSPARENT, is_kf, false);
                if resp.clicked() { new_frame = Some(f); }
                if resp.secondary_clicked() {
                    if is_kf { cam_kf_remove = Some(f); } else { cam_kf_add = Some(f); }
                }
            }
        });
        ui.separator();

        // ── Layer rows ────────────────────────────────────────────────────────
        let layers = &state.project.layers;
        for layer in layers.iter().rev() {
            ui.horizontal(|ui| {
                // Layer name column
                ui.add_sized([header_w, row_h], egui::Label::new(
                    RichText::new(&layer.name).size(11.0)
                ));

                // Frame cells
                for f in 1..=total {
                    let frame_data = layer.frames.iter().find(|fr| fr.frame == f);
                    let is_current = f == current;
                    let is_drawn = frame_data.map(|fd| fd.drawing_id.is_some()).unwrap_or(false);
                    let is_hold = frame_data.map(|fd| fd.hold_source.is_some()).unwrap_or(false);
                    let is_kf = frame_data.map(|fd| fd.is_keyframe).unwrap_or(false);

                    let (fill, stroke_col) = if is_current {
                        (Color32::from_rgb(60, 140, 220), Color32::WHITE)
                    } else if is_drawn || is_hold {
                        let c = if layer.id == sel_layer_id { Color32::from_rgb(90, 90, 180) } else { Color32::from_gray(90) };
                        (c, Color32::TRANSPARENT)
                    } else {
                        (Color32::from_gray(30), Color32::TRANSPARENT)
                    };

                    let resp = draw_frame_cell(ui, frame_w, row_h, fill, stroke_col, is_kf, is_hold);
                    if resp.clicked() {
                        new_frame = Some(f);
                    }
                    if resp.secondary_clicked() {
                        if is_hold {
                            hold_released = Some((layer.id.clone(), f));
                        } else {
                            hold_set = Some((layer.id.clone(), f));
                        }
                    }
                }
            });
        }

        if let Some(f) = new_frame { state.set_frame(f); dirty = true; }
        if let Some((lid, f)) = hold_set { state.set_koma_hold(f, &lid); dirty = true; }
        if let Some((lid, f)) = hold_released { state.release_koma_hold(f, &lid); dirty = true; }
        if let Some(f) = cam_kf_add {
            let mut new_kf = get_camera_at_frame(&state.project.camera_track.keyframes, f);
            new_kf.frame = f;
            state.project.camera_track.keyframes.push(new_kf);
            state.project.camera_track.keyframes.sort_by_key(|k| k.frame);
            dirty = true;
        }
        if let Some(f) = cam_kf_remove {
            if state.project.camera_track.keyframes.len() > 1 {
                state.project.camera_track.keyframes.retain(|k| k.frame != f);
                dirty = true;
            }
        }
    });

    dirty
}

fn draw_frame_cell(
    ui: &mut Ui,
    w: f32, h: f32,
    fill: Color32, stroke_col: Color32,
    is_keyframe: bool, is_hold: bool,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, fill);
        if stroke_col != Color32::TRANSPARENT {
            painter.rect_stroke(rect, 0.0, Stroke::new(1.0, stroke_col));
        }
        if is_keyframe {
            let cx = rect.center().x;
            let cy = rect.center().y;
            let s = 3.0;
            painter.circle_filled(egui::pos2(cx, cy), s, Color32::from_rgb(255, 200, 50));
        } else if is_hold {
            // Draw a small horizontal line to indicate hold
            let cy = rect.center().y;
            painter.hline(rect.left() + 2.0..=rect.right() - 2.0, cy, Stroke::new(2.0, Color32::from_gray(160)));
        }
    }
    resp
}
