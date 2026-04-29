export type WindowMode = "maximized" | "fullscreen" | "normal";
export type CodexMode = "" | "yolo" | "dangerous" | "full-auto";
export type PromptDelivery = "manual" | "file" | "direct" | "auto";
export type AppPlatform = "windows" | "macos" | "linux" | "unknown";

export interface PaneConfig {
  enabled: boolean;
  title: string;
  path: string;
  profile: string;
  startupCommand: string;
  codexMode: CodexMode;
  codexPrompt: string;
  codexTemplate: string;
  codexToolTemplate: string;
  codexPromptDelivery: PromptDelivery;
}

export interface LauncherConfig {
  schemaVersion: 1;
  windowMode: WindowMode;
  layoutMode: "balancedGrid";
  defaultProfile: string;
  panes: PaneConfig[];
}

export interface AppSettings {
  windowsBackendPath: string;
}
