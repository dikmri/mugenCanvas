use egui::{Ui, RichText, Color32};
use crate::model::Tool;
use crate::state::AppState;

pub fn show(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut dirty = false;
    ui.vertical(|ui| {
        ui.add_space(4.0);
        dirty |= tool_btn(ui, state, Tool::Brush,  "✏", "ブラシ (B)");
        dirty |= tool_btn(ui, state, Tool::Eraser, "⌫", "消しゴム (E)");
        dirty |= tool_btn(ui, state, Tool::Fill,   "🪣", "塗りつぶし (G)");
        ui.separator();
        dirty |= tool_btn(ui, state, Tool::Hand,   "✋", "手のひら (H)");
        dirty |= tool_btn(ui, state, Tool::Zoom,   "🔍", "ズーム (Z)");
        ui.separator();
        dirty |= tool_btn(ui, state, Tool::Camera, "🎬", "カメラ (C)");
    });
    dirty
}

fn tool_btn(ui: &mut Ui, state: &mut AppState, tool: Tool, icon: &str, tooltip: &str) -> bool {
    let selected = state.selected_tool == tool;
    let color = if selected { Color32::from_rgb(60, 120, 220) } else { Color32::TRANSPARENT };
    let btn = egui::Button::new(RichText::new(icon).size(20.0))
        .fill(color)
        .min_size(egui::vec2(36.0, 36.0));
    let resp = ui.add(btn).on_hover_text(tooltip);
    if resp.clicked() && !selected {
        state.selected_tool = tool;
        return true;
    }
    false
}
