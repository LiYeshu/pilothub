# PilotHub 0.10.0-alpha.2

PilotHub `0.10.0-alpha.2` 聚焦 Codex Plugin 的可发现性和 macOS 发布包稳定性：

> 安装完成的 Plugin 现在可以在 Codex Skills 目录中以一个明确的入口能力出现，同时 macOS 发布构建会在上传前强制完成 bundle 签名校验。

## 本版亮点

### Codex Skills 目录入口

每个由 PilotHub 管理的 Codex Plugin 都会生成一个独立的启动 Skill。用户可以在 Codex 的 Skills 目录中直接看到“微信公众号内容智能体”等业务入口，而不需要记住内部专业 Skills 的命名空间。

启动 Skill 只负责把用户目标交给对应 Plugin 的统筹能力；专业 Skills 仍由 Codex 按任务需要选择和执行，不会复制或覆盖 Plugin 内部文件。

### Plugin 生命周期与冲突保护

启动 Skill 与所属 Plugin 保持一致的生命周期：

- 安装、修复、诊断、更新和回滚时同步处理；
- 卸载 Plugin 时清理对应的入口 Skill；
- 发现同名用户 Skill 时拒绝覆盖，并给出明确诊断；
- 保留既有 Plugin 的原子安装和失败回滚能力。

### macOS 发布包签名修复

没有 Apple Developer ID 证书的 CI 构建现在显式使用 ad-hoc bundle 签名，并在 macOS runner 上执行：

```text
codesign --verify --deep --strict
```

这修复了此前“应用包看似生成，但严格验签失败”的发布问题。ad-hoc 包仍然未公证，因此 Gatekeeper 可能要求用户移除隔离属性或在系统设置中手动允许打开；这不等同于 Apple Developer ID 签名或公证。

## 验收状态

- `npm run check` 通过：前端 lint、单元测试、生产构建、Rust 格式、Clippy 和 Rust 单元测试；
- 新 Codex 任务可以发现 PilotHub 管理的 Plugin 启动 Skill；
- `LiYeshu/wechat-content-expert-team` 可在 Codex 中显示为“微信公众号内容智能体”，并保留四个内部专业 Skills；
- ChatGPT 聊天可以从 `@` 菜单选择“微信公众号内容智能体”，完成文章、封面和正文配图任务，并将完整图文写入微信公众号草稿箱；
- 本地 macOS bundle 通过 `codesign --verify --deep --strict`；
- 发布工作流对每个 macOS 应用包执行严格验签后才上传。

完整记录见 [Plugin 收尾验收](./PLUGIN-ACCEPTANCE.md)。

## 产品边界

本版仍然只支持 Skill-only Codex Plugin，不新增 MCP 管理、多 Agent runtime、Workflow Engine、Marketplace、云端账号或同步服务。PilotHub 负责获取、安装和治理 Plugin，Codex 负责理解任务并执行 Skills。

## Alpha 提示

这是一个 Alpha 版本。macOS 无证书构建使用 ad-hoc 签名，适合本地试用和发布流程验证，不代表正式分发所需的 Developer ID 签名与公证结果。
