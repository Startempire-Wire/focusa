//! Issue #50 / Spec143 W2 project-discipline bootstrap.
//! Previewable, idempotent, local-only anatomy before Project Genesis.

use super::project_genesis;
use super::project_genesis_support::ProjectGenesisRequest;
use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};
use uuid::Uuid;

use super::project_bootstrap_support::*;

async fn preview(
    Json(req): Json<ProjectBootstrapRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let root = validate_root(&req.project_root, true)?;
    Ok(Json(inspection(&root, &req)))
}

async fn status(
    Query(query): Query<ProjectBootstrapStatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let root = validate_root(&query.project_root, false)?;
    let receipt = read_json(&receipt_path(&root));
    Ok(Json(json!({
        "schema": "focusa.project_bootstrap_status.v1",
        "status": receipt.as_ref().and_then(|value| value.get("status")).and_then(Value::as_str).unwrap_or("not_started"),
        "project_root": root,
        "receipt": receipt,
        "live": {
            "marker": root.join(".focusa-project.json").is_file(),
            "git": root.join(".git").is_dir(),
            "docs": root.join("docs").is_dir(),
            "tasks": root.join(".beads").is_dir(),
            "genesis": root.join(".focusa/genesis/packet.json").is_file(),
        },
        "next_action": if receipt.is_some() { "continue from Project Genesis readiness" } else { "preview bootstrap" },
    })))
}

fn run(root: &Path, binary: &str, args: &[&str]) -> Result<Value, String> {
    let output = Command::new(binary)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    serde_json::from_slice(&output.stdout)
        .or_else(|_| {
            Ok::<Value, serde_json::Error>(
                json!({"status":"ok","stdout":String::from_utf8_lossy(&output.stdout).trim()}),
            )
        })
        .map_err(|error| error.to_string())
}

fn initialize_tasks(
    root: &Path,
    req: &ProjectBootstrapRequest,
    created: &mut Vec<String>,
) -> Result<Value, String> {
    if root.join(".beads").is_dir() {
        return Ok(json!({"provider":"beads","status":"adopted"}));
    }
    let binary = executable(&["bd", "br"]).ok_or_else(|| {
        "Beads provider unavailable; install bd/br or select another approved provider".to_string()
    })?;
    let prefix = req
        .project_id
        .to_ascii_lowercase()
        .replace(|character: char| !character.is_ascii_alphanumeric(), "-");
    let init = run(
        root,
        &binary,
        &[
            "init",
            "--prefix",
            prefix.trim_matches('-'),
            "--json",
            "--no-daemon",
        ],
    )?;
    created.push(".beads".into());
    let mut task_ids: Vec<String> = Vec::new();
    for (index, criterion) in req.acceptance_criteria.iter().enumerate() {
        let priority = index.min(4).to_string();
        let description = format!(
            "Done condition: {criterion}\nEvidence required before closure.\nSource: {}",
            req.specification_ref
                .as_deref()
                .unwrap_or("Project Genesis acceptance")
        );
        let task = run(
            root,
            &binary,
            &[
                "create",
                criterion,
                "--type",
                "task",
                "--priority",
                &priority,
                "--description",
                &description,
                "--json",
                "--no-daemon",
            ],
        )?;
        if let Some(id) = task.get("id").and_then(Value::as_str) {
            if let Some(previous) = task_ids.last() {
                run(
                    root,
                    &binary,
                    &["dep", "add", id, previous, "--json", "--no-daemon"],
                )?;
            }
            task_ids.push(id.to_string());
        }
    }
    Ok(json!({"provider":"beads","status":"initialized","init":init,"task_ids":task_ids}))
}

