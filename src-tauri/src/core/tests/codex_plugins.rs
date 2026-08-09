use std::fs;
use std::process::{ExitStatus, Output};
use std::sync::Mutex;

use anyhow::Result;
use serde_json::{json, Value};
use tempfile::TempDir;

use super::{
    inspect_plugin_root, read_marketplace, CodexCommandRunner, CodexPluginAdapter, PluginSource,
    PILOTHUB_MARKETPLACE_NAME,
};

fn write_plugin(root: &std::path::Path, manifest: &str, skills: &[(&str, &str)]) {
    fs::create_dir_all(root.join(".codex-plugin")).unwrap();
    fs::write(root.join(".codex-plugin/plugin.json"), manifest).unwrap();
    for (name, description) in skills {
        let skill = root.join("skills").join(name);
        fs::create_dir_all(&skill).unwrap();
        fs::write(
            skill.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: {description}\n---\n\n# {name}\n"),
        )
        .unwrap();
    }
}

fn local_source(root: &std::path::Path) -> PluginSource {
    PluginSource {
        source_type: "local".to_string(),
        source_ref: root.to_string_lossy().to_string(),
    }
}

fn test_adapter(temp: &TempDir) -> CodexPluginAdapter {
    CodexPluginAdapter::from_home_with_codex_home(temp.path(), &temp.path().join("codex-home"))
        .unwrap()
}

#[cfg(unix)]
fn success_status() -> ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    ExitStatus::from_raw(0)
}

#[cfg(windows)]
fn success_status() -> ExitStatus {
    use std::os::windows::process::ExitStatusExt;
    ExitStatus::from_raw(0)
}

#[cfg(unix)]
fn failure_status() -> ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    ExitStatus::from_raw(1 << 8)
}

#[cfg(windows)]
fn failure_status() -> ExitStatus {
    use std::os::windows::process::ExitStatusExt;
    ExitStatus::from_raw(1)
}

struct FakeCodexRunner {
    installed: Mutex<bool>,
    calls: Mutex<Vec<Vec<String>>>,
    fail_install: bool,
    fail_list: bool,
    leave_installed_after_remove: bool,
}

impl FakeCodexRunner {
    fn new() -> Self {
        Self {
            installed: Mutex::new(false),
            calls: Mutex::new(Vec::new()),
            fail_install: false,
            fail_list: false,
            leave_installed_after_remove: false,
        }
    }

    fn failing_install() -> Self {
        Self {
            fail_install: true,
            ..Self::new()
        }
    }

    fn failing_list() -> Self {
        Self {
            fail_list: true,
            ..Self::new()
        }
    }

    fn incomplete_remove() -> Self {
        Self {
            leave_installed_after_remove: true,
            ..Self::new()
        }
    }

    fn output(value: Value) -> Output {
        Output {
            status: success_status(),
            stdout: serde_json::to_vec(&value).unwrap(),
            stderr: Vec::new(),
        }
    }
}

impl CodexCommandRunner for FakeCodexRunner {
    fn run(&self, args: &[String]) -> Result<Output> {
        self.calls.lock().unwrap().push(args.to_vec());
        let command = args.join(" ");
        if command == "plugin marketplace list --json" {
            return Ok(Self::output(json!({ "marketplaces": [] })));
        }
        if command.starts_with("plugin marketplace add ") {
            return Ok(Self::output(json!({
                "marketplaceName": PILOTHUB_MARKETPLACE_NAME,
                "alreadyAdded": false
            })));
        }
        if command.starts_with("plugin add ") {
            if self.fail_install {
                return Ok(Output {
                    status: failure_status(),
                    stdout: Vec::new(),
                    stderr: b"simulated install failure".to_vec(),
                });
            }
            *self.installed.lock().unwrap() = true;
            return Ok(Self::output(json!({
                "name": "sample-plugin",
                "marketplaceName": PILOTHUB_MARKETPLACE_NAME
            })));
        }
        if command.starts_with("plugin remove ") {
            if !self.leave_installed_after_remove {
                *self.installed.lock().unwrap() = false;
            }
            return Ok(Self::output(json!({
                "name": "sample-plugin",
                "marketplaceName": PILOTHUB_MARKETPLACE_NAME
            })));
        }
        if command.starts_with("plugin marketplace remove ") {
            return Ok(Self::output(json!({
                "marketplaceName": PILOTHUB_MARKETPLACE_NAME
            })));
        }
        if command == "plugin list --available --json" {
            if self.fail_list {
                return Ok(Output {
                    status: failure_status(),
                    stdout: Vec::new(),
                    stderr: b"simulated list failure".to_vec(),
                });
            }
            let installed = *self.installed.lock().unwrap();
            return Ok(Self::output(json!({
                "installed": if installed {
                    vec![json!({
                        "name": "sample-plugin",
                        "marketplaceName": PILOTHUB_MARKETPLACE_NAME,
                        "version": "1.0.0",
                        "installed": true,
                        "enabled": true,
                        "installedPath": "/tmp/sample-plugin"
                    })]
                } else {
                    Vec::<Value>::new()
                },
                "available": []
            })));
        }
        panic!("unexpected Codex command: {command}");
    }
}

