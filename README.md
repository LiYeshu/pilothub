# PilotHub

PilotHub is an AI Agent extension manager. It installs and configures Skills, MCP servers, Agents, Prompts, and Hooks for environments such as Claude Code and Codex.

The current Alpha has two verified paths:

```text
Independent Skill                 Skill-only Codex Plugin
       ↓                                   ↓
Discover and install              Inspect and validate
       ↓                                   ↓
Sync to Agent directories         Install as one Plugin
       ↓                                   ↓
Use in supported Agents           Use the expert team in Codex
```

## Project status

PilotHub `0.10.0-alpha.2` adds a standard Codex Plugin lifecycle on top of the
verified independent Skill workflow. A multi-Skill Plugin can now be installed,
diagnosed, and removed as one Extension and presented as an AI expert team.

Current implementation:

- Git and local Skill discovery
- Multi-Skill repository scanning through `SKILL.md`
- Central Skill storage
- Antigravity, Codex, and other Agent adapters
- Symlink, junction, and copy sync modes
- SQLite-backed Skill and target records
- Managed Microsoft APM runtime installation and health detection
- Project-scoped APM Skill installation, targeted updates, and consistent uninstall
- Extension collections grouped by source, with health, Agent targets, and scope summaries
- First-launch guidance for users who are new to Skills
- Recommended capability cards with plain-language use cases
- Quick install with automatic Agent detection and target selection
- Installation success guidance with the next action
- Local installation diagnostics for GitHub, Agent targets, directories, and Skill format
- Optional privacy-first product feedback stored only on the device
- Standard Skill-only Codex Plugin inspection and validation
- Atomic Plugin installation with failure rollback
- Isolated `pilothub-local` Codex marketplace registration
- Plugin diagnosis and complete lifecycle management
- AI expert team presentation for multi-Skill Plugins

Planned PilotHub capabilities:

- Skills
- MCP servers
- Agents
- Prompts
- Hooks
- Additional package-manager adapters

The existing `Skill` model remains the persisted core object. Codex Plugins are
managed through their standard manifest and Codex installation state without a
new PilotHub database schema. The native Skill, Microsoft APM, and Skill-only
Codex Plugin paths have passed end-to-end verification.

This release does not add MCP support, a multi-Agent runtime, a workflow engine,
an extension marketplace, or cloud accounts. Codex remains responsible for
selecting and executing the Skills inside a Plugin.

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
3. PilotHub installs the Skill into an Agent project through Microsoft APM.
4. Codex discovers and runs the Skill.
5. Uninstalling it leaves unrelated Skills intact.

The native adapter path has also been verified with Antigravity and Codex.

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
| PR-004 | `baoyu-skills` installation and Agent discovery verification |

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

The first external Skill verification is documented in [docs/E2E-BAOYU.md](docs/E2E-BAOYU.md).
The isolated Microsoft APM lifecycle verification is documented in
[docs/E2E-APM.md](docs/E2E-APM.md).

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
