# Workspace Tabs and Query Template Design

## Goal

Add a second launch workspace without affecting the existing prompt-concatenation flow.

The app should become a multi-workspace shell:

- `vibecoding 项目专用` keeps the current 20-pane prompt composition workflow.
- `query 标注专用` provides a template-driven workflow with anchor-based replacements.

The two workspaces must be isolated in state and behavior. Existing saved configs, launch paths, and prompt composition must keep working unchanged.

## Core Recommendation

Use top-level tabs inside the same Tauri window.

This is better than separate windows because:

- switching is fast
- all workspaces share the same shell and app chrome
- existing launch logic can be reused
- more workspaces can be added later without changing the layout model

The app should feel like one tool with multiple workspaces, not like separate applications.

## Workspace 1: Vibecoding

This workspace is the current product behavior, unchanged in principle.

It keeps:

- up to 20 panes
- prompt concatenation
- prompt import from `---PROMPT---` markdown blocks
- one-click prompt clearing
- automatic title fill from selected folder name
- Codex launch and preview flow

This workspace remains the safe fallback for the current daily routine.

## Workspace 2: Query Annotation

This workspace is for mostly-shared templates with small per-pane variations.

Recommended interaction:

1. Upload a Markdown template file.
2. Open it in an editor that supports text selection.
3. Select a span of text and mark it as an anchor.
4. Give the anchor a name.
5. The template stores that anchor as a replaceable slot.
6. Each pane supplies only the values that differ from the template.
7. The app generates up to 20 Codex launch payloads from the same base template.

### Why this interaction

It is more natural than asking the user to hand-author placeholder syntax everywhere.

It is also lighter than building a full rich-document editor.

The user works with ordinary Markdown, then marks only the parts that should vary.

## Anchor Model

Use anchors as first-class template metadata, not as a separate full editor system.

Suggested behavior:

- selecting text + `Set Anchor` creates a named anchor
- anchors are visible in a side list
- anchors can be reused across panes
- each pane only overrides values that differ
- the rest of the template stays shared

Implementation detail recommendation:

- store anchors as stable tokens in the document model
- keep a sidecar structure for anchor metadata and per-pane overrides
- render the edited template and final launch prompt from the same source of truth

This keeps the user experience simple while still giving the app a stable internal representation.

## Resume Behavior

For the query workspace, support Codex resume launches.

Default behavior:

- if the pane is in resume mode, use `codex resume --last`
- run it in the pane's selected working directory
- keep the workflow simple by default

Optional advanced behavior:

- remember the last known session id per pane
- let the user override it manually if needed

The default should stay `--last`, because that matches the user's normal workflow and avoids unnecessary choices.

## Launch Modes

Each pane should support two launch modes:

- `new`
- `resume`

Suggested defaults:

- vibecoding workspace: `new`
- query workspace: `new`, with `resume` available as an option

For `new`, reuse the current Codex launch flow.

For `resume`, reuse the current working directory selection, then run Codex in resume mode.

## State and Persistence

Use separate persisted workspace state objects.

Recommended shape:

- `app settings` remains shared
- `workspaces.vibecoding` stores the current prompt-composition config
- `workspaces.query` stores the template editor state, anchors, and pane overrides

This prevents one workspace from overwriting the other.

Each workspace should remember:

- pane list
- enabled state
- working directory
- title
- Codex mode
- launch mode
- prompt or template data

Shared app settings should remain shared:

- backend path
- platform detection
- resource paths

## UI Structure

The shell should contain:

- a top tab bar for workspaces
- the current workspace content below it
- shared app-level status at the bottom

For the query workspace, the layout should be:

- template upload and editor area
- anchor list / override panel
- pane generation preview
- existing save / preview / launch actions

For the vibecoding workspace, keep the current dense pane grid.

## Non-Goals

Do not turn the query workspace into a full document editor suite.

Do not remove the existing vibecoding workflow.

Do not force `resume` to require session id input.

Do not mix the two workspace configs together.

## Risks

- Anchor editing can become fragile if we store only raw text ranges.
- Separate workspace persistence needs careful migration from the current single-config file.
- Resume behavior must be tied to the pane working directory, otherwise `--last` can point to the wrong session.

These risks are manageable if the implementation keeps the workspaces separate and treats the pane path as part of the resume context.

## Proposed Rollout

1. Add the tab shell and workspace registry.
2. Move the existing pane grid into the `vibecoding` workspace.
3. Add the `query` workspace scaffold with template upload and anchor model.
4. Add resume mode with `codex resume --last`.
5. Add optional saved session id support later if needed.
6. Keep the two workspaces independently savable and launchable.
