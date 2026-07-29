use serde::Serialize;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

use super::network_proxy::github_http_client;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticStatus {
    Pass,
    Warning,
    Fail,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InstallDiagnosticCheck {
    pub id: String,
    pub status: DiagnosticStatus,
    pub detail: Option<String>,
    pub paths: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InstallDiagnostics {
    pub checks: Vec<InstallDiagnosticCheck>,
}

#[derive(Clone, Debug)]
pub struct DiagnosticAgent {
    pub label: String,
    pub installed: bool,
    pub skills_dir: PathBuf,
}

pub fn check_github_connection(proxy_url: &str) -> InstallDiagnosticCheck {
    let result = github_http_client(proxy_url, Some(5)).and_then(|client| {
        let response = client
            .get("https://api.github.com/rate_limit")
            .header("User-Agent", "PilotHub")
            .send()?;
        if response.status().is_server_error() {
            anyhow::bail!("GitHub returned {}", response.status());
        }
        Ok(())
    });

    InstallDiagnosticCheck {
        id: "github".to_string(),
        status: if result.is_ok() {
            DiagnosticStatus::Pass
        } else {
            DiagnosticStatus::Fail
        },
        detail: result.err().map(|error| error.to_string()),
        paths: Vec::new(),
    }
}

pub fn check_agent_targets(agents: &[DiagnosticAgent]) -> Vec<InstallDiagnosticCheck> {
    let detected = agents
        .iter()
        .filter(|agent| agent.installed)
        .map(|agent| agent.label.clone())
        .collect::<Vec<_>>();
    let agent_check = InstallDiagnosticCheck {
        id: "agents".to_string(),
        status: if detected.is_empty() {
            DiagnosticStatus::Fail
        } else {
            DiagnosticStatus::Pass
        },
        detail: (!detected.is_empty()).then(|| detected.join(", ")),
        paths: Vec::new(),
    };

    let mut failed_paths = Vec::new();
    let mut checked_paths = Vec::new();
    for agent in agents.iter().filter(|agent| agent.installed) {
        let path = agent.skills_dir.to_string_lossy().to_string();
        checked_paths.push(path.clone());
        if probe_writable_path(&agent.skills_dir).is_err() {
            failed_paths.push(path);
        }
    }
    let directory_check = InstallDiagnosticCheck {
        id: "directories".to_string(),
        status: if detected.is_empty() {
            DiagnosticStatus::Warning
        } else if failed_paths.is_empty() {
            DiagnosticStatus::Pass
        } else {
            DiagnosticStatus::Fail
        },
        detail: (!failed_paths.is_empty()).then(|| failed_paths.join("\n")),
        paths: checked_paths,
    };

    vec![agent_check, directory_check]
}

pub fn check_local_skill_source(source: &Path) -> InstallDiagnosticCheck {
    let skill_files = walkdir::WalkDir::new(source)
        .max_depth(4)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && entry.file_name() == "SKILL.md")
        .map(|entry| entry.path().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    InstallDiagnosticCheck {
        id: "skill_format".to_string(),
        status: if source.is_dir() && !skill_files.is_empty() {
            DiagnosticStatus::Pass
        } else {
            DiagnosticStatus::Fail
        },
        detail: if !source.is_dir() {
            Some(source.to_string_lossy().to_string())
        } else {
            None
        },
        paths: skill_files,
    }
}

pub fn git_skill_source_check(result: Result<usize, String>) -> InstallDiagnosticCheck {
    match result {
        Ok(count) if count > 0 => InstallDiagnosticCheck {
            id: "skill_format".to_string(),
            status: DiagnosticStatus::Pass,
            detail: Some(count.to_string()),
            paths: Vec::new(),
        },
        Ok(_) => InstallDiagnosticCheck {
            id: "skill_format".to_string(),
            status: DiagnosticStatus::Fail,
            detail: None,
            paths: Vec::new(),
        },
        Err(error) => InstallDiagnosticCheck {
            id: "skill_format".to_string(),
            status: DiagnosticStatus::Fail,
            detail: Some(error),
            paths: Vec::new(),
        },
    }
}

fn probe_writable_path(path: &Path) -> std::io::Result<()> {
    let existing = path
        .ancestors()
        .find(|candidate| candidate.is_dir())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no existing parent"))?;
    let probe = existing.join(format!(".pilothub-write-test-{}", uuid::Uuid::new_v4()));
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)?;
    std::fs::remove_file(probe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_source_requires_skill_md() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            check_local_skill_source(dir.path()).status,
            DiagnosticStatus::Fail
        );
        std::fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---").unwrap();
        assert_eq!(
            check_local_skill_source(dir.path()).status,
            DiagnosticStatus::Pass
        );
    }

    #[test]
    fn agents_report_detection_and_directory_access() {
        let dir = tempfile::tempdir().unwrap();
        let checks = check_agent_targets(&[DiagnosticAgent {
            label: "Codex".to_string(),
            installed: true,
            skills_dir: dir.path().join("skills"),
        }]);

        assert_eq!(checks[0].status, DiagnosticStatus::Pass);
        assert_eq!(checks[1].status, DiagnosticStatus::Pass);
    }

    #[test]
    fn missing_agents_fail_detection() {
        let checks = check_agent_targets(&[]);
        assert_eq!(checks[0].status, DiagnosticStatus::Fail);
        assert_eq!(checks[1].status, DiagnosticStatus::Warning);
    }

    #[test]
    fn git_scan_result_maps_to_format_status() {
        assert_eq!(git_skill_source_check(Ok(2)).status, DiagnosticStatus::Pass);
        assert_eq!(
            git_skill_source_check(Err("invalid".to_string())).status,
            DiagnosticStatus::Fail
        );
    }
}
