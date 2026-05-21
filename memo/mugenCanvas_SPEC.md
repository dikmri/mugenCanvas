# mugenCanvas 仕様書 / Claude Code 実装指示書

## 0. このドキュメントの目的

このドキュメントは、クロスプラットフォーム対応のデスクトップアニメーション制作アプリ **mugenCanvas** を、Claude Code へそのまま渡して実装を進めるための仕様書です。

mugenCanvas は、**Photoshop 的な描き心地**と、**Adobe Animate 的なタイムライン中心のアニメ制作体験**を組み合わせた、Rust + Tauri 製のデスクトップアプリです。

最初の実装では、完成品レベルの高機能ペイントソフトを一気に作るのではなく、以下を重視した MVP を作成します。

- 無限キャンバス上で自由に描ける
- カメラトラックの赤い枠内だけが最終出力範囲になる
- レイヤーとカメラトラックを Animate 的なタイムラインで管理できる
- ツールウィンドウ、ブラシ・色ウィンドウ、タイムラインウィンドウを自由配置・固定切替できる
- Rust/Tauri の構成として将来的に高性能化しやすい設計にする

---

## 1. アプリ概要

### 1.1 アプリ名

**mugenCanvas**

### 1.2 コンセプト

> 無限に広がるキャンバス上で、Photoshop のように気持ちよく描きながら、Animate のようなタイムラインでアニメーションを作れるクロスプラットフォームアプリ。

### 1.3 対応プラットフォーム

初期ターゲットは以下のデスクトップ OS とします。

- Windows 10 / 11
- macOS
- Linux

### 1.4 技術スタック

- デスクトップフレームワーク: **Tauri v2 系**
- バックエンド / ネイティブ処理: **Rust**
- フロントエンド: **TypeScript + HTML + CSS**
- UI フレームワーク: 原則として軽量な構成を優先
  - 推奨: Vanilla TypeScript + CSS
  - 代替: React / Solid / Svelte などを使ってもよいが、過剰な依存は避ける
- 描画エリア:
  - 初期 MVP: HTML Canvas 2D
  - 将来的な拡張候補: WebGL / WebGPU / Rust 側レンダリング

---

## 2. 重要な設計思想

### 2.1 UI の基本思想

mugenCanvas の UI は、一般的な Photoshop のように画面端までパネルが詰まった状態ではなく、**余白のある自由配置型 UI** とします。

各ウィンドウは、机の上に道具を置くように自由に移動できるイメージです。

対象ウィンドウ:

- ツールウィンドウ
- ブラシ・色ウィンドウ
- タイムラインウィンドウ

各ウィンドウは以下の状態を持ちます。

- floating: 自由移動可能
- pinned: 固定状態
- collapsed: 折りたたみ状態
- hidden: 非表示状態

### 2.2 キャンバスの基本思想

画面には**無限に広い白いキャンバス**が存在します。

ユーザーは上下左右へ自由にスクロール・パンできます。中央付近に赤いカメラ枠が表示され、この赤い枠の中にある絵だけが最終映像として出力されます。

### 2.3 カメラトラックの基本思想

赤い枠は単なるガイドではなく、**カメラトラック**です。

カメラトラックは通常レイヤーと同様にタイムラインで管理されます。

カメラトラックで管理する値:

- x 座標
- y 座標
- scale
- rotation（MVP では任意。余裕があれば実装）
- width
- height
- opacity ではなく visibility

カメラトラックを移動・拡大縮小することで、最終出力時のカメラワークを作成できます。

---

## 3. 画面構成

### 3.1 メイン画面

メイン画面は以下で構成します。

- 上部アプリバー
- 無限キャンバスエリア
- ツールウィンドウ
- ブラシ・色ウィンドウ
- タイムラインウィンドウ
- ステータスバー

### 3.2 上部アプリバー

上部に薄いバーを配置します。

表示項目:

- アプリ名: mugenCanvas
- 新規作成
- 開く
- 保存
- Undo
- Redo
- 再生
- 停止
- 表示倍率
- グリッド表示切替
- 設定

中央には現在のプロジェクト情報を表示します。

例:

```text
新規プロジェクト
1920 x 1080px / 24fps
```

### 3.3 無限キャンバスエリア

画面中央の広い領域です。

仕様:

- 背景は薄いグレーまたはチェック柄
- その上に白い無限キャンバスを表示
- キャンバスは上下左右へ無限にパン可能
- マウスホイールでズーム
- Space + ドラッグでパン
- Ctrl + ホイールでズーム
- カメラ枠は赤色で表示
- カメラ枠の中心・上下左右に赤いガイド線を表示

### 3.4 ツールウィンドウ

左側に初期配置する floating window です。

初期位置:

- x: 40
- y: 120
- width: 120
- height: 460

含めるツール:

- 選択ツール
- 移動ツール
- ブラシツール
- 消しゴムツール
- 塗りつぶしツール
- 図形ツール
- テキストツール
- 手のひらツール
- ズームツール
- 前景色 / 背景色

### 3.5 ブラシ・色ウィンドウ

右側に初期配置する floating window です。

初期位置:

- x: 画面右から 360px 程度
- y: 120
- width: 300
- height: 420

含める機能:

- カラー選択
- スウォッチ
- グラデーション
- ブラシサイズ
- 不透明度
- 最小サイズ
- アンチエイリアス
- ブラシプリセット

MVP では見た目だけでもよい項目:

- スウォッチ
- グラデーション
- ブラシプリセット

実際に動作させる項目:

- カラー選択
- ブラシサイズ
- 不透明度

### 3.6 タイムラインウィンドウ

下部に初期配置する floating window です。

初期位置:

- x: 260
- y: 画面下から 260px 程度
- width: 900
- height: 220

含める要素:

- レイヤー一覧
- カメラトラック
- フレーム番号
- 現在フレームの赤い再生ヘッド
- キーフレーム表示
- レイヤー表示/非表示
- レイヤーロック
- レイヤー追加
- フレーム追加
- Onion Skin 切替

---

## 4. UI レイアウト仕様

### 4.1 floating window 共通仕様

すべての floating window は以下の共通仕様を持ちます。

```ts
type PanelState = 'floating' | 'pinned' | 'collapsed' | 'hidden';

interface FloatingPanel {
  id: string;
  title: string;
  x: number;
  y: number;
  width: number;
  height: number;
  state: PanelState;
  zIndex: number;
}
```

### 4.2 操作仕様

- タイトルバーをドラッグすると移動
- 右上の × で非表示
- ピンアイコンで固定/解除
- ダブルクリックで折りたたみ/展開
- ウィンドウ同士が重なってもよい
- クリックしたウィンドウが最前面に来る
- 初期配置では余白を多く取り、画面全体を詰めすぎない

### 4.3 固定モード

pinned 状態では、ウィンドウを移動できません。

見た目としては以下を変えると分かりやすいです。

- タイトルバーにピンアイコンを表示
- 枠線を少し濃くする
- ドラッグカーソルを通常カーソルに戻す

---

## 5. キャンバス仕様

### 5.1 座標系

キャンバスはワールド座標を持ちます。

```ts
interface Vec2 {
  x: number;
  y: number;
}

interface Viewport {
  offsetX: number;
  offsetY: number;
  zoom: number;
}
```

- 画面座標: 実際のディスプレイ上の座標
- ワールド座標: 無限キャンバス上の座標

### 5.2 基本操作

| 操作 | 挙動 |
|---|---|
| 左ドラッグ | 選択中ツールに応じた操作 |
| Space + 左ドラッグ | キャンバスをパン |
| マウスホイール | 上下スクロール |
| Shift + ホイール | 左右スクロール |
| Ctrl + ホイール | ズーム |
| Ctrl + 0 | カメラ枠を画面中央にフィット |
| Ctrl + 1 | 100% 表示 |

### 5.3 描画仕様

MVP ではラスターベースの描画とします。

実装方針:

- 各レイヤーごとに OffscreenCanvas または内部 canvas を持つ
- 表示時に全レイヤーを合成
- 現在フレームに対応するレイヤーデータを描画

---

## 6. カメラトラック仕様

### 6.1 カメラ枠

カメラ枠は赤い線で表示します。

初期サイズ:

- width: 1920
- height: 1080

初期位置:

- worldX: 0
- worldY: 0

表示仕様:

- 赤い外枠
- 内側の安全領域ガイド
- 上下左右中央の短いガイド線
- 非出力範囲は通常表示のまま。暗くしすぎない

### 6.2 カメラトラックデータ

