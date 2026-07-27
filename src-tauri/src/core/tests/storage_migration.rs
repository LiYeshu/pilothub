use crate::core::skill_store::{SkillRecord, SkillStore};
use crate::core::storage_migration::{migrate_legacy_storage, migration_status, StorageLayout};

fn make_skill(id: &str, central_path: &str) -> SkillRecord {
    SkillRecord {
        id: id.to_string(),
        name: id.to_string(),
        description: None,
        source_type: "local".to_string(),
        source_ref: None,
        source_subpath: None,
        source_revision: None,
        central_path: central_path.to_string(),
        content_hash: None,
        created_at: 1,
        updated_at: 1,
        last_sync_at: None,
        last_seen_at: 1,
        enabled: true,
        status: "ok".to_string(),
    }
}

#[test]
fn status_is_not_required_without_legacy_data() {
    let home = tempfile::tempdir().unwrap();
    let store = SkillStore::new(home.path().join("test.db"));
    store.ensure_schema().unwrap();

    let status = migration_status(home.path(), &store).unwrap();
    assert!(!status.required);
}

#[test]
fn migration_copies_verifies_backs_up_and_rewrites_paths() {
    let home = tempfile::tempdir().unwrap();
    let legacy = home.path().join(".skillshub");
    let skill_dir = legacy.join("example");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "# Example\n").unwrap();

    let store = SkillStore::new(home.path().join("test.db"));
    store.ensure_schema().unwrap();
    store
        .upsert_skill(&make_skill("example", skill_dir.to_string_lossy().as_ref()))
        .unwrap();

    assert!(migration_status(home.path(), &store).unwrap().required);
    let result = migrate_legacy_storage(home.path(), &store).unwrap();
    assert!(!result.required);

    let layout = StorageLayout::from_home(home.path());
    assert!(layout.extensions.join("example/SKILL.md").exists());
    assert!(legacy.join("example/SKILL.md").exists());
    assert!(std::fs::read_dir(&layout.backups).unwrap().next().is_some());

    let migrated = store.get_skill_by_id("example").unwrap().unwrap();
    assert_eq!(
        migrated.central_path,
        layout.extensions.join("example").to_string_lossy()
    );
    assert_eq!(
        store.get_setting("central_repo_path").unwrap().unwrap(),
        layout.extensions.to_string_lossy()
    );
}

#[test]
fn migration_rejects_non_empty_target_without_changing_legacy() {
    let home = tempfile::tempdir().unwrap();
    let legacy = home.path().join(".skillshub/example");
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("SKILL.md"), "# Legacy\n").unwrap();
    let layout = StorageLayout::from_home(home.path());
    std::fs::create_dir_all(&layout.extensions).unwrap();
    std::fs::write(layout.extensions.join("existing"), "keep").unwrap();

    let store = SkillStore::new(home.path().join("test.db"));
    store.ensure_schema().unwrap();
    store
        .upsert_skill(&make_skill("example", legacy.to_string_lossy().as_ref()))
        .unwrap();

    let status = migration_status(home.path(), &store).unwrap();
    assert!(status.required);
    assert_eq!(status.reason.as_deref(), Some("TARGET_NOT_EMPTY"));
    let err = migrate_legacy_storage(home.path(), &store).unwrap_err();
    assert!(err.to_string().starts_with("MIGRATION_CONFLICT|"));
    assert!(legacy.join("SKILL.md").exists());
    assert_eq!(
        std::fs::read_to_string(layout.extensions.join("existing")).unwrap(),
        "keep"
    );
}
