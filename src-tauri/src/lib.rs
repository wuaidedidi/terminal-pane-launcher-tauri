use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use serde::Deserialize;
use tauri::{path::BaseDirectory, Manager};

const MAX_PANES: usize = 20;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LauncherConfig {
    #[serde(default)]
    window_mode: String,
    #[serde(default)]
    default_profile: String,
    #[serde(default)]
    panes: Vec<PaneConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaneConfig {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    title: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    profile: String,
    #[serde(default)]
    startup_command: String,
    #[serde(default)]
    codex_mode: String,
    #[serde(default)]
    codex_prompt: String,
    #[serde(default)]
    codex_template: String,
    #[serde(default)]
    codex_tool_template: String,
    #[serde(default)]
    codex_prompt_delivery: String,
}

#[derive(Clone, Debug)]
struct MacPanePlan {
    pane_number: usize,
    title: String,
    profile: String,
    path: PathBuf,
    shell_command: String,
    preview_command: String,
    delivery: Option<String>,
}

fn project_root() -> Result<PathBuf, String> {
    let cwd = env::current_dir().map_err(|error| error.to_string())?;

    if cwd.join("src-tauri").is_dir() {
        return Ok(cwd);
    }

    if cwd.file_name().and_then(|name| name.to_str()) == Some("src-tauri") {
        return cwd
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| "Unable to resolve project root from src-tauri.".to_string());
    }

    Ok(cwd)
}

fn config_path(file_name: &str) -> Result<PathBuf, String> {
    let path = project_root()?.join("config");
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(path.join(file_name))
}

fn app_config_path(app: &tauri::AppHandle, file_name: &str) -> Result<PathBuf, String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(path.join(file_name))
}

fn codex_run_args_temp_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join("temp")
        .join("codex-run-args");
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

fn cleanup_codex_run_args_temp_files(app: &tauri::AppHandle) -> Result<(), String> {
    let directory = codex_run_args_temp_dir(app)?;
    for entry in fs::read_dir(&directory)
        .map_err(|error| format!("Failed to read {}: {}", directory.display(), error))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if file_type.is_file() && file_name.ends_with("-run-args.md") {
            fs::remove_file(entry.path()).map_err(|error| {
                format!("Failed to remove stale Codex temp file {file_name}: {error}")
            })?;
        }
    }
    Ok(())
}

fn looks_like_windows_backend(path: &Path) -> bool {
    path.join("Start-TerminalLayout.ps1").is_file()
        && path.join("src").join("TerminalLayout.psm1").is_file()
}

#[tauri::command]
fn current_platform() -> &'static str {
    if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}

fn resolve_windows_backend_path(backend_path: Option<String>) -> Result<PathBuf, String> {
    if let Some(value) = backend_path {
        if !value.trim().is_empty() {
            let path = PathBuf::from(value.trim());
            if looks_like_windows_backend(&path) {
                return Ok(path);
            }

            return Err(format!(
                "Windows backend path is invalid: {}",
                path.display()
            ));
        }
    }

    let detected = detect_windows_backend_path()?;
    Ok(PathBuf::from(detected))
}

#[tauri::command]
fn detect_windows_backend_path() -> Result<String, String> {
    if !cfg!(windows) {
        return Ok(String::new());
    }

    let root = project_root()?;

    if looks_like_windows_backend(&root) {
        return Ok(root.display().to_string());
    }

    if let Some(parent) = root.parent() {
        for entry in fs::read_dir(parent).map_err(|error| error.to_string())? {
            let path = entry.map_err(|error| error.to_string())?.path();
            if looks_like_windows_backend(&path) {
                return Ok(path.display().to_string());
            }
        }
    }

    Err("Could not auto-detect the Windows PowerShell backend.".to_string())
}

#[tauri::command]
fn read_config_file(app: tauri::AppHandle) -> Result<String, String> {
    let path = app_config_path(&app, "layout.json")?;
    if !path.is_file() {
        return Ok(String::new());
    }

    fs::read_to_string(path).map_err(|error| error.to_string())
}

