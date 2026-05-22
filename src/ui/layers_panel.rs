use egui::{Ui, Color32, RichText};
use crate::canvas::CanvasState;
use crate::state::AppState;

/// Returns (dirty, layer_removed_id)
pub fn show(ui: &mut Ui, state: &mut AppState, canvas: &mut CanvasState) -> (bool, Option<String>) {
    let mut dirty = false;
    let mut removed: Option<String> = None;
    let mut move_up: Option<String> = None;
    let mut move_down: Option<String> = None;

    ui.horizontal(|ui| {
        ui.heading("レイヤー");
        if ui.button("+").on_hover_text("レイヤー追加").clicked() {
            state.add_layer();
            dirty = true;
        }
    });
    ui.separator();

    // Show layers in reverse order (topmost first in list)
    let layer_ids: Vec<String> = state.project.layers.iter().rev().map(|l| l.id.clone()).collect();

    for layer_id in layer_ids {
        let layer = match state.project.layers.iter_mut().find(|l| l.id == layer_id) {
            Some(l) => l,
            None => continue,
        };
        let selected = layer.id == state.selected_layer_id;
        let bg = if selected { Color32::from_rgb(50, 90, 160) } else { Color32::TRANSPARENT };

        let resp = egui::Frame::none().fill(bg).inner_margin(4.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Visibility toggle
                    let eye = if layer.visible { "👁" } else { "🚫" };
                    if ui.small_button(eye).on_hover_text("表示切替").clicked() {
                        layer.visible = !layer.visible;
                        dirty = true;
                    }
                    // Lock toggle
                    let lock_icon = if layer.locked { "🔒" } else { "🔓" };
                    if ui.small_button(lock_icon).on_hover_text("ロック切替").clicked() {
                        layer.locked = !layer.locked;
                    }
                    // Name (editable on double click — simplified: just show it)
                    ui.label(RichText::new(&layer.name).color(if selected { Color32::WHITE } else { Color32::GRAY }));
                });
            });

        if resp.response.clicked() {
            state.selected_layer_id = layer_id.clone();
        }

        // Context: up/down/delete buttons
        ui.horizontal(|ui| {
            if ui.small_button("↑").on_hover_text("上へ").clicked() {
                move_up = Some(layer_id.clone());
            }
            if ui.small_button("↓").on_hover_text("下へ").clicked() {
                move_down = Some(layer_id.clone());
            }
            if ui.small_button("🗑").on_hover_text("削除").clicked() {
                if state.project.layers.len() > 1 {
                    removed = Some(layer_id.clone());
                    dirty = true;
                }
            }
        });
        ui.separator();
    }

    if let Some(id) = move_up { state.move_layer_up(&id); dirty = true; }
    if let Some(id) = move_down { state.move_layer_down(&id); dirty = true; }
    if let Some(ref id) = removed {
        canvas.remove_layer(id);
        state.remove_layer(id);
    }

    (dirty, removed)
}
