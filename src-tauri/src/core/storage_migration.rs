use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;
use uuid::Uuid;

use super::content_hash::hash_dir;
use super::skill_store::SkillStore;
use super::sync_engine::copy_dir_recursive;

pub const PILOTHUB_DIR_NAME: &str = ".pilothub";
pub const LEGACY_CENTRAL_DIR_NAME: &str = ".skillshub";

#[derive(Clone, Debug)]
pub struct StorageLayout {
    pub root: PathBuf,
    pub extensions: PathBuf,
    pub cache: PathBuf,
    pub logs: PathBuf,
    pub config: PathBuf,
    pub backups: PathBuf,
}

impl StorageLayout {
    pub fn from_home(home: &Path) -> Self {
        let root = home.join(PILOTHUB_DIR_NAME);
        Self {
            extensions: root.join("extensions"),
            cache: root.join("cache"),
            logs: root.join("logs"),
            config: root.join("config"),
            backups: root.join("backups"),
            root,
        }
    }

    pub fn ensure(&self) -> Result<()> {
        for path in [
            &self.extensions,
            &self.cache,
            &self.logs,
            &self.config,
            &self.backups,
        ] {
            std::fs::create_dir_all(path)
                .with_context(|| format!("create PilotHub storage directory {:?}", path))?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct StorageMigrationStatus {
    pub required: bool,
    pub legacy_path: String,
    pub target_path: String,
    pub backup_root: String,
    pub reason: Option<String>,
}

pub fn migration_status(home: &Path, store: &SkillStore) -> Result<StorageMigrationStatus> {
    let layout = StorageLayout::from_home(home);
    let legacy = home.join(LEGACY_CENTRAL_DIR_NAME);
    let skills = store.list_skills()?;
    let has_legacy_records = skills
        .iter()
        .any(|skill| Path::new(&skill.central_path).starts_with(&legacy));
    let legacy_has_content = directory_has_entries(&legacy)?;
    let target_has_content = directory_has_entries(&layout.extensions)?;
    let target_is_selected = store
        .get_setting("central_repo_path")?
        .is_some_and(|path| Path::new(&path) == layout.extensions);
    let required = !target_is_selected && (legacy_has_content || has_legacy_records);
    let reason = if required && target_has_content {
        Some("TARGET_NOT_EMPTY".to_string())
    } else {
        None
    };

    Ok(StorageMigrationStatus {
        required,
        legacy_path: legacy.to_string_lossy().to_string(),
        target_path: layout.extensions.to_string_lossy().to_string(),
        backup_root: layout.backups.to_string_lossy().to_string(),
        reason,
    })
}

pub fn migrate_legacy_storage(home: &Path, store: &SkillStore) -> Result<StorageMigrationStatus> {
    let layout = StorageLayout::from_home(home);
    let legacy = home.join(LEGACY_CENTRAL_DIR_NAME);
    let status = migration_status(home, store)?;
    if !status.required {
        layout.ensure()?;
        return Ok(status);
    }
    if status.reason.as_deref() == Some("TARGET_NOT_EMPTY") {
        anyhow::bail!("MIGRATION_CONFLICT|target_not_empty");
    }

    std::fs::create_dir_all(&layout.root)?;
    std::fs::create_dir_all(&layout.backups)?;
    let migration_id = format!(
        "{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        Uuid::new_v4()
    );
    let backup_dir = layout.backups.join(&migration_id);
    let backup_extensions = backup_dir.join("skillshub");
    let staging = layout
        .root
        .join(format!(".extensions-migration-{migration_id}"));

    let prepare_result = (|| -> Result<()> {
        if legacy.exists() {
            copy_dir_recursive(&legacy, &backup_extensions)
                .context("backup legacy Skill storage")?;
            copy_dir_recursive(&legacy, &staging).context("copy legacy Skill storage")?;
            let source_hash = hash_dir(&legacy).context("hash legacy Skill storage")?;
            let staging_hash = hash_dir(&staging).context("verify migrated Skill storage")?;
            if source_hash != staging_hash {
                anyhow::bail!("MIGRATION_VERIFY_FAILED|content_hash_mismatch");
            }
        } else {
            std::fs::create_dir_all(&staging)?;
        }
        Ok(())
    })();
    if let Err(err) = prepare_result {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(err);
    }

    if layout.extensions.exists() {
        if directory_has_entries(&layout.extensions)? {
            let _ = std::fs::remove_dir_all(&staging);
            anyhow::bail!("MIGRATION_CONFLICT|target_not_empty");
        }
        std::fs::remove_dir(&layout.extensions)?;
    }

    std::fs::rename(&staging, &layout.extensions).context("activate migrated Skill storage")?;
    if let Err(err) = store.rewrite_central_paths(&legacy, &layout.extensions) {
        let _ = std::fs::remove_dir_all(&layout.extensions);
        return Err(err).context("update migrated storage paths");
    }

    layout.ensure()?;
    migration_status(home, store)
}

fn directory_has_entries(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    Ok(std::fs::read_dir(path)
        .with_context(|| format!("read directory {:?}", path))?
        .next()
        .transpose()?
        .is_some())
}

#[cfg(test)]
#[path = "tests/storage_migration.rs"]
mod tests;
