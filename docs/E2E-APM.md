# APM E2E Verification

PilotHub validates Microsoft Agent Package Manager integration in an isolated
temporary workspace. The test never writes to the real Codex, PilotHub, or APM
user directories.

## Verified contract

- APM release: `v0.26.0`
- Package: `JimLiu/baoyu-skills/skills/baoyu-cover-image`
- Target: `codex`
- Deployed Skill: `.agents/skills/baoyu-cover-image/SKILL.md`
- Integrity gate: `apm audit --ci --no-policy --format json`
- Lifecycle: install → audit → targeted update → uninstall
- Cleanup gate: the deployed Skill and its `apm.yml` dependency are removed

Use the virtual subdirectory reference above instead of installing the full
repository with `--skill baoyu-cover-image`. The full repository contains
bundled JavaScript assets with hidden Unicode characters, so APM correctly
blocks the package-wide security scan even though those assets are unrelated to
the selected Skill. The virtual reference narrows resolution and scanning to
the intended Skill directory.

## Run

Download an official APM release, verify its published SHA-256 checksum, and
pass the executable path explicitly:

```bash
PILOTHUB_APM_E2E_BINARY=/absolute/path/to/apm \
  cargo test --manifest-path src-tauri/Cargo.toml \
  core::e2e_apm::installs_and_uninstalls_baoyu_cover_image_with_apm_for_codex \
  -- --ignored --exact --nocapture
```

The test builds the install command through `ApmAdapter`, installs into a
`tempfile` workspace, verifies the deployed Skill and lockfile, runs the APM CI
audit, updates only the selected dependency, and uninstalls it. The final checks
verify that the Skill directory is gone and the dependency no longer appears in
`apm.yml`.