```ts
interface CameraKeyframe {
  frame: number;
  x: number;
  y: number;
  scale: number;
  rotation: number;
  width: number;
  height: number;
}

interface CameraTrack {
  id: 'camera';
  name: 'カメラ';
  visible: boolean;
  locked: boolean;
  keyframes: CameraKeyframe[];
}
```

### 6.3 タイムラインでの扱い

カメラトラックは、通常レイヤーとは別枠で一番上に表示します。

例:

```text
カメラ
レイヤー 3
レイヤー 2
レイヤー 1
背景
```

### 6.4 カメラ操作

カメラ枠を選択すると、以下の操作が可能です。

- ドラッグで移動
- 四隅ハンドルで拡大縮小
- タイムライン上でキーフレーム追加
- フレーム間は線形補間

MVP では rotation は UI に出さなくてもよいですが、データ構造には持たせます。

---

## 7. タイムライン仕様

### 7.1 基本方針

タイムラインは Adobe Animate 的な横方向フレーム管理 UI とします。

構成:

- 左側: レイヤー名一覧
- 右側: フレームグリッド
- 上部: フレーム番号
- 赤い縦線: 現在フレーム

### 7.2 プロジェクト設定

初期値:

```ts
interface ProjectSettings {
  width: number;      // 1920
  height: number;     // 1080
  fps: number;        // 24
  totalFrames: number; // 60
}
```

### 7.3 レイヤー仕様

```ts
interface AnimationLayer {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  frames: AnimationFrame[];
}

interface AnimationFrame {
  frame: number;
  drawingId: string | null;
  isKeyframe: boolean;
}
```

### 7.4 フレーム操作

MVP で実装する操作:

- 現在フレーム変更
- レイヤー追加
- レイヤー削除
- レイヤー名変更
- キーフレーム追加
- 空白キーフレーム追加
- フレームコピー
- フレーム貼り付け

後回しでよい操作:

- トゥイーン
- モーションパス
- 複数フレーム選択
- フレーム範囲ドラッグ

### 7.5 Onion Skin

MVP では簡易 Onion Skin を実装します。

仕様:

- 現在フレームの前後 1 フレームを薄く表示
- 前フレームは薄い青系
- 次フレームは薄い赤系
- ON/OFF 切替可能

色は将来的に設定可能にします。

---

## 8. ツール仕様

### 8.1 ツール一覧

```ts
type ToolType =
  | 'select'
  | 'move'
  | 'brush'
  | 'eraser'
  | 'fill'
  | 'shape'
  | 'text'
  | 'hand'
  | 'zoom'
  | 'camera';
```

### 8.2 ブラシツール

MVP で必要な機能:

- ペン入力 / マウス入力で線を描く
- 色指定
- ブラシサイズ指定
- 不透明度指定
- アンチエイリアス

筆圧対応は後回しでよいですが、将来的に PointerEvent の pressure を使える構造にしておきます。

### 8.3 消しゴムツール

MVP では通常ブラシと同じ形状で、描画済みピクセルを透明にします。

### 8.4 手のひらツール

キャンバスをパンします。

Space キー押下中は一時的に手のひらツールになります。

### 8.5 ズームツール

クリックでズームイン、Alt + クリックでズームアウトします。

---

## 9. データ保存仕様

### 9.1 プロジェクトファイル

拡張子:

```text
.mugencanvas
```

実体は zip 形式とします。

内部構成例:

```text
project.mugencanvas
├─ project.json
├─ drawings/
│  ├─ drawing_001.png
│  ├─ drawing_002.png
│  └─ ...
└─ thumbnails/
   └─ preview.png
```

### 9.2 project.json

```ts
interface ProjectFile {
  app: 'mugenCanvas';
  version: string;
  settings: ProjectSettings;
  viewport: Viewport;
  panels: FloatingPanel[];
  cameraTrack: CameraTrack;
  layers: AnimationLayer[];
}
```

### 9.3 保存操作

MVP では以下を実装します。

- 新規作成
- 保存
- 名前を付けて保存
- 開く

Tauri 側でファイルダイアログとファイル I/O を担当します。

---

## 10. 書き出し仕様

### 10.1 画像書き出し

MVP では、現在フレームを PNG として書き出せるようにします。

出力対象:

- カメラ枠内だけ
- 出力サイズは ProjectSettings の width / height

### 10.2 連番画像書き出し

MVP または次フェーズで実装します。

例:

```text
output_0001.png
output_0002.png
output_0003.png
```

### 10.3 動画書き出し

