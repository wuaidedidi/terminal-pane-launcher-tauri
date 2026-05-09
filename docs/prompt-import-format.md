# Prompt Import Format

`一键导入提示词` 用来从一个 Markdown 文件批量导入 pane 的用户 Prompt。

导入文件只负责 Prompt 内容，不负责 pane 配置。导入时只会写入每个 pane 的 `codexPrompt`，不会修改目录、标题、启用状态、Codex 模式、模板、传递方式或启动命令。

## Format

每个 Prompt 块必须用独立一行的 `---PROMPT---` 开始：

```md
---PROMPT---
第 1 个 prompt 内容

---PROMPT---
第 2 个 prompt 内容
```

解析规则：

- 只识别独立一行的 `---PROMPT---`。
- 从一个 `---PROMPT---` 到下一个 `---PROMPT---` 之间的内容，属于同一个 Prompt。
- Prompt 内部可以使用普通 Markdown，包括标题、列表、空行、代码块和分隔线。
- 最多导入 20 个非空 Prompt 块。
- 超过 20 个时，只导入前 20 个，并在状态栏提示跳过数量。
- 空 Prompt 块会被跳过，并在预览区提示跳过数量。

## Workflow

1. 可选：先点击 `清空启用提示词`，清掉当前启用 pane 的旧 Prompt。
2. 点击 `一键导入提示词`。
3. 选择 `.md` 文件。
4. 导入后的第 1 个 Prompt 写入第 1 行，第 2 个 Prompt 写入第 2 行，依次类推。
5. 手动选择目录。选择目录后，标题会自动填为目录名，仍然可以继续手动修改。

示例文件见 `templates/提示词导入示例.md`。
