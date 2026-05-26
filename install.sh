#!/bin/sh
set -e

REPO="dikmri/mugenCanvas"
BINARY="mugen-canvas"
INSTALL_DIR="${HOME}/.local/bin"

echo "mugenCanvas インストーラー"
echo "--------------------------"

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64) ASSET="mugen-canvas-linux-x86_64" ;;
            *) echo "エラー: 未対応のアーキテクチャです: $ARCH" >&2; exit 1 ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            arm64)  ASSET="mugen-canvas-macos-aarch64" ;;
            x86_64) ASSET="mugen-canvas-macos-x86_64" ;;
            *) echo "エラー: 未対応のアーキテクチャです: $ARCH" >&2; exit 1 ;;
        esac
        ;;
    *)
        echo "エラー: 未対応のOSです: $OS" >&2
        echo "Windows の場合は install.ps1 を使用してください" >&2
        exit 1
        ;;
esac

mkdir -p "$INSTALL_DIR"

URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
DEST="${INSTALL_DIR}/${BINARY}"

echo "ダウンロード中..."

if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$URL" -o "$DEST"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$URL" -O "$DEST"
else
    echo "エラー: curl または wget が必要です" >&2
    exit 1
fi

chmod +x "$DEST"

# Linux: アプリランチャー用 .desktop ファイルを作成
if [ "$OS" = "Linux" ]; then
    DESKTOP_DIR="${HOME}/.local/share/applications"
    mkdir -p "$DESKTOP_DIR"
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
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    fi
fi

# PATH を各シェル設定ファイルに自動追記
PATH_LINE='export PATH="$HOME/.local/bin:$PATH"'
for RC in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.bash_profile" "$HOME/.profile"; do
    if [ -f "$RC" ] && ! grep -qF '.local/bin' "$RC"; then
        echo "$PATH_LINE" >> "$RC"
    fi
done

echo ""
echo "インストール完了: $DEST"
echo ""
if [ "$OS" = "Linux" ]; then
    echo "起動方法:"
    echo "  アプリランチャー (GNOME / KDE 等) で「mugenCanvas」を検索"
    echo "  または新しいターミナルを開いて: mugen-canvas"
elif [ "$OS" = "Darwin" ]; then
    echo "起動方法:"
    echo "  新しいターミナルを開いて: mugen-canvas"
fi
