# Query Public Repo Import Format

`query 标注专用` 的 `一键导入公开仓库` 支持从 Markdown 文件导入最多 `20` 组 repo 数据，并填充到前 20 个 query pane。

导入会重建 query pane 列表：

- 每组数据填充一个 pane。
- `path` 写入 Working directory。
- `title` 自动取路径最后一级目录名。
- `enabled` 设为 `true`。
- `codexMode` 使用 query 默认值 `yolo`。
- `codexLaunchMode` 使用默认值 `new`。
- 其他 pane 配置恢复 query 默认值。
- 超过 20 组会截断并提示。

## Format

每组数据用独立一行的 `---REPO---` 开始。

每组必须包含一行 `path:`，锚点值使用 `[anchor:name] ... [/anchor]` 包裹。`name` 需要对应当前 query 模板里的 `{{name}}`。

```md
---REPO---
path: /Users/Zhuanz/developer/work/project-a

[anchor:task]
修复 src/App.vue 里 query 面板导入后 title 没有自动更新的问题，保持现有保存逻辑不变。
[/anchor]

[anchor:module]
frontend, query-workspace
[/anchor]

---REPO---
path: /Users/Zhuanz/developer/work/project-b

[anchor:task]
检查 backend/api/user.ts 的 createUser 接口，补充重复邮箱的错误处理和 vitest 覆盖。
[/anchor]

[anchor:module]
backend, user
[/anchor]
```

## Notes

- `path:` 需要是本地目录路径，不是 GitHub URL。
- 没有 `path:` 的组会跳过。
- 没有 anchor 的组也可以导入，只会填目录和标题。
- 文件里的 anchor 名如果当前模板没有使用，导入后不会影响最终渲染。
- 当前模板有但导入文件没写的 anchor，会保持为空值。
