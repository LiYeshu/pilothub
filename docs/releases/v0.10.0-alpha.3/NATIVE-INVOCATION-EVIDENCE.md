# PilotHub 0.10.0-alpha.3 Native Plugin 能力探测证据

状态：PR #52 探测证据 + PR #53 原生调用验收

## 结论

PR #52 已证明 PilotHub 可以验证 Plugin 已注册、已安装并处于启用状态。2026-08-09 的 PR #53 Gate 进一步在完全移除 Compatibility Launcher 后启动两次全新、只读、临时 Codex 会话，分别验证原生发现和入口 Skill 调用。

验收后的状态为：

```text
native_registration = true
native_discovery = true
native_invocation = true
verification = verified
mode = native
```

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

## 无 Launcher 新会话验收

测试期间，PilotHub 管理的 `pilothub-wechat-content-expert-team` Launcher 被移到可恢复的临时位置，原生 Plugin 保持安装和启用。

第一次全新会话返回：

```json
{"plugin_detected":true,"plugin_name":"wechat-content-expert-team","contributed_skills":["article-writer","baoyu-article-illustrator","baoyu-cover-image","content-director"]}
```

第二次全新会话明确调用统筹 Skill，返回：

```json
{"invoked_skill":"wechat-content-expert-team:content-director","workflow_steps":["内容策划","文章撰写与润色","封面图生成","文中配图生成与配置"],"ready":true}
```

两次会话均未读取项目文件、未运行工具、未依赖 Launcher。测试完成后 Launcher 已恢复，原生 Plugin 仍为 installed + enabled。

## 自动探测规则

| CLI 状态 | Registration | Discovery | Invocation | Verification |
| --- | ---: | ---: | ---: | --- |
| installed + enabled | true | true | true | verified |
| installed + disabled | true | false | false | failed |
| 未安装或未上报 | false | false | false | failed |
| CLI 执行失败 | false | false | false | failed |

本轮验收已经覆盖当前 Codex Plugin 宿主的原生发现与调用路径。已安装且启用的 Plugin 使用 `native`；禁用、缺失或 CLI 失败时使用 `unavailable`。

## PR #53 迁移规则

- 新安装不再生成 Launcher；
- 列表刷新时只移除带 PilotHub 所有权标记的旧 Launcher；
- 用户所有的同名 Skill 永不删除或覆盖；
- 禁用、缺失或 CLI 失败时不声称原生调用可用；
- 卸载继续清理由 PilotHub 所有的遗留 Launcher。
