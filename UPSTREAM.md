# Upstream

PilotHub is based on:

- Project: Skills Hub
- Repository: https://github.com/qufei1993/skills-hub
- License: MIT
- Baseline commit: `42d3ae1fb863c92f04eb2a2827720ac1d93baef4`
- Baseline tag: `skills-hub-baseline-2026-07-27`

The original repository is configured as the `upstream` Git remote. The PilotHub Fork is configured as `origin`.

## Sync strategy

Fetch upstream changes:

```bash
git fetch upstream --prune
```

Review commits added to upstream:

```bash
git log --oneline develop..upstream/main
```

Review the full patch:

```bash
git diff develop...upstream/main
```

PilotHub does not merge upstream changes automatically. Each upstream update requires review for conflicts with product branding, storage migration, Agent adapters, database migrations, and updater configuration.

Create a dedicated branch for an accepted update:

```bash
git switch develop
git switch -c chore/sync-upstream-YYYY-MM-DD
git merge --no-ff upstream/main
```

Run the complete repository check before merging the update:

```bash
npm run check
```

## Remote layout

```text
origin    https://github.com/LiYeshu/pilothub.git
upstream  https://github.com/qufei1993/skills-hub.git
```

Do not push PilotHub branches or tags to `upstream`.
