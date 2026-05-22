use egui::{Ui, RichText, Color32};
use egui_phosphor::regular as ph;
use crate::model::Tool;
use crate::state::AppState;

pub fn show(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut dirty = false;
    ui.vertical(|ui| {
        ui.add_space(4.0);
        dirty |= tool_btn(ui, state, Tool::Brush,  ph::PENCIL,            "ブラシ (B)");
        dirty |= tool_btn(ui, state, Tool::Eraser, ph::ERASER,            "消しゴム (E)");
        dirty |= tool_btn(ui, state, Tool::Fill,   ph::PAINT_BUCKET,      "塗りつぶし (F)");
        ui.separator();
        dirty |= tool_btn(ui, state, Tool::Hand,   ph::HAND,              "手のひら (H)");
        dirty |= tool_btn(ui, state, Tool::Zoom,   ph::MAGNIFYING_GLASS,  "ズーム (Z)");
        ui.separator();
        dirty |= tool_btn(ui, state, Tool::Camera, ph::FRAME_CORNERS,     "カメラ (C)");
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
