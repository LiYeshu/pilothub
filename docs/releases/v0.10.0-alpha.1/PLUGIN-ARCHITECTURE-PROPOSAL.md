# PilotHub 0.10.0-alpha.1：Codex Plugin 与 AI 专家团架构提案

状态：Frozen for Alpha implementation  
对应版本：`0.10.0-alpha.1`  
对应工程阶段：PR #39  
后续实现：PR #40～#42

## 1. 决策摘要

PilotHub 下一阶段从“单个 Skill 管理器”升级为“可安装业务能力管理器”，但不自建 Agent Runtime。

冻结后的产品与技术关系：

```text
用户理解
AI 专家团 / 业务助手
        ↓
安装与交付
Codex Plugin
        ↓
能力组件
统筹 Skill + 多个专业 Skills
        ↓
运行环境
Codex
```

各层职责：

- **AI 专家团**：用户看到、理解和使用的业务产品。
- **Codex Plugin**：专家团的标准安装包和生命周期对象。
- **Skills**：专家知识、方法、任务拆解和执行流程。
- **Apps / MCP**：专家连接外部系统的工具，后续版本再接入。
- **Codex**：理解用户目标、选择 Skills 并执行任务的运行环境。
- **PilotHub**：获取、验证、预览、安装、更新、卸载和诊断 Plugin。

Alpha.1 只验证：

> 一个包含多个 Skills 的标准 Codex Plugin，能否被 PilotHub 一次安装、统一管理，并在 Codex 中完成一个完整业务任务。

## 2. 为什么采用 Plugin，而不是自定义 Skill Bundle

OpenAI 将 Plugin 定义为可安装的工作流能力包，可以包含多个 Skills，并可进一步包含 Apps、MCP servers 和界面资源。Plugin 因而能够覆盖 PilotHub 未来的扩展边界，而不需要 PilotHub 先创造另一套包格式。

参考：

