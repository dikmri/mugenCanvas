# mugenCanvas 実装計画

> このファイルは開発の進捗に合わせて常に更新する。
> 最終更新: 2026-05-22 (Tauri → eframe 移行完了、パニックバグ修正)

---

## ✅ 完成済み

### アーキテクチャ移行
- [x] **Tauri + TypeScript → 純粋 Rust eframe 移行** (2026-05-22)
  - eframe 0.29 + egui 0.29 + wgpu + winit
  - `cargo run` でプロジェクトルートから起動
  - npm / Node.js / WebView 不要
- [x] ディレクトリ構造整理 (`src-tauri/` → `src/` に移動)
- [x] Windows 日本語フォント読み込み (Meiryo / Yu Gothic / MS Gothic)
- [x] パニックバグ修正: カメラ枠がビューポート外に出ると range out of bounds クラッシュ

### コア / キャンバス
- [x] 無限白キャンバス (タイル分割方式 TILE_SIZE=1024)
- [x] パン (Space+ドラッグ / 手のひらツール)
- [x] ズーム (Ctrl+ホイール / ズームツール)
- [x] キャンバス回転 (R+ドラッグ, Ctrl+R リセット)
- [x] ブラシ描画 (アンチエイリアス・サイズ・不透明度)
- [x] 消しゴムツール (ブラシと独立設定)
- [x] 塗りつぶしツール (BFS flood fill)
- [x] アンドゥ/リドゥ (スナップショット方式, 20 段)
- [x] カメラ枠 (赤枠表示・ドラッグ移動・キーフレーム線形補間)
- [x] PNG 書き出し (ネイティブファイルダイアログ)
- [x] GIF 書き出し (rayon 並列 NeuQuant, 960px 上限)

### レイヤー
- [x] レイヤー追加・削除
- [x] レイヤー名変更
- [x] 表示/非表示・ロック
- [x] 上下並び替え

### タイムライン
- [x] フレーム番号・再生ヘッド
- [x] フレーム移動 (クリック・← / →)
- [x] 再生/停止 (Enter)
- [x] オニオンスキン (前=青・後=赤, 透明度設定)
- [x] コマ (hold) システム
  - [x] 右クリックでコマ指定 / コマ解除
  - [x] hold フレームへの描画ブロック
  - [x] タイムラインに ◯ (描画あり) / · (hold) 表示

### UI
- [x] トップバー (新規・開く・保存・Undo/Redo・書き出し・ズーム倍率)
- [x] ツールバー (ブラシ・消しゴム・塗りつぶし・手・ズーム・カメラ)
- [x] カラーパネル (カラーピッカー・サイズ・不透明度)
- [x] レイヤーパネル
- [x] タイムラインパネル

### データ
- [x] プロジェクト保存 / 読み込み (`.mugencanvas` JSON + タイル PNG base64)
- [x] 旧 TypeScript 版との互換性 (serde camelCase)

---

## 📋 残り実装 (優先度順)

### 🔴 最重要
- [ ] **連番 PNG 書き出し** — 全フレームを一括書き出し (カメラ枠内)

### 🟡 重要 (仕様書 MVP 範囲)
- [ ] **フレームコピー/ペースト** — 描画内容を別フレームに複製
- [ ] **カメラキーフレーム UI** — タイムライン上でキーフレーム追加/削除

### 🟢 実装推奨
- [ ] カメラ枠リサイズ (四隅ハンドルで幅・高さ変更)
- [ ] グリッド表示切替
- [ ] 筆圧対応 (PointerEvent.pressure → ブラシサイズに反映)

### ⬜ MVP 後
- [ ] ブラシプリセット管理
- [ ] ベクターレイヤー
- [ ] テキスト編集・図形ツール
- [ ] カメラトゥイーン (イージング)
- [ ] 複数フレーム選択・範囲操作
- [ ] ショートカットカスタマイズ

---

## 技術メモ

### 技術スタック (現行)
| 役割 | 技術 |
|------|------|
| ウィンドウ・入力 | winit (eframe 0.29 経由) |
| GPU レンダリング | wgpu (eframe 0.29 経由) |
| UI フレームワーク | egui 0.29 |
| キャンバス合成 | CPU (RGBA バッファ) → egui TextureHandle |
| ファイルダイアログ | rfd 0.15 |
| GIF エンコード | gif 0.13 + rayon |
| 保存フォーマット | serde_json + base64 + PNG (image crate) |

### 保存フォーマット (現行)

```json
{
  "app": "mugenCanvas",
  "version": "0.1.0",
  "settings": { "width": 1920, "height": 1080, "fps": 24, "totalFrames": 60 },
  "layers": [ ... ],
  "cameraTrack": { ... },
  "drawings": {
    "layerId:frame:tx:ty": "data:image/png;base64,..."
  }
}
```

- キー形式: `"<layerId>:<frame>:<tile_x>:<tile_y>"`
- タイルサイズ: 1024×1024px RGBA8

### タイル座標系
- ワールド座標 `(wx, wy)` → タイル座標 `(wx / TILE_SIZE, wy / TILE_SIZE)` (floor)
- 描画が存在するタイルだけ生成・保存 (実質無限キャンバス)

### コマ (hold) データモデル
- `AnimationFrame.drawingId != ""` → 描画あり (◯)
- `AnimationFrame.holdSource != ""` → hold フレーム (·)、描画ブロック
- `resolve_layer_frame()` で hold チェーンを解決して描画元フレームを返す

### ビルド / 実行
```bash
# プロジェクトルートから
cargo run            # 開発実行
cargo build --release  # リリースビルド
```

### 既知バグ修正履歴
- 2026-05-22: `draw_hline` / `draw_vline` でカメラ枠がビューポート外 (負の座標) に出ると `range start index out of range` でパニック → 負座標の早期 return を追加