#[tauri::command]
fn read_windows_backend_config(backend_path: Option<String>) -> Result<String, String> {
    let backend = resolve_windows_backend_path(backend_path)?;
    let path = backend.join("config").join("layout.json");
    if !path.is_file() {
        return Err(format!(
            "Windows backend config does not exist: {}",
            path.display()
        ));
    }

    fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read Windows backend config {}: {}", path.display(), error))
}

#[tauri::command]
fn save_config_file(app: tauri::AppHandle, config_json: String) -> Result<(), String> {
    let path = app_config_path(&app, "layout.json")?;
    fs::write(path, config_json).map_err(|error| error.to_string())
}

#[tauri::command]
fn write_clipboard_text(text: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return write_macos_clipboard_text(&text);
    }

    #[cfg(windows)]
    {
        return write_windows_clipboard_text(&text);
    }

    #[cfg(not(any(target_os = "macos", windows)))]
    {
        for (command, args) in [
            ("wl-copy", Vec::<&str>::new()),
            ("xclip", vec!["-selection", "clipboard"]),
            ("xsel", vec!["--clipboard", "--input"]),
        ] {
            if write_clipboard_with_command(command, &args, &text).is_ok() {
                return Ok(());
            }
        }

        Err("No supported clipboard command was found.".to_string())
    }
}

#[cfg(target_os = "macos")]
fn write_macos_clipboard_text(text: &str) -> Result<(), String> {
    use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
    use objc2_foundation::NSString;

    let pasteboard = NSPasteboard::generalPasteboard();
    pasteboard.clearContents();

    let string = NSString::from_str(text);
    let did_write = unsafe { pasteboard.setString_forType(&string, NSPasteboardTypeString) };
    if did_write {
        Ok(())
    } else {
        Err("Failed to write text to the macOS pasteboard.".to_string())
    }
}

#[cfg(windows)]
fn write_windows_clipboard_text(text: &str) -> Result<(), String> {
    let temp_path = env::temp_dir().join(format!(
        "terminal-pane-launcher-clipboard-{}.txt",
        unique_suffix()
    ));
    fs::write(&temp_path, text.as_bytes())
        .map_err(|error| format!("Failed to write clipboard temp file: {error}"))?;

    let escaped_path = temp_path.display().to_string().replace('\'', "''");
    let script = format!(
        "$p = '{}'; $text = [IO.File]::ReadAllText($p, [Text.UTF8Encoding]::new($false)); Set-Clipboard -Value $text",
        escaped_path
    );

    let output_result = Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(script)
        .output()
        .map_err(|error| format!("Failed to start PowerShell Set-Clipboard: {error}"));

    let _ = fs::remove_file(&temp_path);
    let output = output_result?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            "PowerShell Set-Clipboard failed.".to_string()
        } else {
            stderr
        })
    }
}

#[cfg(not(target_os = "macos"))]
fn write_clipboard_with_command(command: &str, args: &[&str], text: &str) -> Result<(), String> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start {command}: {error}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(text.as_bytes())
            .map_err(|error| format!("Failed to write clipboard text to {command}: {error}"))?;
        drop(stdin);
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("Failed to wait for {command}: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!("{command} failed to write clipboard text.")
        } else {
            stderr
        })
    }
}

#[tauri::command]
fn read_template_text(
    app: tauri::AppHandle,
    backend_path: Option<String>,
    template_name: String,
) -> Result<String, String> {
    read_template_file(&app, backend_path, &template_name)
}

