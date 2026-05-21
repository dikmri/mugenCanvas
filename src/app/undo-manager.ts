type TileMap = Map<string, ImageData>;

interface SnapshotEntry {
  layerId: string;
  frame: number;
  tiles: TileMap;
}

export class UndoManager {
  private undoStack: SnapshotEntry[] = [];
  private redoStack: SnapshotEntry[] = [];
  private readonly maxSize = 20;
  private snapshotFn: (layerId: string, frame: number) => TileMap;
  private restoreFn: (layerId: string, frame: number, tiles: TileMap) => void;

  constructor(
    snapshotFn: (layerId: string, frame: number) => TileMap,
    restoreFn: (layerId: string, frame: number, tiles: TileMap) => void,
  ) {
    this.snapshotFn = snapshotFn;
    this.restoreFn = restoreFn;
  }

  snapshot(layerId: string, frame: number): void {
    this.undoStack.push({ layerId, frame, tiles: this.snapshotFn(layerId, frame) });
    if (this.undoStack.length > this.maxSize) this.undoStack.shift();
    this.redoStack = [];
  }

  undo(onRestored: (layerId: string, frame: number) => void): boolean {
    const entry = this.undoStack.pop();
    if (!entry) return false;
    const current = this.snapshotFn(entry.layerId, entry.frame);
    this.redoStack.push({ layerId: entry.layerId, frame: entry.frame, tiles: current });
    if (this.redoStack.length > this.maxSize) this.redoStack.shift();
    this.restoreFn(entry.layerId, entry.frame, entry.tiles);
    onRestored(entry.layerId, entry.frame);
    return true;
  }

  redo(onRestored: (layerId: string, frame: number) => void): boolean {
    const entry = this.redoStack.pop();
    if (!entry) return false;
    const current = this.snapshotFn(entry.layerId, entry.frame);
    this.undoStack.push({ layerId: entry.layerId, frame: entry.frame, tiles: current });
    this.restoreFn(entry.layerId, entry.frame, entry.tiles);
    onRestored(entry.layerId, entry.frame);
    return true;
  }

  get canUndo(): boolean { return this.undoStack.length > 0; }
  get canRedo(): boolean { return this.redoStack.length > 0; }
}
