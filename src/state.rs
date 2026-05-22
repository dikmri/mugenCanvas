use crate::model::*;

pub struct AppState {
    pub project: ProjectFile,
    pub current_frame: u32,
    pub selected_layer_id: String,
    pub selected_tool: Tool,
    pub viewport: Viewport,
    pub brush: BrushSettings,
    pub eraser: BrushSettings,
    pub pen: BrushSettings,
    pub is_playing: bool,
    pub onion_skin: bool,
    pub onion_skin_settings: OnionSkinSettings,
    pub project_path: Option<String>,
    pub layer_counter: u32,
    // Grid
    pub show_grid: bool,
    pub grid_size: u32,
}

impl Default for AppState {
    fn default() -> Self {
        let project = ProjectFile::default();
        let selected_layer_id = project.layers.last().map(|l| l.id.clone()).unwrap_or_default();
        Self {
            project,
            current_frame: 1,
            selected_layer_id,
            selected_tool: Tool::Brush,
            viewport: Viewport::default(),
            brush: BrushSettings::default(),
            eraser: BrushSettings {
                size: 30.0, opacity: 1.0, color: [0, 0, 0], anti_alias: true, min_size: 1.0,
            },
            pen: BrushSettings {
                size: 10.0, opacity: 1.0, color: [0x22, 0x22, 0x22], anti_alias: false, min_size: 0.5,
            },
            is_playing: false,
            onion_skin: false,
            onion_skin_settings: OnionSkinSettings::default(),
            project_path: None,
            layer_counter: 2,
            show_grid: false,
            grid_size: 100,
        }
    }
}

impl AppState {
    pub fn set_frame(&mut self, frame: u32) {
        let total = self.project.settings.total_frames;
        self.current_frame = frame.clamp(1, total);
    }

    pub fn advance_frame(&mut self) {
        let total = self.project.settings.total_frames;
        self.current_frame = if self.current_frame >= total { 1 } else { self.current_frame + 1 };
    }

    pub fn add_layer(&mut self) {
        self.layer_counter += 1;
        let id = format!("layer-{}", self.layer_counter);
        let total = self.project.settings.total_frames;
        let name = format!("レイヤー {}", self.layer_counter);
        let layer = make_layer(&id, &name, total);
        self.selected_layer_id = id.clone();
        self.project.layers.push(layer);
    }

    pub fn remove_layer(&mut self, id: &str) {
        if self.project.layers.len() <= 1 { return; }
        self.project.layers.retain(|l| l.id != id);
        if self.selected_layer_id == id {
            self.selected_layer_id = self.project.layers.last()
                .map(|l| l.id.clone()).unwrap_or_default();
        }
    }

    pub fn move_layer_up(&mut self, id: &str) {
        let layers = &mut self.project.layers;
        if let Some(idx) = layers.iter().position(|l| l.id == id) {
            if idx + 1 < layers.len() {
                layers.swap(idx, idx + 1);
            }
        }
    }

    pub fn move_layer_down(&mut self, id: &str) {
        let layers = &mut self.project.layers;
        if let Some(idx) = layers.iter().position(|l| l.id == id) {
            if idx > 0 {
                layers.swap(idx, idx - 1);
            }
        }
    }

    pub fn mark_frame_drawn(&mut self, layer_id: &str, frame: u32) {
        for layer in &mut self.project.layers {
            if layer.id == layer_id {
                for f in &mut layer.frames {
                    if f.frame == frame && f.drawing_id.is_none() {
                        f.drawing_id = Some("drawn".to_string());
                    }
                }
                break;
            }
        }
    }

    pub fn selected_layer(&self) -> Option<&AnimationLayer> {
        self.project.layers.iter().find(|l| l.id == self.selected_layer_id)
    }

    pub fn active_brush(&self) -> &BrushSettings {
        match self.selected_tool {
            Tool::Eraser => &self.eraser,
            Tool::Pen => &self.pen,
            _ => &self.brush,
        }
    }

    /// Returns true if the current frame on the selected layer is drawable.
    pub fn can_paint(&self) -> bool {
        let layer = match self.selected_layer() {
            Some(l) => l,
            None => return false,
        };
        if layer.locked { return false; }
        let frame_data = layer.frames.iter().find(|f| f.frame == self.current_frame);
        if let Some(f) = frame_data {
            if f.hold_source.is_some() { return false; }
        }
        true
    }

    pub fn set_koma_hold(&mut self, frame: u32, layer_id: &str) {
        let layer = match self.project.layers.iter_mut().find(|l| l.id == layer_id) {
            Some(l) => l,
            None => return,
        };
        // Find nearest drawn frame before 'frame'
        let mut source_frame: Option<u32> = None;
        for f in (1..frame).rev() {
            if let Some(fr) = layer.frames.iter().find(|fr| fr.frame == f) {
                if fr.drawing_id.is_some() { source_frame = Some(f); break; }
                if let Some(src) = fr.hold_source { source_frame = Some(src); break; }
            }
        }
        let src = match source_frame { Some(s) => s, None => return };
        for fr in &mut layer.frames {
            if fr.frame > src && fr.frame <= frame && fr.drawing_id.is_none() {
                fr.hold_source = Some(src);
            }
        }
    }

    pub fn release_koma_hold(&mut self, frame: u32, layer_id: &str) {
        let layer = match self.project.layers.iter_mut().find(|l| l.id == layer_id) {
            Some(l) => l,
            None => return,
        };
        let target = match layer.frames.iter().find(|f| f.frame == frame) {
            Some(f) => f,
            None => return,
        };
        let src = match target.hold_source { Some(s) => s, None => return };
        for fr in &mut layer.frames {
            if fr.frame >= frame && fr.hold_source == Some(src) {
                fr.hold_source = None;
            }
        }
    }
}