fn read_template_file(
    app: &tauri::AppHandle,
    backend_path: Option<String>,
    template_name: &str,
) -> Result<String, String> {
    let file_name = Path::new(template_name)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Template name is invalid.".to_string())?;

    if file_name != template_name {
        return Err("Template name must not contain path separators.".to_string());
    }

    let mut checked_paths = Vec::new();

    if let Ok(root) = project_root() {
        let path = root.join("templates").join(file_name);
        checked_paths.push(path.display().to_string());
        if path.is_file() {
            return fs::read_to_string(&path)
                .map_err(|error| format!("Failed to read template {}: {}", path.display(), error));
        }
    }

    if let Ok(path) = app
        .path()
        .resolve(format!("templates/{file_name}"), BaseDirectory::Resource)
    {
        checked_paths.push(path.display().to_string());
        if path.is_file() {
            return fs::read_to_string(&path)
                .map_err(|error| format!("Failed to read template {}: {}", path.display(), error));
        }
    }

    match resolve_windows_backend_path(backend_path) {
        Ok(backend) => {
            let path = backend.join(file_name);
            checked_paths.push(path.display().to_string());
            if path.is_file() {
                return fs::read_to_string(&path).map_err(|error| {
                    format!("Failed to read template {}: {}", path.display(), error)
                });
            }
        }
        Err(error) => checked_paths.push(format!("Windows backend fallback unavailable: {error}")),
    }

    Err(format!(
        "Template {} was not found. Checked:\n{}",
        template_name,
        checked_paths.join("\n")
    ))
}

#[tauri::command]
fn pick_directory() -> Result<Option<String>, String> {
    if cfg!(windows) {
        return pick_windows_directory();
    }

    if cfg!(target_os = "macos") {
        return pick_macos_directory();
    }

    Err("Directory selection is only implemented for Windows and macOS.".to_string())
}

fn pick_windows_directory() -> Result<Option<String>, String> {
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.FolderBrowserDialog
$dialog.Description = 'Select working directory'
$dialog.ShowNewFolderButton = $true
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
  $encoding = New-Object System.Text.UTF8Encoding $false
  $bytes = $encoding.GetBytes($dialog.SelectedPath)
  [Console]::OpenStandardOutput().Write($bytes, 0, $bytes.Length)
  exit 0
}
exit 1223
"#;

    let output = Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-STA")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(script)
        .output()
        .map_err(|error| error.to_string())?;

    if output.status.success() {
        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok((!selected.is_empty()).then_some(selected));
    }

    if output.status.code() == Some(1223) {
        return Ok(None);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        Err("Windows directory selection failed.".to_string())
    } else {
        Err(stderr)
    }
}

fn pick_macos_directory() -> Result<Option<String>, String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(r#"POSIX path of (choose folder with prompt "Select working directory")"#)
        .output()
        .map_err(|error| error.to_string())?;

    if output.status.success() {
        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok((!selected.is_empty()).then_some(selected));
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.to_lowercase().contains("user canceled") {
        return Ok(None);
    }

    if stderr.is_empty() {
        Err("macOS directory selection failed.".to_string())
    } else {
        Err(stderr)
    }
}

fn parse_launcher_config(config_json: &str) -> Result<LauncherConfig, String> {
    serde_json::from_str(config_json)
        .map_err(|error| format!("Failed to parse launcher config JSON: {error}"))
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn resolve_launcher_path(path: &str) -> PathBuf {
    let trimmed = path.trim();
    if trimmed == "~" {
        return home_dir().unwrap_or_default();
    }

    if let Some(rest) = trimmed.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }

    if let Some(rest) = trimmed.strip_prefix("$HOME/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }

    if let Some(rest) = trimmed.strip_prefix("${HOME}/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }

    PathBuf::from(trimmed)
}

fn uses_codex(pane: &PaneConfig) -> bool {
    !is_blank(&pane.codex_mode)
        || !is_blank(&pane.codex_prompt)
        || pane.codex_prompt_delivery.trim() == "direct"
}

fn normalized_delivery(value: &str) -> Result<String, String> {
    let delivery = if is_blank(value) { "manual" } else { value.trim() };
    match delivery {
        "manual" | "direct" => Ok(delivery.to_string()),
        "file" | "auto" => Ok("manual".to_string()),
        _ => Err("Codex prompt delivery must be manual or direct.".to_string()),
    }
}

fn codex_mode_args(mode: &str) -> Vec<&'static str> {
    let normalized = if is_blank(mode) { "yolo" } else { mode.trim() };
    match normalized {
        "yolo" => vec!["--yolo"],
        "dangerous" => vec!["--dangerously-bypass-approvals-and-sandbox"],
        "full-auto" => vec!["--full-auto"],
        _ => Vec::new(),
    }
}

fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn applescript_quote(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\r', "")
            .replace('\n', "\\n")
    )
}

fn safe_file_stem(title: &str) -> String {
    let stem: String = title
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if stem.is_empty() {
        "pane".to_string()
    } else {
        stem
    }
}

fn unique_suffix() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();

    format!("{}-{}", millis, std::process::id())
}

