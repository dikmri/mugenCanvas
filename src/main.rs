#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod canvas;
mod io;
mod model;
mod state;
mod ui;
mod undo;
mod updater;

fn main() -> eframe::Result<()> {
    setup_crash_logging();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("mugenCanvas")
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "mugenCanvas",
        options,
        Box::new(|cc| Ok(Box::new(app::MugenCanvasApp::new(cc)))),
    )
}

// ─── Crash logging ─────────────────────────────────────────────────────────────

fn get_log_path() -> std::path::PathBuf {
    // Windows: %APPDATA%\mugenCanvas\crash.log
    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = std::path::Path::new(&appdata).join("mugenCanvas");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("crash.log");
    }
    // macOS: ~/Library/Logs/mugenCanvas/crash.log
    #[cfg(target_os = "macos")]
    if let Ok(home) = std::env::var("HOME") {
        let dir = std::path::Path::new(&home)
            .join("Library").join("Logs").join("mugenCanvas");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("crash.log");
    }
    // Linux: ~/.local/share/mugenCanvas/crash.log
    #[cfg(target_os = "linux")]
    if let Ok(home) = std::env::var("HOME") {
        let dir = std::path::Path::new(&home)
            .join(".local").join("share").join("mugenCanvas");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("crash.log");
    }
    // Fallback: exe ディレクトリ
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            return parent.join("mugenCanvas_crash.log");
        }
    }
    std::path::PathBuf::from("mugenCanvas_crash.log")
}

fn setup_crash_logging() {
    use std::io::Write;

    let log_path = get_log_path();

    std::panic::set_hook(Box::new(move |panic_info| {
        let backtrace = std::backtrace::Backtrace::force_capture();

        // 日時 (Unix タイムスタンプ)
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let msg = format!(
            "\n=== mugenCanvas crash (t={}) ===\n{}\n\nbacktrace:\n{}\n",
            ts, panic_info, backtrace
        );

        // ファイルに追記
        let write_result = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .and_then(|mut f| f.write_all(msg.as_bytes()));

        // ログパスをエラー出力にも流す (コンソール版ビルド / デバッグ時)
        eprintln!("{}", msg);
        match write_result {
            Ok(_)  => eprintln!("[crash log] -> {:?}", log_path),
            Err(e) => eprintln!("[crash log] write failed: {}", e),
        }
    }));
}
