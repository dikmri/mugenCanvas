# mugenCanvas

アニメーション制作用デスクトップアプリ。Tauri v2 + Vanilla TypeScript + Vite で構築。

## 主な機能

- **無限キャンバス** — タイル分割方式 (1024×1024px/タイル) により実質無限に描画可能
- **ブラシ / 消しゴム** — アンチエイリアス対応、サイズ・不透明度設定
- **塗りつぶしツール** — BFS flood fill
- **レイヤー** — 追加・削除・名前変更・表示非表示・ロック
- **タイムライン** — フレーム移動・再生・オニオンスキン
- **コマ (hold) システム** — 1コマ/2コマ/3コマ, `.` キーで割り当て/解除
- **カメラ枠** — 赤枠表示・ドラッグ移動・PNG 書き出し
- **アンドゥ/リドゥ** — スナップショット方式 20段
- **プロジェクト保存 / 読み込み** — `.mugencanvas` 形式 (JSON + タイル PNG base64)
- **ダークモード** — system / light / dark 切替

## 技術スタック

| 役割 | 技術 |
|------|------|
| フレームワーク | Tauri v2 |
| フロントエンド | Vanilla TypeScript + Vite |
| バックエンド | Rust |
| レンダリング | Canvas 2D API + OffscreenCanvas (タイル分割) |

## セットアップ

```bash
npm install
npm run dev        # ブラウザ開発サーバー
npm run tauri dev  # Tauri デスクトップアプリ
```

## キーボードショートカット

| キー | 操作 |
|------|------|
| B / E / H / Z / C | ブラシ / 消しゴム / 手 / ズーム / カメラ |
| Space + ドラッグ | パン |
| Ctrl + ホイール | ズーム |
| R + ドラッグ | キャンバス回転 |
| Ctrl+R | 回転リセット |
| ← → / , | フレーム移動 |
| Enter | 再生 / 停止 |
| . | コマ (hold) 割り当て / 解除 |
| Del | ブランク範囲削除 |
| Ctrl+Z / Ctrl+Y | アンドゥ / リドゥ |
| Ctrl+S / Ctrl+O | 保存 / 読み込み |
| Ctrl+N | 新規プロジェクト |

## ディレクトリ構成

```
src/
  app/          # AppState, UndoManager
  canvas/       # CanvasView, Renderer (タイル), BrushEngine, viewport
  panels/       # ToolsPanel, BrushColorPanel, TimelinePanel, FloatingPanel
  types/        # model.ts (型定義)
  ui/           # TopBar, StatusBar
  styles/       # CSS
src-tauri/      # Rust バックエンド
memo/           # 実装計画・仕様書・モックアップ
```