初期 MVP では必須ではありません。

将来的には以下の方法を検討します。

- ffmpeg 同梱またはユーザー指定
- 連番 PNG から mp4 生成

---

## 11. 実装アーキテクチャ

### 11.1 推奨ディレクトリ構成

```text
mugen-canvas/
├─ package.json
├─ tsconfig.json
├─ index.html
├─ src/
│  ├─ main.ts
│  ├─ styles/
│  │  ├─ base.css
│  │  ├─ layout.css
│  │  └─ panels.css
│  ├─ app/
│  │  ├─ app-state.ts
│  │  ├─ project.ts
│  │  └─ commands.ts
│  ├─ canvas/
│  │  ├─ canvas-view.ts
│  │  ├─ viewport.ts
│  │  ├─ renderer.ts
│  │  ├─ camera-track.ts
│  │  └─ brush-engine.ts
│  ├─ timeline/
│  │  ├─ timeline-view.ts
│  │  ├─ timeline-model.ts
│  │  └─ frame-ops.ts
│  ├─ panels/
│  │  ├─ floating-panel.ts
│  │  ├─ tools-panel.ts
│  │  ├─ brush-color-panel.ts
│  │  └─ timeline-panel.ts
│  ├─ tools/
│  │  ├─ tool-manager.ts
│  │  ├─ brush-tool.ts
│  │  ├─ eraser-tool.ts
│  │  ├─ hand-tool.ts
│  │  └─ camera-tool.ts
│  └─ types/
│     └─ model.ts
├─ src-tauri/
│  ├─ Cargo.toml
│  ├─ tauri.conf.json
│  └─ src/
│     ├─ main.rs
│     ├─ project_io.rs
│     └─ export.rs
└─ docs/
   ├─ architecture.md
   ├─ canvas.md
   ├─ timeline.md
   └─ file-format.md
```

### 11.2 フロントエンド責務

- UI 描画
- キャンバス表示
- ユーザー入力処理
- タイムライン表示
- 編集状態管理

### 11.3 Rust/Tauri 側責務

- ファイル保存 / 読み込み
- プロジェクト zip 作成 / 展開
- 画像書き出し補助
- 将来的な動画書き出し
- OS 依存処理

### 11.4 状態管理

巨大なフレームワークを使わず、まずはシンプルな store を作成します。

```ts
interface AppState {
  project: ProjectFile;
  currentFrame: number;
  selectedLayerId: string;
  selectedTool: ToolType;
  viewport: Viewport;
  panels: FloatingPanel[];
  brush: BrushSettings;
}
```

---

## 12. MVP 実装範囲

### 12.1 必須

- Tauri アプリとして起動できる
- mugenCanvas のメイン UI が表示される
- floating window をドラッグ移動できる
- ツールウィンドウが表示される
- ブラシ・色ウィンドウが表示される
- タイムラインウィンドウが表示される
- 無限キャンバスをパン・ズームできる
- ブラシで描ける
- 消しゴムで消せる
- レイヤーを複数持てる
- フレームを切り替えられる
- カメラ枠が赤線で表示される
- カメラ枠を移動できる
- カメラトラックをタイムラインに表示できる
- 現在フレームを PNG 書き出しできる
- プロジェクト保存 / 読み込みができる

### 12.2 後回し

- 高度なブラシエンジン
- 筆圧対応
- ベクターレイヤー
- テキスト編集
- 図形編集
- mp4 書き出し
- 複雑なトゥイーン
- プラグイン機構
- クラウド同期

---

## 13. 見た目の方向性

### 13.1 UI テーマ

初期テーマは light theme とします。

方向性:

- 白〜薄いグレー基調
- 角丸の floating window
- 影は控えめ
- パネルは詰め込みすぎない
- キャンバス周辺に広い余白を残す
- 赤いカメラ枠を目立たせる

### 13.2 色指定

```css
:root {
  --bg-main: #f7f7f5;
  --bg-panel: rgba(255, 255, 255, 0.88);
  --bg-panel-solid: #ffffff;
  --border-subtle: #d8d8d8;
  --text-main: #222222;
  --text-muted: #666666;
  --accent-red: #ff4b4b;
  --accent-blue: #4a90e2;
  --shadow-panel: 0 16px 40px rgba(0, 0, 0, 0.12);
}
```

---

## 14. キーボードショートカット

