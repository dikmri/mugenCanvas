use std::collections::HashMap;

pub struct SnapshotEntry {
    pub layer_id: String,
    pub frame: u32,
    pub tiles: HashMap<(i32, i32), Vec<u8>>,
}

pub struct UndoManager {
    undo_stack: Vec<SnapshotEntry>,
    redo_stack: Vec<SnapshotEntry>,
    max_size: usize,
}

impl Default for UndoManager {
    fn default() -> Self {
        Self { undo_stack: Vec::new(), redo_stack: Vec::new(), max_size: 20 }
    }
}

impl UndoManager {
    pub fn snapshot(&mut self, tiles: HashMap<(i32, i32), Vec<u8>>, layer_id: &str, frame: u32) {
        self.undo_stack.push(SnapshotEntry {
            layer_id: layer_id.to_string(),
            frame,
            tiles,
        });
        if self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current_tiles: HashMap<(i32, i32), Vec<u8>>) -> Option<SnapshotEntry> {
        let entry = self.undo_stack.pop()?;
        self.redo_stack.push(SnapshotEntry {
            layer_id: entry.layer_id.clone(),
            frame: entry.frame,
            tiles: current_tiles,
        });
        if self.redo_stack.len() > self.max_size {
            self.redo_stack.remove(0);
        }
        Some(entry)
    }

    pub fn redo(&mut self, current_tiles: HashMap<(i32, i32), Vec<u8>>) -> Option<SnapshotEntry> {
        let entry = self.redo_stack.pop()?;
        self.undo_stack.push(SnapshotEntry {
            layer_id: entry.layer_id.clone(),
            frame: entry.frame,
            tiles: current_tiles,
        });
        if self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
        Some(entry)
    }

    pub fn can_undo(&self) -> bool { !self.undo_stack.is_empty() }
    pub fn can_redo(&self) -> bool { !self.redo_stack.is_empty() }
    pub fn clear(&mut self) { self.undo_stack.clear(); self.redo_stack.clear(); }
}
