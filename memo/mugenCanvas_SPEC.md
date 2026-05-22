# mugenCanvas 仕様書 / Claude Code 実装指示書

## 0. このドキュメントの目的

このドキュメントは、クロスプラットフォーム対応のデスクトップアニメーション制作アプリ **mugenCanvas** の仕様書です。

mugenCanvas は、**Photoshop 的な描き心地**と、**Adobe Animate 的なタイムライン中心のアニメ制作体験**を組み合わせた、純粋 Rust 製のデスクトップアプリです。

> **2026-05-22 移行完了:** Tauri + TypeScript 構成から **純粋 Rust + eframe (egui 0.29 / wgpu / winit)** 構成へ全面移行済み。

---

## 1. アプリ概要

### 1.1 アプリ名

**mugenCanvas**

### 1.2 コンセプト

> 無限に広がるキャンバス上で、Photoshop のように気持ちよく描きながら、Animate のようなタイムラインでアニメーションを作れるクロスプラットフォームアプリ。

### 1.3 対応プラットフォーム

初期ターゲット:

- Windows 10 / 11
- macOS
- Linux

### 1.4 技術スタック

| 役割 | 技術 |
|------|------|
| ウィンドウ・入力 | winit (eframe 0.29 経由) |
| GPU レンダリング | wgpu (eframe 0.29 経由) |
| UI フレームワーク | egui 0.29 |
| キャンバス合成 | CPU (RGBA バッファ) → egui TextureHandle |
| ファイルダイアログ | rfd 0.15 |
| GIF エンコード | gif 0.13 + rayon (並列 NeuQuant) |
| 保存フォーマット | serde_json + base64 + PNG (image crate) |

ビルド / 実行:

```bash
# プロジェクトルートから (npm / Node.js 不要)
cargo run
cargo build --release
```

---

## 2. 重要な設計思想

### 2.1 UI の基本思想

mugenCanvas の UI は、**余白のある自由配置型 UI** とします。

各パネルは egui の SidePanel / BottomPanel として配置し、中央が描画エリアになります。

- 左パネル: ツール選択
- 右パネル: カラー・ブラシ設定、レイヤー一覧
- 下パネル: タイムライン
- 中央: 無限キャンバスエリア

### 2.2 キャンバスの基本思想

画面には**無限に広い白いキャンバス**が存在します。

ユーザーは上下左右へ自由にパンできます。中央付近に赤いカメラ枠が表示され、この赤い枠の中にある絵だけが最終映像として出力されます。

キャンバスは CPU でタイル合成し、egui の TextureHandle として毎フレーム更新します。

### 2.3 カメラトラックの基本思想

赤い枠は単なるガイドではなく、**カメラトラック**です。

カメラトラックで管理する値:

- x 座標 / y 座標 (ワールド座標)
- scale
- rotation
- width / height

カメラキーフレームをフレーム間で線形補間することでカメラワークを表現します。

---

## 3. 画面構成

### 3.1 メイン画面

- 上部トップバー
- 左パネル: ツール選択ボタン
- 右パネル: カラー・ブラシ設定 / レイヤー一覧
- 下パネル: タイムライン
- 中央: 無限キャンバスエリア

### 3.2 上部トップバー

表示・操作項目:

- アプリ名: mugenCanvas
- 新規作成 / 開く / 保存
- Undo / Redo
- PNG 書き出し / GIF 書き出し
- 表示倍率 (ComboBox)

中央にはプロジェクト情報を表示します:

```text
1920 x 1080px / 24fps
```

### 3.3 無限キャンバスエリア

仕様:

- 背景は薄いグレー
- 白い無限キャンバスを表示 (タイル分割, TILE_SIZE=1024)
- キャンバスは上下左右へ無限にパン可能
- Ctrl + ホイールでズーム
- Space + ドラッグでパン
- 赤いカメラ枠を表示

### 3.4 ツールパネル (左)

含めるツール:

- ブラシツール (B)
- 消しゴムツール (E)
- 塗りつぶしツール (F)
- 手のひらツール (H)
- ズームツール (Z)
- カメラツール (C)

### 3.5 カラー・ブラシパネル (右上)

含める機能:

- カラー選択 (egui ColorPicker)
- ブラシサイズ (スライダー)
- 不透明度 (スライダー)

### 3.6 レイヤーパネル (右下)

含める機能:

- レイヤー一覧 (上から順に上位レイヤー)
- 表示/非表示トグル
- ロックトグル
- 選択 (クリック)
- 追加 / 削除 / 上下移動

### 3.7 タイムラインパネル (下)

含める要素:

- レイヤー名一覧 (左)
- フレームグリッド (右)
- 現在フレームのハイライト (青)
- キーフレームドット (橙)
- hold バー (灰)
- オニオンスキン ON/OFF

---

## 4. キャンバス仕様

### 4.1 座標系

```rust
struct Viewport {
    offset_x: f32,  // ワールド座標原点の画面上の位置
    offset_y: f32,
    zoom: f32,
    rotation: f32,
}
```

- 画面座標: egui ポインタ座標
- ワールド座標: 無限キャンバス上の座標
- 変換: `src/canvas/viewport.rs` の `screen_to_world` / `world_to_screen`

### 4.2 基本操作

| 操作 | 挙動 |
|---|---|
| 左ドラッグ | 選択中ツールに応じた操作 |
| Space + 左ドラッグ | キャンバスをパン |
| マウスホイール | 上下スクロール |
| Ctrl + ホイール | ズーム |
| R + ドラッグ | キャンバス回転 |
| Ctrl + R | 回転リセット |

### 4.3 タイル描画方式

- `TILE_SIZE = 1024`
- キーは `(layer_id, frame, tile_x, tile_y)` の `HashMap`
- 描画が存在するタイルだけ生成
- 合成は CPU で RGBA バッファに全タイルを blit → `egui::TextureHandle` にアップロード

---

## 5. カメラトラック仕様

### 5.1 カメラ枠

初期サイズ: 1920 × 1080

表示仕様:

- 赤い外枠 (1px)
- 上下左右中央の短いガイド線 (中心十字)

### 5.2 カメラトラックデータ

```rust
struct CameraKeyframe {
    frame: u32,
    x: f32,
    y: f32,
    scale: f32,
    rotation: f32,
    width: u32,
    height: u32,
}

struct CameraTrack {
    keyframes: Vec<CameraKeyframe>,
}
```

### 5.3 フレーム間補間

`src/canvas/camera.rs` の `get_camera_at_frame()` でキーフレーム間を線形補間します。

---

## 6. タイムライン仕様

### 6.1 基本方針

Adobe Animate 的な横方向フレーム管理 UI。

### 6.2 プロジェクト設定

```rust
struct ProjectSettings {
    width: u32,        // 1920
    height: u32,       // 1080
    fps: u32,          // 24
    total_frames: u32, // 60
}
```

### 6.3 レイヤー / フレームモデル

```rust
struct AnimationLayer {
    id: String,
    name: String,
    visible: bool,
    locked: bool,
    frames: Vec<AnimationFrame>,
}

struct AnimationFrame {
    frame: u32,
    drawing_id: String,   // "" = 空
    is_keyframe: bool,
    hold_source: String,  // "" = hold なし, 非空 = hold フレーム
}
```

### 6.4 コマ (hold) システム

- `hold_source` が空でないフレームは、`hold_source` が指す描画を表示
- 描画操作は hold フレームではブロック
- タイムライン右クリックでコマ割り当て / 解除

### 6.5 Onion Skin

- 前フレーム: 青系 (デフォルト不透明度 0.3)
- 次フレーム: 赤系 (デフォルト不透明度 0.3)
- ON/OFF 切替可能

---

## 7. ツール仕様

### 7.1 ツール一覧

```rust
enum Tool {
    Brush,
    Eraser,
    Fill,
    Hand,
    Zoom,
    Camera,
}
```

### 7.2 ブラシツール

- アンチエイリアス付きストローク (`src/canvas/brush.rs` の `draw_segment`)
- ブラシ / 消しゴムで独立したサイズ・不透明度設定
- source-over (ブラシ) / destination-out (消しゴム) 合成

### 7.3 塗りつぶしツール

- 4連結 BFS flood fill (`src/canvas/brush.rs` の `flood_fill`)
- アルファ対応 (閾値 128)

### 7.4 手のひらツール

キャンバスをパン。Space キー押下中は一時的に手のひらツール。

### 7.5 カメラツール

カメラ枠をドラッグで移動。

---

## 8. データ保存仕様

### 8.1 プロジェクトファイル

拡張子: `.mugencanvas`

実体は JSON ファイル。タイル PNG は base64 埋め込み。

```json
{
  "app": "mugenCanvas",
  "version": "0.1.0",
  "settings": { "width": 1920, "height": 1080, "fps": 24, "totalFrames": 60 },
  "layers": [ ... ],
  "cameraTrack": { "keyframes": [ ... ] },
  "drawings": {
    "layerId:frame:tx:ty": "data:image/png;base64,..."
  }
}
```

フィールド名は camelCase (旧 TypeScript 版との互換性のため `serde(rename_all = "camelCase")`)。

### 8.2 保存操作

