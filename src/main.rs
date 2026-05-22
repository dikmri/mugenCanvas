#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod canvas;
mod io;
mod model;
mod state;
mod ui;
mod undo;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("mugenCanvas")
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "mugenCanvas",
        options,
        Box::new(|cc| Ok(Box::new(app::MugenCanvasApp::new(cc)))),
    )
}
