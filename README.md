# PilotHub

PilotHub is an AI Agent extension manager. It installs and configures Skills, MCP servers, Agents, Prompts, and Hooks for environments such as Claude Code and Codex.

The first release focuses on one path:

```text
GitHub repository
        ↓
Discover and select a Skill
        ↓
Install it once
        ↓
Use it in Claude Code and Codex
```

## Project status

PilotHub currently uses the verified Skills Hub v0.8.0 codebase. The repository has completed its upstream baseline and product rebrand. The storage migration is implemented in PR-003.

Current implementation:

- Git and local Skill discovery
- Multi-Skill repository scanning through `SKILL.md`
- Central Skill storage
- Claude Code and Codex adapters
- Symlink, junction, and copy sync modes
- SQLite-backed Skill and target records

Planned PilotHub capabilities:

- Skills
- MCP servers
- Agents
- Prompts
- Hooks
- Package-manager adapters, starting with Microsoft APM

The existing `Skill` model remains the core object during the MVP. The project will add a broader `Extension` model after the Skill installation path passes end-to-end verification.

## Architecture

```text
PilotHub
│
├── Desktop UI
│   └── Fork of Skills Hub
│
├── Extension Manager
│   ├── Skills
│   ├── MCP
│   ├── Agents
│   ├── Prompts
│   └── Hooks
│
├── Package Adapter
│   └── Microsoft APM
│
├── Agent Adapter
│   ├── Claude Code
│   └── Codex
│
└── Config Tools
    └── AI Config Sync (later)
```

The desktop app uses React 19 and Tauri 2. Rust modules handle installation, persistence, Git operations, and Agent-specific filesystem sync.

## MVP

The MVP uses [JimLiu/baoyu-skills](https://github.com/JimLiu/baoyu-skills) as its first external extension source.

Acceptance criteria:

1. PilotHub scans the repository and lists its Skills.
2. A user selects and installs `baoyu-cover-image`.
3. PilotHub syncs the Skill to Claude Code and Codex.
4. Both Agents discover and run the Skill.
5. Uninstalling it leaves unrelated Skills intact.

## Out of scope

The first release does not include:

- Refly
- Dify
- Windmill
- A workflow engine
- A marketplace
- Cloud accounts
- PostPilot integration

## Engineering sequence

| Change set | Scope |
| --- | --- |
| PR-001 | Repository governance, upstream record, baseline, and licenses |
| PR-002 | Product name, desktop metadata, UI copy, icons, and updater shutdown |
| PR-003 | Migration from `~/.skillshub` to the `~/.pilothub` directory structure |
| PR-004 | `baoyu-skills` end-to-end verification in Claude Code and Codex |

Brand changes and storage migration stay in separate change sets. APM integration starts after both Agent adapters pass the Skill test.

## Repository model

```text
main       Stable, runnable versions
develop    Integration branch
feature/*  Isolated feature work
upstream   qufei1993/skills-hub
```

See [UPSTREAM.md](UPSTREAM.md) for the upstream sync policy and [docs/BASELINE.md](docs/BASELINE.md) for the verified starting point.

## Local storage

```text
~/.pilothub
├── extensions
├── cache
├── logs
├── config
└── backups
```

When PilotHub finds legacy data under `~/.skillshub`, it asks before migrating. It copies and verifies the data, updates database paths in one transaction, keeps a backup, and does not delete the legacy directory.

## Development

Requirements:

- Node.js 18 or newer
- npm, using the committed `package-lock.json`
- Rust stable
- Tauri system dependencies

```bash
npm install
npm run check
npm run tauri:dev
```

Do not generate a second package-manager lock file.

## License

PilotHub retains the upstream MIT license. See [LICENSE](LICENSE) and [LICENSES](LICENSES/README.md).