async fn apply(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProjectBootstrapRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.confirm != Some(true) {
        return Err(reject(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "bootstrap apply requires confirm=true after preview",
        ));
    }
    let root = validate_root(&req.project_root, true)?;
    if let Some(receipt) = read_json(&receipt_path(&root))
        && receipt["idempotency_key"] == req.idempotency_key
        && matches!(
            receipt["status"].as_str(),
            Some("ready" | "onboarding_required")
        )
    {
        return Ok(Json(
            json!({"replayed":true,"receipt":receipt,"status":receipt["status"]}),
        ));
    }
    let preview = inspection(&root, &req);
    if preview["status"] == "blocked" {
        return Err((StatusCode::PRECONDITION_FAILED, Json(preview)));
    }
    let root_created = !root.exists();
    fs::create_dir_all(&root).map_err(|error| {
        reject(
            StatusCode::INTERNAL_SERVER_ERROR,
            "project_root_create_failed",
            error.to_string(),
        )
    })?;
    let lock_path = root.join(".focusa-bootstrap.lock");
    let lock = OpenOptions::new().write(true).create_new(true).open(&lock_path).map_err(|_| {
        reject(StatusCode::CONFLICT, "bootstrap_in_progress", "Another bootstrap transaction is active; inspect status or retry after it completes")
    })?;
    let mut created = Vec::new();
    if root_created {
        created.push("project_root".into());
    }
    let marker_path = root.join(".focusa-project.json");
    if !marker_path.exists() {
        write_json_atomic(&marker_path, &json!({
            "schema":"focusa.project.v2", "project_id":req.project_id, "canonical_name":req.canonical_name,
            "project_root":root, "workspace_kind":"software_project", "created_at":Utc::now().to_rfc3339(),
        })).map_err(|error| reject(StatusCode::INTERNAL_SERVER_ERROR, "marker_create_failed", error))?;
        created.push(".focusa-project.json".into());
    }
    let settings = root.join(".focusa/settings.json");
    if !settings.exists() {
        write_json_atomic(&settings, &json!({"schema":"focusa.project_settings.v1","discipline_profile":req.discipline_profile.as_deref().unwrap_or("standard_software_project")}))
            .map_err(|error| reject(StatusCode::INTERNAL_SERVER_ERROR, "settings_create_failed", error))?;
        created.push(".focusa/settings.json".into());
    }
    if !root.join("docs").is_dir() {
        fs::create_dir_all(root.join("docs")).map_err(|error| {
            reject(
                StatusCode::INTERNAL_SERVER_ERROR,
                "docs_create_failed",
                error.to_string(),
            )
        })?;
        created.push("docs".into());
    }
    let standard = req
        .discipline_profile
        .as_deref()
        .unwrap_or("standard_software_project")
        == "standard_software_project";
    if req.initialize_git.unwrap_or(standard) && !root.join(".git").is_dir() {
        let result = run(&root, "git", &["init"]).map_err(|error| {
            reject(
                StatusCode::SERVICE_UNAVAILABLE,
                "local_git_init_failed",
                error,
            )
        })?;
        if Command::new("git")
            .args(["remote"])
            .current_dir(&root)
            .output()
            .map(|output| !output.stdout.is_empty())
            .unwrap_or(false)
        {
            return Err(reject(
                StatusCode::CONFLICT,
                "implicit_remote_forbidden",
                "bootstrap never creates or adopts an unapproved remote",
            ));
        }
        created.push(".git".into());
        let _ = result;
    }
    let task_provider = if req.initialize_task_provider.unwrap_or(standard) {
        if req.task_provider.as_deref().unwrap_or("beads") != "beads" {
            return Err(reject(
                StatusCode::NOT_IMPLEMENTED,
                "provider_adapter_required",
                "selected provider requires an approved adapter",
            ));
        }
        initialize_tasks(&root, &req, &mut created).map_err(|error| {
            reject(
                StatusCode::SERVICE_UNAVAILABLE,
                "task_provider_unhealthy",
                error,
            )
        })?
    } else {
        json!({"provider":"none","status":"waived_by_explicit_profile"})
    };

    let genesis_existed = root.join(".focusa/genesis").is_dir();
    let genesis = ProjectGenesisRequest {
        project_root: root.to_string_lossy().to_string(),
        continuity_id: req.continuity_id.clone(),
        idempotency_key: format!("{}:genesis", req.idempotency_key),
        hlt: req.hlt.clone(),
        hlt_confirmed: req.hlt_confirmed,
        desired_end_state: req.desired_end_state.clone(),
        current_state: req.current_state.clone(),
        specification_ref: req.specification_ref.clone(),
        acceptance_criteria: req.acceptance_criteria.clone(),
        task_provider: Some("beads".into()),
        allow_task_decomposition: Some(true),
        confirm: Some(false),
        ..ProjectGenesisRequest::default()
    };
    let Json(staged) = project_genesis::start(State(state.clone()), Json(genesis.clone())).await?;
    let genesis_packet = if staged["status"] == "staged" {
        let mut commit = genesis;
        commit.confirm = Some(true);
        let Json(ready) = project_genesis::commit(State(state), Json(commit)).await?;
        ready
    } else {
        staged
    };
    if !genesis_existed && root.join(".focusa/genesis").is_dir() {
        created.push(".focusa/genesis".into());
    }
    let status = if genesis_packet["status"] == "ready" {
        "ready"
    } else {
        "onboarding_required"
    };
    let receipt = json!({
        "schema":"focusa.project_bootstrap_receipt.v1", "status":status,
        "receipt_id":stable_receipt_id(&root,&req.idempotency_key), "idempotency_key":req.idempotency_key,
        "project_root":root, "created_by_this_transaction":created, "task_provider":task_provider,
        "genesis":genesis_packet, "remote_created":false, "stack_selected":false, "deployment_selected":false,
        "rollback":{"action":"POST /v1/project/bootstrap/repair repair_action=rollback confirm=true","scope":"created_by_this_transaction only"},
        "recorded_at":Utc::now().to_rfc3339(),
        "next_action":if status=="ready" {"continue from the active first Workpoint"} else {"complete the bounded Genesis next action"},
    });
    write_json_atomic(&receipt_path(&root), &receipt).map_err(|error| {
        reject(
            StatusCode::INTERNAL_SERVER_ERROR,
            "bootstrap_receipt_failed",
            error,
        )
    })?;
    drop(lock);
    let _ = fs::remove_file(lock_path);
    Ok(Json(receipt))
}

