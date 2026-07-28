use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use super::{
    PackageManager, PackageManagerAvailability, PackageManagerCommand, PackageManagerContext,
    PackageManagerScope,
};

const APM_ID: &str = "apm";

#[derive(Debug, Clone)]
pub struct ApmAdapter {
    binary: String,
}

impl Default for ApmAdapter {
    fn default() -> Self {
        Self::new("apm")
    }
}

impl ApmAdapter {
    pub fn new(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
        }
    }

    fn command(&self, args: Vec<String>, context: &PackageManagerContext) -> PackageManagerCommand {
        PackageManagerCommand {
            program: self.binary.clone(),
            args,
            working_dir: context.working_dir.clone(),
        }
    }
}

pub fn github_skill_package_reference(repo_url: &str, subpath: &str) -> Result<String> {
    let repo_url = repo_url.trim().trim_end_matches('/');
    let path = subpath.trim().trim_matches('/');
    if repo_url.contains("/tree/") || repo_url.contains("/blob/") {
        bail!("APM installation requires a GitHub repository root URL");
    }
    let repo = repo_url
        .strip_prefix("https://github.com/")
        .context("APM installation currently supports HTTPS GitHub URLs only")?
        .trim_end_matches(".git");
    let segments = repo.split('/').collect::<Vec<_>>();
    if segments.len() != 2 || segments.iter().any(|segment| segment.is_empty()) {
        bail!("invalid GitHub repository URL");
    }
    if path.is_empty()
        || Path::new(path)
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        bail!("invalid Skill subpath");
    }
    Ok(format!("{}/{}", repo.trim_end_matches('/'), path))
}

pub fn uninstall_project_skill(
    adapter: &ApmAdapter,
    package: &str,
    project_root: &Path,
) -> Result<()> {
    let context = PackageManagerContext {
        working_dir: project_root.to_path_buf(),
        scope: PackageManagerScope::Project,
        targets: Vec::new(),
    };
    let command = adapter.uninstall(package, &context)?;
    let output = Command::new(&command.program)
        .args(&command.args)
        .current_dir(&command.working_dir)
        .env("APM_NON_INTERACTIVE", "1")
        .output()
        .with_context(|| format!("run Microsoft APM from {:?}", command.program))?;
    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let detail = [stdout, stderr]
        .into_iter()
        .filter(|message| !message.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if detail.is_empty() {
        bail!(
            "Microsoft APM uninstall failed with exit code {}",
            output.status.code().unwrap_or(-1)
        );
    }
    bail!("Microsoft APM uninstall failed:\n{detail}");
}

impl PackageManager for ApmAdapter {
    fn id(&self) -> &'static str {
        APM_ID
    }

    fn availability(&self) -> PackageManagerAvailability {
        match Command::new(&self.binary).arg("--version").output() {
            Ok(output) if output.status.success() => PackageManagerAvailability {
                available: true,
                version: non_empty_output(&output.stdout),
                message: None,
            },
            Ok(output) => PackageManagerAvailability {
                available: false,
                version: None,
                message: non_empty_output(&output.stderr)
                    .or_else(|| Some(format!("APM exited with status {}", output.status))),
            },
            Err(error) => PackageManagerAvailability {
                available: false,
                version: None,
                message: Some(error.to_string()),
            },
        }
    }

    fn install(
        &self,
        package: &str,
        context: &PackageManagerContext,
    ) -> Result<PackageManagerCommand> {
        validate_package(package)?;
        let mut args = vec!["install".to_string(), package.to_string()];
        append_scope(&mut args, context);
        if !context.targets.is_empty() {
            args.push("--target".to_string());
            args.push(context.targets.join(","));
        }
        Ok(self.command(args, context))
    }

    fn uninstall(
        &self,
        package: &str,
        context: &PackageManagerContext,
    ) -> Result<PackageManagerCommand> {
        validate_package(package)?;
        let mut args = vec!["uninstall".to_string(), package.to_string()];
        append_scope(&mut args, context);
        Ok(self.command(args, context))
    }

    fn update(
        &self,
        package: Option<&str>,
        accept_changes: bool,
        context: &PackageManagerContext,
    ) -> Result<PackageManagerCommand> {
        if let Some(package) = package {
            validate_package(package)?;
        }
        let mut args = vec!["update".to_string()];
        if let Some(package) = package {
            args.push(package.to_string());
        }
        if accept_changes {
            args.push("--yes".to_string());
        }
        append_scope(&mut args, context);
        if !context.targets.is_empty() {
            args.push("--target".to_string());
            args.push(context.targets.join(","));
        }
        Ok(self.command(args, context))
    }

    fn list(&self, context: &PackageManagerContext) -> Result<PackageManagerCommand> {
        let mut args = vec!["list".to_string()];
        append_scope(&mut args, context);
        Ok(self.command(args, context))
    }

    fn doctor(&self, context: &PackageManagerContext) -> Result<PackageManagerCommand> {
        Ok(self.command(vec!["doctor".to_string()], context))
    }
}

