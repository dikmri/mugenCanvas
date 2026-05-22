use egui::{Ui, Color32};
use crate::model::Tool;
use crate::state::AppState;

pub fn show(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut dirty = false;
    ui.add_space(4.0);

    match state.selected_tool {
        Tool::Eraser => {
            ui.label("消しゴム");
            ui.separator();
            dirty |= eraser_controls(ui, state);
        }
        Tool::Pen => {
            ui.label("丸ペン");
            ui.separator();
            dirty |= pen_controls(ui, state);
        }
        _ => {
            ui.label("ブラシ");
            ui.separator();
            dirty |= brush_controls(ui, state);
        }
    }
    dirty
}

fn brush_controls(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut dirty = false;
    let brush = &mut state.brush;

    // Color picker
    let [r, g, b] = brush.color;
    let mut egui_color = Color32::from_rgb(r, g, b);
    ui.label("色");
    if ui.color_edit_button_srgba(&mut egui_color).changed() {
        brush.color = [egui_color.r(), egui_color.g(), egui_color.b()];
        dirty = true;
    }

    ui.add_space(4.0);
    ui.label("サイズ");
    if ui.add(egui::Slider::new(&mut brush.size, 1.0..=200.0).suffix("px")).changed() {
        dirty = true;
    }

    ui.label("不透明度");
    if ui.add(egui::Slider::new(&mut brush.opacity, 0.01..=1.0).custom_formatter(|v, _| format!("{:.0}%", v * 100.0))).changed() {
        dirty = true;
    }

    dirty
}

fn pen_controls(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut dirty = false;
    let pen = &mut state.pen;

    let [r, g, b] = pen.color;
    let mut egui_color = Color32::from_rgb(r, g, b);
    ui.label("色");
    if ui.color_edit_button_srgba(&mut egui_color).changed() {
        pen.color = [egui_color.r(), egui_color.g(), egui_color.b()];
        dirty = true;
    }
    ui.add_space(4.0);
    ui.label("サイズ");
    if ui.add(egui::Slider::new(&mut pen.size, 1.0..=200.0).suffix("px")).changed() {
        dirty = true;
    }
    ui.label("不透明度");
    if ui.add(egui::Slider::new(&mut pen.opacity, 0.01..=1.0).custom_formatter(|v, _| format!("{:.0}%", v * 100.0))).changed() {
        dirty = true;
    }
    dirty
}

fn eraser_controls(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut dirty = false;
    let eraser = &mut state.eraser;

    ui.label("サイズ");
    if ui.add(egui::Slider::new(&mut eraser.size, 1.0..=400.0).suffix("px")).changed() {
        dirty = true;
    }
    dirty
}
