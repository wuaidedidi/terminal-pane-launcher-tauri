import type { AppSettings, LauncherConfig, PaneConfig, PromptDelivery } from "./types";

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

export function createDefaultPane(index: number): PaneConfig {
  return {
    enabled: index === 0,
    title: `Pane ${index + 1}`,
    path: "",
    profile: "",
    startupCommand: "",
    codexMode: "",
    codexPrompt: "",
    codexTemplate: SHARED_TEMPLATES[0],
    codexToolTemplate: TOOL_TEMPLATES[0],
    codexPromptDelivery: "manual",
  };
}

export function createDefaultConfig(): LauncherConfig {
  return {
    schemaVersion: 1,
    windowMode: "maximized",
    layoutMode: "balancedGrid",
    defaultProfile: "Windows PowerShell",
    panes: Array.from({ length: MAX_PANES }, (_, index) => createDefaultPane(index)),
  };
}

export function repairConfig(input: Partial<LauncherConfig> | null | undefined): LauncherConfig {
  const defaults = createDefaultConfig();
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
      codexPromptDelivery:
        (sourcePanes[index]?.codexPromptDelivery as PromptDelivery | undefined) ||
        defaultPane.codexPromptDelivery,
    })),
  };
}

export function createDefaultSettings(): AppSettings {
  return {
    windowsBackendPath: "",
  };
}