async fn repair(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProjectBootstrapRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.repair_action.as_deref() != Some("rollback") {
        return apply(State(state), Json(req)).await;
    }
    if req.confirm != Some(true) {
        return Err(reject(
            StatusCode::PRECONDITION_REQUIRED,
            "rollback_confirmation_required",
            "rollback requires confirm=true",
        ));
    }
    let root = validate_root(&req.project_root, false)?;
    let receipt = read_json(&receipt_path(&root)).ok_or_else(|| {
        reject(
            StatusCode::NOT_FOUND,
            "receipt_missing",
            "no bootstrap receipt to roll back",
        )
    })?;
    if receipt["idempotency_key"] != req.idempotency_key {
        return Err(reject(
            StatusCode::CONFLICT,
            "receipt_mismatch",
            "idempotency key does not own this bootstrap receipt",
        ));
    }
    let created = receipt["created_by_this_transaction"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for item in created.iter().rev().filter_map(Value::as_str) {
        let path = if item == "project_root" {
            root.clone()
        } else {
            root.join(item)
        };
        if item == "project_root" {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|error| {
                reject(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "rollback_failed",
                    error.to_string(),
                )
            })?;
        } else if path.exists() {
            fs::remove_file(path).map_err(|error| {
                reject(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "rollback_failed",
                    error.to_string(),
                )
            })?;
        }
    }
    Ok(Json(
        json!({"schema":"focusa.project_bootstrap_rollback.v1","status":"rolled_back","receipt_id":receipt["receipt_id"],"project_root":root,"preserved_adopted_state":true}),
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/project/bootstrap/preview", post(preview))
        .route("/v1/project/bootstrap/apply", post(apply))
        .route("/v1/project/bootstrap/status", get(status))
        .route("/v1/project/bootstrap/repair", post(repair))
}
