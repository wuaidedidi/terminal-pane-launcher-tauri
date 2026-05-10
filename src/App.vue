<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import {
  MAX_PANES,
  SHARED_TEMPLATES,
  TOOL_TEMPLATES,
  createDefaultQueryWorkspace,
  createDefaultPane,
  createDefaultQueryPane,
  getDeliveryLabel,
  repairConfig,
  repairQueryWorkspace,
} from "./defaults";
import {
  detectWindowsBackendPath,
  getCurrentPlatform,
  launchWindowsBackend,
  loadQueryWorkspace,
  loadConfig,
  loadSettings,
  loadWindowsBackendConfig,
  pickDirectory,
  previewWindowsBackend,
  readTemplateText,
  saveConfig,
  saveQueryWorkspace,
  saveSettings,
  writeClipboardText,
} from "./backend";
import type {
  AppPlatform,
  AppSettings,
  CodexMode,
  LauncherConfig,
  PaneConfig,
  PromptDelivery,
  QueryAnchor,
  QueryPaneConfig,
  QueryWorkspaceState,
} from "./types";

const config = ref<LauncherConfig | null>(null);
const queryWorkspace = ref<QueryWorkspaceState>(createDefaultQueryWorkspace());
const currentPlatform = ref<AppPlatform>("unknown");
const settings = reactive<AppSettings>(loadSettings());
const previewText = ref("");
const status = ref("Ready.");
const isBusy = ref(false);
const isAdvancedOpen = ref(false);
const activeWorkspace = ref<"vibecoding" | "query">("vibecoding");
const editingIndex = ref<number | null>(null);
const promptDraft = ref("");
const modeDraft = ref<CodexMode>("yolo");
const deliveryDraft = ref<PromptDelivery>("manual");
const sharedTemplateDraft = ref(SHARED_TEMPLATES[0]);
const toolTemplateDraft = ref(TOOL_TEMPLATES[0]);
const promptImportInput = ref<HTMLInputElement | null>(null);
const queryTemplateImportInput = ref<HTMLInputElement | null>(null);
const queryEditingIndex = ref<number | null>(null);
const queryAnchorNameDraft = ref("");
const querySelectedTextDraft = ref("");
const queryTemplateEditor = ref<HTMLTextAreaElement | null>(null);
const queryTemplateSelection = ref({ start: 0, end: 0 });
const queryAnchorValuesDraft = ref<Record<string, string>>({});
const isQueryTemplateDialogOpen = ref(false);
const queryTemplateNameDraft = ref("");
const queryTemplateTextDraft = ref("");

const enabledCount = computed(() => config.value?.panes.filter((pane) => pane.enabled).length ?? 0);
const isMacOs = computed(() => currentPlatform.value === "macos");
const gridSummary = computed(() => {
  const count = enabledCount.value;
  if (count <= 1) return "1 pane";
  const columns = Math.min(5, Math.ceil(Math.sqrt(count)));
  const rows = Math.ceil(count / columns);
  return `${columns} columns x ${rows} rows`;
});
const deliveryModes = computed<PromptDelivery[]>(() =>
  isMacOs.value ? ["manual", "direct"] : ["manual", "file", "direct", "auto"],
);
const codexModes: CodexMode[] = ["", "yolo", "dangerous", "full-auto"];
const queryCodexModes: Exclude<CodexMode, "">[] = ["yolo", "dangerous", "full-auto"];
const PROMPT_IMPORT_SEPARATOR = "---PROMPT---";

interface PromptImportParseResult {
  prompts: string[];
  emptyBlockCount: number;
  skippedPromptCount: number;
}

const queryEnabledCount = computed(
  () => queryWorkspace.value?.panes.filter((pane) => pane.enabled).length ?? 0,
);
const queryGridSummary = computed(() => {
  const count = queryEnabledCount.value;
  if (count <= 1) return "1 pane";
  const columns = Math.min(5, Math.ceil(Math.sqrt(count)));
  const rows = Math.ceil(count / columns);
  return `${columns} columns x ${rows} rows`;
});

onMounted(async () => {
  try {
    currentPlatform.value = await getCurrentPlatform();
  } catch (error) {
    currentPlatform.value = "unknown";
    status.value = `Platform detection skipped: ${formatError(error)}`;
  }

  try {
    if (currentPlatform.value === "windows" && !settings.windowsBackendPath) {
      settings.windowsBackendPath = await detectWindowsBackendPath();
      saveSettings(settings);
    }
  } catch (error) {
    status.value = `Backend auto-detect skipped: ${formatError(error)}`;
  }

  try {
    config.value = await loadConfig(currentPlatform.value);
  } catch (error) {
    config.value = repairConfig(null, currentPlatform.value);
    status.value = `Config load failed, using defaults: ${formatError(error)}`;
  }
  if (currentPlatform.value === "macos") {
    config.value.windowMode = "maximized";
  }
  if (shouldResetMacWindowsPreset(config.value)) {
    config.value = resetMacWindowsPreset(config.value);
    await saveConfig(config.value, currentPlatform.value);
    status.value =
      "macOS detected Windows preset directories, so all panes were unchecked for a fresh start.";
  }

  if (
    currentPlatform.value === "windows" &&
    isBlankStarterConfig(config.value) &&
    settings.windowsBackendPath
  ) {
    try {
      config.value = await loadWindowsBackendConfig(settings, currentPlatform.value);
      await saveConfig(config.value, currentPlatform.value);
      status.value = "Imported existing Windows backend config.";
    } catch (error) {
      status.value = `Windows config auto-import skipped: ${formatError(error)}`;
    }
  }

  try {
    queryWorkspace.value = await loadQueryWorkspace(currentPlatform.value);
  } catch (error) {
    queryWorkspace.value = repairQueryWorkspace(null, currentPlatform.value);
    status.value = `Query workspace load failed, using defaults: ${formatError(error)}`;
  }
});

