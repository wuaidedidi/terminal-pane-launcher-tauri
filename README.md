# Terminal Pane Launcher Tauri

这是新版跨平台桌面壳，技术栈是 `Tauri 2 + Vue 3 + TypeScript + Vite`。当前 Windows 版 PowerShell 项目会继续保留，并作为 Windows 后端被这个 Tauri GUI 调用。

## 当前状态

- Windows 后端：已接入现有 `Start-TerminalLayout.ps1` 和 `src/TerminalLayout.psm1`。
- Windows GUI：Tauri/Vue 新界面已搭好，可配置 20 个 pane、Codex Prompt、模板和一键复制。
- macOS 后端：已接入 `iTerm2 + AppleScript/osascript`，支持按 pane 配置启动 split panes。
- 环境脚本：已提供 Windows 和 macOS 的检查/安装脚本。

## 目录关系

建议保持现在这种同级目录结构：

```text
制作一键启动多终端软件/
  制作一键启动多终端软件/          # 原 Windows PowerShell 后端
    Start-TerminalLayout.ps1
    src/TerminalLayout.psm1

  terminal-pane-launcher-tauri/     # 新 Tauri 跨平台 GUI
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

提示词模板会优先从当前 Tauri 项目的 `templates/` 读取，并在打包时一起带入安装包。只有导入旧配置、Windows 预览和 Windows 启动时，Tauri 程序才会自动扫描同级目录，找到包含下面文件的 Windows 后端：

```text
Start-TerminalLayout.ps1
src/TerminalLayout.psm1
```

## 最简单使用方式

### Windows

1. 双击项目根目录里的：

```text
软件Windows环境检查安装脚本.bat
```

2. 脚本会检查并尝试安装：

```text
fnm
Node/npm
Rust/rustup/cargo
Visual Studio Build Tools + C++ 组件
WebView2 Runtime
```

3. 如果脚本安装了 Rust、fnm 或 Visual Studio Build Tools，请关闭当前终端窗口，再重新打开 PowerShell。

4. 进入新 Tauri 目录：

```powershell
cd "D:\work\全栈项目\制作一键启动多终端软件\terminal-pane-launcher-tauri"
```

5. 安装前端依赖：

```powershell
npm install
```

6. 启动真正的 Tauri 桌面软件：

```powershell
npm run tauri:dev
```

也可以直接双击 Tauri 版启动脚本：

```text
软件Windows一键启动Tauri版.bat
```

这个 `.bat` 只负责打开 PowerShell 并执行 `scripts/start-tauri-windows.ps1`，避免 cmd 在中文路径/中文文件名下解析异常。PowerShell 脚本会自动进入当前目录、加载 `fnm` 的 Node 环境、补充 Cargo PATH、检查 `npm/cargo`，缺少 `node_modules` 时自动执行 `npm install`，最后运行 `npm run tauri:dev`。

### macOS

1. 双击项目根目录里的：

```text
软件Mac环境检查安装脚本.command
```

2. 如果 macOS 提示没有执行权限，先在终端执行：

```bash
chmod +x 软件Mac环境检查安装脚本.command
```

也可以不用先 `chmod`，直接用 `bash` 运行一次环境脚本；脚本启动后会自动给两个 macOS `.command` 文件补执行权限：

```bash
bash 软件Mac环境检查安装脚本.command
```

3. 脚本会检查并尝试安装：

```text
Homebrew
fnm
Node/npm
Rust/rustup/cargo
Xcode Command Line Tools
iTerm2（推荐，用于真正的 split panes）
```

4. 安装完成后关闭当前终端，再重新打开终端。

5. 进入项目目录并启动：

```bash
cd "/你的路径/terminal-pane-launcher-tauri"
npm install
npm run tauri:dev
```

也可以直接双击 Tauri 版启动脚本：

```text
软件Mac一键启动Tauri版.command
```

如果 macOS 提示没有执行权限，先执行：

```bash
chmod +x 软件Mac一键启动Tauri版.command
```

如果还没授权，也可以先用 `bash` 跑一次环境脚本，它会自动给环境脚本和启动脚本都补上执行权限：

```bash
bash 软件Mac环境检查安装脚本.command
```

注意：macOS 推荐安装 iTerm2。`Save & Launch` 会优先通过 `osascript` 控制 iTerm2 创建 split panes，并按每行配置进入工作目录、执行 startupCommand 或 Codex；如果没有 iTerm2，会降级用系统 Terminal.app 打开多个窗口。

## 常用命令

检查当前系统环境：

```powershell
npm run env
```

尝试安装当前系统缺失环境：

```powershell
npm run env:install
```

只启动浏览器前端预览：

```powershell
npm run dev
```

启动真正的 Tauri 桌面软件：

```powershell
npm run tauri:dev
```

打包桌面软件：

```powershell
npm run tauri:build
```

## Windows 手动环境安装

如果不想双击脚本，也可以手动执行：

```powershell
winget install --id Rustlang.Rustup -e --source winget
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --source winget
```

安装 Visual Studio Build Tools 时，需要包含：

```text
Desktop development with C++
MSVC
Windows SDK
```

安装后重新打开 PowerShell，检查：

```powershell
rustc --version
cargo --version
node --version
npm --version
```

## 环境脚本

Windows 检查：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-env.ps1
```

