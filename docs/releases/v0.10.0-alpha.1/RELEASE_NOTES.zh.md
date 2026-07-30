# PilotHub 0.10.0-alpha.1

PilotHub `0.10.0-alpha.1` 从“管理独立 Skill”向“管理完整业务能力”迈出第一步：

> 一个包含多个 Skills 的标准 Codex Plugin，可以被 PilotHub 一次安装、统一诊断和完整卸载，并在 Codex 中作为 AI 专家团队完成业务任务。

## 本版亮点

### 安装标准 Codex Plugin

PilotHub 现在支持从 GitHub 或本地目录读取标准 Skill-only Codex Plugin。

安装前会检查：

- `.codex-plugin/plugin.json` 是否存在且有效；
- Plugin 名称和版本是否合法；
- manifest 声明的 Skills 是否真实存在；
- 每个 Skill 是否包含有效的 `SKILL.md`；
- Plugin 内路径是否越过根目录；
- 是否包含 Alpha.1 暂不支持的 MCP、Apps 或 Hooks。

预览阶段不会执行 Plugin 内脚本。

### 原子化安装和失败回滚

PilotHub 在独立 staging 目录准备 Plugin，完成校验后再写入管理目录并注册到 Codex。

如果 Codex marketplace 注册或 Plugin 安装失败，PilotHub 会恢复原有文件和 marketplace 状态，避免留下半安装结果。

PilotHub 使用独立的：

```text
pilothub-local
```

作为 Codex marketplace，不接管用户已有的个人或团队 marketplace。

### Plugin 是真实生命周期对象

Plugin 不会被降级成“把多个 Skills 分别复制到 Codex Skills 目录”。

PilotHub 可以：

- 预览完整 Plugin；
- 一次安装全部组件；
- 查看安装和启用状态；
- 执行健康诊断；
- 完整卸载 Plugin；
- 在失败后重试安装。

既有独立 Skill 安装、Agent 目录同步和 SQLite 数据保持兼容，不需要迁移数据库。

### 多 Skill Plugin 显示为 AI 专家团队

当一个 Plugin 包含多个 Skills 时，Extensions 页面会把它展示为一个面向业务的 AI 专家团队：

```text
公众号运营专家团
├── 内容统筹
├── 文章写作
├── 封面生成
└── 文章配图
```

详情页会显示：

- 统筹角色；
- 专业能力；
- Plugin 来源和版本；
- 安装、启用与诊断状态；
- manifest 提供的示例任务。

只包含一个 Skill 的 Plugin 会显示为独立能力，不会被标记为专家团队。

## 真实业务验收

本版本创建并验证了公共 Plugin：

```text
https://github.com/LiYeshu/wechat-content-expert-team
```

验收链路：

```text
公共 GitHub Plugin
    ↓
PilotHub 预览和校验 4 个 Skills
    ↓
安装到 pilothub-local
    ↓
PilotHub 诊断为健康
    ↓
新的 Codex 任务发现全部 Plugin Skills
    ↓
用户输入完整公众号内容目标
    ↓
产出文章、16:9 封面和两张正文配图
```

Plugin 固定使用 `JimLiu/baoyu-skills` 的明确上游 commit，并保留 MIT 许可证和第三方来源说明。

详细证据见：

- [`PLUGIN-ARCHITECTURE-PROPOSAL.md`](./PLUGIN-ARCHITECTURE-PROPOSAL.md)
- [`CODEX-PLUGIN-ACCEPTANCE.md`](./CODEX-PLUGIN-ACCEPTANCE.md)

## 产品与运行时边界

本版“AI 专家团队”的含义是：

```text
一个 Codex
+
一个统筹 Skill
+
多个专业 Skills
```

PilotHub 负责 Plugin 的获取、理解、安装和治理；Codex 负责理解用户目标、选择 Skills 并执行任务。

本版本不包含：

- MCP 管理；
- Apps 管理；
- 多 Agent runtime；
- Agent handoff；
- Workflow Engine；
- Extension Marketplace；
- 云端账号或同步。

## 验收状态

- 标准 Plugin 校验通过；
- PilotHub Plugin Adapter 生命周期测试通过；
- 安装失败回滚和路径安全测试通过；
- Web 与 Rust CI 通过；
- 公共 Plugin 真实安装和 Codex 发现通过；
- 公众号文章、封面和配图业务任务通过；
- 未修改 SQLite schema；
- 未新增 PilotHub 私有 Plugin manifest。

## Alpha 提示

这是 Codex Plugin 能力的第一个 Alpha 版本。目前只支持 Skill-only Plugin，并要求本机存在可用的 Codex CLI。建议先使用结构明确、来源可信的 Plugin 进行体验。
