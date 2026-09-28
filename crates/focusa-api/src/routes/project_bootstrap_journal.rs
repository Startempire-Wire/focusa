//! Durable fail-closed apply progress: never interpret an interrupted apply as not_started.
use super::{
    project_bootstrap_safety as safety,
    project_bootstrap_support::{
        ProjectBootstrapRequest, receipt_path, stable_receipt_id, write_json_atomic,
    },
};
use chrono::Utc;
use serde_json::{Value, json};
use std::path::Path;

/// Record the next potentially mutating stage before executing it. A crash
/// inside that stage is ambiguous: the receipt explicitly blocks replay and
/// automatic rollback until an owner reconciles any unproven artifact.
pub(super) fn record_stage(
    root: &Path,
    req: &ProjectBootstrapRequest,
    digest: &str,
    created: &[String],
    next_stage: &str,
) -> Result<Value, String> {
    let snapshot = safety::snapshot(root, created)?;
    let progress = json!({
        "schema": "focusa.project_bootstrap_receipt.v1", "status": "applying",
        "request_digest": digest, "idempotency_key": req.idempotency_key,
        "receipt_id": stable_receipt_id(root, &req.idempotency_key),
        "project_root": root, "project_id": req.project_id,
        "created_by_this_transaction": created,
        "created_artifact_snapshot": snapshot,
        "active_stage": next_stage,
        "recorded_at": Utc::now().to_rfc3339(),
        "next_action": "An interrupted apply requires owner review of its active stage before rollback or retry",
    });
    write_json_atomic(&receipt_path(root), &progress)?;
    Ok(progress)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn each_interrupted_stage_remains_uncertain_and_preserves_unproven_content() {
        for (stage, relative) in [
            ("marker_create", ".focusa-project.json"),
            ("settings_create", ".focusa/settings.json"),
            ("docs_create", "docs"),
            ("git_init", ".git"),
            ("task_provider", ".beads"),
            ("genesis", ".focusa/genesis"),
        ] {
            let fixture = tempfile::tempdir().unwrap();
            let root = fixture.path();
            let req = ProjectBootstrapRequest {
                idempotency_key: "stage-key".into(),
                ..ProjectBootstrapRequest::default()
            };
            record_stage(root, &req, "digest", &[], stage).unwrap();
            let artifact = root.join(relative);
            std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
            if relative == ".focusa-project.json" || relative.ends_with(".json") {
                std::fs::write(&artifact, b"unproven later content").unwrap();
            } else {
                std::fs::create_dir(&artifact).unwrap();
            }
            let receipt = safety::read_receipt(&receipt_path(root)).unwrap().unwrap();
            assert_eq!(receipt["active_stage"], stage);
            assert_eq!(receipt["status"], "applying");
            assert!(safety::rollback_plan(root, &receipt).is_err());
            assert!(
                artifact.exists(),
                "interrupted stage {stage} must not delete unproven content"
            );
        }
    }

    #[test]
    fn progress_is_durable_and_never_replays_success() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path();
        let request = ProjectBootstrapRequest {
            project_root: root.display().to_string(),
            project_id: "proof".into(),
            canonical_name: "Proof".into(),
            idempotency_key: "key-1".into(),
            ..ProjectBootstrapRequest::default()
        };
        let first = record_stage(root, &request, "digest-1", &[], "marker_create").unwrap();
        assert_eq!(first["status"], "applying");
        let receipt = safety::read_receipt(&receipt_path(root)).unwrap().unwrap();
        assert_eq!(receipt["active_stage"], "marker_create");
        assert!(safety::validate_apply_receipt(&receipt, root, "key-1", "digest-1").is_err());
        assert!(safety::rollback_plan(root, &receipt).is_err());
        std::fs::write(root.join(".focusa-project.json"), b"later ambiguous marker").unwrap();
        let after = safety::read_receipt(&receipt_path(root)).unwrap().unwrap();
        assert_eq!(after["active_stage"], "marker_create");
        assert_eq!(
            std::fs::read(root.join(".focusa-project.json")).unwrap(),
            b"later ambiguous marker"
        );
    }
}
