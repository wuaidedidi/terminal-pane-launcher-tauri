#!/usr/bin/env bash
set -euo pipefail

INSTALL=0
if [[ "${1:-}" == "--install" ]]; then
  INSTALL=1
fi

has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

version_line() {
  if has_cmd "$1"; then
    "$1" --version 2>/dev/null | head -n 1 || true
  fi
}

has_iterm2() {
  osascript -e 'id of application "iTerm2"' >/dev/null 2>&1
}

print_check() {
  local name="$1"
  local ok="$2"
  local detail="${3:-}"
  local fix="${4:-}"

  if [[ "$ok" == "1" ]]; then
    printf '[OK] %s' "$name"
  else
    printf '[MISS] %s' "$name"
  fi

  if [[ -n "$detail" ]]; then
    printf ' - %s' "$detail"
  fi
  printf '\n'

  if [[ "$ok" != "1" && -n "$fix" ]]; then
    printf '      Fix: %s\n' "$fix"
  fi
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This script is for macOS. Use scripts/check-env.ps1 on Windows."
  exit 1
fi

echo "Checking Tauri development environment for macOS..."
echo

if ! has_cmd brew && [[ "$INSTALL" == "1" ]]; then
  echo "Homebrew is not installed. Installing Homebrew from the official installer..."
  NONINTERACTIVE=1 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
  if [[ -x /opt/homebrew/bin/brew ]]; then
    eval "$(/opt/homebrew/bin/brew shellenv)"
  elif [[ -x /usr/local/bin/brew ]]; then
    eval "$(/usr/local/bin/brew shellenv)"
  fi
fi

if ! has_cmd fnm && [[ "$INSTALL" == "1" ]]; then
  if has_cmd brew; then
    brew install fnm
  else
    echo "Cannot install fnm automatically without Homebrew."
  fi
fi

if has_cmd fnm; then
  eval "$(fnm env --use-on-cd)"
fi

if { ! has_cmd node || ! has_cmd npm; } && has_cmd fnm && [[ "$INSTALL" == "1" ]]; then
  fnm install --lts
  fnm default lts-latest
  eval "$(fnm env --use-on-cd)"
fi

if { ! has_cmd cargo || ! has_cmd rustc; } && [[ "$INSTALL" == "1" ]]; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck source=/dev/null
  source "$HOME/.cargo/env"
fi

if ! has_iterm2 && [[ "$INSTALL" == "1" ]]; then
  if has_cmd brew; then
    brew install --cask iterm2
  else
    echo "Cannot install iTerm2 automatically without Homebrew."
  fi
fi

XCODE_PATH=""
if xcode-select -p >/dev/null 2>&1; then
  XCODE_PATH="$(xcode-select -p)"
elif [[ "$INSTALL" == "1" ]]; then
  echo "Starting Xcode Command Line Tools installer..."
  xcode-select --install || true
fi

echo
print_check "Homebrew" "$(has_cmd brew && echo 1 || echo 0)" "$(version_line brew)" "Install from https://brew.sh/"
print_check "fnm" "$(has_cmd fnm && echo 1 || echo 0)" "$(version_line fnm)" "brew install fnm"
print_check "node" "$(has_cmd node && echo 1 || echo 0)" "$(version_line node)" "fnm install --lts && fnm default lts-latest"
print_check "npm" "$(has_cmd npm && echo 1 || echo 0)" "$(version_line npm)" "Install Node.js through fnm."
print_check "rustup" "$(has_cmd rustup && echo 1 || echo 0)" "$(version_line rustup)" "curl https://sh.rustup.rs -sSf | sh"
print_check "rustc" "$(has_cmd rustc && echo 1 || echo 0)" "$(version_line rustc)" "Install Rust through rustup."
print_check "cargo" "$(has_cmd cargo && echo 1 || echo 0)" "$(version_line cargo)" "Install Rust through rustup."
print_check "Xcode Command Line Tools" "$(xcode-select -p >/dev/null 2>&1 && echo 1 || echo 0)" "$XCODE_PATH" "xcode-select --install"
print_check "iTerm2" "$(has_iterm2 && echo 1 || echo 0)" "Required for macOS split-pane launch" "brew install --cask iterm2"

echo
if has_cmd node && has_cmd npm && has_cmd cargo && has_cmd rustc && xcode-select -p >/dev/null 2>&1 && has_iterm2; then
  echo "Environment looks ready. Try: npm run tauri:dev"
else
  echo "Environment is not ready yet."
  if [[ "$INSTALL" != "1" ]]; then
    echo "Run with installation enabled:"
    echo "  bash scripts/check-env.sh --install"
  else
    echo "Some installers may require a reopened terminal or manual confirmation."
  fi
fi
