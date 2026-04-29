#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

chmod +x "软件Mac环境检查安装脚本.command" "软件Mac一键启动Tauri版.command" 2>/dev/null || true

echo "Starting Terminal Pane Launcher Tauri on macOS..."
echo

if command -v fnm >/dev/null 2>&1; then
  eval "$(fnm env --use-on-cd)"
fi

if ! command -v npm >/dev/null 2>&1; then
  echo "npm was not found. Run 软件Mac环境检查安装脚本.command first, then reopen this launcher."
  echo "Press Enter to close this window."
  read -r _
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo was not found. Run 软件Mac环境检查安装脚本.command first, then reopen this launcher."
  echo "Press Enter to close this window."
  read -r _
  exit 1
fi

if [[ ! -d node_modules ]]; then
  echo "Installing npm dependencies..."
  npm install
fi

npm run tauri:dev
