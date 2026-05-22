use egui::{Ui, RichText};
use egui_phosphor::regular as ph;
use crate::state::AppState;

pub enum TopbarAction {
    New,
    Open,
    Save,
    ExportPng,
    ExportPngSequence,
    ExportGif,
    ExportMp4,
    Undo,
    Redo,
    ToggleGrid,
    None,
}

pub fn show(ui: &mut Ui, state: &mut AppState, can_undo: bool, can_redo: bool) -> TopbarAction {
    let mut action = TopbarAction::None;

    ui.horizontal(|ui| {
        ui.label(RichText::new("mugenCanvas").strong().size(14.0));
        ui.separator();

        if ui.button(ph::FILE_PLUS).on_hover_text("新規 (Ctrl+N)").clicked()  { action = TopbarAction::New; }
        if ui.button(ph::FOLDER_OPEN).on_hover_text("開く (Ctrl+O)").clicked() { action = TopbarAction::Open; }
        if ui.button(ph::FLOPPY_DISK).on_hover_text("保存 (Ctrl+S)").clicked() { action = TopbarAction::Save; }
        ui.separator();

        if ui.add_enabled(can_undo, egui::Button::new(ph::ARROW_COUNTER_CLOCKWISE))
            .on_hover_text("Undo (Ctrl+Z)").clicked() { action = TopbarAction::Undo; }
        if ui.add_enabled(can_redo, egui::Button::new(ph::ARROW_CLOCKWISE))
            .on_hover_text("Redo (Ctrl+Y)").clicked() { action = TopbarAction::Redo; }
        ui.separator();

        if ui.button(ph::UPLOAD_SIMPLE).on_hover_text("現在フレームをPNG書き出し").clicked()      { action = TopbarAction::ExportPng; }
        if ui.button(ph::IMAGES).on_hover_text("連番PNG書き出し (フレーム範囲指定)").clicked()    { action = TopbarAction::ExportPngSequence; }
        if ui.button(ph::FILM_STRIP).on_hover_text("GIFアニメーション書き出し (フレーム範囲指定)").clicked() { action = TopbarAction::ExportGif; }
        if ui.button(ph::VIDEO_CAMERA).on_hover_text("MP4動画書き出し (ffmpeg必須, フレーム範囲指定)").clicked() { action = TopbarAction::ExportMp4; }
        ui.separator();

        let grid_icon = if state.show_grid { ph::GRID_FOUR } else { ph::GRID_FOUR };
        let grid_tint = if state.show_grid { egui::Color32::from_rgb(100, 180, 255) } else { egui::Color32::GRAY };
        if ui.add(egui::Button::new(egui::RichText::new(grid_icon).color(grid_tint)))
            .on_hover_text(format!("グリッド表示 (Ctrl+G)  現在: {}px", state.grid_size))
            .clicked() { action = TopbarAction::ToggleGrid; }
        ui.separator();

        // Project info
        let s = &state.project.settings;
        ui.label(format!("{}×{}px / {}fps / {}f", s.width, s.height, s.fps, s.total_frames));
        ui.separator();

        // Viewport zoom select
        ui.label("ズーム:");
        let zoom_pct = (state.viewport.zoom * 100.0).round() as i32;
        egui::ComboBox::from_id_salt("zoom_select")
            .selected_text(format!("{}%", zoom_pct))
            .width(80.0)
            .show_ui(ui, |ui| {
                for &pct in &[12, 25, 50, 75, 100, 150, 200, 400] {
                    if ui.selectable_label(zoom_pct == pct, format!("{}%", pct)).clicked() {
                        state.viewport.zoom = pct as f32 / 100.0;
                    }
                }
            });

        // Rotation indicator
        let rot_deg = (state.viewport.rotation * 180.0 / std::f32::consts::PI).round() as i32;
        if rot_deg != 0 {
            if ui.button(format!("{}°↺", rot_deg)).on_hover_text("回転リセット (Ctrl+R)").clicked() {
                state.viewport.rotation = 0.0;
            }
        }
    });

    action
}
