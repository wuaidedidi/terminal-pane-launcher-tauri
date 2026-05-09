export type WindowMode = "maximized" | "fullscreen" | "normal";
export type CodexMode = "" | "yolo" | "dangerous" | "full-auto";
export type PromptDelivery = "manual" | "file" | "direct" | "auto";
export type LaunchMode = "new" | "resume";
export type PromptStyle = "composed" | "template";
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
  codexLaunchMode: LaunchMode;
  codexResumeSessionId: string;
  codexPromptStyle: PromptStyle;
}

export interface LauncherConfig {
  schemaVersion: 1;
  windowMode: WindowMode;
  layoutMode: "balancedGrid";
  defaultProfile: string;
  panes: PaneConfig[];
}

export interface QueryAnchor {
  id: string;
  label: string;
  selectedText: string;
}

export interface QueryPaneConfig {
  enabled: boolean;
  title: string;
  path: string;
  profile: string;
  startupCommand: string;
  codexMode: CodexMode;
  codexLaunchMode: LaunchMode;
  codexResumeSessionId: string;
  anchorValues: Record<string, string>;
}

export interface QueryWorkspaceState {
  schemaVersion: 1;
  templateName: string;
  templateText: string;
  anchors: QueryAnchor[];
  panes: QueryPaneConfig[];
}

export interface AppSettings {
  windowsBackendPath: string;
}
