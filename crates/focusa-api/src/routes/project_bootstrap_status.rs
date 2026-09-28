//! Read-only bootstrap observation, with missing receipts treated as uncertain
//! whenever any project-local bootstrap artifact is already present.
use super::{
    project_bootstrap_safety as safety,
    project_bootstrap_support::{ProjectBootstrapStatusQuery, read_receipt, reject, validate_root},
};
use axum::{Json, extract::Query, http::StatusCode};
use serde_json::{Value, json};

pub(super) async fn status(
    Query(query): Query<ProjectBootstrapStatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let root = validate_root(&query.project_root, false)?;
    safety::validate_artifact_paths(&root)
        .map_err(|error| reject(StatusCode::CONFLICT, "bootstrap_path_conflict", error))?;
    let receipt = read_receipt(&root)?;
    let unexplained_artifacts = if receipt.is_none() {
        safety::unverified_artifacts(&root).map_err(|error| {
            reject(
                StatusCode::CONFLICT,
                "bootstrap_status_artifact_unreadable",
                error,
            )
        })?
    } else {
        Vec::new()
    };
    let effective_status = receipt
        .as_ref()
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .unwrap_or(if unexplained_artifacts.is_empty() {
            "not_started"
        } else {
            "unverified_existing_artifacts"
        });
    Ok(Json(json!({
        "schema": "focusa.project_bootstrap_status.v1",
        "status": effective_status,
        "unverified_artifact_paths": unexplained_artifacts,
        "project_root": root,
        "receipt": receipt,
        "live": {
            "marker": root.join(".focusa-project.json").is_file(),
            "git": root.join(".git").is_dir(),
            "docs": root.join("docs").is_dir(),
            "tasks": root.join(".beads").is_dir(),
            "genesis": root.join(".focusa/genesis/packet.json").is_file(),
        },
        "next_action": safety::status_next_action(effective_status),
    })))
}
