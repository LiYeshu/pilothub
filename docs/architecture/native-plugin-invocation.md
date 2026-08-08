# Native Plugin Invocation 与兼容入口

状态：Frozen for `0.10.0-alpha.3` implementation

对应工程阶段：PR #51

成熟度目标：完成 PilotHub L1 Extension Manager

## 1. 决策摘要

PilotHub 将完整 Plugin 作为安装、更新、诊断、回滚和卸载的主对象。宿主能够直接发现并调用 Plugin 时，PilotHub 使用宿主原生入口；只有宿主缺少该能力，或原生入口验证失败时，才创建 PilotHub 管理的兼容 Launcher Skill。

```text
Plugin 安装完成
    ↓
检测宿主能力
    ├── 支持且验证成功 → Native Plugin Entry
    └── 不支持或验证失败 → Compatibility Launcher
```

冻结以下定义：

- **Native Plugin Entry**：由宿主的 Plugin 注册、发现和调用机制提供的入口。
- **Compatibility Launcher**：PilotHub 生成的轻量 Skill，仅把用户目标交给已安装 Plugin 的统筹 Skill。
- **Plugin Skill**：Plugin 内部能力组件，不是专家团的独立产品入口。
- **Invocation Mode**：某个 Plugin 在某个宿主上的实际入口模式，值为 `native`、`compatibility` 或 `unavailable`。

Alpha.3 不通过硬编码客户端版本号判断能力，也不假设 Codex、ChatGPT 或其他 Agent 具有相同的 Plugin 机制。

## 2. 当前基线

`0.10.0-alpha.2` 已完成：

- 标准 `.codex-plugin/plugin.json` 扫描与校验；
- `pilothub-local` marketplace 注册；
- 完整 Plugin 的安装、诊断、更新、回滚和卸载；
- 为每个 Plugin 生成 `pilothub-<plugin-name>` Launcher Skill；
- 在 Codex Skills 目录及 ChatGPT `@` 菜单中发现 Launcher；
- “微信公众号内容智能体”完整业务验收。

Alpha.2 证明了 Plugin 可以交付，但用户可见入口依赖 Launcher。因此 Alpha.3 的工作不是删除已经可用的入口，而是先证明宿主原生入口，再安全降级 Launcher。

## 3. 产品与技术关系

```text
用户看到
业务助手 / 专家团
        ↓
安装与治理对象
Plugin
        ↓
调用入口（按宿主能力选择）
Native Plugin Entry 或 Compatibility Launcher
        ↓
内部能力
统筹 Skill + 专业 Skills
        ↓
执行环境
Codex / ChatGPT
```

PilotHub 负责入口选择和生命周期一致性，但不负责推理、Skill 调度或多 Agent handoff。

## 4. 宿主能力检测

每个宿主 Adapter 必须通过能力探测返回结构化结果，而不是仅判断目录是否存在：

```typescript
type PluginInvocationCapability = {
  host: 'codex' | 'chatgpt'
  nativeRegistration: boolean
  nativeDiscovery: boolean
  nativeInvocation: boolean
  verification: 'verified' | 'failed' | 'unsupported'
  detail?: string
}
```

能力检测至少包含：

1. 宿主是否提供受支持的 Plugin 注册方式；
2. 注册后的 Plugin 是否出现在宿主返回的已安装列表；
3. 用户界面或新任务是否能发现 Plugin 入口；
4. 入口是否能解析到正确 Plugin，而不是同名普通 Skill；
5. 当前用户或工作区策略是否禁止本地 Plugin。

只有注册、发现和调用三项均通过验证，Invocation Mode 才能标记为 `native`。无法自动验证 UI 调用时，不得仅凭“安装命令成功”删除兼容入口。

## 5. 入口选择规则

入口选择按 Plugin、宿主分别计算：

| 原生注册 | 原生发现与调用 | 结果 |
| --- | --- | --- |
| 支持 | 已验证 | `native`，不创建 Launcher |
| 支持 | 验证失败 | `compatibility`，保留诊断信息 |
| 不支持 | 不适用 | `compatibility` |
| 不支持 | Launcher 也不可用 | `unavailable` |

规则：

- Native 是默认优先级，不代表所有环境必须使用 Native。
- Compatibility 是正式支持的回退模式，不是安装失败状态。
- 同一 Plugin 可以在 Codex 使用 Native、在另一个宿主使用 Compatibility。
- 用户可以查看当前模式及原因，但 Alpha.3 不提供强制覆盖探测结果的高级开关。
- Launcher 命名和所有权标记沿用 Alpha.2，禁止覆盖用户同名 Skill。