Windows 检查并安装：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-env.ps1 -Install
```

macOS 检查：

```bash
bash scripts/check-env.sh
```

macOS 检查并安装：

```bash
bash scripts/check-env.sh --install
```

## 功能说明

当前 Tauri GUI 已支持：

- 最多 20 个 pane。
- 每行配置标题、工作目录、Codex 设置。
- Codex 默认 `yolo`。
- Windows 保持原有 Prompt delivery 兼容；macOS 只显示两种：`自动挡/manual` 启动时只运行 Codex，不传长 Prompt；`自动直传/direct` 会把完整合并 Prompt 作为 Codex 初始参数传入。
- 每行最右侧 `一键复制` 会复制完整合并 Prompt。
- Prompt 合并顺序固定为：

```text
1. User Prompt
2. Shared Prompt Template
3. Tool Prompt Template
```

Windows 启动时会调用原后端：

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File <backend>\Start-TerminalLayout.ps1 -ConfigPath <runtime-config>
```

macOS 启动时会调用内置后端：

```text
osascript -> iTerm2 AppleScript；未安装 iTerm2 时降级到 Terminal.app AppleScript
```

macOS 后端会读取当前 Tauri 项目的 `templates/`，并只显示 `manual` 和 `direct` 两种 Codex prompt delivery。`direct` 会把完整合并 Prompt 作为 Codex 初始参数传入；为了避免 AppleScript 超长字符串问题，启动器会用临时 `run-args` 文件做 shell 传输，但不会让 Codex 再去读取 Prompt 文件。Windows 端继续保留旧配置和旧后端的 delivery 兼容。

## 配置文件

Tauri 新 GUI 会保存自己的配置到：

```text
config/layout.json
```

预览/启动时会临时写入：

```text
config/runtime-layout.json
```

原 Windows 后端的配置不会被移动。Windows 上新 GUI 会生成兼容后端的 JSON，再交给后端启动 Windows Terminal；macOS 上则直接由 Tauri 内置后端读取同一份 JSON，优先启动 iTerm2，未安装时降级到 Terminal.app。

## 常见问题

### `npm` 不是内部或外部命令

说明当前终端没有加载 `fnm` 的 Node 环境。推荐重新打开 PowerShell，或者执行：

```powershell
fnm env --use-on-cd | Out-String | Invoke-Expression
```

然后检查：

```powershell
node --version
npm --version
```

### `failed to run cargo metadata`

说明缺少 Rust/Cargo。运行：

```powershell
npm run env:install
```

或者双击：

```text
软件Windows环境检查安装脚本.bat
```

### `搜索源时失败: msstore`

这是 Microsoft Store 源网络失败。脚本已改成安装时强制使用 `--source winget`，避免被 `msstore` 源影响。

如果仍然失败，可以先执行：

```powershell
winget source update
```

然后重新双击 Windows 环境脚本。

### 安装完成后还是找不到 `cargo`

关闭当前终端，重新打开 PowerShell 再检查：

```powershell
cargo --version
rustc --version
```

如果仍然找不到，可以临时补 PATH：

```powershell
$env:Path += ";$env:USERPROFILE\.cargo\bin"
```

### Visual Studio Build Tools 安装后仍报 MSVC 缺失

重新运行：

```powershell
npm run env
```

如果仍然缺失，打开 Visual Studio Installer，确认安装了：

```text
Desktop development with C++
MSVC
Windows SDK
```

## 后续计划

- macOS 后端：继续完善 iTerm2 窗口尺寸/全屏控制和 Terminal.app 兜底体验。
- 打包发布：生成 Windows 安装包和 macOS `.dmg`。
- 配置导入导出：方便在多台机器复用 pane 配置。
- 模板管理：在 GUI 中直接编辑和切换 Prompt 模板。