#[test]
fn inspects_a_valid_skill_only_plugin() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("content-team");
    write_plugin(
        &root,
        r#"{
          "name": "content-team",
          "version": "0.1.0",
          "description": "Create a complete content package",
          "author": { "name": "PilotHub" },
          "license": "MIT",
          "skills": "./skills/",
          "interface": {
            "displayName": "Content Team",
            "capabilities": ["Writing", "Images"],
            "defaultPrompt": ["Create a launch article"]
          }
        }"#,
        &[
            ("article-writer", "Write the article"),
            ("cover-image", "Create the cover"),
        ],
    );

    let preview = inspect_plugin_root(&root, local_source(&root)).unwrap();

    assert!(preview.validation.valid);
    assert_eq!(preview.descriptor.name, "content-team");
    assert_eq!(preview.descriptor.display_name, "Content Team");
    assert_eq!(preview.descriptor.skills.len(), 2);
    assert_eq!(preview.descriptor.default_prompts.len(), 1);
}

#[test]
fn rejects_components_outside_the_alpha_scope() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("connected-team");
    write_plugin(
        &root,
        r#"{
          "name": "connected-team",
          "version": "1.0.0",
          "description": "Uses unsupported components",
          "skills": "./skills/",
          "mcpServers": "./.mcp.json"
        }"#,
        &[("researcher", "Research a topic")],
    );

    let preview = inspect_plugin_root(&root, local_source(&root)).unwrap();

    assert!(!preview.validation.valid);
    assert!(preview
        .validation
        .errors
        .iter()
        .any(|item| item.code == "unsupported_components"));
}

#[test]
fn validates_git_plugins_against_the_repository_name_not_the_cache_folder() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("hashed-cache-folder");
    write_plugin(
        &root,
        r#"{
          "name": "content-team",
          "version": "1.0.0",
          "description": "Content workflows",
          "skills": "./skills/"
        }"#,
        &[("writer", "Write content")],
    );
    let source = PluginSource {
        source_type: "git".to_string(),
        source_ref: "https://github.com/example/content-team.git".to_string(),
    };

    let preview = inspect_plugin_root(&root, source).unwrap();

    assert!(preview.validation.valid);
}

#[test]
fn rejects_plugin_paths_that_escape_the_root() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("unsafe-plugin");
    write_plugin(
        &root,
        r#"{
          "name": "unsafe-plugin",
          "version": "1.0.0",
          "description": "Unsafe path",
          "skills": "../skills/"
        }"#,
        &[],
    );

    let error = inspect_plugin_root(&root, local_source(&root)).unwrap_err();

    assert!(format!("{error:#}").contains("inside the Plugin root"));
}

