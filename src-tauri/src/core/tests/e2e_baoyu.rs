use std::path::Path;

use crate::core::installer::{install_git_skill_from_selection, list_git_skills, InstallResult};
use crate::core::skill_store::{SkillStore, SkillTargetRecord};
use crate::core::sync_engine::sync_dir_for_tool_with_overwrite;
use crate::core::tool_adapters::{adapter_by_key, resolve_default_path};

const REPO_URL: &str = "https://github.com/JimLiu/baoyu-skills";
const SKILL_NAME: &str = "baoyu-cover-image";
const SKILL_SUBPATH: &str = "skills/baoyu-cover-image";

#[test]
#[ignore = "writes to the real PilotHub, Claude Code, and Codex user directories"]
fn installs_baoyu_cover_image_for_claude_and_codex() {
    assert_eq!(
        std::env::var("PILOTHUB_E2E_REAL_HOME").as_deref(),
        Ok("1"),
        "set PILOTHUB_E2E_REAL_HOME=1 to authorize real user-directory writes"
    );

    let home = dirs::home_dir().expect("home directory");
    let central_path = home.join(".pilothub/extensions").join(SKILL_NAME);
    let db_path = home.join(".pilothub/config/pilothub.db");
    let store = SkillStore::new(db_path);
    store.ensure_schema().expect("PilotHub database schema");
    store
        .set_setting(
            "central_repo_path",
            home.join(".pilothub/extensions").to_string_lossy().as_ref(),
        )
        .expect("set PilotHub central repository");

    let targets = ["claude_code", "codex"]
        .into_iter()
        .map(|tool| {
            let adapter = adapter_by_key(tool).expect("known Agent adapter");
            let path = resolve_default_path(&adapter)
                .expect("resolve Agent Skill directory")
                .join(SKILL_NAME);
            (tool, path)
        })
        .collect::<Vec<_>>();

    let app = tauri::test::mock_app();
    let installed = if central_path.exists() {
        let record = store
            .list_skills()
            .expect("list installed Skills")
            .into_iter()
            .find(|record| record.name == SKILL_NAME)
            .expect("installed Skill record");
        InstallResult {
            skill_id: record.id,
            name: record.name,
            central_path: record.central_path.into(),
            content_hash: record.content_hash,
        }
    } else {
        let candidates = list_git_skills(app.handle(), &store, REPO_URL).expect("scan repository");
        assert!(
            candidates
                .iter()
                .any(|item| item.name == SKILL_NAME && item.subpath == SKILL_SUBPATH),
            "baoyu-cover-image was not discovered: {:?}",
            candidates
                .iter()
                .map(|item| (&item.name, &item.subpath))
                .collect::<Vec<_>>()
        );
        install_git_skill_from_selection(app.handle(), &store, REPO_URL, SKILL_SUBPATH, None)
            .expect("install baoyu-cover-image")
    };
    assert_eq!(installed.name, SKILL_NAME);
    assert!(installed.central_path.join("SKILL.md").is_file());

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    for (tool, target) in targets {
        let mode = if std::fs::symlink_metadata(&target).is_ok() {
            assert_target_points_to(&target, &installed.central_path);
            "symlink".to_string()
        } else {
            let outcome =
                sync_dir_for_tool_with_overwrite(tool, &installed.central_path, &target, false)
                    .expect("sync Skill to Agent");
            format!("{:?}", outcome.mode_used).to_lowercase()
        };
        assert!(target.join("SKILL.md").is_file());
        assert_target_points_to(&target, &installed.central_path);
        store
            .upsert_skill_target(&SkillTargetRecord {
                id: uuid::Uuid::new_v4().to_string(),
                skill_id: installed.skill_id.clone(),
                tool: tool.to_string(),
                scope: "global".to_string(),
                project_path: None,
                target_path: target.to_string_lossy().to_string(),
                mode,
                status: "ok".to_string(),
                last_error: None,
                synced_at: Some(now),
            })
            .expect("record Agent target");
    }
}

fn assert_target_points_to(target: &Path, central: &Path) {
    if let Ok(link) = std::fs::read_link(target) {
        assert_eq!(link, central);
    }
}
