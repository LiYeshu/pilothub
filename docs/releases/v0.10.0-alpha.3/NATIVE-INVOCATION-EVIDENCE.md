# PilotHub 0.10.0-alpha.3 Native Plugin 能力探测证据

状态：PR #52 implementation evidence

## 结论

PilotHub 可以通过 Codex CLI 自动验证 Plugin 已注册、已安装并处于启用状态，也可以据此确认 Plugin 内 Skills 应在新会话中进入宿主发现范围。但 CLI 不提供一次无副作用的“模拟用户调用”命令，因此不能仅凭 `plugin list` 把原生调用标记为已验证。

PR #52 采用保守状态：

```text
native_registration = true
native_discovery = true
native_invocation = false
verification = unsupported
```

现有 Compatibility Launcher 保持不变。只有后续完成一次移除 Launcher 后的新会话人工调用验收，PR #53 才能将对应宿主切换为 Native 默认入口。

## 官方产品依据

OpenAI 官方文档说明：

- Plugin 是 ChatGPT 与 Codex 中可发现、安装、分享和发布的包；
- ChatGPT 与 Codex 共享 Plugin 目录；
- 安装后应启动新对话并要求 ChatGPT 或 Codex 使用该 Plugin；
- Plugin 内 Skills 会在安装后的新对话或 CLI session 中可用。

参考：

- [Plugin architecture](https://developers.openai.com/plugins/concepts/plugins)
- [Plugins in ChatGPT and Codex](https://learn.chatgpt.com/docs/plugins)
- [Build plugins](https://learn.chatgpt.com/docs/build-plugins)

## 本机探测环境

探测日期：2026-08-09

```text
Codex CLI: 0.147.0-alpha.6.5
Marketplace: pilothub-local
Plugin: wechat-content-expert-team@pilothub-local
Plugin version: 0.1.0
```

`codex plugin list --json` 返回：

```json
{
  "pluginId": "wechat-content-expert-team@pilothub-local",
  "name": "wechat-content-expert-team",
  "marketplaceName": "pilothub-local",
  "version": "0.1.0",
  "installed": true,
  "enabled": true,
  "installPolicy": "AVAILABLE",
  "authPolicy": "ON_INSTALL"
}
```

该证据足以确认本机原生 Plugin 注册成功且处于发现条件，但不足以区分 ChatGPT `@` 菜单中的业务入口来自原生 Plugin 还是 Alpha.2 Launcher。

## 自动探测规则

| CLI 状态 | Registration | Discovery | Invocation | Verification |
| --- | ---: | ---: | ---: | --- |
| installed + enabled | true | true | false | unsupported |
| installed + disabled | true | false | false | failed |
| 未安装或未上报 | false | false | false | failed |
| CLI 执行失败 | false | false | false | failed |

`unsupported` 表示当前自动探测没有能力证明原生调用，不代表 Plugin 安装失败或运行异常。

## PR #53 前的人工 Gate

必须使用隔离测试 Plugin 或可恢复的现有 Plugin 完成：

```text
确认 Plugin 已安装且 enabled
→ 暂时移除 PilotHub 所有的 Launcher
→ 刷新 ChatGPT / Codex
→ 启动新对话
→ 从原生 Plugin 入口选择该 Plugin
→ 执行一个能明确命中统筹 Skill 的任务
→ 记录调用证据
→ 恢复原环境或进入条件化 Launcher 实现
```

在该 Gate 通过前：

- 不删除现有 Launcher；
- 不把调用模式标记为 `native`；
- 不把 CLI 安装成功等同于 UI 调用成功；
- 不改变 Alpha.2 用户的现有调用路径。
