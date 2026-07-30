use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::WalkDir;

use super::git_fetcher::clone_or_pull;
use super::installer::parse_skill_md_with_reason;
use super::storage_migration::StorageLayout;
use super::sync_engine::copy_dir_recursive;

pub const PILOTHUB_MARKETPLACE_NAME: &str = "pilothub-local";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginSource {
    pub source_type: String,
    pub source_ref: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginSkillDescriptor {
    pub name: String,
    pub description: Option<String>,
    pub relative_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CodexPluginDescriptor {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub source: PluginSource,
    pub manifest_path: String,
    pub skills: Vec<PluginSkillDescriptor>,
    pub capabilities: Vec<String>,
    pub default_prompts: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidationItem {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<ValidationItem>,
    pub warnings: Vec<ValidationItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginPreview {
    pub descriptor: CodexPluginDescriptor,
    pub validation: ValidationReport,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginInstallationStatus {
    pub plugin_name: String,
    pub marketplace_name: String,
    pub target: String,
    pub installed: bool,
    pub enabled: bool,
    pub version: String,
    pub installed_path: Option<String>,
    pub health: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstalledCodexPlugin {
    pub descriptor: CodexPluginDescriptor,
    pub status: PluginInstallationStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginInstallResult {
    pub descriptor: CodexPluginDescriptor,
    pub status: PluginInstallationStatus,
}

#[derive(Clone, Debug, Deserialize)]
struct PluginManifest {
    name: String,
    version: String,
    description: String,
    author: Option<PluginAuthor>,
    license: Option<String>,
    skills: Option<String>,
    #[serde(rename = "mcpServers")]
    mcp_servers: Option<Value>,
    apps: Option<Value>,
    hooks: Option<Value>,
    interface: Option<PluginInterface>,
}

#[derive(Clone, Debug, Deserialize)]
struct PluginAuthor {
    name: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct PluginInterface {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    capabilities: Option<Vec<String>>,
    #[serde(rename = "defaultPrompt")]
    default_prompt: Option<Vec<String>>,
}

trait CodexCommandRunner {
    fn run(&self, args: &[String]) -> Result<Output>;
}

struct SystemCodexRunner;

impl CodexCommandRunner for SystemCodexRunner {
    fn run(&self, args: &[String]) -> Result<Output> {
        let binary = resolve_codex_binary().context("Codex CLI was not found")?;
        Command::new(&binary)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .with_context(|| format!("run Codex CLI at {binary}"))
    }
}

pub struct CodexPluginAdapter {
    layout: StorageLayout,
}

impl CodexPluginAdapter {
    pub fn from_home(home: &Path) -> Result<Self> {
        let layout = StorageLayout::from_home(home);
        layout.ensure()?;
        Ok(Self { layout })
    }

    pub fn inspect(&self, source: &PluginSource, proxy_url: Option<&str>) -> Result<PluginPreview> {
        let root = self.prepare_source(source, proxy_url)?;
        inspect_plugin_root(&root, source.clone())
    }

    pub fn install(
        &self,
        source: &PluginSource,
        proxy_url: Option<&str>,
    ) -> Result<PluginInstallResult> {
        self.install_with_runner(source, proxy_url, &SystemCodexRunner)
    }

    pub fn list(&self) -> Result<Vec<InstalledCodexPlugin>> {
        self.list_with_runner(&SystemCodexRunner)
    }

    pub fn doctor(&self, plugin_name: &str) -> Result<PluginInstallationStatus> {
        let installed = self.list()?;
        installed
            .into_iter()
            .find(|plugin| plugin.descriptor.name == plugin_name)
            .map(|plugin| plugin.status)
            .with_context(|| format!("Plugin {plugin_name} is not managed by PilotHub"))
    }

    pub fn uninstall(&self, plugin_name: &str) -> Result<()> {
        self.uninstall_with_runner(plugin_name, &SystemCodexRunner)
    }

    fn install_with_runner(
        &self,
        source: &PluginSource,
        proxy_url: Option<&str>,
        runner: &dyn CodexCommandRunner,
    ) -> Result<PluginInstallResult> {
        let preview = self.inspect(source, proxy_url)?;
        if !preview.validation.valid {
            let message = preview
                .validation
                .errors
                .iter()
                .map(|item| item.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            anyhow::bail!("PLUGIN_INVALID|{message}");
        }

        let source_root = Path::new(&preview.descriptor.manifest_path)
            .parent()
            .and_then(Path::parent)
            .context("resolve Plugin root from manifest path")?;
        let plugin_name = &preview.descriptor.name;
        let target = self.layout.codex_plugins.join(plugin_name);
        let staging_root = self
            .layout
            .codex_staging
            .join(format!("{plugin_name}-{}", Uuid::new_v4()));
        let staging_plugin = staging_root.join(plugin_name);
        std::fs::create_dir_all(&staging_root)?;

        let install_result = (|| -> Result<PluginInstallResult> {
            copy_dir_recursive(source_root, &staging_plugin).context("stage Codex Plugin files")?;
            inspect_plugin_root(&staging_plugin, source.clone())
                .context("verify staged Codex Plugin")?;

            let backup = if target.exists() {
                let backup = self
                    .layout
                    .codex_backups
                    .join(format!("{plugin_name}-{}", Uuid::new_v4()));
                std::fs::rename(&target, &backup).context("backup existing Plugin")?;
                Some(backup)
            } else {
                None
            };

            if let Err(err) = std::fs::rename(&staging_plugin, &target) {
                if let Some(backup) = &backup {
                    let _ = std::fs::rename(backup, &target);
                }
                return Err(err).context("activate staged Codex Plugin");
            }

            let marketplace_before = read_optional_file(&self.layout.codex_marketplace)?;
            if let Err(err) = self.write_marketplace_entry(&preview.descriptor) {
                let _ = remove_path(&target);
                if let Some(backup) = &backup {
                    let _ = std::fs::rename(backup, &target);
                }
                return Err(err);
            }

            let mut marketplace_added = false;
            let cli_result = (|| -> Result<PluginInstallationStatus> {
                marketplace_added = ensure_marketplace_registered(runner, &self.layout.codex)?;
                let selector = format!("{plugin_name}@{PILOTHUB_MARKETPLACE_NAME}");
                run_json_command(
                    runner,
                    &[
                        "plugin".to_string(),
                        "add".to_string(),
                        selector,
                        "--json".to_string(),
                    ],
                )?;
                find_plugin_status(runner, plugin_name)?.with_context(|| {
                    format!("Codex did not report {plugin_name} after installation")
                })
            })();

            match cli_result {
                Ok(status) => {
                    if let Some(backup) = backup {
                        let _ = remove_path(&backup);
                    }
                    Ok(PluginInstallResult {
                        descriptor: preview.descriptor.clone(),
                        status,
                    })
                }
                Err(err) => {
                    let _ = remove_path(&target);
                    if let Some(backup) = &backup {
                        let _ = std::fs::rename(backup, &target);
                    }
                    restore_optional_file(
                        &self.layout.codex_marketplace,
                        marketplace_before.as_deref(),
                    )?;
                    if marketplace_added {
                        let _ = run_json_command(
                            runner,
                            &[
                                "plugin".to_string(),
                                "marketplace".to_string(),
                                "remove".to_string(),
                                PILOTHUB_MARKETPLACE_NAME.to_string(),
                                "--json".to_string(),
                            ],
                        );
                    }
                    Err(err).context("install Plugin with Codex")
                }
            }
        })();

        let _ = remove_path(&staging_root);
        install_result
    }

    fn list_with_runner(
        &self,
        runner: &dyn CodexCommandRunner,
    ) -> Result<Vec<InstalledCodexPlugin>> {
        let entries = self.marketplace_plugin_names()?;
        let cli = plugin_list_json(runner).unwrap_or_else(|_| json!({ "installed": [] }));
        let mut plugins = Vec::new();
        for name in entries {
            let root = self.layout.codex_plugins.join(&name);
            if !root.exists() {
                continue;
            }
            let source = PluginSource {
                source_type: "managed".to_string(),
                source_ref: root.to_string_lossy().to_string(),
            };
            let preview = inspect_plugin_root(&root, source)?;
            let status = status_from_list_json(&cli, &name).unwrap_or(PluginInstallationStatus {
                plugin_name: name,
                marketplace_name: PILOTHUB_MARKETPLACE_NAME.to_string(),
                target: "codex".to_string(),
                installed: false,
                enabled: false,
                version: preview.descriptor.version.clone(),
                installed_path: None,
                health: "error".to_string(),
                detail: Some("Codex does not report this Plugin as installed".to_string()),
            });
            plugins.push(InstalledCodexPlugin {
                descriptor: preview.descriptor,
                status,
            });
        }
        plugins.sort_by(|left, right| left.descriptor.name.cmp(&right.descriptor.name));
        Ok(plugins)
    }

    fn uninstall_with_runner(
        &self,
        plugin_name: &str,
        runner: &dyn CodexCommandRunner,
    ) -> Result<()> {
        validate_plugin_name(plugin_name)?;
        if find_plugin_status(runner, plugin_name)?.is_some() {
            let selector = format!("{plugin_name}@{PILOTHUB_MARKETPLACE_NAME}");
            run_json_command(
                runner,
                &[
                    "plugin".to_string(),
                    "remove".to_string(),
                    selector,
                    "--json".to_string(),
                ],
            )
            .context("remove Plugin from Codex")?;
        }
        self.remove_marketplace_entry(plugin_name)?;
        remove_path(&self.layout.codex_plugins.join(plugin_name))?;
        Ok(())
    }

    fn prepare_source(&self, source: &PluginSource, proxy_url: Option<&str>) -> Result<PathBuf> {
        match source.source_type.as_str() {
            "local" => expand_home_path(&source.source_ref),
            "git" => {
                let mut hasher = Sha256::new();
                hasher.update(source.source_ref.as_bytes());
                let digest = hex::encode(hasher.finalize());
                let target = self.layout.cache.join("plugin-sources").join(&digest[..20]);
                clone_or_pull(&source.source_ref, &target, None, None, proxy_url)?;
                Ok(target)
            }
            other => anyhow::bail!("unsupported Plugin source type: {other}"),
        }
    }

    fn write_marketplace_entry(&self, descriptor: &CodexPluginDescriptor) -> Result<()> {
        let mut marketplace = read_marketplace(&self.layout.codex_marketplace)?;
        let plugins = marketplace
            .get_mut("plugins")
            .and_then(Value::as_array_mut)
            .context("marketplace plugins must be an array")?;
        plugins.retain(|entry| {
            entry.get("name").and_then(Value::as_str) != Some(descriptor.name.as_str())
        });
        plugins.push(json!({
            "name": descriptor.name,
            "source": {
                "source": "local",
                "path": format!("./plugins/{}", descriptor.name)
            },
            "policy": {
                "installation": "AVAILABLE",
                "authentication": "ON_INSTALL"
            },
            "category": "Productivity"
        }));
        write_json_atomic(&self.layout.codex_marketplace, &marketplace)
    }

    fn remove_marketplace_entry(&self, plugin_name: &str) -> Result<()> {
        let mut marketplace = read_marketplace(&self.layout.codex_marketplace)?;
        let plugins = marketplace
            .get_mut("plugins")
            .and_then(Value::as_array_mut)
            .context("marketplace plugins must be an array")?;
        plugins.retain(|entry| entry.get("name").and_then(Value::as_str) != Some(plugin_name));
        write_json_atomic(&self.layout.codex_marketplace, &marketplace)
    }

    fn marketplace_plugin_names(&self) -> Result<Vec<String>> {
        let marketplace = read_marketplace(&self.layout.codex_marketplace)?;
        Ok(marketplace
            .get("plugins")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.get("name").and_then(Value::as_str))
            .map(str::to_string)
            .collect())
    }
}

fn inspect_plugin_root(root: &Path, source: PluginSource) -> Result<PluginPreview> {
    let manifest_path = root.join(".codex-plugin/plugin.json");
    let bytes = std::fs::read(&manifest_path)
        .with_context(|| format!("read Plugin manifest {:?}", manifest_path))?;
    let manifest: PluginManifest =
        serde_json::from_slice(&bytes).context("parse .codex-plugin/plugin.json")?;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if let Err(err) = validate_plugin_name(&manifest.name) {
        errors.push(validation_item("invalid_name", err.to_string()));
    }
    if source_folder_name(&source, root).as_deref() != Some(manifest.name.as_str()) {
        errors.push(validation_item(
            "folder_name_mismatch",
            "Plugin folder name must match plugin.json name",
        ));
    }
    if !is_strict_semver(&manifest.version) {
        errors.push(validation_item(
            "invalid_version",
            "Plugin version must use semantic versioning",
        ));
    }
    if manifest.description.trim().is_empty() {
        errors.push(validation_item(
            "missing_description",
            "Plugin description is required",
        ));
    }
    if manifest.mcp_servers.is_some() || manifest.apps.is_some() || manifest.hooks.is_some() {
        errors.push(validation_item(
            "unsupported_components",
            "Alpha.1 only supports Skill-only Plugins",
        ));
    }

    let skills_path = manifest.skills.as_deref().unwrap_or("./skills/");
    let skills_root =
        resolve_plugin_relative_path(root, skills_path).context("resolve Plugin skills path")?;
    let mut skills = scan_plugin_skills(root, &skills_root, &mut errors)?;
    skills.sort_by(|left, right| left.name.cmp(&right.name));
    if skills.is_empty() {
        errors.push(validation_item(
            "missing_skills",
            "Plugin must contain at least one valid SKILL.md",
        ));
    }

    if manifest.author.is_none() {
        warnings.push(validation_item(
            "missing_author",
            "Plugin author metadata is recommended",
        ));
    }
    if manifest.license.is_none() {
        warnings.push(validation_item(
            "missing_license",
            "Plugin license metadata is recommended",
        ));
    }

    let interface = manifest.interface.unwrap_or_default();
    let default_prompts = interface.default_prompt.unwrap_or_default();
    if default_prompts.len() > 3 {
        warnings.push(validation_item(
            "prompt_limit",
            "Only the first three default prompts are used by Codex",
        ));
    }

    let descriptor = CodexPluginDescriptor {
        name: manifest.name.clone(),
        display_name: interface
            .display_name
            .unwrap_or_else(|| manifest.name.clone()),
        version: manifest.version,
        description: manifest.description,
        author: manifest.author.map(|author| author.name),
        license: manifest.license,
        source,
        manifest_path: manifest_path.to_string_lossy().to_string(),
        skills,
        capabilities: interface.capabilities.unwrap_or_default(),
        default_prompts: default_prompts.into_iter().take(3).collect(),
    };
    Ok(PluginPreview {
        descriptor,
        validation: ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings,
        },
    })
}

fn scan_plugin_skills(
    plugin_root: &Path,
    skills_root: &Path,
    errors: &mut Vec<ValidationItem>,
) -> Result<Vec<PluginSkillDescriptor>> {
    if !skills_root.is_dir() {
        errors.push(validation_item(
            "invalid_skills_path",
            "Configured skills directory does not exist",
        ));
        return Ok(Vec::new());
    }

    let mut names = HashSet::new();
    let mut skills = Vec::new();
    for entry in WalkDir::new(skills_root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && entry.file_name() == "SKILL.md")
    {
        let skill_dir = entry.path().parent().context("resolve Skill directory")?;
        let (name, description) = match parse_skill_md_with_reason(entry.path()) {
            Ok(value) => value,
            Err(err) => {
                errors.push(validation_item(
                    "invalid_skill",
                    format!("{}: {err}", entry.path().display()),
                ));
                continue;
            }
        };
        if !names.insert(name.clone()) {
            errors.push(validation_item(
                "duplicate_skill",
                format!("Duplicate Skill name: {name}"),
            ));
            continue;
        }
        skills.push(PluginSkillDescriptor {
            name,
            description,
            relative_path: skill_dir
                .strip_prefix(plugin_root)
                .unwrap_or(skill_dir)
                .to_string_lossy()
                .replace('\\', "/"),
        });
    }
    Ok(skills)
}

fn validate_plugin_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 64 {
        anyhow::bail!("Plugin name must contain 1 to 64 characters");
    }
    if name.starts_with('-')
        || name.ends_with('-')
        || name.contains("--")
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        anyhow::bail!("Plugin name must use lower-case kebab-case");
    }
    Ok(())
}

fn is_strict_semver(version: &str) -> bool {
    if version.is_empty() || version.matches('+').count() > 1 {
        return false;
    }
    let (without_build, build) = version
        .split_once('+')
        .map(|(left, right)| (left, Some(right)))
        .unwrap_or((version, None));
    if build.is_some_and(|value| !valid_semver_identifiers(value, false)) {
        return false;
    }
    let (core, prerelease) = without_build
        .split_once('-')
        .map(|(left, right)| (left, Some(right)))
        .unwrap_or((without_build, None));
    if prerelease.is_some_and(|value| !valid_semver_identifiers(value, true)) {
        return false;
    }
    let parts = core.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == &"0" || !part.starts_with('0'))
        })
}

fn valid_semver_identifiers(value: &str, reject_numeric_leading_zero: bool) -> bool {
    !value.is_empty()
        && value.split('.').all(|identifier| {
            !identifier.is_empty()
                && identifier
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && (!reject_numeric_leading_zero
                    || !identifier.bytes().all(|byte| byte.is_ascii_digit())
                    || identifier == "0"
                    || !identifier.starts_with('0'))
        })
}

fn resolve_plugin_relative_path(root: &Path, raw: &str) -> Result<PathBuf> {
    let relative = Path::new(raw);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        anyhow::bail!("Plugin path must remain inside the Plugin root");
    }
    let candidate = root.join(relative);
    let canonical_root = std::fs::canonicalize(root).context("resolve Plugin root")?;
    let canonical_candidate =
        std::fs::canonicalize(&candidate).context("resolve Plugin component path")?;
    if !canonical_candidate.starts_with(&canonical_root) {
        anyhow::bail!("Plugin path must remain inside the Plugin root");
    }
    Ok(candidate)
}

fn validation_item(code: impl Into<String>, message: impl Into<String>) -> ValidationItem {
    ValidationItem {
        code: code.into(),
        message: message.into(),
    }
}

fn source_folder_name(source: &PluginSource, root: &Path) -> Option<String> {
    if source.source_type == "git" {
        return source
            .source_ref
            .split(['?', '#'])
            .next()
            .unwrap_or(&source.source_ref)
            .trim_end_matches('/')
            .rsplit(['/', ':'])
            .next()
            .map(|segment| segment.trim_end_matches(".git").to_string());
    }
    root.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
}

fn expand_home_path(raw: &str) -> Result<PathBuf> {
    let trimmed = raw.trim();
    if trimmed == "~" {
        return dirs::home_dir().context("home directory is unavailable");
    }
    if let Some(rest) = trimmed.strip_prefix("~/") {
        return Ok(dirs::home_dir()
            .context("home directory is unavailable")?
            .join(rest));
    }
    Ok(PathBuf::from(trimmed))
}

fn read_marketplace(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(json!({
            "name": PILOTHUB_MARKETPLACE_NAME,
            "interface": { "displayName": "PilotHub Local" },
            "plugins": []
        }));
    }
    let value: Value =
        serde_json::from_slice(&std::fs::read(path)?).context("parse PilotHub marketplace")?;
    if value.get("name").and_then(Value::as_str) != Some(PILOTHUB_MARKETPLACE_NAME) {
        anyhow::bail!("PilotHub marketplace name does not match {PILOTHUB_MARKETPLACE_NAME}");
    }
    Ok(value)
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<()> {
    let parent = path.parent().context("resolve marketplace parent")?;
    std::fs::create_dir_all(parent)?;
    let temp = parent.join(format!(".marketplace-{}.tmp", Uuid::new_v4()));
    std::fs::write(&temp, serde_json::to_vec_pretty(value)?)?;
    std::fs::rename(&temp, path).context("activate marketplace file")
}

fn read_optional_file(path: &Path) -> Result<Option<Vec<u8>>> {
    if path.exists() {
        Ok(Some(std::fs::read(path)?))
    } else {
        Ok(None)
    }
}

fn restore_optional_file(path: &Path, content: Option<&[u8]>) -> Result<()> {
    match content {
        Some(content) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, content)?;
        }
        None if path.exists() => std::fs::remove_file(path)?,
        None => {}
    }
    Ok(())
}

