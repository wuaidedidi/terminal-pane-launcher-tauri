#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

chmod +x "软件Mac环境检查安装脚本.command" "软件Mac一键启动Tauri版.command" 2>/dev/null || true

echo "Checking and installing the development environment for this software on macOS..."
echo "This includes iTerm2, which is recommended for true split-pane launch."
echo

bash "scripts/check-env.sh" --install

echo
echo "Finished. If Rust, fnm, Homebrew, iTerm2, or Xcode Command Line Tools were installed, close this window and open a new terminal before running the Tauri app."
echo "Press Enter to close this window."
read -r _