| ショートカット | 機能 |
|---|---|
| B | ブラシ |
| E | 消しゴム |
| V | 選択 |
| H | 手のひら |
| Z | ズーム |
| C | カメラツール |
| Space | 一時手のひら |
| Ctrl + Z | Undo |
| Ctrl + Shift + Z | Redo |
| Ctrl + S | 保存 |
| Ctrl + O | 開く |
| Ctrl + N | 新規 |
| Enter | 再生/停止 |
| , | 前フレーム |
| . | 次フレーム |

---

## 15. Claude Code への実装指示

以下の方針で実装してください。

1. まず Tauri v2 + TypeScript の最小アプリを作成する。
2. UI は添付 SVG / ワイヤーフレームの雰囲気に合わせる。
3. Photoshop のような密集 UI ではなく、余白のある floating window 型 UI にする。
4. 最初から高機能化しすぎず、MVP 範囲を優先する。
5. 型定義を先に作成し、機能ごとにファイルを分割する。
6. 各主要ファイルには対応する設計メモを `docs/` 配下に作成する。
7. 状態管理はまずシンプルな AppState で実装する。
8. UI、キャンバス、タイムライン、Tauri ファイル I/O を疎結合にする。
9. 可能な限り TypeScript の型エラーが出ない構成にする。
10. 実装後、最低限以下を確認する。

確認項目:

- `npm run dev` で起動できる
- floating window を移動できる
- キャンバスをパン・ズームできる
- ブラシで線を描ける
- タイムラインの現在フレームを変更できる
- 赤いカメラ枠が表示される
- カメラ枠を移動できる
- 保存 / 読み込みの土台がある

---

## 16. 将来的な拡張案

- 筆圧対応
- ブラシプリセット管理
- カスタムブラシ
- ベクター線
- ラスターレイヤー / ベクターレイヤー切替
- タイムライン上のトゥイーン
- カメラワークのイージング
- 音声トラック
- mp4 / gif 書き出し
- プラグイン API
- ショートカットカスタマイズ
- ダークテーマ
- ワークスペース保存
- パネルレイアウトプリセット

---

## 17. 最初に作るべき具体的なタスク

Claude Code は以下の順番で実装してください。

### Step 1: プロジェクト作成

- Tauri v2 + TypeScript のプロジェクトを作成
- `npm run dev` で起動確認

### Step 2: 静的 UI 作成

- アプリバー
- 中央キャンバス
- ツールウィンドウ
- ブラシ・色ウィンドウ
- タイムラインウィンドウ
- 赤いカメラ枠

### Step 3: floating window 実装

- パネル移動
- z-index 管理
- 固定/解除
- 非表示

### Step 4: キャンバス操作

- パン
- ズーム
- ワールド座標変換

### Step 5: ブラシ描画

- ブラシツール
- 消しゴムツール
- 色/サイズ/不透明度

### Step 6: タイムライン基礎

- フレーム番号表示
- 現在フレーム移動
- レイヤー表示
- カメラトラック表示

### Step 7: カメラトラック

- カメラ枠選択
- 移動
- キーフレーム登録
- フレーム間補間

### Step 8: 保存・読み込み

- project.json 保存
- project.json 読み込み
- 将来の zip 化に備えた構造にする

### Step 9: 書き出し

- 現在フレームを PNG 書き出し
- カメラ枠内だけを出力

---

## 18. 完了条件

MVP の完了条件は以下です。

- mugenCanvas として起動する
- 画面全体が仕様の UI に近い
- パネルが自由配置できる
- 無限キャンバスをパン・ズームできる
- ブラシ描画ができる
- タイムラインでフレームを切り替えられる
- カメラ枠が最終出力範囲として機能する
- カメラトラックがタイムライン上に存在する
- 現在フレームを PNG として書き出せる
- プロジェクト保存/読み込みの最低限が動く

---

## 19. 補足

このアプリは「Photoshop と Animate を合体させたもの」ではありますが、UI を Adobe 製品そっくりにする必要はありません。

重要なのは以下です。

- 描きやすい
- アニメ制作の流れが分かりやすい
- 画面がごちゃごちゃしない
- ウィンドウを自由に配置できる
- 無限キャンバスとカメラ枠という独自性が明確に伝わる

mugenCanvas は、従来のペイントソフトやアニメ制作ソフトよりも、もっと自由に机の上へ紙・道具・タイムラインを広げて作業できるようなアプリを目指します。