fn validate_package(package: &str) -> Result<()> {
    if package.trim().is_empty() {
        bail!("package reference cannot be empty");
    }
    Ok(())
}

fn append_scope(args: &mut Vec<String>, context: &PackageManagerContext) {
    if context.scope == PackageManagerScope::Global {
        args.push("--global".to_string());
    }
}

fn non_empty_output(bytes: &[u8]) -> Option<String> {
    let output = String::from_utf8_lossy(bytes).trim().to_string();
    (!output.is_empty()).then_some(output)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn context(scope: PackageManagerScope) -> PackageManagerContext {
        PackageManagerContext {
            working_dir: PathBuf::from("/tmp/project"),
            scope,
            targets: Vec::new(),
        }
    }

    #[test]
    fn install_builds_project_command_with_targets() {
        let adapter = ApmAdapter::default();
        let mut context = context(PackageManagerScope::Project);
        context.targets = vec!["codex".to_string(), "claude".to_string()];

        let command = adapter
            .install("microsoft/apm-sample-package", &context)
            .unwrap();

        assert_eq!(command.program, "apm");
        assert_eq!(
            command.args,
            [
                "install",
                "microsoft/apm-sample-package",
                "--target",
                "codex,claude"
            ]
        );
        assert_eq!(command.working_dir, PathBuf::from("/tmp/project"));
    }

    #[test]
    fn global_commands_include_global_flag() {
        let adapter = ApmAdapter::default();
        let context = context(PackageManagerScope::Global);

        assert_eq!(
            adapter
                .uninstall("microsoft/apm-sample-package", &context)
                .unwrap()
                .args,
            ["uninstall", "microsoft/apm-sample-package", "--global"]
        );
        assert_eq!(adapter.list(&context).unwrap().args, ["list", "--global"]);
    }

    #[test]
    fn uninstall_builds_project_command_without_global_flag() {
        let adapter = ApmAdapter::default();
        let context = context(PackageManagerScope::Project);

        assert_eq!(
            adapter
                .uninstall("microsoft/apm-sample-package", &context)
                .unwrap()
                .args,
            ["uninstall", "microsoft/apm-sample-package"]
        );
    }

    #[test]
    fn builds_github_skill_package_reference() {
        assert_eq!(
            github_skill_package_reference(
                "https://github.com/JimLiu/baoyu-skills.git",
                "skills/baoyu-cover-image",
            )
            .unwrap(),
            "JimLiu/baoyu-skills/skills/baoyu-cover-image"
        );
    }

    #[test]
    fn rejects_unsafe_or_non_root_github_package_references() {
        assert!(github_skill_package_reference(
            "https://github.com/JimLiu/baoyu-skills/tree/main",
            "skills/baoyu-cover-image",
        )
        .is_err());
        assert!(github_skill_package_reference(
            "https://github.com/JimLiu/baoyu-skills",
            "../baoyu-cover-image",
        )
        .is_err());
    }

    #[test]
    fn update_requires_explicit_consent_flag() {
        let adapter = ApmAdapter::default();
        let context = context(PackageManagerScope::Project);

        assert_eq!(
            adapter.update(None, false, &context).unwrap().args,
            ["update"]
        );
        assert_eq!(
            adapter
                .update(Some("microsoft/apm"), true, &context)
                .unwrap()
                .args,
            ["update", "microsoft/apm", "--yes"]
        );
    }

    #[test]
    fn empty_package_reference_is_rejected() {
        let adapter = ApmAdapter::default();
        let error = adapter
            .install("  ", &context(PackageManagerScope::Project))
            .unwrap_err();
        assert!(error.to_string().contains("cannot be empty"));
    }

    #[test]
    fn unavailable_binary_is_reported_without_running_an_operation() {
        let adapter = ApmAdapter::new("pilothub-missing-apm-test-binary");
        let availability = adapter.availability();
        assert!(!availability.available);
        assert!(availability.version.is_none());
        assert!(availability.message.is_some());
    }
}
