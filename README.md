# Terminal Pane Launcher

[![Tauri](https://img.shields.io/badge/Tauri-2.x-24c8db)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.x-42b883)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-6.x-3178c6)](https://www.typescriptlang.org/)
[![macOS](https://img.shields.io/badge/macOS-iTerm2%20optimized-111111)](https://iterm2.com/)
[![Windows](https://img.shields.io/badge/Windows-Legacy%20backend%20compatible-0078d4)](https://learn.microsoft.com/windows/terminal/)

Terminal Pane Launcher 是一个基于 `Tauri 2 + Vue 3 + TypeScript + Vite` 的跨平台桌面启动器，用来一次性打开多个项目终端 pane，并为 Codex 工作流准备项目目录、模板提示词和启动命令。

当前版本重点优化了 macOS + iTerm2 工作流，同时继续兼容原 Windows PowerShell 后端。

## Highlights

- 最多配置 `20` 个 pane，每个 pane 可独立设置标题、工作目录、启动命令和 Codex 设置。
- macOS 优先使用 iTerm2 创建大窗口 split panes；未安装 iTerm2 时降级为 Terminal.app 多窗口启动。
- macOS 默认使用 `自动直传`，把完整合并 Prompt 作为 Codex 初始参数传入。
- Windows 保持原有 PowerShell 后端兼容，继续调用旧版 `Start-TerminalLayout.ps1`。
- Prompt 模板随应用打包，支持共享模板和工具模板组合。
- 一键复制完整合并 Prompt，Tauri 环境下走原生剪贴板，避免 WebView clipboard 权限问题。
- 打包后可安装到 `/Applications`，支持通过 `Command + Space` 直接启动。

## Platform Behavior

| Platform | Terminal backend | Pane layout | Prompt delivery | Config location |
| --- | --- | --- | --- | --- |
| macOS | iTerm2 + AppleScript，Terminal.app fallback | iTerm2 split panes | `自动直传/direct` 默认，`自动挡/manual` 可选 | `~/Library/Application Support/com.local.terminal-pane-launcher/layout.json` |
| Windows | 原 PowerShell 后端 + Windows Terminal | 由旧后端处理 | 保留旧配置兼容 | Tauri app config + 运行时 JSON |
| Browser preview | Vite dev server | 不启动终端 | 仅预览 UI | `localStorage` |

## macOS Workflow

macOS 是当前主力体验。推荐安装 iTerm2，因为它能稳定创建 split panes，并由 AppleScript 注入启动命令。

### 1. 检查并安装环境

双击：

```text
软件Mac环境检查安装脚本.command
```

如果 macOS 提示没有执行权限，可以在项目目录执行：

```bash
chmod +x 软件Mac环境检查安装脚本.command 软件Mac一键启动Tauri版.command scripts/check-env.sh
bash 软件Mac环境检查安装脚本.command
```

脚本会检查并尽量安装：

```text
Homebrew
fnm
Node/npm
Rust/rustup/cargo
Xcode Command Line Tools
iTerm2
```

### 2. 开发模式启动

开发阶段推荐使用脚本或 Tauri dev 模式，便于查看日志和快速调试：

```bash
npm install
npm run tauri:dev
```

也可以双击：

```text
软件Mac一键启动Tauri版.command
```

### 3. 打包并安装为应用

打包：

```bash
npm run tauri:build
```

产物位置：

```text
src-tauri/target/release/bundle/macos/Terminal Pane Launcher.app
src-tauri/target/release/bundle/dmg/Terminal Pane Launcher_0.1.0_aarch64.dmg
```

安装到 `/Applications`：

```bash
rm -rf "/Applications/Terminal Pane Launcher.app"
cp -R "src-tauri/target/release/bundle/macos/Terminal Pane Launcher.app" "/Applications/Terminal Pane Launcher.app"
xattr -dr com.apple.quarantine "/Applications/Terminal Pane Launcher.app" 2>/dev/null || true
```

安装后可以通过：

```text
Command + Space -> Terminal Pane Launcher
```

直接启动。

### 4. macOS Codex delivery

macOS 只显示两种模式：

| Mode | UI label | Behavior |
| --- | --- | --- |
| `direct` | 自动直传 | 默认。合并完整 Prompt，并作为 Codex 初始参数传入。 |
| `manual` | 自动挡 | 只启动 Codex，不传长 Prompt，适合手动控制。 |

`自动直传` 不会让 Codex 再去读取 Prompt 文件。为了避免 AppleScript 超长字符串问题，启动器会把完整 Prompt 写入自身应用配置目录下的 `temp/codex-run-args/` 临时参数文件，然后让 shell 执行：

```bash
codex --yolo "$(cat /path/to/temp/codex-run-args/run-args.md; rm -f /path/to/temp/codex-run-args/run-args.md)"
```

对 Codex 来说，收到的是完整初始 Prompt。临时文件不写入各个项目目录；应用启动和每次实际启动 pane 前只会清理这个目录下由启动器生成的 `*-run-args.md` 文件。

### 5. app 启动环境

从 `/Applications` 或 Spotlight 启动时，macOS 不会继承终端里的 `PATH`。应用内置了 shell bootstrap，会主动加载：

```text
/opt/homebrew/bin
brew shellenv
fnm env --use-on-cd
~/.nvm/nvm.sh
```

这样打包版也能找到通过 `nvm` 或 `fnm` 安装的 `codex`。

## Windows Workflow

Windows 端继续兼容原 PowerShell 后端，不强行迁移到 macOS 的 direct 策略。

### 1. 检查并安装环境

双击：

```text
软件Windows环境检查安装脚本.bat
```

脚本会检查：

```text
fnm
Node/npm
Rust/rustup/cargo
Visual Studio Build Tools + C++ 组件
WebView2 Runtime
```

### 2. 开发模式启动

```powershell
npm install
npm run tauri:dev
```

也可以双击：

```text
软件Windows一键启动Tauri版.bat
```

该 bat 会调用：

```text
scripts/start-tauri-windows.ps1
```

避免中文路径和 cmd 解析问题。

### 3. Windows 后端目录

Windows 后端应包含：

```text
Start-TerminalLayout.ps1
src/TerminalLayout.psm1
```

Tauri GUI 会自动检测同级目录里的旧后端，也可以在 Advanced 面板里手动指定 Windows backend path。

Windows 启动时会调用：

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File <backend>\Start-TerminalLayout.ps1 -ConfigPath <runtime-config>
```

## Project Layout

推荐目录结构：

```text
workspace/
  old-windows-backend/
    Start-TerminalLayout.ps1
    src/
      TerminalLayout.psm1

  terminal-pane-launcher-tauri/
    src/
    src-tauri/
    scripts/
    templates/
      全栈的提示词留档.md
      跨平台提示词留档.md
      codex的模板.md
      claudecode的模板.md
      prompt-file-instruction.txt
```

模板文件会优先从当前项目的 `templates/` 读取，并在打包时进入应用资源目录。

## Prompt Composition

每个 pane 的完整 Prompt 由三部分组成：

```text
1. User Prompt
2. Shared Prompt Template
3. Tool Prompt Template
```

一键复制会复制同样的合并结果。

## Configuration

当前 Tauri 应用保存 GUI 配置到平台应用配置目录。

macOS：

```text
~/Library/Application Support/com.local.terminal-pane-launcher/layout.json
```

浏览器预览模式：

```text
localStorage
```

旧的项目内配置：

```text
config/layout.json
```

可作为迁移参考，但打包版和当前 Tauri app 运行时不会再依赖项目工作目录，否则通过 Spotlight 启动会不稳定。

## Commands

| Command | Description |
| --- | --- |
| `npm run dev` | 启动 Vite 浏览器预览 |
| `npm run build` | 前端类型检查并构建 |
| `npm run tauri:dev` | 启动 Tauri 开发版 |
| `npm run tauri:build` | 构建 release app 和安装包 |
| `npm run check:env:mac` | 检查 macOS 环境 |
| `npm run install:env:mac` | 检查并安装 macOS 环境 |
| `npm run check:env:windows` | 检查 Windows 环境 |
| `npm run install:env:windows` | 检查并安装 Windows 环境 |

## Troubleshooting

### macOS app 版找不到 Codex

确认终端里能找到：

```bash
command -v codex
```

如果终端里能找到但 app 找不到，请重新打包安装最新版。最新版会自动加载 Homebrew、fnm 和 nvm 环境。

### macOS Spotlight 里出现两个应用

正式使用的是：

```text
/Applications/Terminal Pane Launcher.app
```

如果 Spotlight 还显示构建目录里的副本，可以删除：

```bash
rm -rf "src-tauri/target/release/bundle/macos/Terminal Pane Launcher.app"
```

### macOS pane 打开了但没有启动 Codex

检查该 pane 的 delivery 是否为：

```text
自动直传
```

当前 macOS 逻辑中，只要选择 `自动直传/direct`，即使 `codexMode` 为空，也会启动 Codex 并传入合并 Prompt。

### 一键复制失败

Tauri 环境下复制不依赖浏览器 Clipboard API，而是调用平台剪贴板：

```text
macOS: NSPasteboard
Windows: PowerShell Set-Clipboard
Linux: wl-copy / xclip / xsel
```

如果仍失败，确认系统命令可用。

### Windows npm 或 cargo 找不到

重新打开 PowerShell 后执行：

```powershell
node --version
npm --version
cargo --version
rustc --version
```

如果仍找不到，可以运行：

```powershell
npm run install:env:windows
```

### Visual Studio Build Tools 缺失

打开 Visual Studio Installer，确认安装：

```text
Desktop development with C++
MSVC
Windows SDK
```

## Release Notes

当前版本重点：

- macOS 打包版可通过 Spotlight 启动。
- macOS iTerm2 自动大窗口 split panes。
- macOS 默认自动直传完整 Prompt 到 Codex 初始参数。
- macOS app 版自动加载 nvm/fnm/Homebrew 环境。
- Windows 保持旧后端兼容，不改变原有 PowerShell 启动路径。
- 配置读取失败时会回退默认 20 pane，避免打包版空白页。

## Roadmap

- 配置导入/导出。
- GUI 内编辑模板。
- Windows 端更稳定的直传实验模式。
- macOS iTerm2 布局细节和窗口行为继续优化。
