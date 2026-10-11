//! Read-only bootstrap observation, with missing receipts treated as uncertain
//! whenever any project-local bootstrap artifact is already present.
use super::{
    project_bootstrap_pre_root as pre_root, project_bootstrap_safety as safety,
    project_bootstrap_support::{
        ProjectBootstrapStatusQuery, read_json, read_receipt, reject, validate_root,
    },
};
use axum::{Json, extract::Query, http::StatusCode};
use serde_json::{Value, json};

/// A historical bootstrap receipt may record an old HLT impasse or
/// onboarding_required transaction. The current Genesis lifecycle is the
/// authoritative readiness signal: when the same scoped project's Genesis is
/// ready, status must recommend continuation, not re-bootstrap (#671).
/// Historical transaction truth remains preserved in the receipt.
fn is_genesis_ready(root: &std::path::Path) -> bool {
    let marker = read_json(&root.join(".focusa-project.json")).unwrap_or(Value::Null);
    let binding_ready = marker["genesis_binding"]["status"].as_str() == Some("ready");
    let packet_ready = read_json(&root.join(".focusa/genesis/packet.json"))
        .is_some_and(|packet| packet["status"].as_str() == Some("ready"));
    binding_ready && packet_ready
}

pub(super) async fn status(
    Query(query): Query<ProjectBootstrapStatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let root = validate_root(&query.project_root, true)?;
    safety::validate_artifact_paths(&root)
        .map_err(|error| reject(StatusCode::CONFLICT, "bootstrap_path_conflict", error))?;
    let receipt = read_receipt(&root)?;
    let reservation = pre_root::read(&root)?;
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
    let reservation_status = reservation
        .as_ref()
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str);
    // A historical bootstrap receipt may record an old HLT impasse or
    // onboarding_required transaction. The current Genesis lifecycle is the
    // authoritative readiness signal: when the same scoped project's Genesis is
    // ready, status must recommend continuation, not re-bootstrap (#671).
    // Historical transaction truth remains preserved in `receipt`.
    let genesis_ready = is_genesis_ready(&root);
    let effective_status = if genesis_ready {
        "ready"
    } else {
        receipt
            .as_ref()
            .and_then(|value| value.get("status"))
            .and_then(Value::as_str)
            .unwrap_or(match reservation_status {
                Some("reserved") => "root_creation_uncertain",
                Some("rolled_back") => "rolled_back",
                _ if unexplained_artifacts.is_empty() => "not_started",
                _ => "unverified_existing_artifacts",
            })
    };
    let next_action = if receipt.is_none() && reservation_status == Some("rolled_back") {
        "retain the rolled-back pre-root record; a new project creation requires explicit reconciliation"
    } else {
        safety::status_next_action(effective_status)
    };
    Ok(Json(json!({
        "schema": "focusa.project_bootstrap_status.v1",
        "status": effective_status,
        "genesis_ready": genesis_ready,
        "unverified_artifact_paths": unexplained_artifacts,
        "project_root": root,
        "receipt": receipt,
        "pre_root_journal": reservation,
        "live": {
            "marker": root.join(".focusa-project.json").is_file(),
            "git": root.join(".git").is_dir(),
            "docs": root.join("docs").is_dir(),
            "tasks": root.join(".beads").is_dir(),
            "genesis": root.join(".focusa/genesis/packet.json").is_file(),
        },
        "next_action": next_action,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_root(tag: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "focusa-bootstrap-status-{tag}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".focusa/genesis")).unwrap();
        fs::create_dir_all(root.join(".focusa/bootstrap")).unwrap();
        root
    }

    #[test]
    fn genesis_ready_true_when_binding_and_packet_ready() {
        let root = temp_root("ready");
        fs::write(
            root.join(".focusa-project.json"),
            r#"{"genesis_binding":{"status":"ready"}}"#,
        )
        .unwrap();
        fs::write(
            root.join(".focusa/genesis/packet.json"),
            r#"{"status":"ready"}"#,
        )
        .unwrap();
        assert!(is_genesis_ready(&root));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn genesis_ready_false_when_packet_missing_or_not_ready() {
        let root = temp_root("missing");
        fs::write(
            root.join(".focusa-project.json"),
            r#"{"genesis_binding":{"status":"ready"}}"#,
        )
        .unwrap();
        // No packet file at all.
        assert!(!is_genesis_ready(&root));
        // Packet present but not ready.
        fs::write(
            root.join(".focusa/genesis/packet.json"),
            r#"{"status":"hlt_impasse"}"#,
        )
        .unwrap();
        assert!(!is_genesis_ready(&root));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn genesis_ready_false_when_binding_not_ready() {
        let root = temp_root("binding");
        fs::write(
            root.join(".focusa-project.json"),
            r#"{"genesis_binding":{"status":"pending"}}"#,
        )
        .unwrap();
        fs::write(
            root.join(".focusa/genesis/packet.json"),
            r#"{"status":"ready"}"#,
        )
        .unwrap();
        assert!(!is_genesis_ready(&root));
        let _ = fs::remove_dir_all(&root);
    }
}