fn ensure_marketplace_registered(
    runner: &dyn CodexCommandRunner,
    marketplace_root: &Path,
) -> Result<bool> {
    let list = run_json_command(
        runner,
        &[
            "plugin".to_string(),
            "marketplace".to_string(),
            "list".to_string(),
            "--json".to_string(),
        ],
    )?;
    let exists = list
        .get("marketplaces")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|entry| entry.get("name").and_then(Value::as_str) == Some(PILOTHUB_MARKETPLACE_NAME));
    if !exists {
        run_json_command(
            runner,
            &[
                "plugin".to_string(),
                "marketplace".to_string(),
                "add".to_string(),
                marketplace_root.to_string_lossy().to_string(),
                "--json".to_string(),
            ],
        )?;
        return Ok(true);
    }
    Ok(false)
}

fn find_plugin_status(
    runner: &dyn CodexCommandRunner,
    plugin_name: &str,
) -> Result<Option<PluginInstallationStatus>> {
    let list = plugin_list_json(runner)?;
    Ok(status_from_list_json(&list, plugin_name))
}

fn plugin_list_json(runner: &dyn CodexCommandRunner) -> Result<Value> {
    run_json_command(
        runner,
        &[
            "plugin".to_string(),
            "list".to_string(),
            "--available".to_string(),
            "--json".to_string(),
        ],
    )
}

