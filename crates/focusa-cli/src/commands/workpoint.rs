//! Spec88 Workpoint CLI parity commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub enum WorkpointCmd {
    /// Checkpoint the current typed workpoint before compaction/resume/overflow recovery.
    Checkpoint {
        /// Current mission/objective summary.
        #[arg(long)]
        mission: Option<String>,
        /// Next bounded action slice to resume.
        #[arg(long)]
        next_action: Option<String>,
        /// Beads/work item id.
        #[arg(long)]
        work_item: Option<String>,
        /// Pi/session id.
        #[arg(long)]
        session: Option<String>,
        /// Checkpoint reason, e.g. manual,operator_checkpoint,before_compact,context_overflow.
        #[arg(long, default_value = "manual")]
        reason: String,
        /// Action type, e.g. checkpoint_workpoint, patch_component_binding.
        #[arg(long)]
        action_type: Option<String>,
        /// Action target ref.
        #[arg(long)]
        target_ref: Option<String>,
        /// Do not auto-promote the canonical checkpoint.
        #[arg(long)]
        no_promote: bool,
        /// Mark packet non-canonical/degraded.
        #[arg(long)]
        degraded: bool,
        /// Optional idempotency key accepted for contract parity; currently informational.
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    /// Show the active Workpoint packet.
    Current,
    /// Render a WorkpointResumePacket for Pi continuation.
    Resume {
        /// Render mode: compact_prompt, full_json, operator_summary.
        #[arg(long, default_value = "compact_prompt")]
        mode: String,
    },
    /// Detect whether latest action drifted away from the active workpoint.
    DriftCheck {
        /// Latest action/summary to compare.
        #[arg(long)]
        latest_action: Option<String>,
        /// Expected action type; defaults to active workpoint action intent.
        #[arg(long)]
        expected_action_type: Option<String>,
        /// Emit WorkpointDriftDetected if drift is found.
        #[arg(long)]
        emit: bool,
    },
    /// Resolve candidate active object refs from active Workpoint plus optional hint.
    ResolveObject {
        #[arg(long)]
        hint: Option<String>,
    },
    /// Link an evidence ref/result to the active or specified Workpoint.
    EvidenceLink {
        #[arg(long)]
        workpoint_id: Option<String>,
        #[arg(long)]
        target_ref: String,
        #[arg(long)]
        result: String,
        #[arg(long)]
        evidence_ref: Option<String>,
        #[arg(long, default_value = "focusa-cli")]
        writer_id: String,
    },
}

fn reason_to_api(reason: &str) -> String {
    match reason {
        "session-start" | "session_start" => "session_start",
        "session-resume" | "session_resume" => "session_resume",
        "before-compact" | "before_compact" => "before_compact",
        "after-compact" | "after_compact" => "after_compact",
        "context-overflow" | "context_overflow" => "context_overflow",
        "model-switch" | "model_switch" => "model_switch",
        "fork" => "fork",
        "operator-checkpoint" | "operator_checkpoint" => "operator_checkpoint",
        "manual" => "manual",
        "unknown" => "unknown",
        _ => reason,
    }
    .to_string()
}

fn print_human_summary(resp: &Value, label: &str) {
    let status = resp.get("status").and_then(Value::as_str).unwrap_or("unknown");
    let canonical = resp
        .get("canonical")
        .and_then(Value::as_bool)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let workpoint_id = resp
        .get("workpoint_id")
        .and_then(Value::as_str)
        .unwrap_or("none");
    println!("workpoint {label}: status={status} id={workpoint_id} canonical={canonical}");
    if let Some(summary) = resp.get("rendered_summary").and_then(Value::as_str) {
        println!("  summary: {summary}");
    }
    if let Some(next) = resp.get("next_step_hint").and_then(Value::as_str) {
        println!("  next: {next}");
    }
    if let Some(workpoint) = resp.get("workpoint").or_else(|| resp.get("resume_packet")) {
        if let Some(next_slice) = workpoint.get("next_slice").and_then(Value::as_str) {
            println!("  next_slice: {next_slice}");
        }
        if let Some(action_type) = workpoint
            .pointer("/action_intent/action_type")
            .and_then(Value::as_str)
        {
            println!("  action: {action_type}");
        }
    }
}

pub async fn run(cmd: WorkpointCmd, json_output: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let (label, resp) = match cmd {
        WorkpointCmd::Checkpoint {
            mission,
            next_action,
            work_item,
            session,
            reason,
            action_type,
            target_ref,
            no_promote,
            degraded,
            idempotency_key,
        } => {
            let mut body = json!({
                "mission": mission,
                "next_slice": next_action,
                "work_item_id": work_item,
                "session_id": session,
                "checkpoint_reason": reason_to_api(&reason),
                "canonical": !degraded,
                "promote": !no_promote,
                "idempotency_key": idempotency_key,
            });
            if action_type.is_some() || target_ref.is_some() {
                body["action_intent"] = json!({
                    "action_type": action_type.unwrap_or_else(|| "checkpoint_workpoint".to_string()),
                    "target_ref": target_ref,
                    "verification_hooks": [],
                    "status": "ready",
                });
            }
            ("checkpoint", api.post("/v1/workpoint/checkpoint", &body).await?)
        }
        WorkpointCmd::Current => ("current", api.get("/v1/workpoint/current").await?),
        WorkpointCmd::Resume { mode } => (
            "resume",
            api.post("/v1/workpoint/resume", &json!({ "mode": mode })).await?,
        ),
        WorkpointCmd::DriftCheck {
            latest_action,
            expected_action_type,
            emit,
        } => (
            "drift-check",
            api.post(
                "/v1/workpoint/drift-check",
                &json!({
                    "latest_action": latest_action,
                    "expected_action_type": expected_action_type,
                    "emit": emit,
                }),
            )
            .await?,
        ),
        WorkpointCmd::ResolveObject { hint } => (
            "resolve-object",
            api.post("/v1/workpoint/active-object/resolve", &json!({ "hint": hint })).await?,
        ),
        WorkpointCmd::EvidenceLink { workpoint_id, target_ref, result, evidence_ref, writer_id } => (
            "evidence-link",
            api.post_with_headers(
                "/v1/workpoint/evidence/link",
                &json!({
                    "workpoint_id": workpoint_id,
                    "target_ref": target_ref,
                    "result": result,
                    "evidence_ref": evidence_ref,
                }),
                &[("x-focusa-writer-id", writer_id.as_str())],
            )
            .await?,
        ),
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&resp)?);
    } else {
        print_human_summary(&resp, label);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reason_aliases_match_api_snake_case() {
        assert_eq!(reason_to_api("before-compact"), "before_compact");
        assert_eq!(reason_to_api("context_overflow"), "context_overflow");
        assert_eq!(reason_to_api("operator-checkpoint"), "operator_checkpoint");
        assert_eq!(reason_to_api("nonsense"), "nonsense");
    }

    #[test]
    fn human_summary_reads_resume_packet() {
        let packet = json!({
            "status": "completed",
            "workpoint_id": "wp-1",
            "canonical": true,
            "resume_packet": {
                "next_slice": "Continue Phase 5",
                "action_intent": { "action_type": "resume_workpoint" }
            },
            "next_step_hint": "inject packet"
        });
        print_human_summary(&packet, "resume");
    }
}