function formatError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function getPane(index: number): PaneConfig {
  if (!config.value) {
    throw new Error("Config is not loaded.");
  }

  return config.value.panes[index];
}

function isBlankStarterConfig(value: LauncherConfig): boolean {
  const enabledPanes = value.panes.filter((pane) => pane.enabled);
  if (enabledPanes.length === 0) {
    return true;
  }

  return enabledPanes.every((pane, index) => {
    const title = pane.title.trim();
    const path = pane.path.trim();
    return (
      title === `Pane ${index + 1}` &&
      (path === "%USERPROFILE%" || path === "") &&
      !pane.codexMode &&
      !pane.codexPrompt &&
      !pane.startupCommand
    );
  });
}

function isMissingWorkingDirectory(path: string): boolean {
  const value = path.trim().replace(/[\\/]+$/, "").toLowerCase();
  return value === "" || value === "%userprofile%" || value === "$env:userprofile" || value === "~";
}

function isWindowsStyleWorkingDirectory(path: string): boolean {
  const value = path.trim();
  const lower = value.toLowerCase();
  return (
    /^[a-z]:[\\/]/i.test(value) ||
    lower === "%userprofile%" ||
    lower === "$env:userprofile"
  );
}

function shouldResetMacWindowsPreset(value: LauncherConfig): boolean {
  if (currentPlatform.value !== "macos") {
    return false;
  }

  const enabledPanes = value.panes.filter((pane) => pane.enabled);
  return (
    enabledPanes.length > 0 &&
    enabledPanes.every((pane) => isWindowsStyleWorkingDirectory(pane.path))
  );
}

function resetMacWindowsPreset(value: LauncherConfig): LauncherConfig {
  const repaired = repairConfig(value, "macos");
  return {
    ...repaired,
    defaultProfile: repaired.defaultProfile === "Windows PowerShell" ? "" : repaired.defaultProfile,
    panes: repaired.panes.map((pane) => ({
      ...pane,
      enabled: false,
      path: isWindowsStyleWorkingDirectory(pane.path) ? "" : pane.path,
    })),
  };
}

function validateEnabledWorkingDirectories(value: LauncherConfig): string[] {
  return value.panes.flatMap((pane, index) => {
    if (!pane.enabled || !isMissingWorkingDirectory(pane.path)) {
      return [];
    }

    return [
      `Pane ${index + 1} is enabled, but its working directory is empty or still uses %USERPROFILE%.`,
    ];
  });
}

function showValidationErrors(errors: string[]): void {
  previewText.value = [
    "Enabled panes must choose a real project directory before save, preview, or launch.",
    ...errors,
  ].join("\n");
  status.value = "Please choose a working directory for every enabled pane.";
}

function hasSelectableWorkingDirectory(pane: PaneConfig | QueryPaneConfig): boolean {
  return !isMissingWorkingDirectory(pane.path);
}

function isValueQueryPane(pane: QueryPaneConfig, index: number): boolean {
  return (
    pane.title.trim() !== `Pane ${index + 1}` ||
    pane.path.trim() !== "" ||
    pane.profile.trim() !== "" ||
    pane.startupCommand.trim() !== "" ||
    pane.codexMode !== "yolo" ||
    pane.codexLaunchMode !== "new" ||
    Object.values(pane.anchorValues).some((value) => value.trim() !== "")
  );
}

