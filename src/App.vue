<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import {
  DELIVERY_LABELS,
  MAX_PANES,
  SHARED_TEMPLATES,
  TOOL_TEMPLATES,
  repairConfig,
} from "./defaults";
import {
  detectWindowsBackendPath,
  launchWindowsBackend,
  loadConfig,
  loadSettings,
  loadWindowsBackendConfig,
  pickDirectory,
  previewWindowsBackend,
  readTemplateText,
  saveConfig,
  saveSettings,
  writeClipboardText,
} from "./backend";
import type { AppSettings, CodexMode, LauncherConfig, PaneConfig, PromptDelivery } from "./types";

const config = ref<LauncherConfig | null>(null);
const settings = reactive<AppSettings>(loadSettings());
const previewText = ref("");
const status = ref("Ready.");
const isBusy = ref(false);
const isAdvancedOpen = ref(false);
const editingIndex = ref<number | null>(null);
const promptDraft = ref("");
const modeDraft = ref<CodexMode>("yolo");
const deliveryDraft = ref<PromptDelivery>("manual");
const sharedTemplateDraft = ref(SHARED_TEMPLATES[0]);
const toolTemplateDraft = ref(TOOL_TEMPLATES[0]);

const enabledCount = computed(() => config.value?.panes.filter((pane) => pane.enabled).length ?? 0);
const gridSummary = computed(() => {
  const count = enabledCount.value;
  if (count <= 1) return "1 pane";
  const columns = Math.min(5, Math.ceil(Math.sqrt(count)));
  const rows = Math.ceil(count / columns);
  return `${columns} columns x ${rows} rows`;
});

const codexModes: CodexMode[] = ["", "yolo", "dangerous", "full-auto"];
const deliveryModes: PromptDelivery[] = ["manual", "file", "direct", "auto"];

onMounted(async () => {
  try {
    if (!settings.windowsBackendPath) {
      settings.windowsBackendPath = await detectWindowsBackendPath();
      saveSettings(settings);
    }
  } catch (error) {
    status.value = `Backend auto-detect skipped: ${formatError(error)}`;
  }

  config.value = await loadConfig();
  if (isBlankStarterConfig(config.value) && settings.windowsBackendPath) {
    try {
      config.value = await loadWindowsBackendConfig(settings);
      await saveConfig(config.value);
      status.value = "Imported existing Windows backend config.";
    } catch (error) {
      status.value = `Windows config auto-import skipped: ${formatError(error)}`;
    }
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

async function importWindowsConfig(): Promise<void> {
  try {
    isBusy.value = true;
    config.value = await loadWindowsBackendConfig(settings);
    await saveConfig(config.value);
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

    getPane(index).path = selected;
    status.value = `Pane ${index + 1} working directory selected.`;
  } catch (error) {
    status.value = `Directory selection failed: ${formatError(error)}`;
  } finally {
    isBusy.value = false;
  }
}

function openCodexDialog(index: number): void {
  const pane = getPane(index);
  editingIndex.value = index;
  promptDraft.value = pane.codexPrompt;
  modeDraft.value = pane.codexMode || "yolo";
  deliveryDraft.value = pane.codexPromptDelivery || "manual";
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

async function persistConfig(): Promise<boolean> {
  if (!config.value) return false;

  config.value = repairConfig(config.value);
  const validationErrors = validateEnabledWorkingDirectories(config.value);
  if (validationErrors.length > 0) {
    showValidationErrors(validationErrors);
    return false;
  }

  await saveConfig(config.value);
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
    if (!(await persistConfig())) return;

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
    if (!(await persistConfig())) return;

    const output = await launchWindowsBackend(config.value, settings);
    previewText.value = output || "Windows backend launch command completed.";
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
      <label>
        Window mode
        <select v-if="config" v-model="config.windowMode">
          <option value="maximized">maximized</option>
          <option value="fullscreen">fullscreen</option>
          <option value="normal">normal</option>
        </select>
      </label>
      <div class="settings-summary">
        Enabled panes must choose real project directories before launch.
      </div>
      <button class="soft-button advanced-toggle" @click="isAdvancedOpen = !isAdvancedOpen">
        {{ isAdvancedOpen ? "Hide advanced" : "Advanced" }}
      </button>
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
        <span class="prompt-badge">
          {{ DELIVERY_LABELS[pane.codexPromptDelivery] }}
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
                {{ DELIVERY_LABELS[mode] }}
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
  </main>
</template>
