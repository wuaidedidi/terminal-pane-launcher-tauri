import type { AppPlatform, AppSettings, LauncherConfig, PaneConfig, PromptDelivery } from "./types";

export const MAX_PANES = 20;

export const SHARED_TEMPLATES = [
  "全栈的提示词留档.md",
  "跨平台提示词留档.md",
];

export const TOOL_TEMPLATES = [
  "codex的模板.md",
  "claudecode的模板.md",
];

export const DELIVERY_LABELS: Record<PromptDelivery, string> = {
  manual: "自动挡",
  file: "file",
  direct: "direct",
  auto: "auto (direct/file)",
};

export function getDeliveryLabel(value: PromptDelivery, platform: AppPlatform): string {
  if (platform === "macos") {
    return value === "direct" ? "自动直传" : "自动挡";
  }

  return DELIVERY_LABELS[value];
}

function normalizePromptDelivery(value: unknown, platform: AppPlatform): PromptDelivery {
  if (value === "direct") return "direct";
  if (platform !== "macos" && (value === "file" || value === "auto")) return value;
  return platform === "macos" ? "direct" : "manual";
}

export function createDefaultPane(
  index: number,
  enabled = index === 0,
  platform: AppPlatform = "windows",
): PaneConfig {
  return {
    enabled,
    title: `Pane ${index + 1}`,
    path: "",
    profile: "",
    startupCommand: "",
    codexMode: "",
    codexPrompt: "",
    codexTemplate: SHARED_TEMPLATES[0],
    codexToolTemplate: TOOL_TEMPLATES[0],
    codexPromptDelivery: platform === "macos" ? "direct" : "manual",
  };
}

export function createDefaultConfig(platform: AppPlatform = "windows"): LauncherConfig {
  const enableFirstPane = platform !== "macos";

  return {
    schemaVersion: 1,
    windowMode: "maximized",
    layoutMode: "balancedGrid",
    defaultProfile: platform === "macos" ? "" : "Windows PowerShell",
    panes: Array.from({ length: MAX_PANES }, (_, index) =>
      createDefaultPane(index, enableFirstPane && index === 0, platform),
    ),
  };
}

export function repairConfig(
  input: Partial<LauncherConfig> | null | undefined,
  platform: AppPlatform = "windows",
): LauncherConfig {
  const defaults = createDefaultConfig(platform);
  const sourcePanes = Array.isArray(input?.panes) ? input.panes : [];

  return {
    schemaVersion: 1,
    windowMode: input?.windowMode ?? defaults.windowMode,
    layoutMode: "balancedGrid",
    defaultProfile: input?.defaultProfile ?? defaults.defaultProfile,
    panes: defaults.panes.map((defaultPane, index) => ({
      ...defaultPane,
      ...(sourcePanes[index] ?? {}),
      codexTemplate: sourcePanes[index]?.codexTemplate || defaultPane.codexTemplate,
      codexToolTemplate: sourcePanes[index]?.codexToolTemplate || defaultPane.codexToolTemplate,
      codexPromptDelivery: normalizePromptDelivery(
        sourcePanes[index]?.codexPromptDelivery,
        platform,
      ),
    })),
  };
}

export function createDefaultSettings(): AppSettings {
  return {
    windowsBackendPath: "",
  };
}
