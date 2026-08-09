# PilotHub 0.10.0-alpha.3 Plugin 生命周期证据

状态：PR #54 implementation evidence

## 目标

确保 Native Codex Plugin 在诊断、修复、更新失败和卸载后的状态始终可解释、可恢复、无误报。

## 生命周期规则

### 诊断

Doctor 保持只读，并分别报告：

- Plugin 是否注册；
- Plugin Skills 是否可被宿主发现；
- Native Plugin 是否可调用；
- 当前调用模式是 `native`、`compatibility` 或 `unavailable`。

### 修复

修复操作会重新校验本地 Plugin、重建 PilotHub marketplace 条目、重新注册 Plugin，并重新读取宿主状态。原生调用恢复后，只清理带 PilotHub 所有权标记的旧 Launcher。

### 更新失败

更新继续使用暂存目录和备份目录。Codex 注册失败时恢复上一版本 Plugin、marketplace 内容和原生入口状态，不生成兼容 Launcher。

### 卸载

卸载结束前验证：

- Plugin 文件目录不存在；
- PilotHub marketplace 中不存在该 Plugin；
- PilotHub 所有的旧 Launcher 不存在；
- Codex 不再报告该 Plugin 已安装。

任何残留都会返回 `PLUGIN_UNINSTALL_INCOMPLETE`，避免把部分卸载显示为成功。

## 自动化覆盖

- Native 安装、诊断和完整卸载；
- 修复缺失注册并清理旧 Launcher；
- 失败更新恢复上一版本；
- 卸载时检测 Codex 注册残留；
- 用户所有的同名 Skill 保持不变；
- disabled、missing 和 CLI failure 均不误报 Native 可用。
