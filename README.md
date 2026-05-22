# mugenCanvas

アニメーション制作用デスクトップアプリ。Rust + eframe (egui + wgpu + winit) で構築。

## ダウンロード

| プラットフォーム | ダウンロード |
|---|---|
| Windows (x86_64) | [mugen-canvas-windows-x86_64.exe](https://github.com/dikmri/mugenCanvas/releases/latest/download/mugen-canvas-windows-x86_64.exe) |
| macOS (Apple Silicon) | [mugen-canvas-macos-aarch64](https://github.com/dikmri/mugenCanvas/releases/latest/download/mugen-canvas-macos-aarch64) |
| macOS (Intel) | [mugen-canvas-macos-x86_64](https://github.com/dikmri/mugenCanvas/releases/latest/download/mugen-canvas-macos-x86_64) |
| Linux (x86_64) | [mugen-canvas-linux-x86_64](https://github.com/dikmri/mugenCanvas/releases/latest/download/mugen-canvas-linux-x86_64) |

→ [全リリース一覧](https://github.com/dikmri/mugenCanvas/releases)

## 主な機能

- **無限キャンバス** — タイル分割方式 (1024×1024px/タイル) により実質無限に描画可能
- **ブラシ / 消しゴム** — アンチエイリアス対応、サイズ・不透明度設定
- **塗りつぶしツール** — BFS flood fill
- **レイヤー** — 追加・削除・名前変更・表示非表示・ロック・並び替え
- **タイムライン** — フレーム移動・再生・オニオンスキン
- **コマ (hold) システム** — 右クリックでコマ割り当て/解除
- **カメラ枠** — 赤枠表示・ドラッグ移動・キーフレーム線形補間
- **アンドゥ/リドゥ** — スナップショット方式 20段
- **プロジェクト保存 / 読み込み** — `.mugencanvas` 形式 (JSON + タイル PNG base64)
- **PNG / GIF 書き出し** — ネイティブファイルダイアログ、並列 GIF エンコード

## 技術スタック

| 役割 | 技術 |
|------|------|
| ウィンドウ・入力 | winit (eframe 経由) |
| GPU レンダリング | wgpu (eframe 経由) |
| UI フレームワーク | egui 0.29 |
| キャンバス描画 | CPU タイル合成 → egui TextureHandle |
| ファイルダイアログ | rfd 0.15 |
| GIF エンコード | gif + rayon (並列 NeuQuant) |
| 保存フォーマット | serde_json + base64 PNG タイル |

## セットアップ / 実行

Rust toolchain (stable) が必要。Node.js / npm は不要。

```bash
cargo run            # 開発実行 (プロジェクトルートから)
cargo build --release  # リリースビルド
```

## キーボードショートカット

| キー | 操作 |
|------|------|
| B / E / F / H / Z / C | ブラシ / 消しゴム / 塗りつぶし / 手 / ズーム / カメラ |
| Space + ドラッグ | パン |
| Ctrl + ホイール | ズーム |
| ← → | フレーム移動 |
| Enter | 再生 / 停止 |
| Ctrl+Z / Ctrl+Y | アンドゥ / リドゥ |
| Ctrl+S / Ctrl+O | 保存 / 読み込み |
| Ctrl+N | 新規プロジェクト |

## ディレクトリ構成

```
src/
  main.rs          # エントリーポイント (eframe::run_native)
  app.rs           # MugenCanvasApp (eframe::App 実装、入力処理)
  model.rs         # データモデル (serde camelCase)
  state.rs         # AppState
  undo.rs          # UndoManager (スナップショット方式)
  canvas/
    mod.rs         # CanvasState, タイル管理, CPU 合成, エクスポート
    brush.rs       # draw_segment, flood_fill
    viewport.rs    # 座標変換 (screen ↔ world)
    camera.rs      # カメラキーフレーム線形補間
  ui/
    topbar.rs      # トップバー (メニュー・操作)
    toolbar.rs     # ツール選択
    color_panel.rs # カラー・ブラシ設定
    layers_panel.rs  # レイヤー一覧
    timeline.rs    # タイムライン
  io.rs            # 保存・読み込み・PNG/GIF 書き出し
memo/              # 実装計画・仕様書・モックアップ
```
