import { invoke } from "@tauri-apps/api/core";
import type { AppPlatform, AppSettings, LauncherConfig } from "./types";
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

export async function getCurrentPlatform(): Promise<AppPlatform> {
  if (isTauriRuntime()) {
    return invoke<AppPlatform>("current_platform");
  }

  const platform = window.navigator.platform.toLowerCase();
  if (platform.includes("mac")) return "macos";
  if (platform.includes("win")) return "windows";
  if (platform.includes("linux")) return "linux";
  return "unknown";
}

export async function loadConfig(platform: AppPlatform): Promise<LauncherConfig> {
  let raw = "";

  if (isTauriRuntime()) {
    try {
      raw = await invoke<string>("read_config_file");
    } catch (error) {
      console.warn("Tauri config read failed; using default config.", error);
    }
  }

  if (!raw.trim()) {
    raw = localStorage.getItem(CONFIG_STORAGE_KEY) ?? "";
  }

  if (!raw.trim()) {
    return repairConfig(null, platform);
  }

  try {
    return repairConfig(parseJsonConfig(raw), platform);
  } catch (error) {
    console.warn("Config parse failed; using default config.", error);
    return repairConfig(null, platform);
  }
}

export async function loadWindowsBackendConfig(
  settings: AppSettings,
  platform: AppPlatform,
): Promise<LauncherConfig> {
  if (!isTauriRuntime()) {
    throw new Error("Windows backend config import needs the Tauri desktop runtime.");
  }

  const raw = await invoke<string>("read_windows_backend_config", {
    backendPath: settings.windowsBackendPath,
  });
  return repairConfig(parseJsonConfig(raw), platform);
}

export async function saveConfig(
  config: LauncherConfig,
  platform: AppPlatform,
): Promise<void> {
  const repaired = repairConfig(config, platform);
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
    return "浏览器预览模式：启动平台后端需要在 Tauri 中运行。";
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
    return "浏览器预览模式：启动平台后端需要在 Tauri 中运行。";
  }

  return invoke<string>("launch_windows_backend", {
    backendPath: settings.windowsBackendPath,
    configJson: JSON.stringify(config, null, 2),
    skipPathCheck: false,
  });
}

export async function writeClipboardText(text: string): Promise<void> {
  if (isTauriRuntime()) {
    await invoke("write_clipboard_text", { text });
    return;
  }

  await navigator.clipboard.writeText(text);
}