fn status_from_list_json(value: &Value, plugin_name: &str) -> Option<PluginInstallationStatus> {
    value
        .get("installed")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|entry| {
            entry.get("name").and_then(Value::as_str) == Some(plugin_name)
                && entry.get("marketplaceName").and_then(Value::as_str)
                    == Some(PILOTHUB_MARKETPLACE_NAME)
        })
        .map(|entry| PluginInstallationStatus {
            plugin_name: plugin_name.to_string(),
            marketplace_name: PILOTHUB_MARKETPLACE_NAME.to_string(),
            target: "codex".to_string(),
            installed: entry
                .get("installed")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            enabled: entry
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            version: entry
                .get("version")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            installed_path: entry
                .get("installedPath")
                .and_then(Value::as_str)
                .map(str::to_string),
            health: "healthy".to_string(),
            detail: None,
        })
}

fn run_json_command(runner: &dyn CodexCommandRunner, args: &[String]) -> Result<Value> {
    let output = runner.run(args)?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!(
            "Codex command failed{}",
            if detail.is_empty() {
                String::new()
            } else {
                format!(": {detail}")
            }
        );
    }
    serde_json::from_slice(&output.stdout).context("parse Codex JSON output")
}

fn resolve_codex_binary() -> Option<String> {
    if let Ok(value) = std::env::var("PILOTHUB_CODEX_BIN") {
        if binary_works(&value) {
            return Some(value);
        }
    }
    [
        "codex",
        "/Applications/ChatGPT.app/Contents/Resources/codex",
        "/usr/local/bin/codex",
        "/opt/homebrew/bin/codex",
    ]
    .into_iter()
    .find(|candidate| binary_works(candidate))
    .map(str::to_string)
}

fn binary_works(binary: &str) -> bool {
    Command::new(binary)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn remove_path(path: &Path) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || metadata.is_file() => {
            std::fs::remove_file(path)?;
        }
        Ok(_) => std::fs::remove_dir_all(path)?,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(err.into()),
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/codex_plugins.rs"]
mod tests;
