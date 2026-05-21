// Tauri IPC bridge — gracefully falls back when running in plain browser
type InvokeFn = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
type SaveFn = (opts?: object) => Promise<string | null>;
type OpenFn = (opts?: object) => Promise<string | string[] | null>;

let _invoke: InvokeFn | null = null;
let _dialogSave: SaveFn | null = null;
let _dialogOpen: OpenFn | null = null;

async function loadTauri(): Promise<void> {
  if (_invoke) return;
  try {
    const core = await import('@tauri-apps/api/core');
    _invoke = core.invoke as InvokeFn;
    // Try to load dialog plugin dynamically
    // dialog plugin may not be installed in all environments
    _dialogSave = async () => null;
    _dialogOpen = async () => null;
  } catch {
    throw new Error('Not running in Tauri');
  }
}

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  await loadTauri();
  return _invoke!<T>(cmd, args);
}

export const dialog = {
  async save(opts?: object): Promise<string | null> {
    await loadTauri();
    return _dialogSave!(opts);
  },
  async open(opts?: object): Promise<string | string[] | null> {
    await loadTauri();
    return _dialogOpen!(opts);
  },
};