- 新規作成 (Ctrl+N)
- 保存 (Ctrl+S) — `rfd` ネイティブダイアログ
- 開く (Ctrl+O) — `rfd` ネイティブダイアログ

---

## 9. 書き出し仕様

### 9.1 PNG 書き出し

- 現在フレーム、全レイヤー合成
- カメラ枠内だけを出力
- 出力サイズは ProjectSettings の width × height

### 9.2 GIF 書き出し

- 全フレームを並列エンコード (rayon + NeuQuant)
- 幅 960px を上限にリサイズ
- `gif` crate の `Frame::from_rgba_speed(..., 30)` を使用

### 9.3 連番 PNG 書き出し (未実装)

```text
output_0001.png
output_0002.png
```

---

## 10. 実装アーキテクチャ

### 10.1 ディレクトリ構成

```text
mugenCanvas/
├─ Cargo.toml
├─ src/
│  ├─ main.rs           # eframe::run_native エントリーポイント
│  ├─ app.rs            # MugenCanvasApp (eframe::App 実装)
│  ├─ model.rs          # データモデル (serde camelCase)
│  ├─ state.rs          # AppState
│  ├─ undo.rs           # UndoManager (スナップショット方式, 20段)
│  ├─ io.rs             # 保存・読込・PNG/GIF 書き出し
│  ├─ canvas/
│  │  ├─ mod.rs         # CanvasState, タイル管理, CPU 合成
│  │  ├─ brush.rs       # draw_segment, flood_fill
│  │  ├─ viewport.rs    # 座標変換
│  │  └─ camera.rs      # カメラキーフレーム補間
│  └─ ui/
│     ├─ topbar.rs      # トップバー
│     ├─ toolbar.rs     # ツール選択
│     ├─ color_panel.rs # カラー・ブラシ設定
│     ├─ layers_panel.rs  # レイヤー一覧
│     └─ timeline.rs    # タイムライン
└─ memo/                # 仕様書・実装計画・モックアップ
```

### 10.2 app.rs の責務

- `setup_fonts()` — Windows 日本語フォント読み込み
- `update_canvas_texture()` — CPU 合成 + egui TextureHandle 更新
- `handle_canvas_input()` — ポインタイベント → ツール処理
- `draw_camera_overlay_onto()` — カメラ枠をピクセルバッファに直接描画

### 10.3 状態管理

```rust
struct MugenCanvasApp {
    state: AppState,
    canvas: CanvasState,
    undo_manager: UndoManager,
    // egui テクスチャ、ダイアログ状態 etc.
}
```

---

## 11. キーボードショートカット

| ショートカット | 機能 |
|---|---|
| B | ブラシ |
| E | 消しゴム |
| F | 塗りつぶし |
| H | 手のひら |
| Z | ズーム |
| C | カメラツール |
| Space | 一時手のひら |
| Ctrl + Z | Undo |
| Ctrl + Y | Redo |
| Ctrl + S | 保存 |
| Ctrl + O | 開く |
| Ctrl + N | 新規 |
| Enter | 再生/停止 |
| ← → | フレーム移動 |

---

## 12. MVP 実装範囲

### 12.1 完了済み ✅

- eframe アプリとして起動できる
- 無限キャンバスをパン・ズームできる
- ブラシで描ける / 消しゴムで消せる / 塗りつぶしできる
- レイヤーを複数持てる
- フレームを切り替えられる
- オニオンスキンが機能する
- コマ (hold) システムが機能する
- カメラ枠が赤線で表示される / 移動できる
- カメラキーフレーム線形補間が機能する
- 現在フレームを PNG 書き出しできる
- GIF 書き出しできる
- プロジェクト保存 / 読み込みができる

### 12.2 未実装 (次フェーズ)

- 連番 PNG 書き出し
- フレームコピー/ペースト
- カメラキーフレーム UI (タイムライン上での追加/削除)
- カメラ枠リサイズ

### 12.3 後回し

- 高度なブラシエンジン / 筆圧対応
- ベクターレイヤー
- テキスト編集 / 図形ツール
- mp4 書き出し
- カメラトゥイーン (イージング)
- プラグイン機構

---

## 13. 見た目の方向性

ダークテーマ (egui デフォルト) を基本とします。

- キャンバスエリアは広く確保
- パネルは必要最小限の情報のみ
- 赤いカメラ枠を目立たせる

---

## 14. 将来的な拡張案

- 筆圧対応
- ブラシプリセット管理
- カスタムブラシ
- ベクターレイヤー
- タイムライン上のトゥイーン
- カメラワークのイージング
- 音声トラック
- mp4 書き出し
- プラグイン API
- ショートカットカスタマイズ
- ワークスペース保存
