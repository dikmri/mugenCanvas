use egui::{Ui, RichText, Color32};
use egui_phosphor::regular as ph;
use crate::model::Tool;
use crate::state::AppState;

pub fn show(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut dirty = false;
    ui.vertical(|ui| {
        ui.add_space(4.0);
        dirty |= tool_btn(ui, state, Tool::Brush,  ph::PENCIL,            "ブラシ",     "ブラシ (B)");
        dirty |= tool_btn(ui, state, Tool::Eraser, ph::ERASER,            "消しゴム",   "消しゴム (E)");
        dirty |= tool_btn(ui, state, Tool::Fill,   ph::PAINT_BUCKET,      "塗りつぶし", "塗りつぶし (F)");
        ui.separator();
        dirty |= tool_btn(ui, state, Tool::Hand,   ph::HAND,                   "手のひら",   "手のひら (H)");
        dirty |= tool_btn(ui, state, Tool::Zoom,   ph::MAGNIFYING_GLASS,       "ズーム",     "ズーム (Z)");
        dirty |= tool_btn(ui, state, Tool::Rotate, ph::ARROWS_CLOCKWISE,       "回転",       "回転 (R)");
        ui.separator();
        dirty |= tool_btn(ui, state, Tool::Camera, ph::FRAME_CORNERS,          "カメラ",     "カメラ (C)");
    });
    dirty
}

fn tool_btn(ui: &mut Ui, state: &mut AppState, tool: Tool, icon: &str, label: &str, tooltip: &str) -> bool {
    let selected = state.selected_tool == tool;
    let color = if selected { Color32::from_rgb(60, 120, 220) } else { Color32::TRANSPARENT };
    let text = format!("{} {}", icon, label);
    let btn = egui::Button::new(RichText::new(text).size(15.0))
        .fill(color)
        .min_size(egui::vec2(100.0, 30.0));
    let resp = ui.add(btn).on_hover_text(tooltip);
    if resp.clicked() && !selected {
        state.selected_tool = tool;
        return true;
    }
    false
}
