//! Bounded Project Genesis packet, storage, task-provider, and inference helpers.

use axum::{Json, http::StatusCode};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub(super) const GENESIS_SCHEMA: &str = "focusa.project_genesis.v1";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub(super) struct GenesisTaskInput {
    pub id: Option<String>,
    pub title: String,
    pub status: Option<String>,
    pub priority: Option<i64>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub(super) struct ProjectGenesisRequest {
    pub project_root: String,
    pub continuity_id: String,
    pub idempotency_key: String,
    pub hlt: Option<String>,
    pub hlt_confirmed: Option<bool>,
    pub desired_end_state: Option<String>,
    pub current_state: Option<String>,
    pub specification_ref: Option<String>,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    pub mid_level_goal: Option<String>,
    pub short_term_goal: Option<String>,
    #[serde(default)]
    pub waypoints: Vec<String>,
    pub task_provider: Option<String>,
    #[serde(default)]
    pub tasks: Vec<GenesisTaskInput>,
    pub allow_task_decomposition: Option<bool>,
    pub confirm: Option<bool>,
    pub takeover: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(super) struct ProjectGenesisStatusQuery {
    pub project_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct GenesisReceipt {
    receipt_id: String,
    phase: String,
    status: String,
    recorded_at: String,
    idempotency_key: String,
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
            "code": code,
            "message": message.into(),
            "next_action": "repair the reported Genesis prerequisite, then call project genesis resume",
        })),
    )
}

pub(super) fn canonical_root(raw: &str) -> Result<PathBuf, (StatusCode, Json<Value>)> {
    let path = PathBuf::from(raw);
    if !path.is_absolute() {
        return Err(reject(
            StatusCode::BAD_REQUEST,
            "unsafe_project_root",
            "project_root must be absolute",
        ));
    }
    let root = fs::canonicalize(&path).map_err(|error| {
        reject(
            StatusCode::BAD_REQUEST,
            "project_root_unavailable",
            format!("cannot resolve project_root: {error}"),
        )
    })?;
    if !root.join(".focusa-project.json").is_file() {
        return Err(reject(
            StatusCode::PRECONDITION_FAILED,
            "project_identity_missing",
            "verified .focusa-project.json is required before Genesis",
        ));
    }
    Ok(root)
}

fn genesis_dir(root: &Path) -> PathBuf {
    root.join(".focusa").join("genesis")
}

pub(super) fn packet_path(root: &Path) -> PathBuf {
    genesis_dir(root).join("packet.json")
}

pub(super) fn write_json_atomic(path: &Path, value: &Value) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "missing parent directory".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let tmp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy(),
        Uuid::now_v7()
    ));
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let mut file = File::create(&tmp).map_err(|error| error.to_string())?;
    file.write_all(&bytes).map_err(|error| error.to_string())?;
    file.write_all(b"\n").map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    fs::rename(&tmp, path).map_err(|error| error.to_string())?;
    File::open(parent)
        .and_then(|dir| dir.sync_all())
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

pub(super) fn stable_id(prefix: &str, root: &Path, key: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(root.to_string_lossy().as_bytes());
    hash.update(b"\0");
    hash.update(key.as_bytes());
    format!("{prefix}-{}", &hex::encode(hash.finalize())[..20])
}

