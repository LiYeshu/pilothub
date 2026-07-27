# PilotHub Baseline

## Upstream

- Repository: https://github.com/qufei1993/skills-hub
- Fork: https://github.com/LiYeshu/pilothub
- Baseline commit: `42d3ae1fb863c92f04eb2a2827720ac1d93baef4`
- Baseline tag: `skills-hub-baseline-2026-07-27`
- Baseline tag status: Created locally; remote push pending Git CLI authentication

## Git configuration

- `origin`: https://github.com/LiYeshu/pilothub.git
- `upstream`: https://github.com/qufei1993/skills-hub.git
- `main`: Available locally and on the Fork
- `develop`: Available locally and on the Fork

## Environment

- macOS: 15.3.2 (24D81)
- Node.js: v22.22.3
- Package manager: npm 10.9.8
- Lock file: `package-lock.json`
- Rust: 1.97.1 (`aarch64-apple-darwin`)
- Cargo: 1.97.1
- Tauri: `@tauri-apps/cli` 2.9.6

## Verification

- Frontend install: PASS
- ESLint: PASS
- Frontend tests: PASS (31/31)
- Frontend build: PASS
- Cargo check: PASS
- Tauri dev process launch: PASS
- Claude Code directory detection prerequisite: PASS
- Codex directory detection prerequisite: PASS
- Claude Code runtime detection: PASS
- Codex runtime detection: PASS
- Git working tree clean before this report: PASS

## Known issues

- Rust is installed under `~/.cargo/bin`; the current shell requires that directory in `PATH`.
- The local proxy/TUN resolves `index.crates.io` to the `198.18.0.0/15` range but does not complete TLS forwarding.
- Cargo verification used a command-scoped `rsproxy.cn` source replacement; no project or global Cargo configuration was written.
- The shell locale `C.UTF-8` is unsupported by macOS Perl. Cargo verification used the command-scoped locale `en_US.UTF-8`.
- Tauri compiled and launched `target/debug/app`. Automated window-content inspection was unavailable because macOS denied Apple Events access to System Events.
- All 12 existing `tool_adapters` tests passed. The local `~/.claude` and `~/.codex` detection directories both exist.
- The production build reports existing chunk-size and mixed dynamic/static import warnings.
- `npm install` reports 12 dependency vulnerabilities: 1 low, 1 moderate, and 10 high.
- GitHub CLI is installed; its final OAuth authorization is pending user confirmation in Chrome.
- The baseline tag exists locally but has not been pushed.
