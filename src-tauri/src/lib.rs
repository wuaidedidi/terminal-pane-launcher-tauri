use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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

fn looks_like_windows_backend(path: &Path) -> bool {
    path.join("Start-TerminalLayout.ps1").is_file()
        && path.join("src").join("TerminalLayout.psm1").is_file()
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
fn read_config_file() -> Result<String, String> {
    let path = config_path("layout.json")?;
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
fn save_config_file(config_json: String) -> Result<(), String> {
    let path = config_path("layout.json")?;
    fs::write(path, config_json).map_err(|error| error.to_string())
}

#[tauri::command]
fn read_template_text(
    backend_path: Option<String>,
    template_name: String,
) -> Result<String, String> {
    let file_name = Path::new(&template_name)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Template name is invalid.".to_string())?;

    if file_name != template_name {
        return Err("Template name must not contain path separators.".to_string());
    }

    let backend = resolve_windows_backend_path(backend_path)?;
    let path = backend.join(file_name);
    fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read template {}: {}", path.display(), error))
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
    backend_path: Option<String>,
    config_json: String,
    skip_path_check: bool,
) -> Result<String, String> {
    run_windows_backend(backend_path, config_json, true, skip_path_check)
}

#[tauri::command]
fn launch_windows_backend(
    backend_path: Option<String>,
    config_json: String,
    skip_path_check: bool,
) -> Result<String, String> {
    run_windows_backend(backend_path, config_json, false, skip_path_check)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            detect_windows_backend_path,
            read_config_file,
            read_windows_backend_config,
            save_config_file,
            read_template_text,
            pick_directory,
            preview_windows_backend,
            launch_windows_backend,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
