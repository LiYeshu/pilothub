# Codex Plugin 真实业务验收

## 结论

2026-07-30，PilotHub `0.10.0-alpha.1` 完成首个 Skill-only Codex Plugin 的端到端验收：

```text
公共 GitHub Plugin
  ↓
PilotHub 预览、校验与安装
  ↓
Codex 新任务发现 Plugin Skills
  ↓
统筹 Skill 协调写作、封面和配图 Skills
  ↓
文章、封面和两张文章配图交付成功
```

本次验收证明多个 Skills 可以作为一个业务 Plugin 被一次安装、统一管理，并由 Codex 完成完整内容任务。PilotHub 未增加 MCP、多 Agent runtime、工作流引擎、自定义 manifest 或数据库结构。

## 验收基线

| 项目 | 值 |
| --- | --- |
| PilotHub commit | `7323fa1` |
| Codex CLI | `0.146.0-alpha.3.1` |
| Plugin 仓库 | `https://github.com/LiYeshu/wechat-content-expert-team` |
| Plugin commit | `15f1517` |
| Plugin version | `0.1.0` |
| baoyu-skills 上游 commit | `6b7a2e417500561a5ecdd0b168332f4142584617` |
| 上游许可证 | MIT，Copyright (c) 2026 Jim Liu |

## Plugin 结构

Plugin 使用 Codex 标准 `.codex-plugin/plugin.json`，没有 PilotHub 私有 manifest。

```text
wechat-content-expert-team
├── .codex-plugin
│   └── plugin.json
├── skills
│   ├── content-director
│   │   ├── SKILL.md
│   │   └── agents/openai.yaml
│   ├── article-writer
│   │   ├── SKILL.md
│   │   └── agents/openai.yaml
│   ├── baoyu-cover-image
│   │   └── SKILL.md + scripts/references
│   └── baoyu-article-illustrator
│       └── SKILL.md + scripts/references
├── LICENSE
├── README.md
└── THIRD_PARTY_NOTICES.md
```

角色分工：

- `content-director`：理解业务目标、拆解任务、协调专业 Skills 并检查交付。
- `article-writer`：完成面向微信公众号的中文文章。
- `baoyu-cover-image`：生成文章封面。
- `baoyu-article-illustrator`：规划并生成正文配图。

两个 baoyu Skills 按固定上游 commit 原样引入，来源和许可证记录在 `THIRD_PARTY_NOTICES.md`。上游 Skill frontmatter 中的 `version` 会触发旧版独立 Skill 校验器警告，但 Codex Plugin 校验和 PilotHub 预览均通过，因此本次不修改上游内容。

## Plugin 静态验证

执行官方 Plugin 校验器：

```bash
python3 "$CODEX_HOME/skills/.system/plugin-creator/scripts/validate_plugin.py" .
```

结果：

```text
Plugin validation passed
```

另外，两个本项目编写的 Skills 均通过 `skill-creator` 的 `quick_validate.py`。

## PilotHub 预览、安装与诊断

PilotHub 使用 PR #40 已实现的 `CodexPluginAdapter` 直接检查公共仓库：

```text
https://github.com/LiYeshu/wechat-content-expert-team
```

预览结果：

| 检查项 | 结果 |
| --- | --- |
| manifest | 有效 |
| Skills 数量 | 4 |
| errors | 0 |
| warnings | 0 |
| 安装目标 | Codex |

安装完成后，PilotHub 诊断结果：

```json
{
  "pluginId": "wechat-content-expert-team@pilothub-local",
  "installed": true,
  "enabled": true,
  "version": "0.1.0",
  "health": "healthy",
  "marketplace": "pilothub-local"
}
```

Codex 自身的 `plugin list --json` 同时确认该 Plugin 已启用，来源为 PilotHub 管理目录。

## 新 Codex 任务发现验证

使用全新、临时 Codex 任务，只允许读取活动 Plugin 和 Skill 清单，不允许通过文件搜索猜测结果。

Codex 正确发现：

```text
wechat-content-expert-team:content-director
wechat-content-expert-team:article-writer
wechat-content-expert-team:baoyu-cover-image
wechat-content-expert-team:baoyu-article-illustrator
```

这证明安装结果能被新的 Codex 运行上下文识别，而不只是在 PilotHub 数据中显示为已安装。

## 完整业务任务

随后启动另一个全新的 Codex 任务，输入：

```text
使用 $wechat-content-expert-team:content-director 和该 Plugin 中需要的其他 Skills，
围绕 PilotHub 0.10.0-alpha.1 写一篇 800–1000 中文字符的微信公众号文章。

要求：
1. 面向 AI 开发者与内容创作者；
2. 生成 3 个标题候选并选择 1 个；
3. 说明 PilotHub 可以从 GitHub 安装标准 Skill-only Codex Plugin，
   校验 manifest 与 Skills，失败自动回滚，支持诊断和完整卸载；
4. 说明多 Skill Plugin 在产品中显示为“AI 专家团队”；
5. 明确本版本不包含 MCP 和多 Agent runtime；
6. 生成一张 16:9 中文封面；
7. 生成两张对正文有解释作用的配图；
8. 写入 article.md 和 delivery.md，不执行外部发布。
```

Codex 在执行记录中明确加载了安装缓存内的四个 Plugin Skills，并完成统筹、写作、封面生成和配图生成。

## 业务产物验证

| 产物 | 验证结果 |
| --- | --- |
| `article.md` | 存在且非空 |
| 标题候选 | 3 个 |
| 最终标题 | `PilotHub 0.10.0-alpha.1：把一个插件，变成一支 AI 专家队` |
| 中文字符数 | 832，包含标题候选、摘要和正文 |
| `cover.png` | 1672×941，约 16:9 |
| 安装闭环配图 | 1672×941，文字清晰 |
| 专家团队结构配图 | 1672×941，文字清晰 |
| `delivery.md` | 存在，列出交付物、假设和人工复核项 |
| 外部发布 | 未执行 |

三张 PNG 均通过文件类型、尺寸、非空检查和人工视觉检查。文章中的事实只使用验收任务提供的信息，没有增加 MCP、多 Agent runtime、性能数据或发布日期等未经验证的主张。

本次生成产物保留在验收机临时目录中，不提交二进制图片到 PilotHub 仓库；Plugin 中的提示词和 Skills 足以重新执行同一验收。

## 验收标准

- [x] 公共 GitHub 仓库包含标准 Codex Plugin。
- [x] Plugin 包含一个统筹 Skill 和三个专业 Skills。
- [x] 第三方 Skills 固定上游 commit 并保留许可证说明。
- [x] PilotHub 能预览并校验全部四个 Skills。
- [x] PilotHub 能把完整 Plugin 安装到 Codex。
- [x] PilotHub 诊断结果为已安装、已启用且健康。
- [x] 新 Codex 任务能发现 Plugin 和四个 Skills。
- [x] 用户只输入业务目标即可产出文章、封面和配图。
- [x] PilotHub 没有引入 Plugin 私有格式或 Agent runtime。

## 当前边界

本次没有重复执行破坏性的真实卸载，以便保留 Plugin 供后续人工体验。PR #40 的自动化生命周期测试继续覆盖完整卸载、失败回滚和幂等安装；本次 PR #42 聚焦真实公共仓库、真实安装和真实业务调用。

验收完成后，Plugin 保持安装并启用，用户可在 Codex 中直接调用：

```text
$wechat-content-expert-team:content-director
```
