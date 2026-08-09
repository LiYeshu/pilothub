# PilotHub v0.10.0-alpha.3 Real Environment Acceptance

Date: 2026-08-09

## Scope

This acceptance validates the native Codex Plugin lifecycle introduced by PR #53 and the lifecycle diagnostics added by PR #54. It uses the real `wechat-content-expert-team` Plugin on macOS with Codex `0.147.0-alpha.6.5`.

No content was published and no image generation was invoked during this acceptance.

## Environment

- PilotHub branch: `develop`
- PilotHub base commit: `14e5737bf4616d0a4a59b6a5a61ea53c452b5b3b`
- Plugin: `wechat-content-expert-team@pilothub-local`
- Plugin version: `0.1.0`
- Target: Codex
- Installation source: PilotHub local marketplace

## Acceptance results

| Check | Result | Evidence |
| --- | --- | --- |
| Legacy Launcher migration | Pass | Starting the current `develop` build removed the PilotHub-owned `pilothub-wechat-content-expert-team` Launcher while preserving the native Plugin. |
| Native registration | Pass | `codex plugin list --json` reported `wechat-content-expert-team@pilothub-local` as installed and enabled. |
| Managed source | Pass | Codex resolved the Plugin from `~/.pilothub/codex/plugins/wechat-content-expert-team`. |
| PilotHub health status | Pass | The Extensions page displayed the Plugin as healthy with four professional Skills. |
| Recoverable uninstall | Pass | After backing up the Plugin and marketplace manifest, the Codex registration, managed Plugin directory, marketplace entry, and compatibility Launcher were all absent. |
| Reinstall | Pass | Restoring the managed package and adding the native Plugin returned version `0.1.0` with an installed cache path under the `pilothub-local` marketplace. |
| Fresh-session discovery | Pass | A new ephemeral Codex process discovered the namespaced `wechat-content-expert-team:content-director` Skill. |
| Native invocation | Pass | Codex read the installed `content-director/SKILL.md` and its required `content-project-schema.md`, then returned the four coordinated stages: planning, article writing, cover generation, and inline illustration. |
| Final installed state | Pass | The Plugin was left installed and enabled after acceptance; no compatibility Launcher was recreated. |

## Invocation probe

The final probe ran in a new read-only, ephemeral Codex session. It explicitly prohibited file creation, image generation, and publishing. The successful response was:

```json
{
  "skill_loaded": true,
  "plugin_name": "wechat-content-expert-team",
  "workflow_steps": [
    "planning",
    "article writing",
    "cover generation",
    "inline illustration"
  ]
}
```

## Non-blocking observations

- Codex emitted icon-path warnings for an unrelated bundled spreadsheet Skill. The accepted Plugin does not declare those icon paths.
- Codex could not refresh its remote installed-Plugin catalog during the isolated probe, but local native discovery and invocation still succeeded.
- PilotHub continues to treat the native Plugin as the primary invocation path. The compatibility Launcher remains absent.

## Decision

The native Plugin lifecycle is accepted for `v0.10.0-alpha.3`:

1. an existing Alpha.2 installation migrates away from the PilotHub-owned Launcher;
2. the Plugin remains visible and healthy in PilotHub;
3. uninstall leaves no managed residue;
4. reinstall restores an enabled native Plugin; and
5. a fresh Codex session can discover and invoke the Plugin's namespaced coordinator Skill.

PR #55 can close the Alpha.3 real-environment acceptance gate without additional product-code changes.

## Chat and Work follow-up

A later desktop-host acceptance exposed an important distinction that the original Codex CLI probe did not cover:

| Host surface | Discovery | Invocation | Result |
| --- | --- | --- | --- |
| Chat | The Plugin appeared in the `@` menu and its four Skills were enabled. | The conversation said that no callable `wechat-content-expert-team` instance was available and continued by simulating the workflow. | Not accepted |
| Work | The Plugin completed the coordinated workflow and produced the article, cover, three illustrations, `content-project.yaml`, and `delivery.md`. | Deliverables were present and the task reached ready status. | Pass |

The follow-up changes the interpretation of automated diagnostics:

1. installation and an enabled Codex registration do not prove execution in every host surface;
2. Plugin discovery in the Chat `@` menu does not prove that the current conversation loaded its coordinating Skill; and
3. PilotHub must report Chat, Work, and Codex execution evidence independently.

Until a host exposes machine-readable invocation evidence, PilotHub reports real task execution as unverified rather than inferring it from installation health.
