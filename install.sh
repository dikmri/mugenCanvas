#!/bin/sh
set -e

REPO="dikmri/mugenCanvas"
BINARY="mugen-canvas"
INSTALL_DIR="${HOME}/.local/bin"

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

# インストール先ディレクトリを作成
mkdir -p "$INSTALL_DIR"

# 最新リリースの URL を組み立てる
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
DEST="${INSTALL_DIR}/${BINARY}"

echo "ダウンロード中: $URL"

# curl または wget でダウンロード
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$URL" -o "$DEST"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$URL" -O "$DEST"
else
    echo "エラー: curl または wget が必要です" >&2
    exit 1
fi

chmod +x "$DEST"

echo ""
echo "インストール完了: $DEST"

# PATH に含まれているか確認
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*)
        echo "起動するには: mugen-canvas"
        ;;
    *)
        echo ""
        echo "注意: ${INSTALL_DIR} が PATH に含まれていません。"
        echo "以下の行を ~/.bashrc または ~/.zshrc に追加してください:"
        echo ""
        echo '  export PATH="$HOME/.local/bin:$PATH"'
        echo ""
        echo "追加後: source ~/.bashrc (または ~/.zshrc) を実行してから"
        echo "起動するには: mugen-canvas"
        ;;
esac
