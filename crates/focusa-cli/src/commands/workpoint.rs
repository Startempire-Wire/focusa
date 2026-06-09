//! Spec88 Workpoint CLI parity commands.

use crate::api_client::ApiClient;
use crate::commands::scope::ensure_project_root_scope_safe;
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
        /// Safe project folder/container for canonical Workpoint authority.
        #[arg(long)]
        project_root: Option<String>,
        /// Stable logical workstream id for same-project continuity.
        #[arg(long)]
        continuity_id: Option<String>,
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
        /// Optional idempotency key for safe replay detection.
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    /// Show the active Workpoint packet.
    Current {
        /// Safe project folder/container for scoped lookup.
        #[arg(long)]
        project_root: Option<String>,
        /// Stable logical workstream id for scoped lookup.
        #[arg(long)]
        continuity_id: Option<String>,
    },
    /// Render a WorkpointResumePacket for Pi continuation.
    Resume {
        /// Render mode: compact_prompt, full_json, operator_summary.
        #[arg(long, default_value = "compact_prompt")]
        mode: String,
        /// Safe project folder/container for canonical resume.
        #[arg(long)]
        project_root: Option<String>,
        /// Stable logical workstream id for canonical resume.
        #[arg(long)]
        continuity_id: Option<String>,
        /// Print a paste-ready continuation prompt for non-Pi agents.
        #[arg(long)]
        copy_prompt: bool,
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

fn query_escape(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn current_path(project_root: Option<String>, continuity_id: Option<String>) -> String {
    let mut params = Vec::new();
    if let Some(root) = project_root.filter(|value| !value.trim().is_empty()) {
        params.push(format!("project_root={}", query_escape(&root)));
    }
    if let Some(continuity) = continuity_id.filter(|value| !value.trim().is_empty()) {
        params.push(format!("continuity_id={}", query_escape(&continuity)));
    }
    if params.is_empty() {
        "/v1/workpoint/current".to_string()
    } else {
        format!("/v1/workpoint/current?{}", params.join("&"))
    }
}

fn print_copy_prompt(resp: &Value) {
    println!("Paste this into your AI coding agent:\n");
    if let Some(summary) = resp.get("rendered_summary").and_then(Value::as_str) {
        println!("{summary}\n");
    }
    println!("You are continuing this project under Focusa Workpoint authority.");
    println!(
        "Use the Workpoint packet below as the continuation contract, not the transcript tail."
    );
    println!("Respect operator steering if a fresh instruction conflicts with this packet.\n");
    println!("```json");
    println!(
        "{}",
        serde_json::to_string_pretty(resp).unwrap_or_else(|_| resp.to_string())
    );
    println!("```");
}

fn print_human_summary(resp: &Value, label: &str) {
    let status = resp
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
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
    // Workpoint checkpoint/resume may enqueue reducer events and wait for read-model visibility;
    // keep CLI UX bounded but longer than hot read probes, especially under LowMem backpressure.
    let api = ApiClient::with_timeout_secs(8);
    let mut copy_prompt = false;
    let (label, resp) = match cmd {
        WorkpointCmd::Checkpoint {
            mission,
            next_action,
            work_item,
            session,
            project_root,
            continuity_id,
            reason,
            action_type,
            target_ref,
            no_promote,
            degraded,
            idempotency_key,
        } => {
            ensure_project_root_scope_safe(
                project_root.as_deref(),
                "workpoint checkpoint: project_root",
            )?;
            let mut body = json!({
                "mission": mission,
                "next_slice": next_action,
                "work_item_id": work_item,
                "session_id": session,
                "project_root": project_root,
                "continuity_id": continuity_id,
                "checkpoint_reason": reason_to_api(&reason),
                "canonical": !degraded,
                "promote": !no_promote,
                "idempotency_key": idempotency_key,
            });
            if action_type.is_some() || target_ref.is_some() {
                let target_ref_for_refs = target_ref.clone();
                body["action_intent"] = json!({
                    "action_type": action_type.unwrap_or_else(|| "checkpoint_workpoint".to_string()),
                    "target_ref": target_ref,
                    "verification_hooks": [],
                    "status": "ready",
                });
                if let Some(target) = target_ref_for_refs.filter(|target| !target.trim().is_empty())
                {
                    body["active_object_refs"] = json!([target]);
                }
            }
            (
                "checkpoint",
                api.post("/v1/workpoint/checkpoint", &body).await?,
            )
        }
        WorkpointCmd::Current {
            project_root,
            continuity_id,
        } => {
            ensure_project_root_scope_safe(
                project_root.as_deref(),
                "workpoint current: project_root",
            )?;
            (
                "current",
                api.get(&current_path(project_root, continuity_id)).await?,
            )
        }
        WorkpointCmd::Resume {
            mode,
            project_root,
            continuity_id,
            copy_prompt: should_copy_prompt,
        } => {
            ensure_project_root_scope_safe(
                project_root.as_deref(),
                "workpoint resume: project_root",
            )?;
            copy_prompt = should_copy_prompt;
            (
                "resume",
                api.post(
                    "/v1/workpoint/resume",
                    &json!({
                        "mode": if should_copy_prompt { "compact_prompt" } else { mode.as_str() },
                        "project_root": project_root,
                        "continuity_id": continuity_id,
                    }),
                )
                .await?,
            )
        }
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
            api.post(
                "/v1/workpoint/active-object/resolve",
                &json!({ "hint": hint }),
            )
            .await?,
        ),
        WorkpointCmd::EvidenceLink {
            workpoint_id,
            target_ref,
            result,
            evidence_ref,
            writer_id,
        } => (
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
    } else if copy_prompt {
        print_copy_prompt(&resp);
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
    fn current_path_encodes_scope_query() {
        assert_eq!(
            current_path(
                Some("/tmp/focusa-project".to_string()),
                Some("a b".to_string())
            ),
            "/v1/workpoint/current?project_root=%2Ftmp%2Ffocusa-project&continuity_id=a%20b"
        );
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