pub(super) fn stable_uuid(root: &Path, key: &str) -> Uuid {
    let mut hash = Sha256::new();
    hash.update(root.to_string_lossy().as_bytes());
    hash.update(b"\0");
    hash.update(key.as_bytes());
    let digest = hash.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

pub(super) fn clean(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn discover_beads_tasks(root: &Path) -> Vec<GenesisTaskInput> {
    let path = root.join(".beads").join("issues.jsonl");
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str::<Value>(&line).ok())
        .filter_map(|issue| {
            let title = issue.get("title")?.as_str()?.trim();
            if title.is_empty() {
                return None;
            }
            let blocked_by = issue
                .get("dependencies")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|dependency| {
                    dependency
                        .get("depends_on_id")
                        .or_else(|| dependency.get("id"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .collect();
            Some(GenesisTaskInput {
                id: issue.get("id").and_then(Value::as_str).map(str::to_string),
                title: title.to_string(),
                status: issue
                    .get("status")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                priority: issue.get("priority").and_then(Value::as_i64),
                blocked_by,
                evidence_refs: Vec::new(),
            })
        })
        .collect()
}

fn select_first_task(tasks: &[GenesisTaskInput]) -> Option<GenesisTaskInput> {
    let mut ready = tasks
        .iter()
        .filter(|task| {
            matches!(
                task.status.as_deref().unwrap_or("open"),
                "open" | "in_progress" | "ready"
            ) && task.blocked_by.is_empty()
        })
        .cloned()
        .collect::<Vec<_>>();
    ready.sort_by_key(|task| {
        (
            task.priority.unwrap_or(2),
            task.id.clone().unwrap_or_default(),
        )
    });
    ready.into_iter().next()
}

fn inferred_levels(req: &ProjectGenesisRequest, hlt: &str) -> (String, String, Vec<String>) {
    let desired = clean(req.desired_end_state.as_deref()).unwrap_or_else(|| hlt.to_string());
    let current = clean(req.current_state.as_deref())
        .unwrap_or_else(|| "verified project baseline".to_string());
    let spec = clean(req.specification_ref.as_deref())
        .unwrap_or_else(|| "authoritative acceptance".to_string());
    let mlg = clean(req.mid_level_goal.as_deref()).unwrap_or_else(|| {
        format!("Close the verified gap from {current} to {desired} under {spec}")
    });
    let stg = clean(req.short_term_goal.as_deref()).unwrap_or_else(|| {
        let first = req
            .acceptance_criteria
            .first()
            .cloned()
            .unwrap_or_else(|| desired.clone());
        format!("Produce evidence for the next unsatisfied acceptance condition: {first}")
    });
    let waypoints = if req.waypoints.is_empty() {
        req.acceptance_criteria.clone()
    } else {
        req.waypoints.clone()
    };
    (mlg, stg, waypoints)
}

pub(super) fn build_staged_packet(
    root: &Path,
    req: &ProjectGenesisRequest,
    existing_hlt: Option<String>,
) -> Value {
    let supplied_hlt = clean(req.hlt.as_deref());
    let hlt = supplied_hlt.clone().or(existing_hlt);
    let hlt_confirmed = supplied_hlt.is_none() || req.hlt_confirmed.unwrap_or(false);
    let provider = clean(req.task_provider.as_deref()).unwrap_or_else(|| {
        if root.join(".beads").join("issues.jsonl").is_file() {
            "beads".to_string()
        } else {
            "provider_neutral".to_string()
        }
    });
    let mut tasks = if req.tasks.is_empty() && provider == "beads" {
        discover_beads_tasks(root)
    } else {
        req.tasks.clone()
    };
    let (mlg, stg, waypoints) = hlt
        .as_deref()
        .map(|hlt| inferred_levels(req, hlt))
        .unwrap_or_default();
    if tasks.is_empty() && req.allow_task_decomposition.unwrap_or(false) {
        tasks = waypoints
            .iter()
            .enumerate()
            .map(|(index, waypoint)| GenesisTaskInput {
                id: Some(format!("genesis-task-{}", index + 1)),
                title: waypoint.clone(),
                status: Some("open".to_string()),
                priority: Some(index as i64),
                blocked_by: if index == 0 {
                    Vec::new()
                } else {
                    vec![format!("genesis-task-{index}")]
                },
                evidence_refs: Vec::new(),
            })
            .collect();
    }
    let first_task = select_first_task(&tasks);
    let missing = [
        (hlt.is_none() || !hlt_confirmed, "hlt"),
        (
            clean(req.specification_ref.as_deref()).is_none() || req.acceptance_criteria.is_empty(),
            "specification_and_acceptance",
        ),
        (
            clean(req.current_state.as_deref()).is_none()
                || clean(req.desired_end_state.as_deref()).is_none(),
            "current_and_desired_state",
        ),
        (waypoints.is_empty(), "waypoints"),
        (tasks.is_empty(), "task_graph"),
        (first_task.is_none(), "first_workpoint_candidate"),
    ]
    .into_iter()
    .filter_map(|(is_missing, name)| is_missing.then_some(name))
    .collect::<Vec<_>>();
    let status = if hlt.is_none() || !hlt_confirmed {
        "hlt_impasse"
    } else if missing.is_empty() {
        "staged"
    } else {
        "incomplete"
    };
    let receipt_id = stable_id("genesis", root, &req.idempotency_key);
    json!({
        "schema": GENESIS_SCHEMA,
        "status": status,
        "project_identity": {"project_root": root, "verified": true},
        "bootstrap_receipt": GenesisReceipt { receipt_id: receipt_id.clone(), phase: "inventory".into(), status: status.into(), recorded_at: Utc::now().to_rfc3339(), idempotency_key: req.idempotency_key.clone() },
        "hlt": hlt,
        "hlt_status": if status == "hlt_impasse" { "missing_required" } else { "canonical_explicit" },
        "specification_and_acceptance": {"specification_ref": req.specification_ref, "acceptance_criteria": req.acceptance_criteria},
        "current_and_desired_state": {"current_state": req.current_state, "desired_end_state": req.desired_end_state},
        "mlg": mlg,
        "stg": stg,
        "waypoints": waypoints,
        "task_provider_and_task_graph": {"provider": provider, "tasks": tasks},
        "first_workpoint_candidate": first_task,
        "coordination_owner": Value::Null,
        "first_workpoint": Value::Null,
        "readiness_receipt": Value::Null,
        "missing_links": missing,
        "authority": {"operator_steering_precedence": true, "scope": root, "continuity_id": req.continuity_id},
        "freshness": {"observed_at": Utc::now().to_rfc3339()},
        "provenance": {"hlt": if supplied_hlt.is_some() { "operator_request" } else { "existing_trajectory" }, "lower_levels": "spec143_deliberate_inference", "tasks": provider},
        "confidence": if req.mid_level_goal.is_some() && req.short_term_goal.is_some() && !req.waypoints.is_empty() { "high" } else { "medium" },
        "idempotency_key": req.idempotency_key,
        "evidence_refs": (req.specification_ref.iter().cloned().collect::<Vec<_>>()),
        "next_action": if status == "hlt_impasse" { "answer one concise HLT intent question" } else if status == "staged" { "commit Project Genesis" } else { "supply the listed missing links" },
        "recovery_tools": ["focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"],
    })
}