fn write_codex_launcher_file(
    app: &tauri::AppHandle,
    pane_title: &str,
    content: &str,
) -> Result<PathBuf, String> {
    let directory = codex_run_args_temp_dir(app)?;

    let file_name = format!(
        "{}-{}-run-args.md",
        unique_suffix(),
        safe_file_stem(pane_title)
    );
    let path = directory.join(file_name);
    fs::write(&path, content)
        .map_err(|error| format!("Failed to write {}: {}", path.display(), error))?;
    Ok(path)
}

fn new_codex_merged_prompt(
    app: &tauri::AppHandle,
    pane: &PaneConfig,
) -> Result<String, String> {
    let template_name = if is_blank(&pane.codex_template) {
        "全栈的提示词留档.md"
    } else {
        pane.codex_template.trim()
    };
    let tool_template_name = if is_blank(&pane.codex_tool_template) {
        "codex的模板.md"
    } else {
        pane.codex_tool_template.trim()
    };
    let shared = read_template_file(app, None, template_name)?;
    let tool = read_template_file(app, None, tool_template_name)?;
    let mut parts = Vec::new();

    if !is_blank(&pane.codex_prompt) {
        parts.push("## User Prompt".to_string());
        parts.push(pane.codex_prompt.trim().to_string());
    }

    parts.push(format!("## Shared Prompt Template: {template_name}"));
    parts.push(shared.trim().to_string());
    parts.push(format!("## Tool Prompt Template: {tool_template_name}"));
    parts.push(tool.trim().to_string());

    Ok(parts.join("\n\n"))
}

fn build_codex_shell_command(
    app: &tauri::AppHandle,
    pane: &PaneConfig,
    _working_directory: &Path,
    preview: bool,
) -> Result<(String, String, String), String> {
    let mut command_parts = vec!["codex".to_string()];
    let mut preview_parts = vec!["codex".to_string()];

    for argument in codex_mode_args(&pane.codex_mode) {
        command_parts.push(argument.to_string());
        preview_parts.push(argument.to_string());
    }

    let delivery = normalized_delivery(&pane.codex_prompt_delivery)?;
    if delivery == "manual" {
        return Ok((
            command_parts.join(" "),
            preview_parts.join(" "),
            delivery,
        ));
    }

    let merged_prompt = new_codex_merged_prompt(app, pane)?;

    let prompt_argument = if preview {
        format!("<direct prompt, {} chars>", merged_prompt.len())
    } else {
        let argument_file = write_codex_launcher_file(
            app,
            &pane.title,
            &merged_prompt,
        )?;
        let quoted_path = sh_quote(&argument_file.display().to_string());
        format!("\"$(cat {quoted_path}; rm -f {quoted_path})\"")
    };

    command_parts.push(prompt_argument);
    preview_parts.push(format!("<direct prompt, {} chars>", merged_prompt.len()));

    Ok((
        command_parts.join(" "),
        preview_parts.join(" "),
        delivery,
    ))
}

fn wrap_shell_command(path: &Path, title: &str, command: &str) -> String {
    let mut script = format!("{} cd {} || exit 1; ", mac_shell_bootstrap(), sh_quote(&path.display().to_string()));

    if !is_blank(title) {
        script.push_str(&format!(
            "printf '\\033]0;%s\\007' {}; ",
            sh_quote(title.trim())
        ));
    }

    if is_blank(command) {
        script.push_str("exec \"${SHELL:-/bin/zsh}\" -l");
    } else {
        script.push_str(command.trim());
        script.push_str("; exec \"${SHELL:-/bin/zsh}\" -l");
    }

    script
}