- [OpenAI：Plugins in Codex](https://help.openai.com/en/articles/20001256-plugins-in-codex/)
- [OpenAI Developers](https://developers.openai.com/)
- [Codex Plugin structure](https://developers.openai.com/codex/plugins/build/#plugin-structure)

本阶段明确不创建：

```text
pilot.extension.yaml
skill-bundle.yaml
自定义 Plugin manifest
自定义 entry_skill 字段
```

PilotHub 直接适配 Codex Plugin，而不是在 Codex Plugin 外再包一层新的公开安装标准。

## 3. 官方 Plugin 包结构

最小 Skill-only Plugin：

```text
wechat-content-plugin/
├── .codex-plugin/
│   └── plugin.json
└── skills/
    ├── content-director/
    │   └── SKILL.md
    ├── article-writer/
    │   └── SKILL.md
    ├── cover-image/
    │   └── SKILL.md
    └── article-illustrator/
        └── SKILL.md
```

未来可扩展为：

```text
wechat-content-plugin/
├── .codex-plugin/
│   └── plugin.json
├── skills/
├── assets/
├── .mcp.json
└── .app.json
```

Alpha.1 的有效包必须满足：

1. 存在 `.codex-plugin/plugin.json`。
2. `plugin.json` 中的 `name`、版本、作者和展示信息有效。
3. `skills` 指向 Plugin 内真实存在的 Skills。
4. 每个 Skill 具有有效的 `SKILL.md`。
5. manifest、Skill 和资源路径不能逃逸 Plugin 根目录。
6. 预览阶段不执行 Plugin 内脚本。

示例 manifest：

```json
{
  "name": "wechat-content-plugin",
  "version": "0.1.0",
  "description": "Plan and produce complete WeChat content packages.",
  "author": {
    "name": "PilotHub"
  },
  "license": "MIT",
  "skills": "./skills/",
  "interface": {
    "displayName": "公众号运营专家团",
    "shortDescription": "完成选题、写作、封面和文章配图",
    "longDescription": "由内容统筹和多个专业 Skills 组成的公众号内容工作流。",
    "developerName": "PilotHub",
    "category": "Productivity",
    "capabilities": ["Writing", "Image generation"],
    "defaultPrompt": [
      "围绕以下主题完成一篇公众号文章，并生成封面和配图："
    ]
  }
}
```

`content-director` 是普通 Skill。它通过自身 `SKILL.md` 描述适用任务、步骤和专业能力协作方式，由 Codex 在适合的任务中选择。PilotHub 不向官方 manifest 添加 `entry_skill`、`orchestrator` 或 `experts` 等未定义字段。

## 4. PilotHub Extension、Plugin 与 Skill 的关系

冻结以下领域模型：

```text
Extension
├── CodexPlugin
│   ├── Skill
│   ├── Skill
│   └── Skill
├── MCP Extension       future
├── Agent Extension     future
├── Prompt Extension    future
└── Hook Extension      future
```

定义：

- **Extension** 是 PilotHub 的产品级总称。
- **CodexPlugin** 是 Alpha.1 新增的真实安装对象。
- **Skill** 是 Plugin 内的能力组件，也继续支持当前独立安装方式。
- 当前 Extensions 页面按来源聚合已安装 Skills，属于展示视图，不等同于 Plugin 安装记录。

兼容原则：

1. 已安装的独立 Skills 保持原状。
2. 当前 `skills` 表和 Skill 安装器不做迁移。
3. Plugin 安装不能被降级为“把多个 Skills 分别复制到 Codex Skills 目录”。
4. Codex Plugin Adapter 负责 Plugin 的完整生命周期。
5. 一个 Plugin 内的 Skills 在 PilotHub 中可查看，但 Plugin 是更新和卸载的主对象。

## 5. Alpha.1 应用层模型

PR #40 先增加应用层 DTO，不修改 SQLite schema：

```typescript
type CodexPluginDescriptor = {
  name: string
  version: string
  description: string
  author: string
  source: PluginSource
  manifestPath: string
  skills: PluginSkillDescriptor[]
  capabilities: string[]
  defaultPrompts: string[]
}

type PluginInstallationStatus = {
  pluginName: string
  marketplaceName: string
  target: 'codex'
  installed: boolean
  version: string
  health: 'healthy' | 'warning' | 'error'
}
```

数据来源：

- 包信息来自官方 `.codex-plugin/plugin.json`。
- Skill 信息来自 Plugin 内 `SKILL.md`。
- 安装状态由 Codex Plugin CLI 和 PilotHub 管理的本地 marketplace 共同确认。
- PilotHub 不复制一份 Plugin manifest 到 SQLite。

Alpha.1 不新增 `plugins`、`plugin_skills` 或 `extensions` 数据表。只有在 PR #40 验证“仅靠标准 manifest 与 Codex 安装状态无法可靠恢复生命周期”后，才允许为后续版本提出数据库迁移。

## 6. CodexPluginAdapter

PilotHub 新增独立适配器，不替换当前 Skill installer：

```text
ExtensionManager
├── NativeSkillManager
└── CodexPluginAdapter
```

建议接口：

```rust
trait PluginManager {
    fn inspect(&self, source: &PluginSource) -> Result<CodexPluginDescriptor>;
    fn validate(&self, plugin: &CodexPluginDescriptor) -> Result<ValidationReport>;
    fn install(&self, request: &PluginInstallRequest) -> Result<PluginInstallResult>;
    fn update(&self, request: &PluginUpdateRequest) -> Result<PluginInstallResult>;
    fn uninstall(&self, plugin_name: &str) -> Result<PluginUninstallResult>;
    fn list(&self) -> Result<Vec<PluginInstallationStatus>>;
    fn doctor(&self, plugin_name: &str) -> Result<PluginDoctorReport>;
}
```

职责边界：

| 能力 | NativeSkillManager | CodexPluginAdapter |
|---|---:|---:|
| 安装单个 Skill | 是 | 否 |
| 同步到多个 Agent Skills 目录 | 是 | 否 |
| 验证 `.codex-plugin/plugin.json` | 否 | 是 |
| 注册 Codex marketplace | 否 | 是 |
| 安装、更新、卸载完整 Plugin | 否 | 是 |
| 管理 Plugin 内多个 Skills | 否 | 是 |

## 7. PilotHub 本地 Marketplace

Alpha.1 只支持当前用户的本地 Codex 环境。PilotHub 使用独立 marketplace，避免直接接管用户已有的个人或团队 marketplace。

建议目录：

```text
~/.pilothub/
└── codex/
    ├── marketplace.json
    ├── plugins/
    │   └── wechat-content-plugin/
    ├── staging/
    └── backups/
```

marketplace 固定名称：

```text
pilothub-local
```

首次安装时：

```text
准备 Plugin
    ↓
验证 manifest 与 Skills
    ↓
写入 PilotHub 本地 marketplace
    ↓
向 Codex 注册 marketplace
    ↓
codex plugin add <plugin>@pilothub-local
    ↓
codex plugin list 验证
```

卸载时使用 Codex 提供的 Plugin 删除能力，并清理 PilotHub 管理的包版本。不得通过删除单个 Skills 目录模拟 Plugin 卸载。

当前本地 Codex CLI 已确认提供：

```text
codex plugin add
codex plugin list
codex plugin remove
codex plugin marketplace
```

PR #40 实现前必须继续核对每个子命令参数和幂等行为，不在架构文档中假定未验证的参数。

## 8. 安装事务与回滚

安装流程：

```text
Acquire
  ↓
Inspect
  ↓
Validate
  ↓
Preview
  ↓
Stage
  ↓
Register marketplace
  ↓
Install in Codex
  ↓
Verify
  ↓
Commit
```

约束：

1. 下载和解包先进入 `staging`。
2. 验证失败时不得改变 Codex 或现有 Plugin。
3. 更新前保留上一版本和 marketplace 快照。
4. Codex 安装失败时恢复 marketplace 和上一版本。
5. 只有 Codex 状态验证成功后，PilotHub 才显示“运行正常”。
6. 重试必须幂等，不能生成重复 marketplace 条目。
7. 同名不同来源或签名信息变化时必须要求用户确认。

## 9. 安装预览与安全边界

安装前至少展示：

```text
Plugin 名称
版本
作者
来源
许可证
包含的 Skills
文件与资源类型
目标运行环境
将创建或修改的目录
```

Alpha.1 安全检查：

- manifest schema 和必填字段；
- Skill 数量和 `SKILL.md` 有效性；
- 绝对路径、`..`、符号链接逃逸；
- 重复 Skill 名称；
- 超大文件和异常文件数量；
- 来源与安装版本；
- Codex CLI 是否可用；
- marketplace 目录是否可写。

Apps、MCP 和安装脚本属于更高权限边界。Alpha.1 检测到这些组件时只展示“当前版本不支持”，不得静默忽略后继续安装。

## 10. 个人、项目与工作区边界

Codex Plugin 的可用性可能受到产品形态、工作区角色和管理员策略影响。PilotHub 必须区分：

| 场景 | Alpha.1 | 说明 |
|---|---:|---|
| 当前用户、本地 Codex | 支持 | PR #40 的唯一安装目标 |
| 项目或团队 marketplace | 不支持 | 后续验证共享和更新策略 |
| 受管理 Codex 工作区 | 不支持 | 需要管理员策略与分发边界 |
| ChatGPT Plugin 安装 | 不支持 | 不假设与本地 Codex 完全相同 |

PilotHub 不应把“本地安装成功”表述为“已部署到整个组织”。

## 11. 专家团的运行模型

Alpha.1 是逻辑专家团：

```text
一个 Codex
    +
内容统筹 Skill
    +
多个专业 Skills
    +
Plugin 内资源
```

用户输入业务目标：

```text
根据 PilotHub 0.10.0-alpha.1 的发布内容，
写一篇公众号文章，并生成标题、封面和文章配图。
```

预期执行：

```text
Codex 理解目标
    ↓
选择内容统筹 Skill
    ↓
按工作流使用专业 Skills
    ├── 选题与标题
    ├── 文章写作
    ├── 封面生成
    └── 文章配图
    ↓
汇总交付
```

PilotHub 不负责推理、任务调度、上下文传递或 Handoff。

## 12. Alpha.1 明确不做

```text
MCP 安装与授权
Apps 安装与授权
真实多 Agent Runtime
Agent Handoff
Workflow Engine
Plugin Marketplace 产品
云端账号与同步
AI 自动组合 Plugin
通用 Extension manifest
SQLite Extension 重构
```

若 Plugin 包含上述未支持组件，PilotHub 应阻止安装并说明原因。

## 13. 后续 PR 计划

### PR #40：Skill-only Plugin MVP

实现：

- 扫描标准 Codex Plugin；
- 解析和验证 `.codex-plugin/plugin.json`；
- 预览多个 Skills；
- 建立 `pilothub-local` marketplace；
- 一次安装到本地 Codex；
- 统一列出、诊断和卸载；
- 不修改 SQLite schema。

验收：

```text
导入一个预组装的 Skill-only Plugin
    ↓
PilotHub 显示 Plugin 与全部 Skills
    ↓
一次安装
    ↓
Codex 能发现 Plugin 内 Skills
    ↓
PilotHub 能正确诊断和完整卸载
```

### PR #41：专家团产品体验

实现：

- “专家团 / 业务助手”展示；
- 统筹能力与专业能力说明；
- 默认业务指令；
- 安装成功后的使用入口；
- 技术详情展示 Plugin、Skills、路径和版本。

不改变 Plugin 包格式。

### PR #42：真实业务验收

使用 `baoyu-skills` 相关能力组装第一个验证 Plugin：

```text
公众号运营专家团
├── 内容统筹
├── 文章写作
├── baoyu-cover-image
└── article-illustrator
```

验收链路：

```text
PilotHub 获取 Plugin
    ↓
预览与验证
    ↓
一次安装到 Codex
    ↓
用户输入完整公众号任务
    ↓
多个 Skills 协同
    ↓
成功产出文章、封面和配图
```

## 14. Alpha.1 完成标准

以下条件全部满足，才能认为 `0.10.0-alpha.1` 成立：

1. PilotHub 管理的对象是完整 Codex Plugin，而不是若干 Skill 复制任务。
2. Plugin 使用官方 `.codex-plugin/plugin.json`，没有 PilotHub 自定义公开 manifest。
3. 一个 Plugin 至少包含一个统筹 Skill 和两个专业 Skills。
4. 用户一次操作即可完成安装。
5. Codex 能在新任务中发现并使用 Plugin 内 Skills。
6. PilotHub 能显示 Plugin、Skills、版本、来源和健康状态。
7. 安装失败可回滚，不留下假成功状态。
8. Plugin 可以被完整卸载。
9. 一个真实业务任务由多个 Skills 协同完成。
10. PilotHub 没有新增多 Agent Runtime、MCP 或 Marketplace 产品边界。

## 15. 最终冻结定义

```text
专家团
= 用户理解的产品

Codex Plugin
= 标准安装包和生命周期对象

Skills
= 专家知识、方法和工作流

Apps / MCP
= 后续接入的外部工具

Codex
= 理解、选择和执行能力的运行环境

PilotHub
= Plugin 的获取、验证、安装、更新、卸载和治理中心
```

PR #40 只能在上述边界内实现 Skill-only Plugin MVP。任何新增公开 manifest、数据库 schema、MCP、Apps、真实多 Agent 调度或 Marketplace 产品能力，都必须另行提出架构变更。
