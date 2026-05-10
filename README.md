# Terminal Pane Launcher

[![Tauri](https://img.shields.io/badge/Tauri-2.x-24c8db)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.x-42b883)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-6.x-3178c6)](https://www.typescriptlang.org/)
[![macOS](https://img.shields.io/badge/macOS-iTerm2%20optimized-111111)](https://iterm2.com/)
[![Windows](https://img.shields.io/badge/Windows-Legacy%20backend%20compatible-0078d4)](https://learn.microsoft.com/windows/terminal/)

Terminal Pane Launcher 是一个基于 `Tauri 2 + Vue 3 + TypeScript + Vite` 的跨平台桌面启动器，用来一次性打开多个终端 pane，并为 Codex 工作流准备好项目目录、提示词模板和启动参数。

现在的重点有两条：

1. macOS 体验优先，默认偏向 iTerm2 的 split panes。
2. 保留 Windows 旧后端兼容，同时新增一个更适合模板化工作的 `query 标注专用` 工作区。

## 一眼看懂

- 最多 `20` 个 pane。
- 每个 pane 都可以单独配置目录、标题、Codex 模式和启动方式。
- macOS 会优先使用 iTerm2；如果没有 iTerm2，再退回 Terminal.app。
- Windows 继续兼容旧的 PowerShell 后端，不破坏原有工作流。
- 支持把 Markdown 提示词批量导入到 pane。
- 支持 Markdown 模板、锚点和差异值驱动的 query 工作区。

## 工作区

顶部有两个页签，它们互不干扰：

| 工作区 | 用途 | 适合什么场景 |
| --- | --- | --- |
| `vibecoding 项目专用` | 继续使用当前的 prompt 拼接式 20 pane 流程 | 需要快速起多个相似 Codex pane 的日常项目 |
| `query 标注专用` | 使用 Markdown 模板、锚点和 pane 差异值生成启动内容 | 需要为不同项目快速生成定制化启动参数 |

`query 标注专用` 里，Codex 的 `resume` 会优先按当前 pane 目录查找最近会话，再用 `codex resume <session-id> <prompt>` 启动。这样模板 prompt 不会被误当成 session id。找不到目录匹配的会话时，会退回 `codex resume --last`。

## 快速开始

### macOS

1. 双击环境检查脚本：

```text
软件Mac环境检查安装脚本.command
```

2. 开发模式启动：

```bash
npm install
npm run tauri:dev
```

3. 打包并安装：

```bash
npm run tauri:build
rm -rf "/Applications/Terminal Pane Launcher.app"
cp -R "src-tauri/target/release/bundle/macos/Terminal Pane Launcher.app" "/Applications/Terminal Pane Launcher.app"
xattr -dr com.apple.quarantine "/Applications/Terminal Pane Launcher.app" 2>/dev/null || true
```

打包产物会生成在：

```text
src-tauri/target/release/bundle/macos/Terminal Pane Launcher.app
src-tauri/target/release/bundle/dmg/Terminal Pane Launcher_0.1.0_aarch64.dmg
```

### Windows

1. 双击环境检查脚本：

```text
软件Windows环境检查安装脚本.bat
```

2. 开发模式启动：

```powershell
npm install
npm run tauri:dev
```

3. 旧后端需要保留这些文件：

```text
Start-TerminalLayout.ps1
src/TerminalLayout.psm1
```

## macOS 行为

macOS 版本的启动逻辑会先尝试 iTerm2 的 split panes。没有 iTerm2 时，会退回到 Terminal.app 多窗口方案。

### Prompt 传递方式

macOS 有两种 Codex 传递方式：

| Mode | UI 名称 | 行为 |
| --- | --- | --- |
| `direct` | 自动直传 | 把完整合并 Prompt 作为 Codex 初始参数传入 |
| `manual` | 自动挡 | 只启动 Codex，不传长 Prompt |

为了避免 AppleScript 超长字符串问题，启动器会先把完整 Prompt 写入应用配置目录下的临时文件，再让 shell 读取：

```bash
codex --yolo "$(cat /path/to/temp/codex-run-args/run-args.md; rm -f /path/to/temp/codex-run-args/run-args.md)"
```

App 启动时也会自动补齐这些常见环境：

```text
/opt/homebrew/bin
brew shellenv
fnm env --use-on-cd
~/.nvm/nvm.sh
```

## Prompt 规则

### Prompt 导入

点击 `一键导入提示词` 可以从 Markdown 文件批量导入最多 `20` 个用户 Prompt。

- 只导入 Prompt 文本，不写 pane 配置。
- 目录、标题、启用状态、模板和传递方式都会沿用当前面板设置。
- 每个 Prompt 用独立一行的 `---PROMPT---` 分隔。
- 超过 `20` 段会直接截断。

详细规则见 [`docs/prompt-import-format.md`](docs/prompt-import-format.md)，示例文件见 [`templates/提示词导入示例.md`](templates/提示词导入示例.md)。

### Query 模板

`query 标注专用` 使用 Markdown 模板来做定制化启动。

- 模板里使用 `{{anchor}}` 作为锚点占位符。
- 选中文本后点 `设为锚点`，模板会自动插入锚点。
- 每个 pane 可以为锚点填写自己的差异值。
- `上传模板` 和 `自定义模板` 都会覆盖当前 query 模板。
- `选择目录` 后会自动把目录名填到 title。
- `Codex` 默认是 `yolo`。
- 可以一键把全部 pane 切成 `resume`，也可以一键切回 `new`。
- `resume` 默认按 pane 目录找最近会话。

如果你只想快速起一批相似 prompt，这个模式会比手工拼每个 pane 更轻。

## 配置位置

当前 Tauri 应用保存 GUI 配置到平台应用配置目录。

### macOS

```text
~/Library/Application Support/com.local.terminal-pane-launcher/layout.json
```

### 浏览器预览

```text
localStorage
```

### 旧项目内配置

```text
config/layout.json
```

这个旧路径只作为迁移参考，打包版和当前 Tauri app 运行时不会再依赖项目工作目录。

## 项目结构

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

## 常用命令

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

## 常见问题

### macOS app 里找不到 Codex

先确认终端本身能找到：

```bash
command -v codex
```

如果终端里能找到但 app 里找不到，重新打包安装最新版。当前版本会自动加载 Homebrew、fnm 和 nvm 环境。

### query 的 resume 没有恢复到预期会话

先检查该 pane 目录下是否真的存在 Codex 会话。当前实现会优先找同目录最近会话，再用 `codex resume <session-id> <prompt>` 启动；如果找不到对应会话，会退回 `codex resume --last`。

### macOS pane 打开了但没有启动 Codex

检查该 pane 的 delivery 是否为 `自动直传`。只有在 `direct` 模式下，macOS 才会把完整 Prompt 直接送给 Codex。

### 一键复制失败

Tauri 环境下复制不依赖浏览器 Clipboard API，而是调用平台剪贴板：

```text
macOS: NSPasteboard
Windows: PowerShell Set-Clipboard
Linux: wl-copy / xclip / xsel
```

如果仍失败，先确认系统命令可用。

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

## 版本说明

当前版本重点：

- macOS 打包版可通过 Spotlight 启动。
- macOS iTerm2 自动大窗口 split panes。
- macOS 默认自动直传完整 Prompt 到 Codex 初始参数。
- `query 标注专用` 支持模板、锚点、自动选目录、全选有目录 pane 和一键清空。
- `query 标注专用` 的 resume 会优先按 pane 目录恢复最近会话。
- Windows 保持旧后端兼容，不改变原有 PowerShell 启动路径。
- 配置读取失败时会回退默认 20 pane，避免打包版空白页。

## Roadmap

- 配置导入/导出。
- GUI 内编辑模板。
- Windows 端更稳定的直传实验模式。
- macOS iTerm2 布局细节和窗口行为继续优化。
