# mugenCanvas Windows インストーラー
# 使い方: irm https://raw.githubusercontent.com/dikmri/mugenCanvas/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo     = "dikmri/mugenCanvas"
$Asset    = "mugen-canvas-windows-x86_64.exe"
$Binary   = "mugen-canvas.exe"
$InstallDir = "$env:LOCALAPPDATA\mugenCanvas"

Write-Host "mugenCanvas インストーラー"
Write-Host "--------------------------"

# ダウンロード
$Url  = "https://github.com/$Repo/releases/latest/download/$Asset"
$Dest = "$InstallDir\$Binary"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Write-Host "ダウンロード中..."
Invoke-WebRequest -Uri $Url -OutFile $Dest -UseBasicParsing

# ユーザー PATH に追加（永続化）
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    # 現在のセッションにも反映
    $env:PATH = "$env:PATH;$InstallDir"
}

# スタートメニューにショートカット作成
$StartMenu = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
$Shell     = New-Object -ComObject WScript.Shell
$Shortcut  = $Shell.CreateShortcut("$StartMenu\mugenCanvas.lnk")
$Shortcut.TargetPath  = $Dest
$Shortcut.Description = "mugenCanvas - アニメーション制作アプリ"
$Shortcut.Save()

Write-Host ""
Write-Host "インストール完了: $Dest"
Write-Host ""
Write-Host "起動方法:"
Write-Host "  スタートメニューで「mugenCanvas」を検索"
Write-Host "  またはコマンドプロンプト / PowerShell を再起動して: mugen-canvas"
