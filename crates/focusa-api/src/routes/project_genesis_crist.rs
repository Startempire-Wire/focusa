//! Durable C.R.I.S.T. state transitions and receipt-backed validation.

use super::super::project_genesis_support::{stable_id, write_json_atomic};
use chrono::Utc;
use serde_json::{Value, json};
use std::path::Path;

fn allowed_transition(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("created", "project_scope_verified")
            | ("project_scope_verified", "context_collecting")
            | ("context_collecting", "context_ready")
            | ("context_ready", "role_drafting")
            | ("role_drafting", "role_pending_operator")
            | ("role_pending_operator", "role_approved")
            | ("role_approved", "interviewing")
            | ("interviewing", "interview_ready")
            | ("interview_ready", "spec_workbench_created")
            | ("spec_workbench_created", "spec_in_review")
            | ("spec_in_review", "spec_approved")
            | ("spec_approved", "task_plan_drafting")
            | ("task_plan_drafting", "task_plan_pending_operator")
            | ("task_plan_pending_operator", "tasks_materialized")
            | ("tasks_materialized", "first_workpoint_ready")
            | ("first_workpoint_ready", "operational")
    )
}

pub(super) fn initialize_crist_state(root: &Path, packet: &mut Value) -> Result<(), Value> {
    if !packet["transition_receipts"]
        .as_array()
        .is_none_or(Vec::is_empty)
    {
        return Ok(());
    }
    let target_stage = packet["crist_stage"]
        .as_str()
        .unwrap_or("project_scope_verified")
        .to_string();
    packet["crist_stage"] = json!("created");
    packet["revision"] = json!(0);
    packet["resolved_project_operating_profile"]["crist_state"] =
        json!({"stage": "created", "revision": 0, "status": "active"});
    record_crist_transition(
        root,
        packet,
        "project_scope_verified",
        "verify_project_scope",
    )?;
    if target_stage == "context_collecting" {
        record_crist_transition(
            root,
            packet,
            "context_collecting",
            "begin_context_collection",
        )?;
    }
    Ok(())
}

pub(super) fn record_crist_transition(
    root: &Path,
    packet: &mut Value,
    target_stage: &str,
    action: &str,
) -> Result<Value, Value> {
    if packet["transition_receipts"].as_array().is_none() || packet["receipts"].as_array().is_none()
    {
        return Err(json!({
            "schema": "focusa.crist.transition_receipt.v1",
            "outcome": "rejected",
            "reason_code": "invalid_genesis_state_shape",
            "target_stage": target_stage,
            "action": action,
        }));
    }
    let current_stage = packet["crist_stage"]
        .as_str()
        .unwrap_or("created")
        .to_string();
    let revision = packet["revision"].as_u64().unwrap_or(0);
    let attempt = packet["transition_receipts"]
        .as_array()
        .map_or(1, |receipts| receipts.len() + 1);
    let accepted = allowed_transition(&current_stage, target_stage);
    let receipt_key = format!(
        "{}:{revision}:{attempt}:{current_stage}:{target_stage}:{action}",
        packet["idempotency_key"].as_str().unwrap_or_default()
    );
    let receipt_id = stable_id("crist-transition", root, &receipt_key);
    let receipt = json!({
        "schema": "focusa.crist.transition_receipt.v1",
        "receipt_id": receipt_id.clone(),
        "continuity_id": packet["continuity_id"],
        "owner_ref": packet["ownership"]["owner_ref"],
        "from_stage": current_stage,
        "target_stage": target_stage,
        "action": action,
        "attempt": attempt,
        "outcome": if accepted { "accepted" } else { "rejected" },
        "reason_code": if accepted { "allowed_transition" } else { "invalid_crist_transition" },
        "state_revision_before": revision,
        "state_revision_after": if accepted { revision + 1 } else { revision },
        "recorded_at": Utc::now().to_rfc3339(),
    });
    let receipt_path = root
        .join(".focusa/project-genesis/transition-receipts")
        .join(format!("{receipt_id}.json"));
    if let Err(error) = write_json_atomic(&receipt_path, &receipt) {
        return Err(json!({
            "schema": "focusa.crist.transition_receipt.v1",
            "receipt_id": receipt_id,
            "continuity_id": packet["continuity_id"],
            "from_stage": current_stage,
            "target_stage": target_stage,
            "action": action,
            "attempt": attempt,
            "outcome": "rejected",
            "reason_code": "transition_receipt_persist_failed",
            "persistence_error": error,
            "state_revision_before": revision,
            "state_revision_after": revision,
            "recorded_at": Utc::now().to_rfc3339(),
        }));
    }
    packet["transition_receipts"]
        .as_array_mut()
        .expect("validated transition_receipts array")
        .push(receipt.clone());
    packet["receipts"]
        .as_array_mut()
        .expect("validated receipts array")
        .push(json!(receipt_id));
    if accepted {
        packet["crist_stage"] = json!(target_stage);
        packet["revision"] = json!(revision + 1);
        packet["updated_at"] = json!(Utc::now().to_rfc3339());
        packet["resolved_project_operating_profile"]["crist_state"] = json!({
            "stage": target_stage,
            "revision": revision + 1,
            "status": "active",
        });
        Ok(receipt)
    } else {
        Err(receipt)
    }
}
