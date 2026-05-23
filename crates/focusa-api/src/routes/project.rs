//! Spec96 ProjectIdentity discovery and verification routes.
//!
//! ProjectIdentity is a bounded, hot-path-safe orientation record. It composes
//! filesystem/project signals; it does not select work or mutate cognitive state.

use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Deserialize, Default)]
pub struct ProjectIdentityQuery {
    pub cwd: Option<String>,
    pub project_root: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectVerifyRequest {
    pub cwd: Option<String>,
    pub project_root: Option<String>,
    pub project_id: Option<String>,
    pub canonical_name: Option<String>,
    pub repo_remote: Option<String>,
}

#[derive(Debug, Clone)]
struct ProjectSignal {
    source: &'static str,
    root: Option<String>,
    confidence: &'static str,
    independent: bool,
    details: Value,
}

#[derive(Debug, Clone)]
struct IdentityCandidate {
    project_id: String,
    canonical_name: String,
    project_root: String,
    repo_remote: Option<String>,
    beads_prefix: Option<String>,
    workspace_kind: Option<String>,
    fingerprint: String,
    confidence: &'static str,
    status: &'static str,
    signals: Vec<ProjectSignal>,
    mismatches: Vec<Value>,
    verified_at: String,
}

fn clean(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn unsafe_project_root_reason(value: &str) -> Option<&'static str> {
    let root = value.trim().trim_end_matches('/');
    if root.is_empty() {
        return Some("missing_project_root");
    }
    match root {
        "/" | "/root" | "/home" | "/tmp" | "/var" | "/usr" | "/opt" => Some("unsafe_broad_project_root"),
        _ if root.strip_prefix("/home/").is_some_and(|rest| !rest.contains('/')) => Some("unsafe_user_home_project_root"),
        _ => None,
    }
}

fn expand_home(path: &str) -> PathBuf {
    if path == "~"
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home);
    }
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

fn normalize_path(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .trim_end_matches('/')
        .to_string()
}

fn resolve_start(cwd: Option<&str>, project_root: Option<&str>) -> PathBuf {
    let candidate = clean(project_root)
        .or_else(|| clean(cwd))
        .or_else(|| std::env::var("FOCUSA_PROJECT_ROOT").ok())
        .or_else(|| std::env::var("FOCUSA_HOME").ok())
        .map(|value| expand_home(&value));
    let raw = candidate.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    if raw.is_absolute() {
        raw
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(raw)
    }
}

