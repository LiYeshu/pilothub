use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use super::skill_store::SkillStore;
use super::tool_adapters::adapter_by_key;

const ENABLED_KEY: &str = "product_feedback_enabled_v1";
const EVENTS_KEY: &str = "product_feedback_events_v1";
const MAX_EVENTS: usize = 500;

const ALLOWED_EVENTS: [&str; 3] = ["install_start", "install_success", "install_fail"];
const ALLOWED_SOURCES: [&str; 2] = ["github", "local"];
const ALLOWED_FAILURE_CODES: [&str; 8] = [
    "network",
    "permission",
    "agent_unavailable",
    "target_exists",
    "invalid_skill",
    "duplicate",
    "cancelled",
    "unknown",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductFeedbackEvent {
    pub id: String,
    pub event_name: String,
    pub timestamp_ms: u64,
    pub source_kind: String,
    pub target_agents: Vec<String>,
    pub failure_code: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProductFeedbackStatus {
    pub enabled: bool,
    pub event_count: usize,
}

pub fn get_product_feedback_status(store: &SkillStore) -> Result<ProductFeedbackStatus> {
    Ok(ProductFeedbackStatus {
        enabled: is_enabled(store)?,
        event_count: load_events(store)?.len(),
    })
}

pub fn set_product_feedback_enabled(
    store: &SkillStore,
    enabled: bool,
) -> Result<ProductFeedbackStatus> {
    store.set_setting(ENABLED_KEY, if enabled { "true" } else { "false" })?;
    get_product_feedback_status(store)
}

pub fn clear_product_feedback(store: &SkillStore) -> Result<ProductFeedbackStatus> {
    store.set_setting(EVENTS_KEY, "[]")?;
    get_product_feedback_status(store)
}

pub fn record_product_feedback_event(
    store: &SkillStore,
    event_name: &str,
    source_kind: &str,
    target_agents: &[String],
    failure_code: Option<&str>,
) -> Result<bool> {
    if !is_enabled(store)? {
        return Ok(false);
    }

    let event_name = allowed_value(event_name, &ALLOWED_EVENTS, "event")?;
    let source_kind = allowed_value(source_kind, &ALLOWED_SOURCES, "source")?;
    let failure_code = failure_code
        .map(|code| allowed_value(code, &ALLOWED_FAILURE_CODES, "failure code"))
        .transpose()?;
    let mut agents = target_agents
        .iter()
        .map(|agent| {
            if adapter_by_key(agent).is_some() {
                agent.clone()
            } else {
                "other".to_string()
            }
        })
        .collect::<Vec<_>>();
    agents.sort();
    agents.dedup();

    let mut events = load_events(store)?;
    events.push(ProductFeedbackEvent {
        id: uuid::Uuid::new_v4().to_string(),
        event_name,
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system time is before UNIX epoch")?
            .as_millis() as u64,
        source_kind,
        target_agents: agents,
        failure_code,
    });
    if events.len() > MAX_EVENTS {
        events.drain(..events.len() - MAX_EVENTS);
    }
    store.set_setting(EVENTS_KEY, &serde_json::to_string(&events)?)?;
    Ok(true)
}

fn is_enabled(store: &SkillStore) -> Result<bool> {
    Ok(store
        .get_setting(ENABLED_KEY)?
        .is_some_and(|value| value == "true"))
}

fn load_events(store: &SkillStore) -> Result<Vec<ProductFeedbackEvent>> {
    let Some(raw) = store.get_setting(EVENTS_KEY)? else {
        return Ok(Vec::new());
    };
    serde_json::from_str(&raw).context("parse local product feedback events")
}

fn allowed_value(value: &str, allowed: &[&str], label: &str) -> Result<String> {
    if allowed.contains(&value) {
        Ok(value.to_string())
    } else {
        anyhow::bail!("unsupported product feedback {label}: {value}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, SkillStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = SkillStore::new(dir.path().join("db.sqlite"));
        store.ensure_schema().unwrap();
        (dir, store)
    }

    #[test]
    fn feedback_is_disabled_by_default() {
        let (_dir, store) = store();
        let status = get_product_feedback_status(&store).unwrap();
        assert!(!status.enabled);
        assert_eq!(status.event_count, 0);
    }

    #[test]
    fn disabled_feedback_does_not_store_events() {
        let (_dir, store) = store();
        let stored = record_product_feedback_event(
            &store,
            "install_start",
            "github",
            &["codex".to_string()],
            None,
        )
        .unwrap();
        assert!(!stored);
        assert_eq!(get_product_feedback_status(&store).unwrap().event_count, 0);
    }

    #[test]
    fn records_only_sanitized_fields_when_enabled() {
        let (_dir, store) = store();
        set_product_feedback_enabled(&store, true).unwrap();
        record_product_feedback_event(
            &store,
            "install_fail",
            "local",
            &["codex".to_string(), "private-agent".to_string()],
            Some("permission"),
        )
        .unwrap();

        let events = load_events(&store).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target_agents, vec!["codex", "other"]);
        assert_eq!(events[0].failure_code.as_deref(), Some("permission"));
    }

    #[test]
    fn rejects_unrecognized_values_and_can_clear_events() {
        let (_dir, store) = store();
        set_product_feedback_enabled(&store, true).unwrap();
        assert!(
            record_product_feedback_event(&store, "file_opened", "github", &[], None,).is_err()
        );
        record_product_feedback_event(&store, "install_success", "github", &[], None).unwrap();
        assert_eq!(clear_product_feedback(&store).unwrap().event_count, 0);
    }
}
