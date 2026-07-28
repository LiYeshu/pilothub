use std::process::{Command, Output};

use crate::core::package_managers::apm::{uninstall_project_skill, ApmAdapter};
use crate::core::package_managers::{
    PackageManager, PackageManagerCommand, PackageManagerContext, PackageManagerScope,
};

const BAOYU_COVER_IMAGE_REF: &str = "JimLiu/baoyu-skills/skills/baoyu-cover-image";

#[test]
#[ignore = "downloads baoyu-skills and executes an external APM binary"]
fn installs_and_uninstalls_baoyu_cover_image_with_apm_for_codex() {
    let binary = std::env::var("PILOTHUB_APM_E2E_BINARY")
        .expect("set PILOTHUB_APM_E2E_BINARY to an APM executable");
    let workspace = tempfile::tempdir().expect("create isolated APM workspace");
    let adapter = ApmAdapter::new(binary);
    let availability = adapter.availability();
    assert!(
        availability.available,
        "APM unavailable: {:?}",
        availability.message
    );

    let context = PackageManagerContext {
        working_dir: workspace.path().to_path_buf(),
        scope: PackageManagerScope::Project,
        targets: vec!["codex".to_string()],
    };
    let install = adapter
        .install(BAOYU_COVER_IMAGE_REF, &context)
        .expect("build APM install command");
    assert_success(run_adapter_command(&install), "apm install");

    let installed_skill = workspace
        .path()
        .join(".agents/skills/baoyu-cover-image/SKILL.md");
    assert!(installed_skill.is_file(), "{installed_skill:?} is missing");
    assert!(workspace.path().join("apm.lock.yaml").is_file());

    let audit = Command::new(&install.program)
        .args(["audit", "--ci", "--no-policy", "--format", "json"])
        .current_dir(workspace.path())
        .env("APM_NON_INTERACTIVE", "1")
        .output()
        .expect("run APM audit");
    assert_success(audit, "apm audit");

    uninstall_project_skill(&adapter, BAOYU_COVER_IMAGE_REF, workspace.path())
        .expect("uninstall APM Skill");
    assert!(!workspace
        .path()
        .join(".agents/skills/baoyu-cover-image")
        .exists());
    let manifest =
        std::fs::read_to_string(workspace.path().join("apm.yml")).expect("read APM manifest");
    assert!(!manifest.contains("jimliu/baoyu-skills/skills/baoyu-cover-image"));
}

fn run_adapter_command(command: &PackageManagerCommand) -> Output {
    Command::new(&command.program)
        .args(&command.args)
        .current_dir(&command.working_dir)
        .env("APM_NON_INTERACTIVE", "1")
        .output()
        .expect("run APM adapter command")
}

fn assert_success(output: Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