fn find_upwards(start: &Path, name: &str) -> Option<PathBuf> {
    let mut cur = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        let candidate = cur.join(name);
        if candidate.exists() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

fn read_marker(marker_root: &Path) -> Option<Value> {
    let path = marker_root.join(".focusa-project.json");
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
}

fn marker_project_root(marker_root: &Path, marker: &Value) -> String {
    marker
        .get("project_root")
        .and_then(Value::as_str)
        .map(expand_home)
        .map(|path| normalize_path(&path))
        .unwrap_or_else(|| normalize_path(marker_root))
}

fn read_git_remote(git_root: &Path) -> Option<String> {
    let config_path = git_root.join(".git").join("config");
    let text = fs::read_to_string(config_path).ok()?;
    let mut in_origin = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_origin = trimmed == "[remote \"origin\"]";
            continue;
        }
        if in_origin
            && let Some(rest) = trimmed.strip_prefix("url")
            && let Some((_, value)) = rest.split_once('=')
        {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn workspace_kind(root: &Path) -> Option<&'static str> {
    if root.join("Cargo.toml").exists() {
        Some("rust-workspace")
    } else if root.join("package.json").exists() {
        Some("node-workspace")
    } else if root.join("go.mod").exists() {
        Some("go-module")
    } else if root.join("pyproject.toml").exists() {
        Some("python-project")
    } else {
        None
    }
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("project")
        .to_string()
}

fn stable_fingerprint(parts: &[String]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in parts.join("|").as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("project-fnv1a64:{hash:016x}")
}

fn signal_json(signal: &ProjectSignal) -> Value {
    json!({
        "source": signal.source,
        "root": signal.root,
        "confidence": signal.confidence,
        "independent": signal.independent,
        "details": signal.details,
    })
}

fn discover_identity(cwd: Option<&str>, project_root: Option<&str>) -> IdentityCandidate {
    let start = resolve_start(cwd, project_root);
    let start_root = normalize_path(&start);
    let now = chrono::Utc::now().to_rfc3339();
    let mut signals = Vec::<ProjectSignal>::new();

    signals.push(ProjectSignal {
        source: "cwd",
        root: Some(start_root.clone()),
        confidence: "low",
        independent: false,
        details: json!({"path": start_root}),
    });

    if let Some(operator_root) = clean(project_root) {
        let root = normalize_path(&expand_home(&operator_root));
        signals.push(ProjectSignal {
            source: "operator_supplied_scope",
            root: Some(root.clone()),
            confidence: "medium",
            independent: true,
            details: json!({"project_root": root}),
        });
    }

    let marker_root = find_upwards(&start, ".focusa-project.json");
    let marker = marker_root.as_ref().and_then(|root| read_marker(root));
    if let (Some(root), Some(marker_value)) = (&marker_root, &marker) {
        let root_value = marker_project_root(root, marker_value);
        signals.push(ProjectSignal {
            source: "root_marker",
            root: Some(root_value),
            confidence: "high",
            independent: true,
            details: json!({
                "marker_path": root.join(".focusa-project.json").to_string_lossy(),
                "schema": marker_value.get("schema"),
                "project_id": marker_value.get("project_id"),
                "canonical_name": marker_value.get("canonical_name"),
            }),
        });
    }

    let git_root = find_upwards(&start, ".git");
    let repo_remote = git_root.as_ref().and_then(|root| read_git_remote(root));
    if let Some(root) = &git_root {
        signals.push(ProjectSignal {
            source: "git_root",
            root: Some(normalize_path(root)),
            confidence: "high",
            independent: true,
            details: json!({"repo_remote": repo_remote}),
        });
    }

    let beads_root = find_upwards(&start, ".beads");
    if let Some(root) = &beads_root {
        signals.push(ProjectSignal {
            source: "beads_root",
            root: Some(normalize_path(root)),
            confidence: "high",
            independent: true,
            details: json!({"beads_dir": root.join(".beads").to_string_lossy()}),
        });
    }

    let workspace_root = ["Cargo.toml", "package.json", "go.mod", "pyproject.toml"]
        .iter()
        .find_map(|name| find_upwards(&start, name));
    let workspace_kind = workspace_root.as_ref().and_then(|root| workspace_kind(root));
    if let Some(root) = &workspace_root {
        signals.push(ProjectSignal {
            source: "workspace_file",
            root: Some(normalize_path(root)),
            confidence: "medium",
            independent: true,
            details: json!({"workspace_kind": workspace_kind}),
        });
    }

    let caller_supplied_scope = clean(project_root).is_some() || clean(cwd).is_some();
    if let Ok(daemon_cwd) = std::env::current_dir() {
        signals.push(ProjectSignal {
            source: "daemon_working_directory",
            root: Some(normalize_path(&daemon_cwd)),
            confidence: "medium",
            independent: !caller_supplied_scope,
            details: json!({"working_directory": daemon_cwd.to_string_lossy(), "authority_note": "independent only when no explicit project/cwd override is supplied"}),
        });
    }

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for signal in signals.iter().filter(|signal| signal.independent) {
        if let Some(root) = &signal.root {
            *counts.entry(root.clone()).or_default() += 1;
        }
    }
    let canonical_root = counts
        .iter()
        .max_by_key(|(_, count)| **count)
        .map(|(root, _)| root.clone())
        .or_else(|| signals.iter().find_map(|signal| signal.root.clone()))
        .unwrap_or_else(|| start_root.clone());

    let mut mismatches = Vec::new();
    for signal in &signals {
        if let Some(root) = &signal.root
            && signal.independent
            && root != &canonical_root
        {
            mismatches.push(json!({
                "source": signal.source,
                "expected": canonical_root,
                "actual": root,
                "severity": if signal.source == "root_marker" { "high" } else { "medium" },
            }));
        }
    }

    let unsafe_reason = unsafe_project_root_reason(&canonical_root);
    if let Some(reason) = unsafe_reason {
        mismatches.push(json!({
            "source": "project_root_authority",
            "expected": "specific project/repo root",
            "actual": canonical_root,
            "severity": "high",
            "reason": reason,
        }));
    }

    let matching_independent = signals
        .iter()
        .filter(|signal| signal.independent && signal.root.as_deref() == Some(canonical_root.as_str()))
        .count();
    let confidence = if unsafe_reason.is_some() {
        "low"
    } else if mismatches.is_empty() && matching_independent >= 2 {
        "high"
    } else if mismatches.is_empty() && matching_independent == 1 {
        "medium"
    } else {
        "low"
    };
    let status = if unsafe_reason.is_some() {
        "unsafe_project_root"
    } else if !mismatches.is_empty() {
        "mismatch"
    } else if matching_independent >= 2 {
        "verified"
    } else if matching_independent == 1 {
        "degraded"
    } else {
        "cwd_only"
    };

    let marker_project_id = marker
        .as_ref()
        .and_then(|value| value.get("project_id"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let marker_name = marker
        .as_ref()
        .and_then(|value| value.get("canonical_name"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let marker_remote = marker
        .as_ref()
        .and_then(|value| value.get("repo_remote"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let marker_beads = marker
        .as_ref()
        .and_then(|value| value.get("beads_prefix"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let marker_workspace = marker
        .as_ref()
        .and_then(|value| value.get("workspace_kind"))
        .and_then(Value::as_str)
        .map(str::to_string);

    let project_id = marker_project_id.unwrap_or_else(|| basename(&canonical_root));
    let canonical_name = marker_name.unwrap_or_else(|| project_id.clone());
    let repo_remote = marker_remote.or(repo_remote);
    let beads_prefix = marker_beads.or_else(|| Some(project_id.clone()));
    let workspace_kind = marker_workspace.or_else(|| workspace_kind.map(str::to_string));
    let fingerprint = stable_fingerprint(&[
        project_id.clone(),
        canonical_name.clone(),
        canonical_root.clone(),
        repo_remote.clone().unwrap_or_default(),
        beads_prefix.clone().unwrap_or_default(),
    ]);

    IdentityCandidate {
        project_id,
        canonical_name,
        project_root: canonical_root,
        repo_remote,
        beads_prefix,
        workspace_kind,
        fingerprint,
        confidence,
        status,
        signals,
        mismatches,
        verified_at: now,
    }
}

fn candidate_payload(candidate: IdentityCandidate, expected: Option<&ProjectVerifyRequest>) -> Value {
    let mut mismatches = candidate.mismatches.clone();
    if let Some(expected) = expected {
        if let Some(project_id) = clean(expected.project_id.as_deref())
            && project_id != candidate.project_id
        {
            mismatches.push(json!({"source":"operator_expected_project_id", "expected": project_id, "actual": candidate.project_id, "severity":"high"}));
        }
        if let Some(name) = clean(expected.canonical_name.as_deref())
            && name != candidate.canonical_name
        {
            mismatches.push(json!({"source":"operator_expected_canonical_name", "expected": name, "actual": candidate.canonical_name, "severity":"medium"}));
        }
        if let Some(remote) = clean(expected.repo_remote.as_deref())
            && candidate.repo_remote.as_deref() != Some(remote.as_str())
        {
            mismatches.push(json!({"source":"operator_expected_repo_remote", "expected": remote, "actual": candidate.repo_remote, "severity":"medium"}));
        }
    }
    let verified = candidate.status == "verified" && mismatches.is_empty() && unsafe_project_root_reason(&candidate.project_root).is_none();
    let canonical = verified && candidate.confidence == "high";
    let status = if mismatches.is_empty() { "completed" } else { "degraded" };
    let identity_status = if candidate.status == "unsafe_project_root" {
        "unsafe_project_root"
    } else if mismatches.is_empty() {
        candidate.status
    } else {
        "mismatch"
    };
    json!({
        "status": status,
        "canonical": canonical,
        "degraded": !canonical,
        "project_identity": {
            "schema": "focusa.project_identity.v1",
            "status": identity_status,
            "project_id": candidate.project_id,
            "canonical_name": candidate.canonical_name,
            "project_root": candidate.project_root,
            "repo_remote": candidate.repo_remote,
            "beads_prefix": candidate.beads_prefix,
            "workspace_kind": candidate.workspace_kind,
            "fingerprint": candidate.fingerprint,
            "confidence": candidate.confidence,
            "signals": candidate.signals.iter().map(signal_json).collect::<Vec<_>>(),
            "mismatches": mismatches,
            "verified_at": candidate.verified_at,
            "authority_boundary": "project_root_plus_fingerprint",
        },
        "verification": {
            "verified": verified,
            "quorum_rule": "high confidence requires at least two independent matching signals; cwd-only is degraded",
            "matching_independent_signals": candidate.signals.iter().filter(|signal| signal.independent && signal.root.as_deref() == Some(candidate.project_root.as_str())).count(),
            "required_recovery": if verified { Value::Null } else { json!("resolve mismatched project signals or provide explicit project_root after checking current repo") },
        },
        "next_tools": ["focusa_project_identity", "focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"],
        "details": {"tool_result_v1": {
            "ok": verified,
            "status": status,
            "canonical": canonical,
            "degraded": !canonical,
            "failure_class": if verified { Value::Null } else { json!("scope_mismatch") },
            "retry": {"safe": verified, "posture": if verified { "safe_retry" } else { "do_not_retry_unchanged" }},
            "side_effects": [],
            "evidence_refs": [],
            "next_tools": ["focusa_project_identity", "focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"]
        }}
    })
}

pub(crate) fn project_identity_payload_for_scope(cwd: Option<&str>, project_root: Option<&str>) -> Value {
    candidate_payload(discover_identity(cwd, project_root), None)
}

async fn identity(Query(query): Query<ProjectIdentityQuery>) -> Json<Value> {
    Json(project_identity_payload_for_scope(
        query.cwd.as_deref(),
        query.project_root.as_deref(),
    ))
}

async fn verify(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<ProjectVerifyRequest>,
) -> Json<Value> {
    Json(candidate_payload(
        discover_identity(body.cwd.as_deref(), body.project_root.as_deref()),
        Some(&body),
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/project/identity", get(identity))
        .route("/v1/project/verify", post(verify))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_project(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("focusa-project-test-{name}-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(&root).expect("create temp project");
        root
    }

    #[test]
    fn git_beads_workspace_quorum_verifies_identity() {
        let root = temp_project("quorum");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".git/config"), "[remote \"origin\"]\n\turl = https://example.test/focusa.git\n").unwrap();
        fs::create_dir_all(root.join(".beads")).unwrap();
        fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        let candidate = discover_identity(root.to_str(), None);
        assert_eq!(candidate.status, "verified");
        assert_eq!(candidate.confidence, "high");
        assert!(candidate.mismatches.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn conflicting_marker_degrades_identity() {
        let root = temp_project("mismatch");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".git/config"), "").unwrap();
        fs::write(root.join(".focusa-project.json"), r#"{"schema":"focusa.project.v1","project_id":"other","project_root":"/definitely/not/this/project"}"#).unwrap();
        let candidate = discover_identity(root.to_str(), None);
        assert_eq!(candidate.status, "mismatch");
        assert!(!candidate.mismatches.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn broad_root_never_verifies_as_project_identity() {
        let candidate = discover_identity(Some("/root"), Some("/root"));
        assert_eq!(candidate.status, "unsafe_project_root");
        assert_eq!(candidate.confidence, "low");
        assert!(candidate.mismatches.iter().any(|item| item.get("source").and_then(Value::as_str) == Some("project_root_authority")));
        let payload = candidate_payload(candidate, None);
        assert_eq!(payload.pointer("/project_identity/status").and_then(Value::as_str), Some("unsafe_project_root"));
        assert_eq!(payload.pointer("/details/tool_result_v1/failure_class").and_then(Value::as_str), Some("scope_mismatch"));
    }
}