#[test]
fn writes_a_valid_pilothub_marketplace_entry() {
    let temp = TempDir::new().unwrap();
    let adapter = test_adapter(&temp);
    let root = temp.path().join("sample-plugin");
    write_plugin(
        &root,
        r#"{
          "name": "sample-plugin",
          "version": "1.0.0",
          "description": "Sample",
          "skills": "./skills/"
        }"#,
        &[("sample-skill", "Run the sample workflow")],
    );
    let preview = inspect_plugin_root(&root, local_source(&root)).unwrap();

    adapter
        .write_marketplace_entry(&preview.descriptor)
        .unwrap();

    let marketplace = read_marketplace(&adapter.layout.codex_marketplace).unwrap();
    assert_eq!(
        marketplace.get("name").and_then(Value::as_str),
        Some(PILOTHUB_MARKETPLACE_NAME)
    );
    let entry = &marketplace["plugins"][0];
    assert_eq!(entry["name"], "sample-plugin");
    assert_eq!(entry["source"]["path"], "./plugins/sample-plugin");
    assert_eq!(entry["policy"]["installation"], "AVAILABLE");
    assert_eq!(entry["policy"]["authentication"], "ON_INSTALL");
}

#[test]
fn installs_diagnoses_and_uninstalls_as_one_plugin() {
    let temp = TempDir::new().unwrap();
    let adapter = test_adapter(&temp);
    let root = temp.path().join("sample-plugin");
    write_plugin(
        &root,
        r#"{
          "name": "sample-plugin",
          "version": "1.0.0",
          "description": "Sample",
          "skills": "./skills/"
        }"#,
        &[
            ("sample-skill", "Run the sample workflow"),
            ("second-skill", "Run another workflow"),
        ],
    );
    let source = local_source(&root);
    let runner = FakeCodexRunner::new();

    let result = adapter.install_with_runner(&source, None, &runner).unwrap();

    assert!(result.status.installed);
    assert_eq!(result.descriptor.skills.len(), 2);
    assert!(adapter
        .layout
        .codex_plugins
        .join("sample-plugin/.codex-plugin/plugin.json")
        .exists());
    let launcher = adapter.codex_skills.join("pilothub-sample-plugin");
    assert!(!launcher.exists());
    let installed = adapter.list_with_runner(&runner).unwrap();
    assert_eq!(installed.len(), 1);
    assert_eq!(installed[0].status.health, "healthy");
    assert!(installed[0].status.invocation.native_registration);
    assert!(installed[0].status.invocation.native_discovery);
    assert!(installed[0].status.invocation.native_invocation);
    assert_eq!(installed[0].status.invocation.mode, "native");
    assert_eq!(installed[0].status.invocation.verification, "verified");
    assert!(!installed[0].status.catalog.visible);

    adapter
        .uninstall_with_runner("sample-plugin", &runner)
        .unwrap();

    assert!(!adapter.layout.codex_plugins.join("sample-plugin").exists());
    assert!(!launcher.exists());
    let marketplace = read_marketplace(&adapter.layout.codex_marketplace).unwrap();
    assert_eq!(marketplace["plugins"].as_array().unwrap().len(), 0);
}

#[test]
fn enabled_plugin_uses_verified_native_invocation() {
    let status = super::status_from_list_json(
        &json!({
            "installed": [{
                "name": "sample-plugin",
                "marketplaceName": PILOTHUB_MARKETPLACE_NAME,
                "version": "1.0.0",
                "installed": true,
                "enabled": true
            }]
        }),
        "sample-plugin",
    )
    .unwrap();

    assert!(status.invocation.native_registration);
    assert!(status.invocation.native_discovery);
    assert!(status.invocation.native_invocation);
    assert_eq!(status.invocation.mode, "native");
    assert_eq!(status.invocation.verification, "verified");
}

#[test]
fn disabled_plugin_fails_native_discovery_detection() {
    let status = super::status_from_list_json(
        &json!({
            "installed": [{
                "name": "sample-plugin",
                "marketplaceName": PILOTHUB_MARKETPLACE_NAME,
                "version": "1.0.0",
                "installed": true,
                "enabled": false
            }]
        }),
        "sample-plugin",
    )
    .unwrap();

    assert!(status.invocation.native_registration);
    assert!(!status.invocation.native_discovery);
    assert!(!status.invocation.native_invocation);
    assert_eq!(status.invocation.mode, "unavailable");
    assert_eq!(status.invocation.verification, "failed");
}