fn mac_default_profile(config: &LauncherConfig) -> String {
    let value = config.default_profile.trim();
    if value.is_empty()
        || value.eq_ignore_ascii_case("windows powershell")
        || value.eq_ignore_ascii_case("powershell")
        || value.eq_ignore_ascii_case("command prompt")
    {
        String::new()
    } else {
        value.to_string()
    }
}

fn action_with_profile(action: &str, profile: &str) -> String {
    if is_blank(profile) {
        format!("{action} with default profile")
    } else {
        format!(
            "{action} with profile {}",
            applescript_quote(profile),
        )
    }
}

fn mac_plan_command(plan: &MacPanePlan, preview: bool) -> &str {
    if preview {
        plan.preview_command.as_str()
    } else {
        plan.shell_command.as_str()
    }
}

fn mac_shell_bootstrap() -> &'static str {
    r#"export PATH="/opt/homebrew/bin:/opt/homebrew/sbin:/usr/local/bin:$PATH";
if [ -r /etc/zprofile ]; then . /etc/zprofile >/dev/null 2>&1 || true; fi;
if command -v brew >/dev/null 2>&1; then eval "$(brew shellenv)" >/dev/null 2>&1 || true; fi;
if command -v fnm >/dev/null 2>&1; then eval "$(fnm env --use-on-cd)" >/dev/null 2>&1 || true; fi;
if [ -s "$HOME/.nvm/nvm.sh" ]; then . "$HOME/.nvm/nvm.sh" >/dev/null 2>&1 || true; fi;
if command -v nvm >/dev/null 2>&1; then nvm use default >/dev/null 2>&1 || nvm use node >/dev/null 2>&1 || true; fi;"#
}