## 6. 生命周期一致性

Plugin 是唯一主生命周期对象。入口不能拥有独立版本，也不能脱离 Plugin 单独更新。

### 安装

```text
验证 Plugin
→ 安装并注册 Plugin
→ 探测原生调用能力
→ 选择并创建入口
→ 验证 Plugin 与入口
→ 提交事务
```

入口验证失败时，安装事务应回退到 Compatibility；两种入口都不可用时安装失败并回滚。

### 诊断与修复

Doctor 应分别报告：

- Plugin 包、manifest、版本和来源；
- marketplace 与宿主注册状态；
- Invocation Mode；
- Native 验证结果；
- Launcher 所有权、文件和目标 Skill；
- 重复入口或残留入口。

修复时先重新探测 Native，再决定修复、保留或移除 Launcher。

### 更新与回滚

- 更新前保存 Plugin、marketplace 和入口状态快照；
- 新版本验证成功后重新探测入口能力；
- 更新失败恢复 Plugin 版本及原 Invocation Mode；
- 模式切换必须与 Plugin 更新处于同一事务；
- 回滚后不得留下指向新版本 Skill 的 Launcher。

### 卸载

卸载顺序：

```text
移除宿主 Plugin 注册
→ 移除 PilotHub 所有的 Launcher
→ 清理 marketplace 条目和托管包
→ 验证 Plugin、入口和临时文件均无残留
```

任何非 PilotHub 所有的同名 Skill 均不得删除。

## 7. Alpha.2 升级策略

Alpha.2 已安装的 Plugin 默认存在 Launcher。升级到 Alpha.3 时执行幂等迁移：

1. 读取 Launcher 的 `.pilothub-plugin-launcher.json` 所有权标记；
2. 确认对应 Plugin 仍由 `pilothub-local` 管理；
3. 探测宿主原生入口；
4. Native 验证成功后移除 PilotHub 所有的 Launcher；
5. Native 未验证时保留 Launcher，并标记 `compatibility`；
6. 用户所有或来源不明的 Skill 不参与自动清理。

迁移可重复执行，不生成重复入口，不改变 Plugin 版本。

## 8. 状态模型与 UI 语义

Alpha.3 可先扩展应用层状态，不因入口模式新增公开 manifest：

```typescript
type PluginInvocationStatus = {
  host: string
  mode: 'native' | 'compatibility' | 'unavailable'
  visible: boolean
  detail?: string
}
```

用户界面使用：

```text
调用入口：原生 Plugin
```

或：

```text
调用入口：兼容模式
原因：当前宿主尚未验证原生 Plugin 调用
```

不得继续把 Launcher 表述为 Plugin 本身，也不得把 Compatibility 状态显示为“运行异常”。

## 9. 安全与产品边界

Alpha.3 继续冻结：

- 不新增 PilotHub 自定义 Plugin manifest；
- 不引入 MCP、Apps 或连接授权；
- 不实现 Assistant Instance、品牌 Profile 或秘密存储；
- 不实现 Workflow Engine、任务状态或记忆层；
- 不实现多 Agent Runtime；
- 不建设 Marketplace 产品；
- 不将本地用户安装表述为组织级部署。

## 10. L1 完成标准

以下条件全部满足，PilotHub 才完成 L1 Extension Manager：

1. Plugin 是统一安装与生命周期对象。
2. 支持的宿主默认使用经过验证的 Native Plugin Entry。
3. 不支持 Native 的宿主可自动使用 Compatibility Launcher。
4. 用户能看到每个宿主的 Invocation Mode 和诊断原因。
5. 更新、修复和回滚不会生成重复或失效入口。
6. 卸载后 Plugin、marketplace、Launcher 和临时文件无残留。
7. Alpha.2 已安装 Plugin 可以幂等迁移。
8. “微信公众号内容智能体”通过安装、调用、更新、回滚和卸载验收。
9. 失败操作能够恢复到上一个可调用状态。
10. 未引入 L2 Assistant Instance 或 L3 Workflow Runtime。

## 11. 实现顺序

```text
PR #51  架构与路线冻结
PR #52  宿主能力检测和 Native 注册验证
PR #53  Compatibility Launcher 条件化与迁移
PR #54  生命周期、Doctor 和 UI 状态收口
PR #55  真实升级、回滚与卸载验收
PR #56  v0.10.0-alpha.3 发布
```

PR #52 必须先输出真实宿主探测证据。没有证明 Native 入口可发现、可调用之前，不得删除 Alpha.2 Launcher。