#[test]
fn missing_plugin_has_no_native_invocation_capability() {
    assert!(super::status_from_list_json(&json!({ "installed": [] }), "sample-plugin").is_none());

    let capability =
        super::unavailable_invocation_capability("Codex does not report this Plugin as installed");
    assert!(!capability.native_registration);
    assert!(!capability.native_discovery);
    assert!(!capability.native_invocation);
    assert_eq!(capability.mode, "unavailable");
    assert_eq!(capability.verification, "failed");
}

#[test]
fn cli_list_failure_reports_no_native_invocation_capability() {
    let temp = TempDir::new().unwrap();
    let adapter = test_adapter(&temp);
    let root = temp.path().join("sample-plugin");
    write_plugin(
        &root,
        r#"{
          "name": "sample-plugin",
          "version": "1.0.0",
          "description": "Sample",
          "skills": "./skills/"
        }"#,
        &[("sample-skill", "Run the sample workflow")],
    );
    let runner = FakeCodexRunner::new();
    adapter
        .install_with_runner(&local_source(&root), None, &runner)
        .unwrap();

    let installed = adapter
        .list_with_runner(&FakeCodexRunner::failing_list())
        .unwrap();

    assert_eq!(installed.len(), 1);
    assert_eq!(installed[0].status.health, "error");
    assert!(!installed[0].status.invocation.native_registration);
    assert!(!installed[0].status.invocation.native_discovery);
    assert!(!installed[0].status.invocation.native_invocation);
    assert_eq!(installed[0].status.invocation.mode, "unavailable");
    assert_eq!(installed[0].status.invocation.verification, "failed");
}

#[test]
fn rolls_back_files_and_marketplace_when_codex_install_fails() {
    let temp = TempDir::new().unwrap();
    let adapter = test_adapter(&temp);
    let root = temp.path().join("sample-plugin");
    write_plugin(
        &root,
        r#"{
          "name": "sample-plugin",
          "version": "1.0.0",
          "description": "Sample",
          "skills": "./skills/"
        }"#,
        &[("sample-skill", "Run the sample workflow")],
    );
    let runner = FakeCodexRunner::failing_install();

    let error = adapter
        .install_with_runner(&local_source(&root), None, &runner)
        .unwrap_err();

    assert!(format!("{error:#}").contains("simulated install failure"));
    assert!(!adapter.layout.codex_plugins.join("sample-plugin").exists());
    assert!(!adapter.layout.codex_marketplace.exists());
    assert!(runner
        .calls
        .lock()
        .unwrap()
        .iter()
        .any(|args| args.join(" ").starts_with("plugin marketplace remove ")));
    assert!(!adapter.codex_skills.join("pilothub-sample-plugin").exists());
}

#[test]
fn listing_migrates_a_pilothub_owned_legacy_launcher() {
    let temp = TempDir::new().unwrap();
    let adapter = test_adapter(&temp);
    let root = temp.path().join("sample-plugin");
    write_plugin(
        &root,
        r#"{
          "name": "sample-plugin",
          "version": "1.0.0",
          "description": "Sample",
          "skills": "./skills/"
        }"#,
        &[("sample-skill", "Run the sample workflow")],
    );
    let runner = FakeCodexRunner::new();
    adapter
        .install_with_runner(&local_source(&root), None, &runner)
        .unwrap();
    let launcher = adapter.codex_skills.join("pilothub-sample-plugin");
    let descriptor = adapter
        .inspect(&local_source(&root), None)
        .unwrap()
        .descriptor;
    super::write_catalog_launcher(&launcher, &descriptor).unwrap();
    assert!(launcher.exists());

    let installed = adapter.list_with_runner(&runner).unwrap();

    assert_eq!(installed[0].status.invocation.mode, "native");
    assert!(!installed[0].status.catalog.visible);
    assert!(!launcher.exists());
}

