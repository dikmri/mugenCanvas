use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tool {
    Brush,
    Eraser,
    Fill,
    Hand,
    Zoom,
    Camera,
}

impl Default for Tool {
    fn default() -> Self { Tool::Brush }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub total_frames: u32,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self { width: 1920, height: 1080, fps: 24, total_frames: 60 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraKeyframe {
    pub frame: u32,
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    pub rotation: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraTrack {
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub keyframes: Vec<CameraKeyframe>,
}

impl Default for CameraTrack {
    fn default() -> Self {
        Self {
            name: "カメラ".to_string(),
            visible: true,
            locked: false,
            keyframes: vec![CameraKeyframe {
                frame: 1, x: 0.0, y: 0.0, scale: 1.0, rotation: 0.0,
                width: 1920.0, height: 1080.0,
            }],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationFrame {
    pub frame: u32,
    pub drawing_id: Option<String>,
    pub hold_source: Option<u32>,
    pub is_keyframe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationLayer {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub frames: Vec<AnimationFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrushSettings {
    pub size: f32,
    pub opacity: f32,
    /// RGB 0-255
    pub color: [u8; 3],
    pub anti_alias: bool,
    pub min_size: f32,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self { size: 15.0, opacity: 1.0, color: [0x22, 0x22, 0x22], anti_alias: true, min_size: 0.2 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnionSkinSettings {
    pub prev_color: [u8; 3],
    pub prev_alpha: f32,
    pub next_color: [u8; 3],
    pub next_alpha: f32,
}

impl Default for OnionSkinSettings {
    fn default() -> Self {
        Self {
            prev_color: [0x4a, 0x90, 0xe2],
            prev_alpha: 0.5,
            next_color: [0xff, 0x4b, 0x4b],
            next_alpha: 0.4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Viewport {
    pub offset_x: f32,
    pub offset_y: f32,
    pub zoom: f32,
    pub rotation: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self { offset_x: 0.0, offset_y: 0.0, zoom: 1.0, rotation: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFile {
    pub app: String,
    pub version: String,
    pub settings: ProjectSettings,
    pub viewport: Viewport,
    pub camera_track: CameraTrack,
    pub layers: Vec<AnimationLayer>,
}

impl Default for ProjectFile {
    fn default() -> Self {
        let settings = ProjectSettings::default();
        let total = settings.total_frames;
        Self {
            app: "mugenCanvas".to_string(),
            version: "0.1.0".to_string(),
            settings,
            viewport: Viewport::default(),
            camera_track: CameraTrack::default(),
            layers: vec![
                make_layer("layer-bg", "背景", total),
                make_layer("layer-1", "レイヤー 1", total),
            ],
        }
    }
}

pub fn make_layer(id: &str, name: &str, total_frames: u32) -> AnimationLayer {
    AnimationLayer {
        id: id.to_string(),
        name: name.to_string(),
        visible: true,
        locked: false,
        frames: (1..=total_frames).map(|f| AnimationFrame {
            frame: f,
            drawing_id: None,
            hold_source: None,
            is_keyframe: f == 1,
        }).collect(),
    }
}

/// Given a layer and frame number, return the frame index that contains the actual
/// pixel data (accounting for hold-source chains). Returns None if blank.
pub fn resolve_layer_frame(layer: &AnimationLayer, frame: u32) -> Option<u32> {
    let f = layer.frames.iter().find(|f| f.frame == frame)?;
    if f.drawing_id.is_some() { return Some(frame); }
    if let Some(src) = f.hold_source { return Some(src); }
    None
}
