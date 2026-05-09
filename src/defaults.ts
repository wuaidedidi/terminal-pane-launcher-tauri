import type {
  AppPlatform,
  AppSettings,
  LauncherConfig,
  PaneConfig,
  PromptDelivery,
  QueryPaneConfig,
  QueryWorkspaceState,
} from "./types";

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
    codexLaunchMode: "new",
    codexResumeSessionId: "",
    codexPromptStyle: "composed",
  };
}

export function createDefaultQueryPane(
  index: number,
  enabled = index === 0,
  platform: AppPlatform = "windows",
): QueryPaneConfig {
  return {
    enabled,
    title: `Pane ${index + 1}`,
    path: "",
    profile: "",
    startupCommand: "",
    codexMode: "yolo",
    codexLaunchMode: "new",
    codexResumeSessionId: "",
    anchorValues: {},
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
      codexLaunchMode: sourcePanes[index]?.codexLaunchMode === "resume" ? "resume" : "new",
      codexResumeSessionId: sourcePanes[index]?.codexResumeSessionId ?? "",
      codexPromptStyle: sourcePanes[index]?.codexPromptStyle === "template" ? "template" : "composed",
    })),
  };
}

export function createDefaultQueryWorkspace(
  platform: AppPlatform = "windows",
): QueryWorkspaceState {
  return {
    schemaVersion: 1,
    templateName: "",
    templateText: "",
    anchors: [],
    panes: Array.from({ length: MAX_PANES }, (_, index) =>
      createDefaultQueryPane(index, index === 0, platform),
    ),
  };
}

export function repairQueryWorkspace(
  input: Partial<QueryWorkspaceState> | null | undefined,
  platform: AppPlatform = "windows",
): QueryWorkspaceState {
  const defaults = createDefaultQueryWorkspace(platform);
  const sourcePanes = Array.isArray(input?.panes) ? input.panes : [];
  const anchors = Array.isArray(input?.anchors) ? input.anchors : [];

  return {
    schemaVersion: 1,
    templateName: input?.templateName ?? defaults.templateName,
    templateText: input?.templateText ?? defaults.templateText,
    anchors: anchors.map((anchor, index) => ({
      id: anchor?.id ?? `anchor-${index + 1}`,
      label: anchor?.label ?? `anchor_${index + 1}`,
      selectedText: anchor?.selectedText ?? "",
    })),
    panes: defaults.panes.map((defaultPane, index) => ({
      ...defaultPane,
      ...(sourcePanes[index] ?? {}),
      codexMode: sourcePanes[index]?.codexMode || defaultPane.codexMode,
      codexLaunchMode: sourcePanes[index]?.codexLaunchMode === "resume" ? "resume" : "new",
      codexResumeSessionId: "",
      anchorValues: {
        ...anchors.reduce<Record<string, string>>((acc, anchor) => {
          acc[anchor.label] = "";
          return acc;
        }, {}),
        ...(sourcePanes[index]?.anchorValues ?? {}),
      },
    })),
  };
}

export function createDefaultSettings(): AppSettings {
  return {
    windowsBackendPath: "",
  };
}
