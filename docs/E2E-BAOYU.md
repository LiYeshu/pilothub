# baoyu-skills end-to-end verification

Date: 2026-07-27

## Source

- Repository: https://github.com/JimLiu/baoyu-skills
- Selected Skill: `skills/baoyu-cover-image`
- Skill name from frontmatter: `baoyu-cover-image`

The repository currently contains 21 public `SKILL.md` files under its main Skill collection and one release helper under `.claude/skills`.

## PilotHub flow

The verification used PilotHub's existing core modules:

1. `list_git_skills` cloned and scanned the multi-Skill repository.
2. The scan returned the `baoyu-cover-image` candidate at the expected subpath.
3. `install_git_skill_from_selection` installed only that candidate.
4. `sync_dir_for_tool_with_overwrite` synchronized it to the Claude Code and Codex adapters without overwrite.
5. `SkillStore` persisted the installed Skill and both global targets.

## Verified state

| Check | Result |
| --- | --- |
| PilotHub source | `~/.pilothub/extensions/baoyu-cover-image` |
| Claude Code target | `~/.claude/skills/baoyu-cover-image` |
| Codex target | `~/.codex/skills/baoyu-cover-image` |
| Claude Code sync mode | Symlink |
| Codex sync mode | Symlink |
| Both links point to the PilotHub source | PASS |
| `SKILL.md` readable through all three paths | PASS |
| Database source type and subpath | `git`, `skills/baoyu-cover-image` |
| Database Skill and target status | `ok` |
| Existing target overwritten | No |

## Repeatable test

The ignored Rust test `installs_baoyu_cover_image_for_claude_and_codex` exercises the real user directories. It refuses to run unless the operator explicitly sets:

```bash
PILOTHUB_E2E_REAL_HOME=1
```

The test also stops before installation if the central Skill or either Agent target already exists.

## Remaining runtime check

Filesystem discovery is verified. This machine did not expose standalone `claude` or `codex` CLI executables during the run, so invoking the Skill inside a fresh Agent process and generating a cover image remains a manual acceptance step. Image generation is intentionally outside this installation test.
