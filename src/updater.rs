use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const API_URL: &str = "https://api.github.com/repos/dikmri/mugenCanvas/releases/latest";

// ─── Public types ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
}

pub type CheckResult = Result<Option<UpdateInfo>, String>;

pub struct ApplyState {
    pub downloaded: u64,
    pub total: u64,
    pub finished: Option<Result<(), String>>,
}

// ─── Background check ─────────────────────────────────────────────────────────

pub fn spawn_check(rx: Arc<Mutex<Option<CheckResult>>>) {
    std::thread::spawn(move || {
        let r = check_update();
        *rx.lock().unwrap() = Some(r);
    });
}

// ─── Core logic ───────────────────────────────────────────────────────────────

pub fn check_update() -> CheckResult {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .into();

    let mut resp = agent
        .get(API_URL)
        .header("User-Agent", "mugenCanvas-updater")
        .call()
        .map_err(|e| format!("ネットワークエラー: {}", e))?;

    let body = resp
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("レスポンス読み込み失敗: {}", e))?;

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("JSON解析失敗: {}", e))?;

    let tag = json["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v');

    if tag.is_empty() {
        return Ok(None);
    }
    if !is_newer(tag, CURRENT_VERSION) {
        return Ok(None);
    }

    let asset_name = platform_asset_name();
    if asset_name == "unknown" {
        return Ok(None);
    }

    let url = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets
                .iter()
                .find(|a| a["name"].as_str() == Some(asset_name))
                .and_then(|a| a["browser_download_url"].as_str())
        })
        .ok_or_else(|| {
            format!(
                "このプラットフォーム向けバイナリが見つかりません: {}",
                asset_name
            )
        })?;

    Ok(Some(UpdateInfo {
        version: tag.to_string(),
        download_url: url.to_string(),
    }))
}

pub fn spawn_apply(info: UpdateInfo, state: Arc<Mutex<ApplyState>>) {
    std::thread::spawn(move || {
        let result = apply_update(&info, |dl, total| {
            let mut s = state.lock().unwrap();
            s.downloaded = dl;
            s.total = total;
        });
        state.lock().unwrap().finished = Some(result);
    });
}

fn apply_update(info: &UpdateInfo, on_progress: impl Fn(u64, u64)) -> Result<(), String> {
    let agent: ureq::Agent = ureq::Agent::config_builder().build().into();

    let mut resp = agent
        .get(&info.download_url)
        .header("User-Agent", "mugenCanvas-updater")
        .call()
        .map_err(|e| format!("ダウンロードエラー: {}", e))?;

    let total = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let tmp = std::env::temp_dir().join(format!("mugen-canvas-{}.tmp", info.version));
    let mut file =
        std::fs::File::create(&tmp).map_err(|e| format!("一時ファイル作成失敗: {}", e))?;

    let mut reader = resp.body_mut().as_reader();
    let mut buf = [0u8; 65536];
    let mut downloaded = 0u64;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("読み込みエラー: {}", e))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .map_err(|e| format!("書き込みエラー: {}", e))?;
        downloaded += n as u64;
        on_progress(downloaded, total);
    }
    drop(file);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("権限設定失敗: {}", e))?;
    }

    self_replace::self_replace(&tmp)
        .map_err(|e| format!("バイナリ置換失敗: {}", e))?;
    let _ = std::fs::remove_file(&tmp);

    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

pub fn is_newer(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> (u32, u32, u32) {
        let mut p = s.splitn(3, '.');
        let a = p.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        let b = p.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        let c = p.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        (a, b, c)
    };
    parse(latest) > parse(current)
}

fn platform_asset_name() -> &'static str {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "mugen-canvas-windows-x86_64.exe"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "mugen-canvas-macos-aarch64"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "mugen-canvas-macos-x86_64"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "mugen-canvas-linux-x86_64"
    } else {
        "unknown"
    }
}

// ─── Update dialog state (shared with app.rs) ─────────────────────────────────

pub enum UpdateDialog {
    Hidden,
    Checking,
    Available(UpdateInfo),
    Applying(Arc<Mutex<ApplyState>>),
    Done(Instant),      // timestamp when done was set; relaunch after brief delay
    NotFound,
    Error(String),
}