#[test]
fn native_install_preserves_an_unmanaged_legacy_launcher_name() {
    let temp = TempDir::new().unwrap();
    let adapter = test_adapter(&temp);
    let root = temp.path().join("sample-plugin");
    write_plugin(
        &root,
        r#"{
          "name": "sample-plugin",
          "version": "1.0.0",
          "description": "Sample",
          "skills": "./skills/"
        }"#,
        &[("sample-skill", "Run the sample workflow")],
    );
    let launcher = adapter.codex_skills.join("pilothub-sample-plugin");
    fs::create_dir_all(&launcher).unwrap();
    fs::write(launcher.join("SKILL.md"), "user-owned").unwrap();

    let result = adapter
        .install_with_runner(&local_source(&root), None, &FakeCodexRunner::new())
        .unwrap();

    assert_eq!(result.status.invocation.mode, "native");
    assert_eq!(
        fs::read_to_string(launcher.join("SKILL.md")).unwrap(),
        "user-owned"
    );
    assert!(adapter.layout.codex_plugins.join("sample-plugin").exists());
}

#[test]
fn repair_restores_registration_and_removes_owned_legacy_launcher() {
    let temp = TempDir::new().unwrap();
    let adapter = test_adapter(&temp);
    let root = temp.path().join("sample-plugin");
    write_plugin(
        &root,
        r#"{
          "name": "sample-plugin",
          "version": "1.0.0",
          "description": "Sample",
          "skills": "./skills/"
        }"#,
        &[("sample-skill", "Run the sample workflow")],
    );
    let runner = FakeCodexRunner::new();
    adapter
        .install_with_runner(&local_source(&root), None, &runner)
        .unwrap();
    *runner.installed.lock().unwrap() = false;
    let descriptor = adapter
        .inspect(&local_source(&root), None)
        .unwrap()
        .descriptor;
    let launcher = adapter.codex_skills.join("pilothub-sample-plugin");
    super::write_catalog_launcher(&launcher, &descriptor).unwrap();

    let status = adapter
        .repair_with_runner("sample-plugin", &runner)
        .unwrap();

    assert!(status.installed);
    assert_eq!(status.invocation.mode, "native");
    assert!(!launcher.exists());
}

#[test]
fn failed_native_update_preserves_the_previous_plugin() {
    let temp = TempDir::new().unwrap();
    let adapter = test_adapter(&temp);
    let root = temp.path().join("sample-plugin");
    write_plugin(
        &root,
        r#"{
          "name": "sample-plugin",
          "version": "1.0.0",
          "description": "Sample",
          "skills": "./skills/"
        }"#,
        &[("sample-skill", "Run the sample workflow")],
    );
    adapter
        .install_with_runner(&local_source(&root), None, &FakeCodexRunner::new())
        .unwrap();
    let installed_manifest = adapter
        .layout
        .codex_plugins
        .join("sample-plugin/.codex-plugin/plugin.json");
    let original = fs::read_to_string(&installed_manifest).unwrap();

    let error = adapter
        .install_with_runner(
            &local_source(&root),
            None,
            &FakeCodexRunner::failing_install(),
        )
        .unwrap_err();

    assert!(format!("{error:#}").contains("simulated install failure"));
    assert_eq!(fs::read_to_string(installed_manifest).unwrap(), original);
    assert!(!adapter.codex_skills.join("pilothub-sample-plugin").exists());
}

#[test]
fn uninstall_reports_a_codex_registration_residue() {
    let temp = TempDir::new().unwrap();
    let adapter = test_adapter(&temp);
    let root = temp.path().join("sample-plugin");
    write_plugin(
        &root,
        r#"{
          "name": "sample-plugin",
          "version": "1.0.0",
          "description": "Sample",
          "skills": "./skills/"
        }"#,
        &[("sample-skill", "Run the sample workflow")],
    );
    let runner = FakeCodexRunner::incomplete_remove();
    adapter
        .install_with_runner(&local_source(&root), None, &runner)
        .unwrap();

    let error = adapter
        .uninstall_with_runner("sample-plugin", &runner)
        .unwrap_err();

    assert!(format!("{error:#}").contains("PLUGIN_UNINSTALL_INCOMPLETE"));
}
