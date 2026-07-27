# baoyu-skills end-to-end verification

Date: 2026-07-27

## Source

- Repository: https://github.com/JimLiu/baoyu-skills
- Selected Skill: `skills/baoyu-cover-image`
- Skill name from frontmatter: `baoyu-cover-image`

The repository currently contains 21 public `SKILL.md` files under its main Skill collection and one release helper under `.claude/skills`.

## PilotHub flow

The verification used PilotHub's existing core modules:

1. `scan_git_skill_collection` cloned and scanned the multi-Skill repository.
2. The Collection preview resolved the repository name, author, MIT license, Skill count, and per-Skill content markers without changing the database.
3. The scan returned the `baoyu-cover-image` candidate at the expected subpath.
4. `install_git_skill_from_selection` installed only that candidate.
5. `sync_dir_for_tool_with_overwrite` synchronized it to the Antigravity and Codex adapters without overwrite.
6. `SkillStore` persisted the installed Skill and both global targets.

## Verified state

| Check | Result |
| --- | --- |
| PilotHub source | `~/.pilothub/extensions/baoyu-cover-image` |
| Antigravity target | `~/.gemini/config/skills/baoyu-cover-image` |
| Codex target | `$CODEX_HOME/skills/baoyu-cover-image` |
| Antigravity sync mode | Symlink |
| Codex sync mode | Symlink |
| Both links point to the PilotHub source | PASS |
| `SKILL.md` readable through all three paths | PASS |
| Database source type and subpath | `git`, `skills/baoyu-cover-image` |
| Database Skill and target status | `ok` |
| Existing target overwritten | No |
| Fresh Codex process discovery | `DISCOVERED` |
| Collection identity | `JimLiu/baoyu-skills` |
| Collection license | `MIT` |

On this machine, `CODEX_HOME` resolves to `~/.codex_lys`. A fresh Codex CLI process loaded the Skill through the PilotHub symlink, reported the exact name `baoyu-cover-image`, and identified its blocking first step as loading `EXTEND.md` preferences.

The Codex adapter now honors `CODEX_HOME`. When the variable is unavailable, it prefers the standard `~/.codex` directory when configured, detects one configured `~/.codex_*` home, and otherwise falls back to `~/.codex`.

## Repeatable test

The ignored Rust test `installs_baoyu_cover_image_for_antigravity_and_codex` exercises the real user directories. It refuses to run unless the operator explicitly sets:

```bash
PILOTHUB_E2E_REAL_HOME=1
```

The test installs the Skill when absent. On repeat runs, it verifies the existing central Skill and targets, then creates only missing Agent links.

## Runtime coverage

Codex filesystem discovery and fresh-process loading are verified. Antigravity filesystem discovery is verified at its configured global Skill directory. Its CLI manages the IDE but does not expose a headless Agent prompt command, so conversational invocation remains an IDE acceptance step. Image generation is intentionally outside this installation test.