fn build_mac_pane_plans(
    app: &tauri::AppHandle,
    config: &LauncherConfig,
    preview: bool,
    require_existing_path: bool,
) -> Result<Vec<MacPanePlan>, String> {
    let enabled: Vec<(usize, &PaneConfig)> = config
        .panes
        .iter()
        .enumerate()
        .filter(|(_, pane)| pane.enabled)
        .collect();

    let mut errors = Vec::new();
    if enabled.is_empty() {
        errors.push("Enable at least one pane.".to_string());
    }

    if enabled.len() > MAX_PANES {
        errors.push(format!("Pane count cannot exceed {MAX_PANES}."));
    }

    let mut needs_codex = false;
    let default_profile = mac_default_profile(config);
    let mut plans = Vec::new();

    for (source_index, pane) in enabled {
        let pane_number = source_index + 1;
        let path = resolve_launcher_path(&pane.path);
        if pane.path.trim().is_empty() {
            errors.push(format!("Pane {pane_number} path is required."));
            continue;
        }

        if require_existing_path && !path.is_dir() {
            errors.push(format!(
                "Pane {pane_number} path does not exist: {}",
                path.display()
            ));
            continue;
        }

        let profile = if is_blank(&pane.profile) {
            default_profile.clone()
        } else {
            pane.profile.trim().to_string()
        };

        let (inner_command, preview_inner_command, delivery) = if uses_codex(pane) {
            needs_codex = true;
            let (command, preview_command, delivery) =
                build_codex_shell_command(app, pane, &path, preview)?;
            (command, preview_command, Some(delivery))
        } else if !is_blank(&pane.startup_command) {
            (
                pane.startup_command.trim().to_string(),
                pane.startup_command.trim().to_string(),
                None,
            )
        } else {
            (String::new(), String::new(), None)
        };

        plans.push(MacPanePlan {
            pane_number,
            title: pane.title.trim().to_string(),
            profile,
            shell_command: wrap_shell_command(&path, &pane.title, &inner_command),
            preview_command: wrap_shell_command(&path, &pane.title, &preview_inner_command),
            path,
            delivery,
        });
    }

    if needs_codex && !mac_command_exists("codex") {
        errors.push(
            "At least one pane uses Codex, but `codex` was not found from the macOS login shell."
                .to_string(),
        );
    }

    if errors.is_empty() {
        Ok(plans)
    } else {
        Err(errors
            .into_iter()
            .map(|error| format!("- {error}"))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

fn mac_command_exists(command_name: &str) -> bool {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let script = format!(
        "{} command -v {}",
        mac_shell_bootstrap(),
        sh_quote(command_name)
    );
    Command::new(shell)
        .arg("-lc")
        .arg(script)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn get_grid_columns<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
    if items.is_empty() {
        return Vec::new();
    }

    let column_count = (items.len() as f64).sqrt().ceil().min(5.0) as usize;
    let mut columns = Vec::new();

    for column_index in 0..column_count {
        let mut column = Vec::new();
        let mut pane_index = column_index;
        while pane_index < items.len() {
            column.push(items[pane_index].clone());
            pane_index += column_count;
        }

        if !column.is_empty() {
            columns.push(column);
        }
    }

    columns
}

fn build_iterm_applescript(plans: &[MacPanePlan], window_mode: &str, preview: bool) -> String {
    let columns = get_grid_columns(plans);
    let mut lines = vec![
        "tell application \"Finder\"".to_string(),
        "  set desktopBounds to bounds of window of desktop".to_string(),
        "end tell".to_string(),
        "set leftEdge to (item 1 of desktopBounds) + 24".to_string(),
        "set topEdge to (item 2 of desktopBounds) + 48".to_string(),
        "set rightEdge to (item 3 of desktopBounds) - 24".to_string(),
        "set bottomEdge to (item 4 of desktopBounds) - 48".to_string(),
        "tell application id \"com.googlecode.iterm2\"".to_string(),
        "  activate".to_string(),
    ];

    let first = &columns[0][0];
    lines.push(format!("  {}", action_with_profile("create window", &first.profile)));
    lines.push("  delay 0.2".to_string());
    lines.push("  set bounds of current window to {leftEdge, topEdge, rightEdge, bottomEdge}".to_string());
    lines.push("  set pane_0 to current session of current window".to_string());
    lines.push("  tell pane_0".to_string());
    lines.push(format!(
        "    write text {}",
        applescript_quote(mac_plan_command(first, preview))
    ));
    lines.push("  end tell".to_string());

    let mut pane_variable_index = 1usize;
    let mut top_panes = vec!["pane_0".to_string()];

    for column in columns.iter().skip(1) {
        let plan = &column[0];
        let variable = format!("pane_{pane_variable_index}");
        lines.push(format!("    tell pane_0"));
        lines.push(format!(
            "      set {variable} to ({})",
            action_with_profile("split vertically", &plan.profile)
        ));
        lines.push("    end tell".to_string());
        lines.push(format!("    tell {variable}"));
        lines.push(format!(
            "      write text {}",
            applescript_quote(mac_plan_command(plan, preview))
        ));
        lines.push("    end tell".to_string());
        top_panes.push(variable);
        pane_variable_index += 1;
    }

    for (column_index, column) in columns.iter().enumerate() {
        let top_pane = &top_panes[column_index];
        for plan in column.iter().skip(1) {
            let variable = format!("pane_{pane_variable_index}");
            lines.push(format!("    tell {top_pane}"));
            lines.push(format!(
                "      set {variable} to ({})",
                action_with_profile("split horizontally", &plan.profile)
            ));
            lines.push("    end tell".to_string());
            lines.push(format!("    tell {variable}"));
            lines.push(format!(
                "      write text {}",
                applescript_quote(mac_plan_command(plan, preview))
            ));
            lines.push("    end tell".to_string());
            pane_variable_index += 1;
        }
    }

    if window_mode.trim() == "fullscreen" {
        lines.push("    -- Fullscreen is requested; use the iTerm2 profile/window shortcut if the window is not already fullscreen.".to_string());
    } else if window_mode.trim() == "maximized" {
        lines.push("    -- Maximized is requested; iTerm2 keeps the normal macOS window controls for this launch.".to_string());
    }

    lines.push("end tell".to_string());
    lines.join("\n")
}

fn build_terminal_applescript(plans: &[MacPanePlan], window_mode: &str, preview: bool) -> String {
    let mut lines = vec![
        "tell application \"Terminal\"".to_string(),
        "  activate".to_string(),
    ];

    for plan in plans {
        lines.push(format!(
            "  do script {}",
            applescript_quote(mac_plan_command(plan, preview))
        ));
    }

    if window_mode.trim() == "fullscreen" {
        lines.push("  -- Fullscreen is requested; Terminal.app opens normal windows through AppleScript.".to_string());
    } else if window_mode.trim() == "maximized" {
        lines.push("  -- Maximized is requested; Terminal.app opens normal windows through AppleScript.".to_string());
    }

    lines.push("end tell".to_string());
    lines.join("\n")
}

fn build_macos_preview(
    plans: &[MacPanePlan],
    window_mode: &str,
    script: &str,
    terminal_name: &str,
    layout_description: &str,
) -> String {
    let columns = get_grid_columns(plans);
    let row_count = columns.iter().map(Vec::len).max().unwrap_or(0);
    let mut lines = vec![
        format!("macOS {terminal_name} launch plan"),
        format!("Panes: {}", plans.len()),
        format!("Layout: {layout_description}"),
        format!("Window mode: {}", if is_blank(window_mode) { "normal" } else { window_mode }),
        String::new(),
    ];

    if terminal_name == "iTerm2" {
        lines.push(format!("iTerm2 split grid: {} columns x {} rows", columns.len(), row_count));
        lines.push(String::new());
    }

    for plan in plans {
        lines.push(format!("Pane {}: {}", plan.pane_number, if is_blank(&plan.title) { "(untitled)" } else { &plan.title }));
        lines.push(format!("  Directory: {}", plan.path.display()));
        if terminal_name == "iTerm2" {
            lines.push(format!(
                "  iTerm2 profile: {}",
                if is_blank(&plan.profile) {
                    "default"
                } else {
                    &plan.profile
                }
            ));
        }
        if let Some(delivery) = &plan.delivery {
            lines.push(format!("  Codex prompt delivery: {delivery}"));
        }
        lines.push(format!("  Command: {}", plan.preview_command));
    }

    lines.push(String::new());
    lines.push("AppleScript preview:".to_string());
    lines.push(script.to_string());
    lines.join("\n")
}

fn mac_application_available(name: &str, bundle_id: Option<&str>) -> Result<bool, String> {
    let script = if let Some(bundle_id) = bundle_id {
        format!(r#"id of application id "{}""#, bundle_id.replace('"', ""))
    } else {
        format!(r#"id of application "{}""#, name.replace('"', ""))
    };
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|error| format!("Failed to check {name} availability: {error}"))?;

    Ok(output.status.success())
}

fn run_macos_backend(
    app: tauri::AppHandle,
    config_json: String,
    preview: bool,
    skip_path_check: bool,
) -> Result<String, String> {
    if !cfg!(target_os = "macos") {
        return Err("The macOS backend only runs on macOS.".to_string());
    }

    let config = parse_launcher_config(&config_json)?;
    if !preview {
        cleanup_codex_run_args_temp_files(&app)?;
    }
    let plans = build_mac_pane_plans(&app, &config, preview, !skip_path_check)?;
    let window_mode = if is_blank(&config.window_mode) {
        "normal"
    } else {
        config.window_mode.trim()
    };
    let use_iterm = mac_application_available("iTerm2", Some("com.googlecode.iterm2"))?;
    let (terminal_name, layout_description, script) = if use_iterm {
        (
            "iTerm2",
            "split panes",
            build_iterm_applescript(&plans, window_mode, preview),
        )
    } else {
        (
            "Terminal.app",
            "separate Terminal windows (iTerm2 not installed)",
            build_terminal_applescript(&plans, window_mode, preview),
        )
    };

    if preview {
        return Ok(build_macos_preview(
            &plans,
            window_mode,
            &script,
            terminal_name,
            layout_description,
        ));
    }

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|error| format!("Failed to launch {terminal_name} through osascript: {error}"))?;

    if output.status.success() {
        Ok(format!(
            "macOS {terminal_name} launch started.\n{}",
            build_macos_preview(
                &plans,
                window_mode,
                &script,
                terminal_name,
                layout_description,
            )
        ))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!("{terminal_name} AppleScript launch failed."))
        } else {
            Err(stderr)
        }
    }
}

fn run_windows_backend(
    backend_path: Option<String>,
    config_json: String,
    preview: bool,
    skip_path_check: bool,
) -> Result<String, String> {
    if !cfg!(windows) {
        return Err("The current backend runner only supports Windows.".to_string());
    }

    let backend = resolve_windows_backend_path(backend_path)?;
    let script = backend.join("Start-TerminalLayout.ps1");
    let runtime_config = config_path("runtime-layout.json")?;
    fs::write(&runtime_config, config_json).map_err(|error| error.to_string())?;

    if !preview {
        let mut validation_command = Command::new("powershell.exe");
        validation_command
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(&script)
            .arg("-ConfigPath")
            .arg(&runtime_config)
            .arg("-Preview");

        if skip_path_check {
            validation_command.arg("-SkipPathCheck");
        }

        let validation_output = validation_command.output().map_err(|error| error.to_string())?;
        let validation_stdout = String::from_utf8_lossy(&validation_output.stdout)
            .trim()
            .to_string();
        let validation_stderr = String::from_utf8_lossy(&validation_output.stderr)
            .trim()
            .to_string();

        if !validation_output.status.success() {
            if validation_stderr.is_empty() {
                return Err(validation_stdout);
            }

            return Err(validation_stderr);
        }

        let mut launch_command = Command::new("powershell.exe");
        launch_command
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(script)
            .arg("-ConfigPath")
            .arg(runtime_config);

        if skip_path_check {
            launch_command.arg("-SkipPathCheck");
        }

        launch_command.spawn().map_err(|error| error.to_string())?;
        return Ok(format!("Launch started.\n{}", validation_stdout));
    }

    let mut command = Command::new("powershell.exe");
    command
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(script)
        .arg("-ConfigPath")
        .arg(runtime_config);

    if preview {
        command.arg("-Preview");
    }

    if skip_path_check {
        command.arg("-SkipPathCheck");
    }

    let output = command.output().map_err(|error| error.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        return Ok(stdout);
    }

    if stderr.is_empty() {
        Err(stdout)
    } else {
        Err(stderr)
    }
}

#[tauri::command]
fn preview_windows_backend(
    app: tauri::AppHandle,
    backend_path: Option<String>,
    config_json: String,
    skip_path_check: bool,
) -> Result<String, String> {
    if cfg!(windows) {
        return run_windows_backend(backend_path, config_json, true, skip_path_check);
    }

    if cfg!(target_os = "macos") {
        return run_macos_backend(app, config_json, true, skip_path_check);
    }

    Err("Preview is only implemented for Windows and macOS.".to_string())
}

#[tauri::command]
fn launch_windows_backend(
    app: tauri::AppHandle,
    backend_path: Option<String>,
    config_json: String,
    skip_path_check: bool,
) -> Result<String, String> {
    if cfg!(windows) {
        return run_windows_backend(backend_path, config_json, false, skip_path_check);
    }

    if cfg!(target_os = "macos") {
        return run_macos_backend(app, config_json, false, skip_path_check);
    }

    Err("Launch is only implemented for Windows and macOS.".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let _ = cleanup_codex_run_args_temp_files(&app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            current_platform,
            detect_windows_backend_path,
            read_config_file,
            read_windows_backend_config,
            save_config_file,
            write_clipboard_text,
            read_template_text,
            pick_directory,
            preview_windows_backend,
            launch_windows_backend,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
