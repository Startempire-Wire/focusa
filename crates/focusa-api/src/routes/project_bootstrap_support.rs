//! Project bootstrap request, inspection, receipt, and provider helpers.

use axum::{Json, http::StatusCode};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub(super) struct ProjectBootstrapRequest {
    pub project_root: String,
    pub project_id: String,
    pub canonical_name: String,
    pub continuity_id: String,
    pub idempotency_key: String,
    pub discipline_profile: Option<String>,
    pub initialize_git: Option<bool>,
    pub initialize_task_provider: Option<bool>,
    pub task_provider: Option<String>,
    pub hlt: Option<String>,
    pub hlt_confirmed: Option<bool>,
    pub desired_end_state: Option<String>,
    pub current_state: Option<String>,
    pub specification_ref: Option<String>,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    pub confirm: Option<bool>,
    pub repair_action: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(super) struct ProjectBootstrapStatusQuery {
    pub project_root: String,
}

pub(super) fn reject(
    status: StatusCode,
    code: &str,
    message: impl Into<String>,
) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "status": "blocked",
            "failure_class": code,
            "message": message.into(),
            "next_action": "inspect the bootstrap preview and exact recovery before retrying",
        })),
    )
}

pub(super) fn validate_root(
    raw: &str,
    allow_missing: bool,
) -> Result<PathBuf, (StatusCode, Json<Value>)> {
    let path = PathBuf::from(raw);
    if !path.is_absolute()
        || path == Path::new("/")
        || path == Path::new("/root")
        || path == Path::new("/home")
        || path == Path::new("/tmp")
    {
        return Err(reject(
            StatusCode::BAD_REQUEST,
            "unsafe_project_root",
            "project_root must be an explicit safe absolute child path",
        ));
    }
    if path.exists() {
        fs::canonicalize(path).map_err(|error| {
            reject(
                StatusCode::BAD_REQUEST,
                "project_root_unavailable",
                format!("cannot resolve project_root: {error}"),
            )
        })
    } else if allow_missing {
        Ok(path)
    } else {
        Err(reject(
            StatusCode::NOT_FOUND,
            "project_root_missing",
            "project root does not exist; preview/apply can create it",
        ))
    }
}

pub(super) fn receipt_path(root: &Path) -> PathBuf {
    root.join(".focusa").join("bootstrap").join("receipt.json")
}

pub(super) fn write_json_atomic(path: &Path, value: &Value) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| "missing parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = parent.join(format!(".receipt-{}.tmp", Uuid::now_v7()));
    let mut file = File::create(&temporary).map_err(|error| error.to_string())?;
    file.write_all(&serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    file.write_all(b"\n").map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    fs::rename(temporary, path).map_err(|error| error.to_string())?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| error.to_string())
}

pub(super) fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

pub(super) fn stable_receipt_id(root: &Path, key: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(root.to_string_lossy().as_bytes());
    hash.update([0]);
    hash.update(key.as_bytes());
    format!("bootstrap-{}", &hex::encode(hash.finalize())[..20])
}

pub(super) fn executable(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        Command::new("sh")
            .args(["-c", &format!("command -v {name}")])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

pub(super) fn inspection(root: &Path, req: &ProjectBootstrapRequest) -> Value {
    let standard = req
        .discipline_profile
        .as_deref()
        .unwrap_or("standard_software_project")
        == "standard_software_project";
    let wants_git = req.initialize_git.unwrap_or(standard);
    let wants_tasks = req.initialize_task_provider.unwrap_or(standard);
    let provider = req.task_provider.as_deref().unwrap_or("beads");
    let root_exists = root.exists();
    let marker_exists = root.join(".focusa-project.json").is_file();
    let git_exists = root.join(".git").is_dir();
    let docs_exists = root.join("docs").is_dir();
    let beads_exists = root.join(".beads").is_dir();
    let provider_binary = executable(&["bd", "br"]);
    let mut blockers = Vec::new();
    if wants_tasks && provider == "beads" && provider_binary.is_none() {
        blockers.push("task_provider_unavailable: install or select an approved provider");
    }
    let planned_changes = [
        (!root_exists).then_some("create project folder"),
        (!marker_exists).then_some("create Focusa project marker and settings"),
        (!docs_exists).then_some("create docs folder"),
        (wants_git && !git_exists).then_some("initialize local Git repository without a remote"),
        (wants_tasks && provider == "beads" && !beads_exists)
            .then_some("initialize project-local Beads task provider"),
        Some("stage and commit Project Genesis when authority is complete"),
    ]
    .into_iter()
    .flatten()
    .map(str::to_string)
    .collect::<Vec<_>>();
    json!({
        "schema": "focusa.project_bootstrap_preview.v1",
        "status": if blockers.is_empty() { "preview_ready" } else { "blocked" },
        "project_root": root,
        "discipline_profile": req.discipline_profile.as_deref().unwrap_or("standard_software_project"),
        "observed": {
            "root": root_exists,
            "marker": marker_exists,
            "local_git": git_exists,
            "docs": docs_exists,
            "beads": beads_exists,
            "task_provider_binary": provider_binary,
        },
        "planned_changes": planned_changes,
        "preserved_choices": ["programming language", "framework", "remote", "deployment target", "domain"],
        "blockers": blockers,
        "rollback": "remove only objects listed as created_by_this_transaction; never remove adopted project state",
        "verification": ["marker guard", "local git has no remotes", "task provider health", "Genesis readiness", "first Workpoint"],
        "next_action": if wants_tasks && provider != "beads" { "supply an approved provider adapter" } else { "apply with confirm=true" },
    })
}
