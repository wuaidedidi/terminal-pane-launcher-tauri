import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, LauncherConfig } from "./types";
import { createDefaultSettings, repairConfig } from "./defaults";

const CONFIG_STORAGE_KEY = "terminal-pane-launcher.config";
const SETTINGS_STORAGE_KEY = "terminal-pane-launcher.settings";

function parseJsonConfig(raw: string): Partial<LauncherConfig> {
  return JSON.parse(raw.replace(/^\uFEFF/, "")) as Partial<LauncherConfig>;
}

function isTauriRuntime(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export async function detectWindowsBackendPath(): Promise<string> {
  if (!isTauriRuntime()) {
    return "";
  }

  return invoke<string>("detect_windows_backend_path");
}

export async function loadConfig(): Promise<LauncherConfig> {
  if (isTauriRuntime()) {
    const raw = await invoke<string>("read_config_file");
    if (raw.trim().length > 0) {
      return repairConfig(parseJsonConfig(raw));
    }
  }

  const local = localStorage.getItem(CONFIG_STORAGE_KEY);
  return repairConfig(local ? parseJsonConfig(local) : null);
}

export async function loadWindowsBackendConfig(
  settings: AppSettings,
): Promise<LauncherConfig> {
  if (!isTauriRuntime()) {
    throw new Error("Windows backend config import needs the Tauri desktop runtime.");
  }

  const raw = await invoke<string>("read_windows_backend_config", {
    backendPath: settings.windowsBackendPath,
  });
  return repairConfig(parseJsonConfig(raw));
}

export async function saveConfig(config: LauncherConfig): Promise<void> {
  const repaired = repairConfig(config);
  const json = JSON.stringify(repaired, null, 2);

  if (isTauriRuntime()) {
    await invoke("save_config_file", { configJson: json });
  } else {
    localStorage.setItem(CONFIG_STORAGE_KEY, json);
  }
}

export function loadSettings(): AppSettings {
  const local = localStorage.getItem(SETTINGS_STORAGE_KEY);
  if (!local) {
    return createDefaultSettings();
  }

  return {
    ...createDefaultSettings(),
    ...(JSON.parse(local) as Partial<AppSettings>),
  };
}

export function saveSettings(settings: AppSettings): void {
  localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings, null, 2));
}

export async function readTemplateText(
  settings: AppSettings,
  templateName: string,
): Promise<string> {
  if (!isTauriRuntime()) {
    return `# ${templateName}\n\n浏览器预览模式不会读取本地模板文件。`;
  }

  return invoke<string>("read_template_text", {
    backendPath: settings.windowsBackendPath,
    templateName,
  });
}

export async function pickDirectory(): Promise<string | null> {
  if (!isTauriRuntime()) {
    throw new Error("Directory selection needs the Tauri desktop runtime.");
  }

  return invoke<string | null>("pick_directory");
}

export async function previewWindowsBackend(
  config: LauncherConfig,
  settings: AppSettings,
): Promise<string> {
  if (!isTauriRuntime()) {
    return "浏览器预览模式：启动 Windows 后端需要在 Tauri 中运行。";
  }

  return invoke<string>("preview_windows_backend", {
    backendPath: settings.windowsBackendPath,
    configJson: JSON.stringify(config, null, 2),
    skipPathCheck: true,
  });
}

export async function launchWindowsBackend(
  config: LauncherConfig,
  settings: AppSettings,
): Promise<string> {
  if (!isTauriRuntime()) {
    return "浏览器预览模式：启动 Windows 后端需要在 Tauri 中运行。";
  }

  return invoke<string>("launch_windows_backend", {
    backendPath: settings.windowsBackendPath,
    configJson: JSON.stringify(config, null, 2),
    skipPathCheck: false,
  });
}

export async function writeClipboardText(text: string): Promise<void> {
  await navigator.clipboard.writeText(text);
}
