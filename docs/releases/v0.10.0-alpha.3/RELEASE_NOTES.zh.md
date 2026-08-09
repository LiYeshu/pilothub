# PilotHub v0.10.0-alpha.3

本版本完成 Codex 原生 Plugin 调用链收口：PilotHub 负责安装、更新、修复、卸载和诊断 Plugin，Codex 直接发现并调用 Plugin 内的命名空间 Skills。

## 主要变化

### 原生 Plugin 成为默认入口

- 新安装不再生成额外的技能目录 Launcher；
- 已安装并启用的 Plugin 直接报告原生注册、发现和调用能力；
- Alpha.2 遗留 Launcher 会在读取 Plugin 列表时安全迁移；
- 只清理带 PilotHub 所有权标记的 Launcher，不触碰用户自行创建的同名 Skill。

### 生命周期诊断与修复

- Doctor 保持只读，分别展示 Plugin 注册、Skill 发现和调用状态；
- Repair 可重新校验文件、重建 marketplace 条目并恢复 Codex 注册；
- 更新失败时恢复上一版本 Plugin、marketplace 内容和原生入口状态；
- 卸载后验证文件、marketplace、旧 Launcher 和 Codex 注册均无残留。

### 真实环境验收

`wechat-content-expert-team` 已在真实 macOS 环境完成：

1. Alpha.2 旧 Launcher 自动迁移；
2. 原生 Plugin 保持已安装、已启用和健康状态；
3. 可恢复卸载后无托管残留；
4. 重新安装后恢复原生注册；
5. 全新只读 Codex 会话成功发现并调用 `wechat-content-expert-team:content-director`。

调用探测读取了统筹 Skill 及其项目状态协议，并正确返回内容策划、文章写作、封面生成和文章配图四阶段。测试没有创建内容文件、调用图片生成或执行发布。

## 质量检查

- 前端 lint 通过；
- 49 项前端测试通过；
- 前端生产构建通过；
- Rust 格式与 Clippy 通过；
- 179 项 Rust 测试通过；
- 2 项需要真实外部环境的 E2E 测试按设计忽略。

## 当前边界

本版本仍是 Skill-only Codex Plugin Alpha：

- 不增加 MCP 管理；
- 不实现多 Agent Runtime；
- 不实现 Workflow Engine；
- 不增加 Marketplace 或云端账号；
- PilotHub 不负责运行专家团队，实际理解、调度和执行仍由 Codex 完成。

## macOS 说明

没有 Apple Developer ID 证书的 macOS 安装包继续使用 ad-hoc 签名并在发布前执行严格验签，但尚未经过 Apple notarization。部分系统首次启动时仍可能需要按项目文档处理 Gatekeeper 提示。
