#!/bin/sh
set -e

REPO="dikmri/mugenCanvas"
BINARY="mugen-canvas"
INSTALL_DIR="${HOME}/.local/bin"
DESKTOP_DIR="${HOME}/.local/share/applications"

echo "mugenCanvas インストーラー"
echo "--------------------------"

# アーキテクチャ確認
ARCH=$(uname -m)
case "$ARCH" in
    x86_64) ASSET="mugen-canvas-linux-x86_64" ;;
    *)
        echo "エラー: 未対応のアーキテクチャです: $ARCH" >&2
        echo "サポート対象: x86_64" >&2
        exit 1
        ;;
esac

# ディレクトリ作成
mkdir -p "$INSTALL_DIR"
mkdir -p "$DESKTOP_DIR"

# ダウンロード
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
DEST="${INSTALL_DIR}/${BINARY}"

echo "ダウンロード中..."

if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$URL" -o "$DEST"
elif command -v wget >/dev/null 2>&1; then
    wget -q --show-progress "$URL" -O "$DEST"
else
    echo "エラー: curl または wget が必要です" >&2
    exit 1
fi

chmod +x "$DEST"

# アプリランチャー用 .desktop ファイルを作成
cat > "${DESKTOP_DIR}/mugen-canvas.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=mugenCanvas
Comment=アニメーション制作アプリ
Exec=${DEST}
Icon=applications-graphics
Categories=Graphics;2DGraphics;
Terminal=false
DESKTOP

# デスクトップデータベース更新（コマンドがあれば）
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

# ~/.local/bin を PATH に自動追記（まだ含まれていない場合）
PATH_LINE='export PATH="$HOME/.local/bin:$PATH"'
for RC in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
    if [ -f "$RC" ] && ! grep -qF '.local/bin' "$RC"; then
        echo "$PATH_LINE" >> "$RC"
    fi
done

echo ""
echo "インストール完了: $DEST"
echo ""
echo "起動方法:"
echo "  アプリランチャー (GNOME / KDE 等) で「mugenCanvas」を検索"
echo "  または新しいターミナルを開いて: mugen-canvas"