async function selectValuePanes(): Promise<void> {
  if (!config.value) return;

  try {
    isBusy.value = true;
    let selectedCount = 0;
    config.value.panes.forEach((pane) => {
      pane.enabled = hasSelectableWorkingDirectory(pane);
      if (pane.enabled) {
        selectedCount += 1;
      }
    });
    await saveConfig(config.value, currentPlatform.value);
    status.value = `Selected ${selectedCount} pane(s) with working directories.`;
    previewText.value = `Selected ${selectedCount} pane(s) that have real project directories.`;
  } catch (error) {
    status.value = `Select value panes failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function selectValueQueryPanes(): Promise<void> {
  if (!queryWorkspace.value) return;

  try {
    isBusy.value = true;
    let selectedCount = 0;
    queryWorkspace.value.panes.forEach((pane) => {
      pane.enabled = hasSelectableWorkingDirectory(pane);
      if (pane.enabled) {
        selectedCount += 1;
      }
    });
    await saveQueryWorkspace(queryWorkspace.value, currentPlatform.value);
    status.value = `Selected ${selectedCount} query pane(s) with working directories.`;
    previewText.value = `Selected ${selectedCount} query pane(s) that have real project directories.`;
  } catch (error) {
    status.value = `Select query value panes failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

function parsePromptImportMarkdown(raw: string): PromptImportParseResult {
  const normalized = raw.replace(/^\uFEFF/, "").replace(/\r\n?/g, "\n");
  const sections = normalized.split(/^[ \t]*---PROMPT---[ \t]*$/gm);
  const promptSections = sections.slice(1);
  const nonEmptyPrompts = promptSections
    .map((section) => section.trim())
    .filter(Boolean);

  return {
    prompts: nonEmptyPrompts.slice(0, MAX_PANES),
    emptyBlockCount: promptSections.length - nonEmptyPrompts.length,
    skippedPromptCount: Math.max(0, nonEmptyPrompts.length - MAX_PANES),
  };
}

function openPromptImportPicker(): void {
  promptImportInput.value?.click();
}

async function handlePromptImportFile(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";

  if (!file || !config.value) {
    return;
  }
  const currentConfig = config.value;

  if (!file.name.toLowerCase().endsWith(".md")) {
    status.value = "Prompt import needs a .md file.";
    return;
  }

  try {
    isBusy.value = true;
    const parsed = parsePromptImportMarkdown(await file.text());

    if (parsed.prompts.length === 0) {
      status.value = `No prompt blocks found. Use ${PROMPT_IMPORT_SEPARATOR} as the separator.`;
      previewText.value = [
        `Prompt import skipped: ${file.name}`,
        `Use one ${PROMPT_IMPORT_SEPARATOR} line before each prompt block.`,
      ].join("\n");
      return;
    }

    parsed.prompts.forEach((prompt, index) => {
      if (currentConfig.panes[index]) {
        currentConfig.panes[index].codexPrompt = prompt;
      }
    });

    await saveConfig(currentConfig, currentPlatform.value);

    previewText.value = [
      `Prompt import: ${file.name}`,
      `Imported: ${parsed.prompts.length}`,
      parsed.skippedPromptCount > 0 ? `Skipped extra prompts: ${parsed.skippedPromptCount}` : "",
      parsed.emptyBlockCount > 0 ? `Skipped empty prompt blocks: ${parsed.emptyBlockCount}` : "",
      "Only imported prompt text was updated. Other pane settings and non-imported prompts were kept.",
    ]
      .filter(Boolean)
      .join("\n");
    status.value =
      parsed.skippedPromptCount > 0
        ? `Imported ${parsed.prompts.length} prompt(s), skipped ${parsed.skippedPromptCount} extra prompt(s).`
        : `Imported ${parsed.prompts.length} prompt(s) from ${file.name}.`;
  } catch (error) {
    status.value = `Prompt import failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function clearEnabledPrompts(): Promise<void> {
  if (!config.value) return;

  const currentConfig = config.value;
  const enabledPanes = currentConfig.panes.filter((pane) => pane.enabled);
  if (enabledPanes.length === 0) {
    status.value = "No enabled panes to reset.";
    return;
  }

  try {
    isBusy.value = true;
    currentConfig.panes.forEach((pane, index) => {
      if (pane.enabled) {
        currentConfig.panes[index] = createDefaultPane(index, false, currentPlatform.value);
      }
    });
    await saveConfig(currentConfig, currentPlatform.value);
    previewText.value = `Reset ${enabledPanes.length} enabled pane(s) to default blank state.`;
    status.value = `Reset ${enabledPanes.length} enabled pane(s) to default state.`;
  } catch (error) {
    status.value = `Reset failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function clearEnabledQueryPanes(): Promise<void> {
  if (!queryWorkspace.value) return;

  const workspace = queryWorkspace.value;
  const valuePaneIndexes = workspace.panes.flatMap((pane, index) =>
    isValueQueryPane(pane, index) || pane.enabled ? [index] : [],
  );
  if (valuePaneIndexes.length === 0) {
    status.value = "No query panes to reset.";
    return;
  }

  try {
    isBusy.value = true;
    const anchorValues = workspace.anchors.reduce<Record<string, string>>((acc, anchor) => {
      acc[anchor.label] = "";
      return acc;
    }, {});
    valuePaneIndexes.forEach((index) => {
      workspace.panes[index] = {
        ...createDefaultQueryPane(index, false, currentPlatform.value),
        anchorValues: { ...anchorValues },
      };
    });
    queryWorkspace.value = repairQueryWorkspace(workspace, currentPlatform.value);
    syncQueryAnchorValues();
    await saveQueryWorkspace(queryWorkspace.value, currentPlatform.value);
    previewText.value = `Reset ${valuePaneIndexes.length} query pane(s) to default state.`;
    status.value = `Reset ${valuePaneIndexes.length} query pane(s).`;
  } catch (error) {
    status.value = `Query reset failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function setAllQueryLaunchModes(mode: "new" | "resume"): Promise<void> {
  if (!queryWorkspace.value) return;

  try {
    isBusy.value = true;
    queryWorkspace.value.panes.forEach((pane) => {
      pane.codexLaunchMode = mode;
    });
    await saveQueryWorkspace(queryWorkspace.value, currentPlatform.value);
    status.value = `Set all query panes to ${mode}.`;
    previewText.value = `All query panes now launch in ${mode} mode.`;
  } catch (error) {
    status.value = `Query launch mode update failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

function getQueryPane(index: number): QueryPaneConfig {
  if (!queryWorkspace.value) {
    throw new Error("Query workspace is not loaded.");
  }

  return queryWorkspace.value.panes[index];
}

function syncQueryAnchorValues(): void {
  if (!queryWorkspace.value) return;

  const labels = queryWorkspace.value.anchors.map((anchor) => anchor.label);
  queryWorkspace.value.panes.forEach((pane) => {
    labels.forEach((label) => {
      if (!(label in pane.anchorValues)) {
        pane.anchorValues[label] = "";
      }
    });
  });
}

function extractQueryAnchorsFromTemplate(templateText: string): QueryAnchor[] {
  const matches = [...templateText.matchAll(/\{\{\s*([a-zA-Z0-9_-]+)\s*\}\}/g)];
  const seen = new Set<string>();
  return matches.flatMap((match) => {
    const label = match[1];
    if (seen.has(label)) {
      return [];
    }
    seen.add(label);
    return [
      {
        id: label,
        label,
        selectedText: "",
      },
    ];
  });
}

function slugifyQueryAnchorLabel(value: string): string {
  const slug = value
    .trim()
    .toLowerCase()
    .replace(/[\s/\\]+/g, "_")
    .replace(/[^a-z0-9_-]/g, "_")
    .replace(/_+/g, "_")
    .replace(/^_+|_+$/g, "");

  return slug || "anchor";
}

function applyQueryTemplateText(name: string, text: string): void {
  if (!queryWorkspace.value) return;

  const previousAnchors = new Map(
    queryWorkspace.value.anchors.map((anchor) => [anchor.label, anchor] as const),
  );
  queryWorkspace.value.templateName = name.trim() || "自定义模板";
  queryWorkspace.value.templateText = text.replace(/^\uFEFF/, "");
  queryWorkspace.value.anchors = extractQueryAnchorsFromTemplate(queryWorkspace.value.templateText).map(
    (anchor) => ({
      ...anchor,
      selectedText: previousAnchors.get(anchor.label)?.selectedText ?? anchor.selectedText,
    }),
  );
  syncQueryAnchorValues();
}

function openQueryTemplateDialog(): void {
  if (!queryWorkspace.value) return;

  queryTemplateNameDraft.value = queryWorkspace.value.templateName || "自定义模板";
  queryTemplateTextDraft.value = queryWorkspace.value.templateText;
  isQueryTemplateDialogOpen.value = true;
}

function closeQueryTemplateDialog(): void {
  isQueryTemplateDialogOpen.value = false;
}

async function applyQueryTemplateDialog(): Promise<void> {
  try {
    isBusy.value = true;
    applyQueryTemplateText(queryTemplateNameDraft.value, queryTemplateTextDraft.value);
    await saveQueryWorkspace(queryWorkspace.value, currentPlatform.value);
    previewText.value = `Loaded custom query template: ${queryWorkspace.value.templateName}`;
    status.value = `Custom query template saved.`;
    closeQueryTemplateDialog();
  } catch (error) {
    status.value = `Custom template save failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function importQueryTemplateFile(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";

  if (!file || !queryWorkspace.value) {
    return;
  }

  try {
    isBusy.value = true;
    const text = await file.text();
    applyQueryTemplateText(file.name, text);
    await saveQueryWorkspace(queryWorkspace.value, currentPlatform.value);
    previewText.value = `Loaded query template: ${file.name}`;
    status.value = `Loaded ${file.name} into query workspace.`;
  } catch (error) {
    status.value = `Template import failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

function updateQueryTemplateSelection(): void {
  const editor = queryTemplateEditor.value;
  if (!editor) return;

  queryTemplateSelection.value = {
    start: editor.selectionStart ?? 0,
    end: editor.selectionEnd ?? 0,
  };
  querySelectedTextDraft.value = editor.value.slice(
    queryTemplateSelection.value.start,
    queryTemplateSelection.value.end,
  );
}

function openQueryAnchorDialog(index: number): void {
  if (!queryWorkspace.value) return;

  queryEditingIndex.value = index;
  const pane = queryWorkspace.value.panes[index];
  queryAnchorValuesDraft.value = { ...pane.anchorValues };
}

function closeQueryAnchorDialog(): void {
  queryEditingIndex.value = null;
  queryAnchorValuesDraft.value = {};
}

function applyQueryAnchorDialog(): void {
  if (queryEditingIndex.value === null || !queryWorkspace.value) return;

  const pane = getQueryPane(queryEditingIndex.value);
  pane.anchorValues = {
    ...pane.anchorValues,
    ...queryAnchorValuesDraft.value,
  };
  status.value = `Query pane ${queryEditingIndex.value + 1} anchor values updated.`;
  previewText.value = `Updated anchor values for query pane ${queryEditingIndex.value + 1}.`;
  closeQueryAnchorDialog();
}

function applyQueryAnchorFromSelection(): void {
  if (!queryWorkspace.value) return;
  const editor = queryTemplateEditor.value;
  if (!editor) return;

  const { start, end } = queryTemplateSelection.value;
  const selectedText = querySelectedTextDraft.value.trim();
  if (!selectedText || start === end) {
    status.value = "Please select text in the template before creating an anchor.";
    return;
  }

  const baseLabel = slugifyQueryAnchorLabel(queryAnchorNameDraft.value || selectedText);
  let label = baseLabel;
  let suffix = 2;
  while (queryWorkspace.value.anchors.some((anchor) => anchor.label === label)) {
    label = `${baseLabel}_${suffix}`;
    suffix += 1;
  }

  const templateText = queryWorkspace.value.templateText;
  queryWorkspace.value.templateText =
    templateText.slice(0, start) + `{{${label}}}` + templateText.slice(end);
  queryWorkspace.value.anchors.push({
    id: label,
    label,
    selectedText,
  });
  queryWorkspace.value.panes.forEach((pane) => {
    pane.anchorValues[label] = "";
  });
  queryTemplateSelection.value = { start, end: start + label.length + 4 };
  previewText.value = `Anchor ${label} added.`;
  status.value = `Anchor ${label} created.`;
  closeQueryAnchorDialog();
}

function renderQueryPrompt(pane: QueryPaneConfig): string {
  if (!queryWorkspace.value) return "";

  return queryWorkspace.value.anchors.reduce((result, anchor) => {
    const value = pane.anchorValues[anchor.label]?.trim() || anchor.selectedText || "";
    return result.replace(new RegExp(`\\{\\{\\s*${anchor.label}\\s*\\}\\}`, "g"), value);
  }, queryWorkspace.value.templateText);
}

function buildQueryLaunchConfig(): LauncherConfig | null {
  if (!queryWorkspace.value) return null;

  const repaired = repairConfig(null, currentPlatform.value);
  return {
    ...repaired,
    panes: repaired.panes.map((defaultPane, index) => {
      const sourcePane = queryWorkspace.value?.panes[index];
      const renderedPrompt = sourcePane ? renderQueryPrompt(sourcePane) : "";
      return {
        ...defaultPane,
        ...(sourcePane ?? {}),
        enabled: sourcePane?.enabled ?? defaultPane.enabled,
        title: sourcePane?.title ?? defaultPane.title,
        path: sourcePane?.path ?? defaultPane.path,
        profile: sourcePane?.profile ?? defaultPane.profile,
        startupCommand: sourcePane?.startupCommand ?? defaultPane.startupCommand,
        codexMode: sourcePane?.codexMode || "yolo",
        codexPrompt: renderedPrompt,
        codexTemplate: "",
        codexToolTemplate: "",
        codexPromptDelivery: "direct",
        codexLaunchMode: sourcePane?.codexLaunchMode ?? "new",
        codexResumeSessionId: "",
        codexPromptStyle: "template",
      };
    }),
  };
}

async function persistQueryWorkspace(): Promise<boolean> {
  if (!queryWorkspace.value) return false;

  const knownAnchors = new Map(
    queryWorkspace.value.anchors.map((anchor) => [anchor.label, anchor] as const),
  );
  queryWorkspace.value.anchors = extractQueryAnchorsFromTemplate(queryWorkspace.value.templateText).map(
    (anchor) => ({
      ...anchor,
      selectedText: knownAnchors.get(anchor.label)?.selectedText ?? anchor.selectedText,
    }),
  );
  queryWorkspace.value = repairQueryWorkspace(queryWorkspace.value, currentPlatform.value);
  syncQueryAnchorValues();
  await saveQueryWorkspace(queryWorkspace.value, currentPlatform.value);
  status.value = `Saved query workspace.`;
  return true;
}

async function handleQuerySave(): Promise<void> {
  try {
    isBusy.value = true;
    await persistQueryWorkspace();
  } catch (error) {
    status.value = `Query save failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function handleQueryPreview(): Promise<void> {
  try {
    isBusy.value = true;
    if (!(await persistQueryWorkspace())) return;

    const launchConfig = buildQueryLaunchConfig();
    if (!launchConfig) return;

    previewText.value = await previewWindowsBackend(launchConfig, settings);
    status.value = `Previewed ${queryEnabledCount.value} query pane(s).`;
  } catch (error) {
    status.value = `Query preview failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function handleQueryLaunch(): Promise<void> {
  try {
    isBusy.value = true;
    if (!(await persistQueryWorkspace())) return;

    const launchConfig = buildQueryLaunchConfig();
    if (!launchConfig) return;

    const output = await launchWindowsBackend(launchConfig, settings);
    previewText.value = output || "Launch command completed.";
    status.value = `Launched ${queryEnabledCount.value} query pane(s).`;
  } catch (error) {
    status.value = `Query launch failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function chooseQueryWorkingDirectory(index: number): Promise<void> {
  try {
    isBusy.value = true;
    const selected = await pickDirectory();
    if (!selected) {
      status.value = `Query pane ${index + 1} directory selection cancelled.`;
      return;
    }

    const pane = getQueryPane(index);
    const title = getDirectoryTitle(selected);
    pane.path = selected;
    if (title) {
      pane.title = title;
    }
    status.value = title
      ? `Query pane ${index + 1} directory selected and title set to ${title}.`
      : `Query pane ${index + 1} working directory selected.`;
  } catch (error) {
    status.value = `Query directory selection failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function importWindowsConfig(): Promise<void> {
  try {
    isBusy.value = true;
    config.value = await loadWindowsBackendConfig(settings, currentPlatform.value);
    await saveConfig(config.value, currentPlatform.value);
    status.value = "Imported existing Windows backend config.";
  } catch (error) {
    status.value = `Import failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function chooseWorkingDirectory(index: number): Promise<void> {
  try {
    isBusy.value = true;
    const selected = await pickDirectory();
    if (!selected) {
      status.value = `Pane ${index + 1} directory selection cancelled.`;
      return;
    }

    const pane = getPane(index);
    const title = getDirectoryTitle(selected);
    pane.path = selected;
    if (title) {
      pane.title = title;
    }
    status.value = title
      ? `Pane ${index + 1} directory selected and title set to ${title}.`
      : `Pane ${index + 1} working directory selected.`;
  } catch (error) {
    status.value = `Directory selection failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

function getDirectoryTitle(path: string): string {
  return path.trim().replace(/[\\/]+$/, "").split(/[\\/]/).pop()?.trim() ?? "";
}

function openCodexDialog(index: number): void {
  const pane = getPane(index);
  editingIndex.value = index;
  promptDraft.value = pane.codexPrompt;
  modeDraft.value = pane.codexMode || "yolo";
  deliveryDraft.value = pane.codexPromptDelivery || "manual";
  if (isMacOs.value && deliveryDraft.value !== "direct") {
    deliveryDraft.value = "direct";
  }
  sharedTemplateDraft.value = pane.codexTemplate || SHARED_TEMPLATES[0];
  toolTemplateDraft.value = pane.codexToolTemplate || TOOL_TEMPLATES[0];
}

function closeCodexDialog(): void {
  editingIndex.value = null;
  promptDraft.value = "";
}

function applyCodexDialog(): void {
  if (editingIndex.value === null || !config.value) return;

  const pane = getPane(editingIndex.value);
  pane.codexMode = modeDraft.value;
  pane.codexPromptDelivery = deliveryDraft.value;
  pane.codexTemplate = sharedTemplateDraft.value;
  pane.codexToolTemplate = toolTemplateDraft.value;
  pane.codexPrompt = promptDraft.value;
  status.value = `Pane ${editingIndex.value + 1} Codex settings updated.`;
  closeCodexDialog();
}

async function copyMergedPrompt(index: number): Promise<void> {
  if (!config.value) return;

  try {
    isBusy.value = true;
    const pane = getPane(index);
    const shared = await readTemplateText(settings, pane.codexTemplate || SHARED_TEMPLATES[0]);
    const tool = await readTemplateText(settings, pane.codexToolTemplate || TOOL_TEMPLATES[0]);
    const merged = [
      pane.codexPrompt.trim() ? `## User Prompt\n${pane.codexPrompt.trim()}` : "",
      `## Shared Prompt Template: ${pane.codexTemplate}\n${shared.trim()}`,
      `## Tool Prompt Template: ${pane.codexToolTemplate}\n${tool.trim()}`,
    ]
      .filter(Boolean)
      .join("\n\n");

    await writeClipboardText(merged);
    status.value = `Pane ${index + 1} merged prompt copied. Length: ${merged.length}`;
  } catch (error) {
    status.value = `Copy failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function persistConfig(validateBeforeRun = false): Promise<boolean> {
  if (!config.value) return false;

  config.value = repairConfig(config.value, currentPlatform.value);
  if (validateBeforeRun) {
    const validationErrors = validateEnabledWorkingDirectories(config.value);
    if (validationErrors.length > 0) {
      showValidationErrors(validationErrors);
      return false;
    }
  }

  await saveConfig(config.value, currentPlatform.value);
  saveSettings(settings);
  status.value = `Saved ${MAX_PANES} pane config.`;
  return true;
}

async function handleSave(): Promise<void> {
  try {
    isBusy.value = true;
    await persistConfig();
  } catch (error) {
    status.value = `Save failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function handlePreview(): Promise<void> {
  if (!config.value) return;

  try {
    isBusy.value = true;
    if (!(await persistConfig(true))) return;

    previewText.value = await previewWindowsBackend(config.value, settings);
    status.value = `Previewed ${enabledCount.value} pane(s).`;
  } catch (error) {
    status.value = `Preview failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

async function handleLaunch(): Promise<void> {
  if (!config.value) return;

  try {
    isBusy.value = true;
    if (!(await persistConfig(true))) return;

    const output = await launchWindowsBackend(config.value, settings);
    previewText.value = output || "Launch command completed.";
    status.value = `Launched ${enabledCount.value} pane(s).`;
  } catch (error) {
    status.value = `Launch failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}
</script>

<template>
  <main class="shell">
    <section class="workspace-tabs">
      <button
        class="workspace-tab"
        :class="{ active: activeWorkspace === 'vibecoding' }"
        @click="activeWorkspace = 'vibecoding'"
      >
        vibecoding 项目专用
      </button>
      <button
        class="workspace-tab"
        :class="{ active: activeWorkspace === 'query' }"
        @click="activeWorkspace = 'query'"
      >
        query 标注专用
      </button>
    </section>

    <div v-show="activeWorkspace === 'vibecoding'">
    <section class="hero">
      <div>
        <p class="eyebrow">Tauri cross-platform shell</p>
        <h1>Codex Terminal Pane Launcher</h1>
        <p class="subtitle">
          保留当前 Windows PowerShell 后端，先用 Tauri + Vue 重新做跨平台 GUI。
        </p>
      </div>
      <div class="stats-card">
        <span>{{ enabledCount }}</span>
        <small>enabled panes</small>
        <strong>{{ gridSummary }}</strong>
      </div>
    </section>

    <section class="settings-bar">
      <label v-if="!isMacOs">
        Window mode
        <select v-if="config" v-model="config.windowMode">
          <option value="maximized">maximized</option>
          <option value="fullscreen">fullscreen</option>
          <option value="normal">normal</option>
        </select>
      </label>
      <label v-else>
        macOS launch window
        <div class="readonly-field">Optimized large iTerm2 window</div>
      </label>
      <div class="settings-summary">
        Enabled panes must choose real project directories before launch.
      </div>
      <div class="settings-actions">
        <button class="soft-button" :disabled="isBusy" @click="selectValuePanes">
          全选有目录 pane
        </button>
        <button class="soft-button" :disabled="isBusy" @click="clearEnabledPrompts">
          清空启用提示词
        </button>
        <button class="import-button" :disabled="isBusy" @click="openPromptImportPicker">
          一键导入提示词
        </button>
        <button class="soft-button advanced-toggle" @click="isAdvancedOpen = !isAdvancedOpen">
          {{ isAdvancedOpen ? "Hide advanced" : "Advanced" }}
        </button>
      </div>
      <input
        ref="promptImportInput"
        class="file-input"
        type="file"
        accept=".md,text/markdown"
        @change="handlePromptImportFile"
      />
    </section>

    <section class="advanced-panel" v-if="isAdvancedOpen">
      <label class="backend-field">
        Windows backend path
        <input
          v-model="settings.windowsBackendPath"
          placeholder="Auto-detected from sibling Windows project"
        />
      </label>
      <div class="advanced-actions">
        <button class="import-button" :disabled="isBusy" @click="importWindowsConfig">
          Import legacy Windows config
        </button>
        <p>
          Use this only when migrating old Windows PowerShell launcher settings. It can
          overwrite the current pane rows.
        </p>
      </div>
    </section>

    <section class="pane-board" v-if="config">
      <div class="pane-header">
        <span>#</span>
        <span>Enabled</span>
        <span>Title</span>
        <span>Working directory</span>
        <span>Browse</span>
        <span>Codex</span>
        <span>Prompt</span>
        <span>Copy</span>
      </div>

      <div
        class="pane-row"
        :class="{ 'is-invalid': pane.enabled && isMissingWorkingDirectory(pane.path) }"
        v-for="(pane, index) in config.panes"
        :key="index"
      >
        <span class="pane-index">{{ index + 1 }}</span>
        <input type="checkbox" v-model="pane.enabled" />
        <input v-model="pane.title" />
        <input
          v-model="pane.path"
          :class="{ invalid: pane.enabled && isMissingWorkingDirectory(pane.path) }"
          placeholder="Choose project folder first"
        />
        <button class="soft-button browse-button" :disabled="isBusy" @click="chooseWorkingDirectory(index)">
          Browse
        </button>
        <button class="soft-button" @click="openCodexDialog(index)">
          {{ pane.codexPrompt || pane.codexMode ? "Codex*" : "Codex" }}
        </button>
        <select v-if="isMacOs" class="prompt-select" v-model="pane.codexPromptDelivery">
          <option v-for="mode in deliveryModes" :key="mode" :value="mode">
            {{ getDeliveryLabel(mode, currentPlatform) }}
          </option>
        </select>
        <span v-else class="prompt-badge">
          {{ getDeliveryLabel(pane.codexPromptDelivery, currentPlatform) }}
        </span>
        <button class="copy-button" :disabled="isBusy" @click="copyMergedPrompt(index)">
          一键复制
        </button>
      </div>
    </section>

    <section class="preview-panel">
      <textarea v-model="previewText" readonly placeholder="Preview output will appear here." />
    </section>

    <footer class="action-bar">
      <span>{{ status }}</span>
      <div>
        <button :disabled="isBusy" @click="handleSave">Save</button>
        <button :disabled="isBusy" @click="handlePreview">Preview</button>
        <button class="launch-button" :disabled="isBusy" @click="handleLaunch">
          Save & Launch
        </button>
      </div>
    </footer>

    <div class="modal-backdrop" v-if="editingIndex !== null">
      <section class="modal">
        <header>
          <div>
            <p class="eyebrow">Pane {{ editingIndex + 1 }}</p>
            <h2>Codex startup prompt</h2>
          </div>
          <button class="icon-button" @click="closeCodexDialog">x</button>
        </header>

        <div class="modal-grid">
          <label>
            Mode
            <select v-model="modeDraft">
              <option v-for="mode in codexModes" :key="mode" :value="mode">
                {{ mode || "off" }}
              </option>
            </select>
          </label>
          <label>
            Delivery
            <select v-model="deliveryDraft">
              <option v-for="mode in deliveryModes" :key="mode" :value="mode">
                {{ getDeliveryLabel(mode, currentPlatform) }}
              </option>
            </select>
          </label>
          <label>
            Shared template
            <select v-model="sharedTemplateDraft">
              <option v-for="template in SHARED_TEMPLATES" :key="template" :value="template">
                {{ template }}
              </option>
            </select>
          </label>
          <label>
            Tool template
            <select v-model="toolTemplateDraft">
              <option v-for="template in TOOL_TEMPLATES" :key="template" :value="template">
                {{ template }}
              </option>
            </select>
          </label>
        </div>

        <textarea
          class="prompt-editor"
          v-model="promptDraft"
          placeholder="把本次项目需求写在这里；一键复制会自动拼接后面的两个模板。"
        />

        <footer>
          <button @click="closeCodexDialog">Cancel</button>
          <button class="launch-button" @click="applyCodexDialog">OK</button>
        </footer>
      </section>
    </div>
    </div>

    <div v-show="activeWorkspace === 'query'">
      <section class="hero query-hero">
        <div>
          <p class="eyebrow">Template driven workspace</p>
          <h1>Query Template Launcher</h1>
          <p class="subtitle">
            上传一份 Markdown 模板，标注锚点后为最多 20 个 pane 生成不同的启动版本。
          </p>
        </div>
        <div class="stats-card">
          <span>{{ queryEnabledCount }}</span>
          <small>enabled panes</small>
          <strong>{{ queryGridSummary }}</strong>
        </div>
      </section>

      <section class="settings-bar query-toolbar">
        <label>
          Template file
          <div class="toolbar-row">
            <button class="import-button" :disabled="isBusy" @click="queryTemplateImportInput?.click()">
              上传模板
            </button>
            <button class="soft-button" :disabled="isBusy" @click="openQueryTemplateDialog">
              自定义模板
            </button>
            <input v-model="queryWorkspace.templateName" placeholder="Template name" />
          </div>
        </label>
        <label>
          Anchor name
          <div class="toolbar-row">
            <input v-model="queryAnchorNameDraft" placeholder="选中文字后输入锚点名" />
            <button class="soft-button" :disabled="isBusy" @click="applyQueryAnchorFromSelection">
              设为锚点
            </button>
          </div>
        </label>
        <div class="settings-summary">
          <button class="soft-button" :disabled="isBusy" @click="selectValueQueryPanes">
            全选有目录 pane
          </button>
          <button class="soft-button" :disabled="isBusy" @click="clearEnabledQueryPanes">
            一键清空
          </button>
          <button class="soft-button" :disabled="isBusy" @click="setAllQueryLaunchModes('resume')">
            全部 resume
          </button>
          <button class="soft-button" :disabled="isBusy" @click="setAllQueryLaunchModes('new')">
            全部 new
          </button>
        </div>
      </section>

      <input
        ref="queryTemplateImportInput"
        class="file-input"
        type="file"
        accept=".md,text/markdown"
        @change="importQueryTemplateFile"
      />

      <section class="query-workbench" v-if="queryWorkspace">
        <div class="query-template-panel">
          <label>
            Template markdown
            <textarea
              ref="queryTemplateEditor"
              v-model="queryWorkspace.templateText"
              class="query-template-editor"
              placeholder="Upload a markdown template or edit it here."
              @mouseup="updateQueryTemplateSelection"
              @keyup="updateQueryTemplateSelection"
              @select="updateQueryTemplateSelection"
            />
          </label>
        </div>

        <aside class="query-anchor-panel">
          <h2>Anchors</h2>
          <div v-if="queryWorkspace.anchors.length === 0" class="query-empty">
            还没有锚点，先在左侧模板中选中文字再添加。
          </div>
          <div v-else class="query-anchor-list">
            <div class="query-anchor-item" v-for="anchor in queryWorkspace.anchors" :key="anchor.id">
              <strong>{{ anchor.label }}</strong>
              <small>{{ anchor.selectedText || "selected text" }}</small>
            </div>
          </div>
        </aside>
      </section>

      <section class="pane-board query-pane-board" v-if="queryWorkspace">
        <div class="pane-header query-pane-header">
          <span>#</span>
          <span>Enabled</span>
          <span>Title</span>
          <span>Working directory</span>
          <span>Browse</span>
          <span>Codex</span>
          <span>Mode</span>
          <span>Anchors</span>
        </div>

        <div
          class="pane-row query-pane-row"
          v-for="(pane, index) in queryWorkspace.panes"
          :key="index"
        >
          <span class="pane-index">{{ index + 1 }}</span>
          <input type="checkbox" v-model="pane.enabled" />
          <input v-model="pane.title" />
          <input v-model="pane.path" placeholder="Choose project folder first" />
          <button
            class="soft-button browse-button"
            :disabled="isBusy"
            @click="chooseQueryWorkingDirectory(index)"
          >
            Browse
          </button>
          <select v-model="pane.codexMode">
            <option v-for="mode in queryCodexModes" :key="mode" :value="mode">
              {{ mode }}
            </option>
          </select>
          <select v-model="pane.codexLaunchMode">
            <option value="new">new</option>
            <option value="resume">resume</option>
          </select>
          <button class="soft-button" @click="openQueryAnchorDialog(index)">Edit</button>
        </div>
      </section>

      <footer class="action-bar">
        <span>{{ status }}</span>
        <div>
          <button :disabled="isBusy" @click="handleQuerySave">Save</button>
          <button :disabled="isBusy" @click="handleQueryPreview">Preview</button>
          <button class="launch-button" :disabled="isBusy" @click="handleQueryLaunch">
            一键启动
          </button>
        </div>
      </footer>

      <div class="modal-backdrop" v-if="queryEditingIndex !== null">
        <section class="modal">
          <header>
            <div>
              <p class="eyebrow">Pane {{ queryEditingIndex + 1 }}</p>
              <h2>Anchor values</h2>
            </div>
            <button class="icon-button" @click="closeQueryAnchorDialog">x</button>
          </header>

          <div class="modal-grid" v-if="queryWorkspace">
            <label v-for="anchor in queryWorkspace.anchors" :key="anchor.id">
              {{ anchor.label }}
              <input
                v-model="queryAnchorValuesDraft[anchor.label]"
                :placeholder="anchor.selectedText || anchor.label"
              />
            </label>
          </div>

          <footer>
            <button @click="closeQueryAnchorDialog">Cancel</button>
            <button class="launch-button" @click="applyQueryAnchorDialog">OK</button>
          </footer>
        </section>
      </div>

      <div class="modal-backdrop" v-if="isQueryTemplateDialogOpen">
        <section class="modal query-template-modal">
          <header>
            <div>
              <p class="eyebrow">Query template</p>
              <h2>自定义模板</h2>
            </div>
            <button class="icon-button" @click="closeQueryTemplateDialog">x</button>
          </header>

          <label>
            Template name
            <input v-model="queryTemplateNameDraft" placeholder="自定义模板" />
          </label>
          <label>
            Markdown
            <textarea
              v-model="queryTemplateTextDraft"
              class="query-template-draft"
              placeholder="在这里粘贴 Markdown 模板，使用 {{anchor}} 标注可替换位置。"
            />
          </label>

          <footer>
            <button @click="closeQueryTemplateDialog">Cancel</button>
            <button class="launch-button" :disabled="isBusy" @click="applyQueryTemplateDialog">
              Save
            </button>
          </footer>
        </section>
      </div>
    </div>
  </main>
</template>
