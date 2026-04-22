//! Continuous work loop control/status routes.

use crate::routes::permissions::{forbid, permission_context};
use crate::server::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::types::{
    Action, BlockerClass, FocusaEvent, ProjectRunId, SpecLinkedTaskPacket, TaskClass,
    WorkLoopPolicy, WorkLoopPolicyOverrides, WorkLoopPreset, WorkLoopStatus,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

const WRITER_HEADER: &str = "x-focusa-writer-id";
const APPROVAL_HEADER: &str = "x-focusa-approval";
const PROJECT_ROOT: &str = "/home/wirebot/focusa";
const PI_RPC_BIN: &str = "/opt/cpanel/ea-nodejs20/bin/pi";

#[derive(Debug, Deserialize)]
pub struct EnableWorkLoopRequest {
    pub project_run_id: Option<ProjectRunId>,
    pub preset: Option<WorkLoopPreset>,
    pub policy_overrides: Option<WorkLoopPolicyOverrides>,
    pub root_work_item_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReasonRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckpointRequest {
    pub checkpoint_id: Option<focusa_core::types::CheckpointId>,
    pub summary: String,
}

#[derive(Debug, Deserialize)]
pub struct SelectNextRequest {
    pub parent_work_item_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DelegationRequest {
    pub delegate_id: String,
    pub scope: String,
    pub amendment_summary: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PauseFlagsRequest {
    pub destructive_confirmation_required: bool,
    pub governance_decision_pending: bool,
    pub operator_override_active: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SessionAttachRequest {
    pub adapter: String,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DecisionContextRequest {
    pub current_ask: Option<String>,
    pub ask_kind: Option<String>,
    pub scope_kind: Option<String>,
    pub carryover_policy: Option<String>,
    pub excluded_context_reason: Option<String>,
    pub excluded_context_labels: Option<Vec<String>>,
    pub source_turn_id: Option<String>,
    pub operator_steering_detected: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PiDriverStartRequest {
    pub cwd: Option<String>,
    pub models: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PiDriverPromptRequest {
    pub message: String,
    pub streaming_behavior: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransportEventRequest {
    pub sequence: u64,
    pub kind: String,
    pub session_id: Option<String>,
    pub turn_id: Option<String>,
    pub summary: Option<String>,
}

fn bad_request(message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": message.into() })),
    )
}

fn conflict(
    message: impl Into<String>,
    active_writer: Option<String>,
) -> (StatusCode, Json<Value>) {
    (
        StatusCode::CONFLICT,
        Json(json!({
            "error": message.into(),
            "active_writer": active_writer,
        })),
    )
}

fn writer_id_from_headers(headers: &HeaderMap) -> Result<String, (StatusCode, Json<Value>)> {
    headers
        .get(WRITER_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| bad_request(format!("missing required header: {WRITER_HEADER}")))
}

fn require_approval(headers: &HeaderMap, reason: &str) -> Result<(), (StatusCode, Json<Value>)> {
    let approved = headers
        .get(APPROVAL_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .map(|v| matches!(v, "true" | "1" | "approved"))
        .unwrap_or(false);
    if approved {
        Ok(())
    } else {
        Err((
            StatusCode::PRECONDITION_REQUIRED,
            Json(json!({
                "error": "explicit approval required",
                "reason": reason,
                "required_header": APPROVAL_HEADER,
            })),
        ))
    }
}

async fn ensure_writer_claim(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<String, (StatusCode, Json<Value>)> {
    let writer_id = writer_id_from_headers(headers)?;
    let mut guard = state.active_writer.write().await;
    match guard.as_deref() {
        Some(existing) if existing != writer_id => Err(conflict(
            "continuous work loop already claimed by another writer",
            guard.clone(),
        )),
        Some(_) => Ok(writer_id),
        None => {
            *guard = Some(writer_id.clone());
            Ok(writer_id)
        }
    }
}

async fn release_writer_claim(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<Option<String>, (StatusCode, Json<Value>)> {
    let writer_id = writer_id_from_headers(headers)?;
    let mut guard = state.active_writer.write().await;
    match guard.as_deref() {
        Some(existing) if existing != writer_id => Err(conflict(
            "continuous work loop claimed by another writer",
            guard.clone(),
        )),
        Some(_) => Ok(guard.take()),
        None => Ok(None),
    }
}

async fn ensure_claimed_writer_matches_for_context(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<Option<String>, (StatusCode, Json<Value>)> {
    let active_writer = state.active_writer.read().await.clone();
    let Some(active_writer) = active_writer else {
        return Ok(None);
    };

    let writer_id = writer_id_from_headers(headers)?;
    if writer_id != active_writer {
        return Err(conflict(
            "continuous work context write rejected: active writer claim belongs to another writer",
            Some(active_writer),
        ));
    }

    Ok(Some(writer_id))
}

async fn worktree_status_snapshot() -> Value {
    let top = match Command::new("git")
        .args([
            "-c",
            "safe.directory=/home/wirebot/focusa",
            "rev-parse",
            "--show-toplevel",
        ])
        .current_dir(PROJECT_ROOT)
        .output()
        .await
    {
        Ok(top) if top.status.success() => top,
        Ok(top) => {
            return json!({
                "git_available": true,
                "in_worktree": false,
                "clean": false,
                "repo_root_hint": PROJECT_ROOT,
                "error": String::from_utf8_lossy(&top.stderr).trim().to_string(),
            });
        }
        Err(e) => {
            return json!({
                "git_available": false,
                "in_worktree": false,
                "clean": false,
                "error": e.to_string(),
            });
        }
    };

    let repo_root = String::from_utf8_lossy(&top.stdout).trim().to_string();
    let status = match Command::new("git")
        .args([
            "-c",
            "safe.directory=/home/wirebot/focusa",
            "status",
            "--porcelain",
        ])
        .current_dir(&repo_root)
        .output()
        .await
    {
        Ok(status) if status.status.success() => status,
        Ok(_) => {
            return json!({
                "git_available": true,
                "in_worktree": true,
                "clean": false,
                "repo_root": repo_root,
                "error": "git status unsuccessful",
            });
        }
        Err(e) => {
            return json!({
                "git_available": true,
                "in_worktree": true,
                "clean": false,
                "repo_root": repo_root,
                "error": e.to_string(),
            });
        }
    };

    let dirty = String::from_utf8_lossy(&status.stdout)
        .lines()
        .take(10)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let diff_stat = Command::new("git")
        .args([
            "-c",
            "safe.directory=/home/wirebot/focusa",
            "diff",
            "--stat",
        ])
        .current_dir(&repo_root)
        .output()
        .await
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());
    json!({
        "git_available": true,
        "in_worktree": true,
        "clean": dirty.is_empty(),
        "repo_root": repo_root,
        "sample_changes": dirty,
        "diff_stat": diff_stat,
        "forbidden_without_explicit_approval": ["git reset --hard", "git clean", "git restore"],
    })
}

async fn alternate_ready_work_snapshot(
    current_task: Option<&focusa_core::types::SpecLinkedTaskPacket>,
) -> Value {
    let Some(task) = current_task else {
        return json!({ "exists": false });
    };
    let Some(parent_work_item_id) = task.dependencies.first() else {
        return json!({ "exists": false });
    };

    let output = match Command::new("bd")
        .args(["show", parent_work_item_id, "--json"])
        .current_dir(PROJECT_ROOT)
        .output()
        .await
    {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            return json!({
                "exists": false,
                "error": String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }
        Err(e) => return json!({ "exists": false, "error": e.to_string() }),
    };

    let payload: Vec<Value> = match serde_json::from_slice(&output.stdout) {
        Ok(payload) => payload,
        Err(e) => return json!({ "exists": false, "error": e.to_string() }),
    };
    let Some(parent) = payload.first() else {
        return json!({ "exists": false });
    };
    let Some(dependents) = parent.get("dependents").and_then(Value::as_array) else {
        return json!({ "exists": false });
    };
    let candidate = dependents.iter().find(|dep| {
        dep.get("id").and_then(Value::as_str) != Some(task.work_item_id.as_str())
            && matches!(
                dep.get("status").and_then(Value::as_str),
                Some("open") | Some("in_progress")
            )
    });

    match candidate {
        Some(dep) => json!({
            "exists": true,
            "work_item_id": dep.get("id").and_then(Value::as_str),
            "title": dep.get("title").and_then(Value::as_str),
        }),
        None => json!({ "exists": false }),
    }
}

fn build_blocker_package(
    wl: &focusa_core::types::WorkLoopState,
    alternate_ready_work: Value,
) -> Option<Value> {
    let blocker_class = wl.last_blocker_class?;
    let current_task = wl.current_task.as_ref();
    let linked_spec_requirement =
        current_task.and_then(|task| task.linked_spec_refs.first().cloned());
    let mut recovery_attempts = vec!["self-recovery on same task".to_string()];
    if wl.consecutive_failures_for_task_class > 0 {
        recovery_attempts.push(format!(
            "repeated recovery attempts: {}",
            wl.consecutive_failures_for_task_class
        ));
    }
    let mut fallback_attempts = Vec::new();
    if let Some(worker) = wl.active_worker.as_ref() {
        fallback_attempts.push(format!("worker route: {}", worker.worker_id));
        if !worker.fallback_available {
            fallback_attempts.push("fallback worker already selected".to_string());
        }
    }

    let retries_remaining = wl
        .policy
        .max_consecutive_failures
        .saturating_sub(wl.consecutive_failures_for_task_class);
    let self_recovery_allowed = retries_remaining > 0
        && !wl.pause_flags.operator_override_active
        && !wl.pause_flags.destructive_confirmation_required
        && !wl.pause_flags.governance_decision_pending
        && !matches!(
            blocker_class,
            focusa_core::types::BlockerClass::Governance
                | focusa_core::types::BlockerClass::Permission
        );

    let (exact_operator_decision_needed, recommended_next_action) = if self_recovery_allowed {
        (
            "no immediate operator decision required unless retry budget is exhausted".to_string(),
            format!(
                "retry self-recovery on the blocked task (remaining retry budget: {retries_remaining})"
            ),
        )
    } else if wl.pause_flags.operator_override_active {
        (
            "confirm override intent and choose whether to resume, pause longer, or stop"
                .to_string(),
            "honor operator override before any further autonomous step".to_string(),
        )
    } else if wl.pause_flags.destructive_confirmation_required {
        (
            "approve or reject the destructive/high-risk action".to_string(),
            "provide explicit approval or redirect to a safer path".to_string(),
        )
    } else if wl.pause_flags.governance_decision_pending
        || blocker_class == focusa_core::types::BlockerClass::Governance
    {
        (
            "resolve the governance-sensitive decision or amend policy/spec".to_string(),
            "choose the governing outcome, then resume with updated authority".to_string(),
        )
    } else if alternate_ready_work.get("exists").and_then(Value::as_bool) == Some(true) {
        (
            "decide whether to defer the blocked task and switch to alternate ready work"
                .to_string(),
            "defer the blocked task and continue with the alternate ready item".to_string(),
        )
    } else {
        (
            "review blocker package because no valid ready work remains".to_string(),
            "escalate to the operator because retries and alternate ready work are exhausted"
                .to_string(),
        )
    };

    Some(json!({
        "blocker_class": blocker_class,
        "affected_work_item_id": current_task.map(|task| task.work_item_id.clone()),
        "linked_spec_requirement": linked_spec_requirement,
        "recovery_attempts_made": recovery_attempts,
        "fallback_attempts_made": fallback_attempts,
        "alternate_ready_work": alternate_ready_work,
        "exact_operator_decision_needed": exact_operator_decision_needed,
        "recommended_next_action": recommended_next_action,
    }))
}

fn continuation_boundary_reason(wl: &focusa_core::types::WorkLoopState) -> Option<&'static str> {
    if wl.decision_context.operator_steering_detected {
        return Some("operator steering detected");
    }
    if wl.pause_flags.governance_decision_pending {
        return Some("governance decision pending");
    }
    None
}

fn transport_health_for_status(wl: &focusa_core::types::WorkLoopState) -> Value {
    json!({
        "status": if wl.status == focusa_core::types::WorkLoopStatus::TransportDegraded {
            "degraded"
        } else {
            "healthy"
        },
        "last_reason": wl.last_blocker_reason,
    })
}

fn execution_environment_for_status(
    transport_session_state: Option<&str>,
    worktree: &Value,
) -> Value {
    let git_available = worktree
        .get("git_available")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let in_worktree = worktree
        .get("in_worktree")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let worktree_clean = worktree
        .get("clean")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let transport_attached = matches!(
        transport_session_state,
        Some(
            "attached"
                | "running"
                | "turn_active"
                | "streaming"
                | "turn_completed"
                | "agent_completed"
        )
    );

    let affordance_status = if git_available && in_worktree && transport_attached {
        "available"
    } else {
        "blocked"
    };
    let affordance_reason = if affordance_status == "available" {
        Some("Git worktree and transport session are available for non-destructive code-edit execution".to_string())
    } else {
        Some(format!(
            "Missing execution prerequisites: git_available={git_available}, in_worktree={in_worktree}, transport_attached={transport_attached}"
        ))
    };

    json!({
        "context_kind": if in_worktree { "local_dev" } else { "constrained_runtime" },
        "facts": [
            {
                "id": "fact_git_available",
                "fact_key": "git_available",
                "fact_value": git_available,
                "source": "worktree_status_snapshot"
            },
            {
                "id": "fact_in_worktree",
                "fact_key": "in_worktree",
                "fact_value": in_worktree,
                "source": "worktree_status_snapshot"
            },
            {
                "id": "fact_worktree_clean",
                "fact_key": "worktree_clean",
                "fact_value": worktree_clean,
                "source": "worktree_status_snapshot"
            },
            {
                "id": "fact_transport_attached",
                "fact_key": "transport_session_attached",
                "fact_value": transport_attached,
                "source": "work_loop.transport_session_state"
            }
        ],
        "affordances": [
            {
                "id": "affordance_safe_local_code_edit",
                "affordance_kind": "safe_local_edit_available",
                "status": affordance_status,
                "recommended": affordance_status == "available" && worktree_clean,
                "reason": affordance_reason,
                "required_fact_ids": [
                    "fact_git_available",
                    "fact_in_worktree",
                    "fact_transport_attached"
                ]
            }
        ]
    })
}

fn extract_assistant_text(message: &Value) -> Option<String> {
    if let Some(text) = message.as_str() {
        return Some(text.to_string());
    }
    if let Some(text) = message.get("content").and_then(Value::as_str) {
        return Some(text.to_string());
    }
    message
        .get("content")
        .and_then(Value::as_array)
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| {
                    part.get("text")
                        .and_then(Value::as_str)
                        .map(|s| s.to_string())
                        .or_else(|| {
                            part.get("content")
                                .and_then(Value::as_str)
                                .map(|s| s.to_string())
                        })
                })
                .collect::<Vec<_>>()
                .join("")
        })
        .filter(|s| !s.is_empty())
}

fn render_continuous_turn_prompt(
    task: &SpecLinkedTaskPacket,
    mission: Option<String>,
    focus: Option<String>,
    last_checkpoint: Option<String>,
) -> String {
    let acceptance = if task.acceptance_criteria.is_empty() {
        "- satisfy the authoritative spec and verification requirements".to_string()
    } else {
        task.acceptance_criteria
            .iter()
            .map(|item| format!("- {}", item))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let scope = if task.allowed_scope.is_empty() {
        "(inherit current mission scope)".to_string()
    } else {
        task.allowed_scope.join(", ")
    };
    let refs = if task.linked_spec_refs.is_empty() {
        "(none)".to_string()
    } else {
        task.linked_spec_refs.join(", ")
    };
    format!(
        "Continuous work mode.\nWork item: {id} — {title}\nMission: {mission}\nFocus: {focus}\nAllowed scope: {scope}\nLinked specs: {refs}\nAcceptance criteria:\n{acceptance}\nLast checkpoint: {checkpoint}\nExecute the next concrete step only within scope. Verify before claiming completion. If blocked, say why explicitly.",
        id = task.work_item_id,
        title = task.title,
        mission = mission.unwrap_or_else(|| "(none)".to_string()),
        focus = focus.unwrap_or_else(|| "(none)".to_string()),
        checkpoint = last_checkpoint.unwrap_or_else(|| "(none)".to_string()),
    )
}

async fn dispatch_pi_prompt(
    state: &Arc<AppState>,
    message: String,
) -> Result<(), (StatusCode, Json<Value>)> {
    let mut guard = state.pi_rpc_session.lock().await;
    let Some(session) = guard.as_mut() else {
        return Err(bad_request("pi rpc driver not active"));
    };
    let msg =
        json!({"id": format!("prompt-{}", Uuid::now_v7()), "type":"prompt", "message": message})
            .to_string()
            + "\n";
    session
        .stdin
        .write_all(msg.as_bytes())
        .await
        .map_err(|e| bad_request(format!("failed writing prompt: {e}")))?;
    Ok(())
}

async fn defer_work_item_for_alternate_switch(work_item_id: &str, reason: &str) {
    let note = format!(
        "Continuous loop deferred for alternate-ready switch: {}",
        reason.chars().take(180).collect::<String>()
    );
    let _ = Command::new("bd")
        .args([
            "update",
            work_item_id,
            "--defer",
            "+1d",
            "--append-notes",
            &note,
        ])
        .current_dir(PROJECT_ROOT)
        .output()
        .await;
}

async fn maybe_auto_advance_from_blocked(
    state: &Arc<AppState>,
    reason: &str,
) -> Result<bool, (StatusCode, Json<Value>)> {
    let (enabled, status, current_task, boundary_reason) = {
        let focusa = state.focusa.read().await;
        (
            focusa.work_loop.enabled,
            focusa.work_loop.status,
            focusa.work_loop.current_task.clone(),
            continuation_boundary_reason(&focusa.work_loop),
        )
    };

    let blocked = status == WorkLoopStatus::Blocked;
    if !enabled || !blocked || boundary_reason.is_some() {
        return Ok(false);
    }

    let Some(task) = current_task else {
        if maybe_select_global_ready_work_item(state).await? {
            let _ = state
                .command_tx
                .send(Action::CheckpointContinuousLoop {
                    checkpoint_id: Uuid::now_v7(),
                    summary: format!(
                        "auto-advanced from blocked state without bound task ({})",
                        reason.chars().take(120).collect::<String>()
                    ),
                })
                .await;
            sleep(Duration::from_millis(120)).await;
            return Ok(true);
        }
        return Ok(false);
    };

    if blocked {
        defer_work_item_for_alternate_switch(&task.work_item_id, reason).await;
    }

    let parent_work_item_id = task
        .tranche_id
        .clone()
        .unwrap_or_else(|| task.work_item_id.clone());

    state
        .command_tx
        .send(Action::SelectNextContinuousSubtask {
            parent_work_item_id,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    let _ = state
        .command_tx
        .send(Action::CheckpointContinuousLoop {
            checkpoint_id: Uuid::now_v7(),
            summary: format!(
                "auto-advanced from blocked task {} ({})",
                task.work_item_id,
                reason.chars().take(120).collect::<String>()
            ),
        })
        .await;

    sleep(Duration::from_millis(120)).await;
    Ok(true)
}

fn item_text(item: &Value, key: &str) -> String {
    item.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn is_ontology_work_item(item: &Value) -> bool {
    let haystack = format!(
        "{} {} {}",
        item_text(item, "title"),
        item_text(item, "description"),
        item_text(item, "notes")
    )
    .to_ascii_lowercase();
    haystack.contains("ontology")
}

fn extract_work_item_id_and_title(item: &Value) -> Option<(String, String)> {
    let id = item.get("id").and_then(Value::as_str)?;
    if id.trim().is_empty() {
        return None;
    }
    let title = item
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("untitled work item")
        .to_string();
    Some((id.to_string(), title))
}

async fn maybe_select_global_ready_work_item(
    state: &Arc<AppState>,
) -> Result<bool, (StatusCode, Json<Value>)> {
    let boundary_reason = {
        let focusa = state.focusa.read().await;
        continuation_boundary_reason(&focusa.work_loop)
    };
    if boundary_reason.is_some() {
        return Ok(false);
    }

    let all_items_output = Command::new("bd")
        .args(["list", "--all", "--limit", "0", "--json"])
        .current_dir(PROJECT_ROOT)
        .output()
        .await
        .map_err(|e| bad_request(format!("failed to run bd list --all --json: {e}")))?;

    let selected_from_ontology = if all_items_output.status.success() {
        let all_items = serde_json::from_slice::<Vec<Value>>(&all_items_output.stdout)
            .map_err(|e| bad_request(format!("failed to parse bd list json: {e}")))?;

        ["in_progress", "open"].iter().find_map(|target_status| {
            all_items.iter().find_map(|item| {
                let status_matches = item
                    .get("status")
                    .and_then(Value::as_str)
                    .map(|s| s == *target_status)
                    .unwrap_or(false);
                if !status_matches || !is_ontology_work_item(item) {
                    return None;
                }
                extract_work_item_id_and_title(item)
            })
        })
    } else {
        None
    };

    let selected = if let Some(priority_item) = selected_from_ontology {
        Some(priority_item)
    } else {
        let output = Command::new("bd")
            .args(["ready", "--json"])
            .current_dir(PROJECT_ROOT)
            .output()
            .await
            .map_err(|e| bad_request(format!("failed to run bd ready --json: {e}")))?;
        if !output.status.success() {
            return Ok(false);
        }

        let ready_items = serde_json::from_slice::<Vec<Value>>(&output.stdout)
            .map_err(|e| bad_request(format!("failed to parse bd ready json: {e}")))?;
        ready_items.iter().find_map(extract_work_item_id_and_title)
    };

    let Some((work_item_id, title)) = selected else {
        return Ok(false);
    };

    let task_run_id = {
        let focusa = state.focusa.read().await;
        focusa.work_loop.run.task_run_id
    };

    state
        .command_tx
        .send(Action::SetContinuousWorkItem {
            task_run_id,
            packet: SpecLinkedTaskPacket {
                work_item_id,
                title: title.clone(),
                task_class: focusa_core::types::TaskClass::Unknown,
                linked_spec_refs: vec![
                    "docs/79-focusa-governed-continuous-work-loop.md".to_string(),
                ],
                acceptance_criteria: vec![],
                required_verification_tier: Some("task-class".to_string()),
                allowed_scope: vec![],
                dependencies: vec![],
                tranche_id: None,
                blocker_class: None,
                checkpoint_summary: None,
            },
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    sleep(Duration::from_millis(120)).await;
    Ok(true)
}

pub async fn maybe_dispatch_continuous_turn_prompt(
    state: &Arc<AppState>,
    reason: &str,
) -> Result<bool, (StatusCode, Json<Value>)> {
    let _ = maybe_auto_advance_from_blocked(state, reason).await?;

    let (
        enabled,
        status,
        task_run_id,
        current_task,
        mission,
        focus,
        last_checkpoint_id,
        last_turn_requested_at,
        status_heartbeat_ms,
        transport_session_state,
        boundary_reason,
    ) = {
        let focusa = state.focusa.read().await;
        let active_frame = focusa
            .focus_stack
            .active_id
            .and_then(|aid| focusa.focus_stack.frames.iter().find(|f| f.id == aid));
        (
            focusa.work_loop.enabled,
            focusa.work_loop.status,
            focusa.work_loop.run.task_run_id,
            focusa.work_loop.current_task.clone(),
            active_frame.map(|f| f.focus_state.intent.clone()),
            active_frame.map(|f| f.focus_state.current_state.clone()),
            focusa
                .work_loop
                .run
                .last_checkpoint_id
                .map(|v| v.to_string()),
            focusa.work_loop.last_turn_requested_at,
            focusa.work_loop.policy.status_heartbeat_ms,
            focusa.work_loop.transport_session_state.clone(),
            continuation_boundary_reason(&focusa.work_loop),
        )
    };

    if !enabled
        || !matches!(
            status,
            WorkLoopStatus::SelectingReadyWork
                | WorkLoopStatus::Idle
                | WorkLoopStatus::AwaitingHarnessTurn
                | WorkLoopStatus::AdvancingTask
                | WorkLoopStatus::EvaluatingOutcome
        )
    {
        return Ok(false);
    }
    if boundary_reason.is_some() {
        return Ok(false);
    }

    if current_task.is_none() {
        if maybe_select_global_ready_work_item(state).await? {
            let refreshed_task = {
                let focusa = state.focusa.read().await;
                focusa.work_loop.current_task.clone()
            };
            if let Some(task) = refreshed_task {
                state
                    .command_tx
                    .send(Action::RequestNextContinuousTurn {
                        task_run_id,
                        work_item_id: Some(task.work_item_id.clone()),
                        reason: "re-selected work after unassigned turn state".to_string(),
                    })
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "error": format!("dispatch failed: {e}") })),
                        )
                    })?;

                let prompt =
                    render_continuous_turn_prompt(&task, mission, focus, last_checkpoint_id);
                dispatch_pi_prompt(state, prompt).await?;
                return Ok(true);
            }
        }

        if status != WorkLoopStatus::Blocked {
            let _ = state
                .command_tx
                .send(Action::EmitEvent {
                    event: FocusaEvent::ContinuousTurnBlocked {
                        blocker_class: BlockerClass::SpecGap,
                        reason: "no ready work available for autonomous continuation".to_string(),
                        work_item_id: None,
                    },
                })
                .await;
        }

        return Ok(false);
    }

    if let Some(last_turn_at) = last_turn_requested_at {
        let since_last_turn_ms = (Utc::now() - last_turn_at).num_milliseconds().max(0) as u64;
        if status == WorkLoopStatus::AwaitingHarnessTurn {
            let reprompt_stale_ms = status_heartbeat_ms.saturating_mul(3).max(1_500);
            if since_last_turn_ms < reprompt_stale_ms {
                return Ok(false);
            }
        } else if since_last_turn_ms < status_heartbeat_ms {
            return Ok(false);
        }
    }
    let Some(task) = current_task else {
        return Ok(false);
    };

    let task_requires_local_edit_affordance = matches!(
        task.task_class,
        TaskClass::Code | TaskClass::Refactor | TaskClass::Integration | TaskClass::Architecture
    );
    if task_requires_local_edit_affordance {
        let worktree = worktree_status_snapshot().await;
        let execution_environment =
            execution_environment_for_status(transport_session_state.as_deref(), &worktree);
        let safe_local_edit_affordance = execution_environment
            .get("affordances")
            .and_then(Value::as_array)
            .and_then(|items| {
                items.iter().find(|item| {
                    item.get("id").and_then(Value::as_str)
                        == Some("affordance_safe_local_code_edit")
                })
            });
        let affordance_status = safe_local_edit_affordance
            .and_then(|item| item.get("status"))
            .and_then(Value::as_str)
            .unwrap_or("blocked");
        if affordance_status != "available" {
            let affordance_reason = safe_local_edit_affordance
                .and_then(|item| item.get("reason"))
                .and_then(Value::as_str)
                .unwrap_or("safe_local_edit_available is blocked in current execution environment")
                .to_string();
            state
                .command_tx
                .send(Action::PauseContinuousWork {
                    reason: format!(
                        "execution affordance blocked before dispatch: safe_local_edit_available; {affordance_reason}"
                    ),
                })
                .await
                .map_err(|e| (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("dispatch failed: {e}") })),
                ))?;
            return Ok(false);
        }
    }

    state
        .command_tx
        .send(Action::RequestNextContinuousTurn {
            task_run_id,
            work_item_id: Some(task.work_item_id.clone()),
            reason: reason.to_string(),
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    let prompt = render_continuous_turn_prompt(&task, mission, focus, last_checkpoint_id);
    dispatch_pi_prompt(state, prompt).await?;
    Ok(true)
}

fn budget_remaining_for_status(wl: &focusa_core::types::WorkLoopState) -> Value {
    let policy = &wl.policy;
    let elapsed_ms = wl
        .enabled_at
        .map(|ts| (Utc::now() - ts).num_milliseconds().max(0) as u64)
        .unwrap_or(0);
    json!({
        "max_turns": policy.max_turns,
        "max_wall_clock_ms": policy.max_wall_clock_ms,
        "max_retries": policy.max_retries,
        "max_consecutive_failures": policy.max_consecutive_failures,
        "max_consecutive_low_productivity_turns": policy.max_consecutive_low_productivity_turns,
        "max_same_subproblem_retries": policy.max_same_subproblem_retries,
        "status_heartbeat_ms": policy.status_heartbeat_ms,
        "turn_count": wl.turn_count,
        "elapsed_wall_clock_ms": elapsed_ms,
        "cooldown_ms": policy.cooldown_ms,
        "last_turn_requested_at": wl.last_turn_requested_at,
        "remaining_turn_budget": policy.max_turns.map(|max| max.saturating_sub(wl.turn_count)),
        "remaining_wall_clock_ms": policy.max_wall_clock_ms.map(|max| max.saturating_sub(elapsed_ms)),
        "remaining_failure_budget": policy
            .max_consecutive_failures
            .saturating_sub(wl.consecutive_failures_for_task_class),
        "remaining_low_productivity_budget": policy
            .max_consecutive_low_productivity_turns
            .saturating_sub(wl.consecutive_low_productivity_turns),
        "remaining_same_subproblem_budget": policy
            .max_same_subproblem_retries
            .saturating_sub(wl.consecutive_same_work_item_retries),
    })
}

fn next_work_risk_class_for_status(wl: &focusa_core::types::WorkLoopState) -> &'static str {
    let Some(task) = wl.current_task.as_ref() else {
        return "none";
    };
    let title = task.title.to_ascii_lowercase();
    if task
        .allowed_scope
        .iter()
        .any(|scope| scope.to_ascii_lowercase().contains("governance"))
        || matches!(
            wl.last_blocker_class,
            Some(
                focusa_core::types::BlockerClass::Governance
                    | focusa_core::types::BlockerClass::Permission
            )
        )
        || [
            "delete",
            "drop",
            "remove",
            "rename",
            "migrate",
            "rewrite",
            "destructive",
            "governance",
        ]
        .iter()
        .any(|needle| title.contains(needle))
    {
        "high"
    } else if matches!(
        task.task_class,
        focusa_core::types::TaskClass::Architecture | focusa_core::types::TaskClass::Integration
    ) {
        "medium"
    } else {
        "low"
    }
}

fn resume_payload_for_status(wl: &focusa_core::types::WorkLoopState) -> Value {
    json!({
        "last_checkpoint_id": wl.run.last_checkpoint_id,
        "last_safe_reentry_prompt_basis": wl.last_safe_reentry_prompt_basis,
        "restored_context_summary": wl.restored_context_summary,
        "last_blocker_reason": wl.last_blocker_reason,
        "last_completed_turn_summary": wl.last_observed_summary,
        "continuation_eligibility": wl.enabled && !wl.pause_flags.operator_override_active,
        "current_transport_health": if wl.status == focusa_core::types::WorkLoopStatus::TransportDegraded {
            "degraded"
        } else {
            "healthy"
        },
        "current_ask_and_scope_posture": json!({
            "current_ask": wl.decision_context.current_ask,
            "ask_kind": wl.decision_context.ask_kind,
            "scope_kind": wl.decision_context.scope_kind,
            "carryover_policy": wl.decision_context.carryover_policy,
            "excluded_context_reason": wl.decision_context.excluded_context_reason,
            "excluded_context_labels": wl.decision_context.excluded_context_labels,
            "work_item": wl.current_task.as_ref().map(|task| json!({
                "work_item_id": task.work_item_id,
                "allowed_scope": task.allowed_scope,
                "linked_spec_refs": task.linked_spec_refs,
            })),
        }),
    })
}

fn commitment_lifecycle_for_status(wl: &focusa_core::types::WorkLoopState) -> Value {
    let active_commitment = wl.current_task.as_ref().map(|task| {
        json!({
            "commitment_id": format!("commitment:{}", task.work_item_id),
            "work_item_id": task.work_item_id,
            "commitment_kind": "continuous_work_item",
            "status": if matches!(wl.status, focusa_core::types::WorkLoopStatus::Blocked | focusa_core::types::WorkLoopStatus::Paused) {
                "at_risk"
            } else {
                "active"
            }
        })
    });

    let decay_pressure =
        wl.consecutive_low_productivity_turns + wl.consecutive_same_work_item_retries;
    let persistence_posture = if wl.current_task.is_none() {
        "none"
    } else if decay_pressure == 0 && wl.consecutive_failures_for_task_class == 0 {
        "stable"
    } else {
        "stressed"
    };

    let release_state = if wl.current_task.is_none() && wl.last_completed_task_id.is_some() {
        "released_on_completion"
    } else if wl.current_task.is_none() && wl.last_blocker_reason.is_some() {
        "released_on_blocker"
    } else if wl.current_task.is_none() {
        "released_or_unbound"
    } else {
        "bound"
    };

    json!({
        "active_commitment": active_commitment,
        "creation_semantics": {
            "trigger": "SetContinuousWorkItem",
            "evidence_fields": ["current_task.work_item_id", "run.task_run_id"],
            "created_when": wl.current_task.as_ref().map(|task| format!("commitment:{}", task.work_item_id)),
        },
        "persistence_semantics": {
            "posture": persistence_posture,
            "policy": "commitment remains bound across turns unless completion, blocker escalation, or explicit pause/stop release occurs",
            "inhibits_drift_via": ["current_task pinning", "same-work-item retry tracking", "pause_flags"],
        },
        "decay_semantics": {
            "decay_pressure": decay_pressure,
            "failure_pressure": wl.consecutive_failures_for_task_class,
            "decay_triggers": {
                "low_productivity_turns": wl.consecutive_low_productivity_turns,
                "same_subproblem_retries": wl.consecutive_same_work_item_retries,
                "task_class_failures": wl.consecutive_failures_for_task_class,
            },
            "decay_posture": if decay_pressure > 0 || wl.consecutive_failures_for_task_class > 0 {
                "decaying"
            } else {
                "healthy"
            },
        },
        "release_semantics": {
            "state": release_state,
            "release_conditions": [
                "verification-backed completion transition",
                "explicit pause/stop or operator override",
                "blocker escalation when continuation is no longer productive"
            ],
            "last_completed_task_id": wl.last_completed_task_id,
            "last_blocker_reason": wl.last_blocker_reason,
        }
    })
}

fn safe_rate(numerator: u64, denominator: u64) -> Option<f64> {
    if denominator == 0 {
        None
    } else {
        Some(numerator as f64 / denominator as f64)
    }
}

fn secondary_loop_quality_metrics_for_status(
    s: &focusa_core::types::FocusaState,
    wl: &focusa_core::types::WorkLoopState,
) -> Value {
    let verification_result_events = s.telemetry.verification_result_events;
    let decision_consult_events = s.telemetry.decision_consult_events;
    let scope_contamination_events = s.telemetry.scope_contamination_events;
    let subject_hijack_prevented_events = s.telemetry.subject_hijack_prevented_events;
    let subject_hijack_occurred_events = s.telemetry.subject_hijack_occurred_events;

    json!({
        "verification_result_events": verification_result_events,
        "decision_consult_events": decision_consult_events,
        "scope_contamination_events": scope_contamination_events,
        "subject_hijack_prevented_events": subject_hijack_prevented_events,
        "subject_hijack_occurred_events": subject_hijack_occurred_events,
        "useful_events": s.telemetry.secondary_loop_useful_events,
        "low_quality_events": s.telemetry.secondary_loop_low_quality_events,
        "archived_events": s.telemetry.secondary_loop_archived_events,
        "decision_consult_rate": safe_rate(decision_consult_events, verification_result_events),
        "scope_contamination_rate": safe_rate(scope_contamination_events, verification_result_events),
        "subject_hijack_rate": safe_rate(subject_hijack_occurred_events, verification_result_events),
        "verification_coverage_rate": safe_rate(verification_result_events, wl.turn_count as u64),
        "verification_coverage_denominator": wl.turn_count,
    })
}

fn metacognitive_outcome_contracts() -> Value {
    json!([
        {
            "contract_id": "self_monitoring_signal",
            "category": "self_regulation",
            "machine_check_fields": ["quality_trace_events", "objective_counts", "continuation_decision_counts"]
        },
        {
            "contract_id": "strategy_selection_signal",
            "category": "cognitive_strategy",
            "machine_check_fields": ["objective_counts", "dominant_objective", "continuation_decision_counts"]
        },
        {
            "contract_id": "progress_regulation_signal",
            "category": "progress_control",
            "machine_check_fields": ["continuation_decision_counts", "non_closure_objective_events", "non_closure_objective_rate"]
        },
        {
            "contract_id": "transfer_to_new_context_signal",
            "category": "transfer_learning",
            "machine_check_fields": ["objective_counts", "non_closure_objective_events"]
        },
        {
            "contract_id": "motivation_ownership_signal",
            "category": "motivation",
            "machine_check_fields": ["objective_counts", "continuation_decision_counts"]
        },
        {
            "contract_id": "social_emotional_perspective_signal",
            "category": "social_emotional",
            "machine_check_fields": ["objective_counts", "non_closure_objective_events"]
        },
        {
            "contract_id": "teaching_regulation_signal",
            "category": "instructor_regulation",
            "machine_check_fields": ["objective_counts", "non_closure_objective_rate"]
        }
    ])
}

fn secondary_loop_objective_profile_for_status(s: &focusa_core::types::FocusaState) -> Value {
    let mut objective_counts = std::collections::BTreeMap::<String, u64>::new();
    let mut continuation_decision_counts = std::collections::BTreeMap::<String, u64>::new();
    let mut quality_trace_events = 0_u64;
    let mut non_closure_objective_events = 0_u64;

    for payload in s
        .telemetry
        .trace_events
        .iter()
        .filter(|event| {
            event.get("event_type").and_then(Value::as_str) == Some("verification_result")
        })
        .filter_map(|event| event.get("payload"))
        .filter(|payload| {
            payload.get("verification_kind").and_then(Value::as_str)
                == Some("secondary_loop_quality")
        })
    {
        quality_trace_events += 1;

        let objective = payload
            .get("loop_objective")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|objective| !objective.is_empty())
            .unwrap_or("continuous_turn_outcome_quality")
            .to_string();
        if objective != "continuous_turn_outcome_quality" {
            non_closure_objective_events += 1;
        }
        *objective_counts.entry(objective).or_insert(0) += 1;

        let continuation_decision = payload
            .get("continuation_decision")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|decision| !decision.is_empty())
            .unwrap_or("unknown")
            .to_string();
        *continuation_decision_counts
            .entry(continuation_decision)
            .or_insert(0) += 1;
    }

    let dominant_objective = objective_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(objective, _)| objective.clone());

    json!({
        "quality_trace_events": quality_trace_events,
        "objective_counts": objective_counts,
        "continuation_decision_counts": continuation_decision_counts,
        "non_closure_objective_events": non_closure_objective_events,
        "non_closure_objective_rate": safe_rate(non_closure_objective_events, quality_trace_events),
        "dominant_objective": dominant_objective,
        "metacognitive_outcome_contracts": metacognitive_outcome_contracts(),
    })
}

fn secondary_loop_eval_bundle_for_status(
    s: &focusa_core::types::FocusaState,
    wl: &focusa_core::types::WorkLoopState,
) -> Value {
    let promoted = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| entry.promotion_status == "promoted")
        .count() as u64;
    let rejected = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| entry.promotion_status == "rejected")
        .count() as u64;

    let retained_as_projection = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| entry.promotion_status == "retained_as_projection")
        .count() as u64;
    let deferred_for_review = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| entry.promotion_status == "deferred_for_review")
        .count() as u64;
    let archived_failed_attempt = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| entry.promotion_status == "archived_failed_attempt")
        .count() as u64;
    let archived = s.telemetry.secondary_loop_archived_events + archived_failed_attempt;

    let recent_entries: Vec<focusa_core::types::SecondaryLoopLedgerEntry> = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .rev()
        .take(20)
        .cloned()
        .collect();
    let trace_handles: Vec<String> = recent_entries
        .iter()
        .map(|entry| format!("trace://{}", entry.trace_id))
        .collect();
    let ledger_refs: Vec<String> = recent_entries
        .iter()
        .map(|entry| entry.proposal_id.clone())
        .collect();

    json!({
        "task_id": wl
            .current_task
            .as_ref()
            .map(|task| task.work_item_id.clone())
            .or_else(|| wl.last_completed_task_id.clone()),
        "scenario_id": wl
            .run
            .task_run_id
            .map(|id| id.to_string())
            .or_else(|| wl.decision_context.source_turn_id.clone()),
        "model_runtime_configuration": {
            "rfm_level": format!("{:?}", s.rfm.level),
            "autonomy_level": format!("{:?}", wl.current_autonomy_level.unwrap_or(s.autonomy.level)),
            "work_loop_status": format!("{:?}", wl.status),
            "transport_session_state": wl.transport_session_state,
            "policy": {
                "max_turns": wl.policy.max_turns,
                "max_wall_clock_ms": wl.policy.max_wall_clock_ms,
                "max_consecutive_low_productivity_turns": wl.policy.max_consecutive_low_productivity_turns,
                "max_same_subproblem_retries": wl.policy.max_same_subproblem_retries,
            }
        },
        "secondary_loop_kind_invoked": "continuous_turn_outcome_quality",
        "secondary_loop_objective_profile": secondary_loop_objective_profile_for_status(s),
        "trace_handles": trace_handles,
        "promotion_rejection_archival_result": {
            "promoted": promoted,
            "retained_as_projection": retained_as_projection,
            "deferred_for_review": deferred_for_review,
            "rejected": rejected,
            "archived_failed_attempt": archived_failed_attempt,
            "archived": archived,
        },
        "latency_token_cost_impact": {
            "total_prompt_tokens": s.telemetry.total_prompt_tokens,
            "total_completion_tokens": s.telemetry.total_completion_tokens,
            "verification_result_events": s.telemetry.verification_result_events,
            "useful_events": s.telemetry.secondary_loop_useful_events,
            "low_quality_events": s.telemetry.secondary_loop_low_quality_events,
        },
        "final_task_outcome": {
            "last_completed_task_id": wl.last_completed_task_id,
            "last_blocker_class": wl.last_blocker_class,
            "last_blocker_reason": wl.last_blocker_reason,
            "last_observed_summary": wl.last_observed_summary,
        },
        "ledger_refs": ledger_refs,
    })
}

fn secondary_loop_acceptance_hooks_for_status(s: &focusa_core::types::FocusaState) -> Value {
    let quality_payloads: Vec<&Value> = s
        .telemetry
        .trace_events
        .iter()
        .filter(|event| {
            event.get("event_type").and_then(Value::as_str) == Some("verification_result")
        })
        .filter_map(|event| event.get("payload"))
        .filter(|payload| {
            payload.get("verification_kind").and_then(Value::as_str)
                == Some("secondary_loop_quality")
        })
        .collect();

    let useful_quality_traces = quality_payloads
        .iter()
        .filter(|payload| payload.get("loop_quality").and_then(Value::as_str) == Some("useful"))
        .count() as u64;
    let low_quality_traces = quality_payloads
        .iter()
        .filter(|payload| {
            payload.get("loop_quality").and_then(Value::as_str) == Some("low_quality")
        })
        .count() as u64;
    let suppressed_irrelevant_suggestions = quality_payloads
        .iter()
        .filter(|payload| {
            payload.get("continuation_decision").and_then(Value::as_str) == Some("suppress")
        })
        .count() as u64;

    let rejected_or_deferred = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| {
            matches!(
                entry.promotion_status.as_str(),
                "rejected" | "deferred_for_review"
            )
        })
        .count() as u64;
    let archived_attempts = s
        .telemetry
        .secondary_loop_ledger
        .iter()
        .filter(|entry| entry.promotion_status == "archived_failed_attempt")
        .count() as u64
        + s.telemetry.secondary_loop_archived_events;

    let mut comparative_outcomes_by_task: std::collections::BTreeMap<String, (bool, bool)> =
        std::collections::BTreeMap::new();
    for entry in &s.telemetry.secondary_loop_ledger {
        let Some(correlation_id) = entry.correlation_id.as_deref() else {
            continue;
        };
        let slot = comparative_outcomes_by_task
            .entry(correlation_id.to_string())
            .or_insert((false, false));
        if entry.promotion_status == "promoted" {
            slot.0 = true;
        } else if matches!(
            entry.promotion_status.as_str(),
            "rejected" | "deferred_for_review" | "archived_failed_attempt"
        ) {
            slot.1 = true;
        }
    }
    let comparative_improvement_pairs = comparative_outcomes_by_task
        .values()
        .filter(|(has_promoted, has_baseline_failure)| *has_promoted && *has_baseline_failure)
        .count() as u64;

    json!({
        "bounded_improvement_over_no_secondary_baseline": comparative_improvement_pairs > 0
            || (useful_quality_traces > low_quality_traces && useful_quality_traces > 0),
        "irrelevant_secondary_suggestion_suppressed": suppressed_irrelevant_suggestions > 0
            || s.telemetry.subject_hijack_occurred_events > 0,
        "verification_rejection_observed": rejected_or_deferred > 0,
        "decay_or_archival_observed": archived_attempts > 0,
        "evidence_counts": {
            "quality_trace_events": quality_payloads.len(),
            "useful_quality_traces": useful_quality_traces,
            "low_quality_traces": low_quality_traces,
            "suppressed_irrelevant_suggestions": suppressed_irrelevant_suggestions,
            "rejected_or_deferred_outcomes": rejected_or_deferred,
            "archived_outcomes": archived_attempts,
            "comparative_improvement_pairs": comparative_improvement_pairs,
        }
    })
}

fn secondary_loop_closure_replay_evidence_for_status(
    wl: &focusa_core::types::WorkLoopState,
    replay_summary: &focusa_core::replay::SecondaryLoopComparativeReplaySummary,
) -> Value {
    let mut correlation_candidates = Vec::new();

    if let Some(task_run_id) = wl.run.task_run_id {
        correlation_candidates.push(task_run_id.to_string());
    }
    if let Some(current_task) = wl.current_task.as_ref()
        && !correlation_candidates
            .iter()
            .any(|candidate| candidate == &current_task.work_item_id)
    {
        correlation_candidates.push(current_task.work_item_id.clone());
    }
    if let Some(last_completed_task_id) = wl.last_completed_task_id.as_ref()
        && !correlation_candidates
            .iter()
            .any(|candidate| candidate == last_completed_task_id)
    {
        correlation_candidates.push(last_completed_task_id.clone());
    }

    let matched_pair = correlation_candidates.iter().find_map(|candidate| {
        replay_summary
            .task_pairs
            .iter()
            .find(|pair| pair.correlation_id == *candidate)
    });

    json!({
        "correlation_candidates": correlation_candidates,
        "replay_events_scanned": replay_summary.replay_events_scanned,
        "secondary_loop_outcome_events": replay_summary.secondary_loop_outcome_events,
        "comparative_improvement_pairs": replay_summary.comparative_improvement_pairs,
        "current_task_pair_observed": matched_pair
            .map(|pair| pair.comparative_improvement_observed)
            .unwrap_or(false),
        "current_task_pair_id": matched_pair.map(|pair| pair.correlation_id.as_str()),
        "current_task_pair_promoted_outcomes": matched_pair.map(|pair| pair.promoted_outcomes),
        "current_task_pair_non_promoted_outcomes": matched_pair
            .map(|pair| pair.non_promoted_outcomes),
    })
}

fn secondary_loop_replay_surface_payloads_for_status(
    wl: &focusa_core::types::WorkLoopState,
    replay_summary: &Result<focusa_core::replay::SecondaryLoopComparativeReplaySummary, String>,
) -> (Value, Value) {
    match replay_summary {
        Ok(summary) => {
            let closure_evidence = secondary_loop_closure_replay_evidence_for_status(wl, summary);
            (
                json!({ "status": "ok", "summary": summary }),
                json!({ "status": "ok", "evidence": closure_evidence }),
            )
        }
        Err(error) => (
            json!({ "status": "error", "error": error }),
            json!({ "status": "error", "error": error }),
        ),
    }
}

fn secondary_loop_replay_consumer_payload_for_status(
    wl: &focusa_core::types::WorkLoopState,
    replay_summary: &Result<focusa_core::replay::SecondaryLoopComparativeReplaySummary, String>,
) -> Value {
    let (secondary_loop_replay_comparative, secondary_loop_closure_replay_evidence) =
        secondary_loop_replay_surface_payloads_for_status(wl, replay_summary);

    match replay_summary {
        Ok(_) => json!({
            "status": "ok",
            "secondary_loop_replay_comparative": secondary_loop_replay_comparative,
            "secondary_loop_closure_replay_evidence": secondary_loop_closure_replay_evidence,
        }),
        Err(error) => json!({
            "status": "error",
            "error": error,
            "secondary_loop_replay_comparative": secondary_loop_replay_comparative,
            "secondary_loop_closure_replay_evidence": secondary_loop_closure_replay_evidence,
        }),
    }
}

fn secondary_loop_continuity_gate_for_status(
    replay_summary: &Result<focusa_core::replay::SecondaryLoopComparativeReplaySummary, String>,
    replay_consumer_payload: &Value,
) -> Value {
    let current_task_pair_observed = replay_consumer_payload
        .get("secondary_loop_closure_replay_evidence")
        .and_then(|value| value.get("evidence"))
        .and_then(|value| value.get("current_task_pair_observed"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let current_task_pair_id = replay_consumer_payload
        .get("secondary_loop_closure_replay_evidence")
        .and_then(|value| value.get("evidence"))
        .and_then(|value| value.get("current_task_pair_id"))
        .and_then(Value::as_str);

    match replay_summary {
        Ok(_) => json!({
            "state": "open",
            "fail_closed": false,
            "reason": "replay_consumer_ok",
            "replay_status": "ok",
            "current_task_pair_observed": current_task_pair_observed,
            "current_task_pair_id": current_task_pair_id,
            "requires_replay_consumer_ok": true,
        }),
        Err(error) => json!({
            "state": "fail-closed",
            "fail_closed": true,
            "reason": "replay_consumer_error",
            "error": error,
            "replay_status": "error",
            "current_task_pair_observed": false,
            "current_task_pair_id": Value::Null,
            "requires_replay_consumer_ok": true,
        }),
    }
}

fn secondary_loop_closure_bundle_for_status(
    s: &focusa_core::types::FocusaState,
    wl: &focusa_core::types::WorkLoopState,
    replay_summary: &Result<focusa_core::replay::SecondaryLoopComparativeReplaySummary, String>,
) -> Value {
    let secondary_loop_quality_metrics = secondary_loop_quality_metrics_for_status(s, wl);
    let secondary_loop_eval_bundle = secondary_loop_eval_bundle_for_status(s, wl);
    let secondary_loop_acceptance_hooks = secondary_loop_acceptance_hooks_for_status(s);
    let replay_consumer_payload =
        secondary_loop_replay_consumer_payload_for_status(wl, replay_summary);
    let secondary_loop_continuity_gate =
        secondary_loop_continuity_gate_for_status(replay_summary, &replay_consumer_payload);

    let project_status = if wl.enabled {
        if wl.current_task.is_none()
            && wl.last_completed_task_id.is_some()
            && wl.status == focusa_core::types::WorkLoopStatus::AdvancingTask
        {
            "completing"
        } else {
            "active"
        }
    } else {
        "idle"
    };

    let tranche_status = match (&wl.run.tranche_run_id, &wl.current_task, &wl.status) {
        (Some(_), Some(_), _) => "active",
        (Some(_), None, focusa_core::types::WorkLoopStatus::AdvancingTask) => "completed",
        (Some(_), None, _) => "advancing",
        _ => "none",
    };

    json!({
        "status": "ok",
        "doc": "78",
        "work_loop": {
            "enabled": wl.enabled,
            "status": wl.status,
            "project_status": project_status,
            "tranche_status": tranche_status,
            "current_task": wl.current_task,
            "last_completed_task_id": wl.last_completed_task_id,
            "last_continue_reason": wl.last_continue_reason,
            "last_blocker_reason": wl.last_blocker_reason,
        },
        "secondary_loop_quality_metrics": secondary_loop_quality_metrics,
        "secondary_loop_eval_bundle": secondary_loop_eval_bundle,
        "secondary_loop_acceptance_hooks": secondary_loop_acceptance_hooks,
        "secondary_loop_replay_consumer": replay_consumer_payload,
        "secondary_loop_continuity_gate": secondary_loop_continuity_gate,
        "evidence_contract": {
            "watchdog_consumer": "scripts/work_loop_watchdog.sh",
            "replay_consumer_route": "/v1/work-loop/replay/closure-evidence",
            "continuity_gate_policy": "fail-closed when replay consumer is unavailable",
        },
    })
}

async fn status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }

    let s = state.focusa.read().await;
    let wl = &s.work_loop;
    let active_writer = state.active_writer.read().await.clone();
    let driver_snapshot = {
        let guard = state.pi_rpc_session.lock().await;
        guard.as_ref().map(|session| {
            json!({
                "adapter": "pi-rpc",
                "session_id": session.session_id,
                "cwd": session.cwd,
                "uptime_ms": session.started_at.elapsed().as_millis(),
            })
        })
    };
    let worktree = worktree_status_snapshot().await;
    let alternate_ready_work = alternate_ready_work_snapshot(wl.current_task.as_ref()).await;
    let blocker_package = build_blocker_package(wl, alternate_ready_work.clone());
    let transport_health = transport_health_for_status(wl);
    let execution_environment =
        execution_environment_for_status(wl.transport_session_state.as_deref(), &worktree);
    let budget_remaining = budget_remaining_for_status(wl);
    let resume_payload = resume_payload_for_status(wl);
    let commitment_lifecycle = commitment_lifecycle_for_status(wl);
    let secondary_loop_quality_metrics = secondary_loop_quality_metrics_for_status(&s, wl);
    let secondary_loop_eval_bundle = secondary_loop_eval_bundle_for_status(&s, wl);
    let secondary_loop_acceptance_hooks = secondary_loop_acceptance_hooks_for_status(&s);
    let secondary_loop_replay_summary =
        focusa_core::replay::secondary_loop_comparative_summary_from_replay(
            &state.persistence,
            &focusa_core::replay::ReplayConfig {
                from: None,
                until: None,
                session_id: s.session.as_ref().map(|session| session.session_id),
                frame_id: None,
            },
        )
        .map_err(|error| error.to_string());
    let secondary_loop_replay_consumer =
        secondary_loop_replay_consumer_payload_for_status(wl, &secondary_loop_replay_summary);
    let secondary_loop_continuity_gate = secondary_loop_continuity_gate_for_status(
        &secondary_loop_replay_summary,
        &secondary_loop_replay_consumer,
    );
    let secondary_loop_replay_comparative = secondary_loop_replay_consumer
        .get("secondary_loop_replay_comparative")
        .cloned()
        .unwrap_or_else(|| {
            json!({
                "status": "error",
                "error": "missing secondary_loop_replay_comparative payload",
            })
        });
    let secondary_loop_closure_replay_evidence = secondary_loop_replay_consumer
        .get("secondary_loop_closure_replay_evidence")
        .cloned()
        .unwrap_or_else(|| {
            json!({
                "status": "error",
                "error": "missing secondary_loop_closure_replay_evidence payload",
            })
        });
    let pending_proposals = focusa_core::pre::pending_count(&s.pre);
    let next_work_risk_class = next_work_risk_class_for_status(wl);
    Ok(Json(json!({
        "enabled": wl.enabled,
        "status": wl.status,
        "project_status": if wl.enabled {
            if wl.current_task.is_none() && wl.last_completed_task_id.is_some() && wl.status == focusa_core::types::WorkLoopStatus::AdvancingTask {
                "completing"
            } else {
                "active"
            }
        } else {
            "idle"
        },
        "tranche_status": match (&wl.run.tranche_run_id, &wl.current_task, &wl.status) {
            (Some(_), Some(_), _) => "active",
            (Some(_), None, focusa_core::types::WorkLoopStatus::AdvancingTask) => "completed",
            (Some(_), None, _) => "advancing",
            _ => "none",
        },
        "authorship_mode": wl.authorship_mode,
        "policy": wl.policy,
        "run": wl.run,
        "identity_summary": {
            "project_run_id": wl.run.project_run_id,
            "tranche_run_id": wl.run.tranche_run_id,
            "task_run_id": wl.run.task_run_id,
            "worker_session_id": wl.run.worker_session_id,
            "last_checkpoint_id": wl.run.last_checkpoint_id,
        },
        "current_task": wl.current_task,
        "last_completed_task_id": wl.last_completed_task_id,
        "last_recorded_bd_transition_id": wl.last_recorded_bd_transition_id,
        "last_blocker_class": wl.last_blocker_class,
        "last_blocker_reason": wl.last_blocker_reason,
        "last_continue_reason": wl.last_continue_reason,
        "last_observed_summary": wl.last_observed_summary,
        "last_checkpoint_id": wl.run.last_checkpoint_id,
        "consecutive_failures_for_task_class": wl.consecutive_failures_for_task_class,
        "pause_flags": wl.pause_flags,
        "decision_context": wl.decision_context,
        "continuation_inputs": {
            "active_mission": { "intent": s.focus_stack.frames.iter().find(|f| Some(f.id) == s.focus_stack.active_id).map(|f| f.focus_state.intent.clone()), "frame_id": s.focus_stack.active_id },
            "current_ask": wl.decision_context.current_ask,
            "ask_kind": wl.decision_context.ask_kind,
            "scope_kind": wl.decision_context.scope_kind,
            "carryover_policy": wl.decision_context.carryover_policy,
            "excluded_context_reason": wl.decision_context.excluded_context_reason,
            "excluded_context_labels": wl.decision_context.excluded_context_labels,
            "operator_steering_detected": wl.decision_context.operator_steering_detected,
            "pending_proposals_requiring_resolution": wl.pending_proposals_requiring_resolution.max(pending_proposals),
            "autonomy_level": wl.current_autonomy_level.unwrap_or(s.autonomy.level),
            "autonomy_scope": s.autonomy.granted_scope,
            "verification_required": wl.current_task.as_ref().map(|task| task.required_verification_tier.clone()),
            "next_work_risk_class": wl.next_work_risk_class.clone().unwrap_or_else(|| next_work_risk_class.to_string()),
            "budget_caps": {
                "max_turns": wl.policy.max_turns,
                "max_wall_clock_ms": wl.policy.max_wall_clock_ms,
                "max_retries": wl.policy.max_retries,
                "max_consecutive_failures": wl.policy.max_consecutive_failures,
                "max_same_subproblem_retries": wl.policy.max_same_subproblem_retries,
            },
            "operator_overrides": wl.pause_flags,
            "recent_checkpoint_state": {
                "last_checkpoint_id": wl.run.last_checkpoint_id,
                "last_safe_reentry_prompt_basis": wl.last_safe_reentry_prompt_basis,
                "restored_context_summary": wl.restored_context_summary,
            }
        },
        "delegated_authorship": wl.delegated_authorship,
        "transport": {
            "adapter": wl.transport_adapter,
            "session_state": wl.transport_session_state,
            "last_event_kind": wl.last_transport_event_kind,
            "last_event_summary": wl.last_transport_event_summary,
            "last_event_sequence": wl.last_transport_event_sequence,
            "abort_reason": wl.transport_abort_reason,
            "daemon_supervised_session": driver_snapshot,
        },
        "active_worker": wl.active_worker,
        "blocker_package": blocker_package,
        "active_writer": active_writer,
        "transport_health": transport_health,
        "execution_environment": execution_environment,
        "budget_remaining": budget_remaining,
        "secondary_loop_quality_metrics": secondary_loop_quality_metrics,
        "secondary_loop_eval_artifacts": {
            "ledger_size": s.telemetry.secondary_loop_ledger.len(),
            "recent_entries": s
                .telemetry
                .secondary_loop_ledger
                .iter()
                .rev()
                .take(20)
                .cloned()
                .collect::<Vec<_>>(),
        },
        "secondary_loop_eval_bundle": secondary_loop_eval_bundle,
        "secondary_loop_acceptance_hooks": secondary_loop_acceptance_hooks,
        "secondary_loop_replay_consumer": secondary_loop_replay_consumer,
        "secondary_loop_replay_comparative": secondary_loop_replay_comparative,
        "secondary_loop_closure_replay_evidence": secondary_loop_closure_replay_evidence,
        "secondary_loop_continuity_gate": secondary_loop_continuity_gate,
        "resume_payload": resume_payload,
        "commitment_lifecycle": commitment_lifecycle,
        "governance": {
            "writer_header_required": WRITER_HEADER,
            "approval_header_required_for_enable": APPROVAL_HEADER,
            "explicit_enable_approval_required": true,
            "policy_owner": "daemon",
            "api_role": "dispatch_and_observability_only",
            "extension_role": "bridge_only_not_cognitive_authority",
            "llm_authority": "executor_only_unless_explicitly_delegated",
            "operator_override_supersedes_loop": true,
        },
        "worktree": worktree,
    })))
}

async fn closure_replay_evidence(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }

    let s = state.focusa.read().await;
    let wl = &s.work_loop;

    let secondary_loop_replay_summary =
        focusa_core::replay::secondary_loop_comparative_summary_from_replay(
            &state.persistence,
            &focusa_core::replay::ReplayConfig {
                from: None,
                until: None,
                session_id: s.session.as_ref().map(|session| session.session_id),
                frame_id: None,
            },
        )
        .map_err(|error| error.to_string());

    let replay_consumer_payload =
        secondary_loop_replay_consumer_payload_for_status(wl, &secondary_loop_replay_summary);
    let secondary_loop_continuity_gate = secondary_loop_continuity_gate_for_status(
        &secondary_loop_replay_summary,
        &replay_consumer_payload,
    );

    let mut payload = replay_consumer_payload;
    if let Some(obj) = payload.as_object_mut() {
        obj.insert(
            "secondary_loop_continuity_gate".to_string(),
            secondary_loop_continuity_gate,
        );
    }

    Ok(Json(payload))
}

async fn closure_replay_bundle(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }

    let s = state.focusa.read().await;
    let wl = &s.work_loop;

    let secondary_loop_replay_summary =
        focusa_core::replay::secondary_loop_comparative_summary_from_replay(
            &state.persistence,
            &focusa_core::replay::ReplayConfig {
                from: None,
                until: None,
                session_id: s.session.as_ref().map(|session| session.session_id),
                frame_id: None,
            },
        )
        .map_err(|error| error.to_string());

    Ok(Json(secondary_loop_closure_bundle_for_status(
        &s,
        wl,
        &secondary_loop_replay_summary,
    )))
}

async fn enable(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<EnableWorkLoopRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    require_approval(
        &headers,
        "continuous work enable crosses a governance boundary and must be explicitly approved",
    )?;
    let writer_id = ensure_writer_claim(&state, &headers).await?;

    let preset = payload.preset.unwrap_or_default();
    let policy =
        WorkLoopPolicy::with_overrides(preset, payload.policy_overrides.unwrap_or_default());
    let action = Action::EnableContinuousWork {
        project_run_id: payload.project_run_id.unwrap_or_else(Uuid::now_v7),
        policy,
    };
    state.command_tx.send(action).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("dispatch failed: {e}") })),
        )
    })?;

    let root_work_item_id = if let Some(root) = payload.root_work_item_id.clone() {
        Some(root)
    } else {
        let focusa = state.focusa.read().await;
        focusa
            .focus_stack
            .active_id
            .and_then(|aid| focusa.focus_stack.frames.iter().find(|f| f.id == aid))
            .map(|frame| frame.beads_issue_id.clone())
            .filter(|id| !id.is_empty())
    };

    if let Some(parent_work_item_id) = root_work_item_id {
        state
            .command_tx
            .send(Action::SelectNextContinuousSubtask {
                parent_work_item_id,
            })
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("dispatch failed: {e}") })),
                )
            })?;
        let _ = maybe_dispatch_continuous_turn_prompt(
            &state,
            "continuous work enabled with ready work selected",
        )
        .await;
    }

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn pause(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ReasonRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_id = writer_id_from_headers(&headers)?;
    let active_writer = state.active_writer.read().await.clone();
    if active_writer.as_deref().is_some() && active_writer.as_deref() != Some(writer_id.as_str()) {
        return Err(conflict(
            "continuous work loop claimed by another writer",
            active_writer,
        ));
    }

    state
        .command_tx
        .send(Action::PauseContinuousWork {
            reason: payload.reason.unwrap_or_default(),
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn resume(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ReasonRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_id = ensure_writer_claim(&state, &headers).await?;

    state
        .command_tx
        .send(Action::ResumeContinuousWork {
            reason: payload.reason.unwrap_or_default(),
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn select_next(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SelectNextRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_id = ensure_writer_claim(&state, &headers).await?;

    state
        .command_tx
        .send(Action::SelectNextContinuousSubtask {
            parent_work_item_id: payload.parent_work_item_id,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;
    let _ = maybe_dispatch_continuous_turn_prompt(
        &state,
        "ready work selected for continuous execution",
    )
    .await;

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn set_decision_context(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<DecisionContextRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_id = ensure_claimed_writer_matches_for_context(&state, &headers).await?;

    state
        .command_tx
        .send(Action::SetContinuousDecisionContext {
            current_ask: payload.current_ask,
            ask_kind: payload.ask_kind,
            scope_kind: payload.scope_kind,
            carryover_policy: payload.carryover_policy,
            excluded_context_reason: payload.excluded_context_reason,
            excluded_context_labels: payload.excluded_context_labels,
            source_turn_id: payload.source_turn_id,
            operator_steering_detected: payload.operator_steering_detected,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    Ok(Json(
        json!({ "status": "accepted", "writer_id": writer_id }),
    ))
}

async fn start_pi_driver(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<PiDriverStartRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    let writer_id = ensure_writer_claim(&state, &headers).await?;

    let mut guard = state.pi_rpc_session.lock().await;
    if guard.is_some() {
        return Err(conflict("pi rpc driver already active", Some(writer_id)));
    }

    let session_id = format!("pi-rpc-{}", Uuid::now_v7());
    let mut cmd = Command::new(PI_RPC_BIN);
    let base_path = std::env::var("PATH").unwrap_or_default();
    let node_bin_dir = "/opt/cpanel/ea-nodejs20/bin";
    let merged_path = if base_path.split(':').any(|segment| segment == node_bin_dir) {
        base_path
    } else if base_path.is_empty() {
        node_bin_dir.to_string()
    } else {
        format!("{node_bin_dir}:{base_path}")
    };

    cmd.env("PATH", merged_path)
        .args(["--mode", "rpc", "--no-session"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(models) = payload.models.as_deref() {
        cmd.args(["--models", models]);
    }
    if let Some(cwd) = payload.cwd.as_deref() {
        cmd.current_dir(cwd);
    }

    let mut child = cmd.spawn().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("failed to spawn pi rpc: {e}")})),
        )
    })?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| bad_request("pi rpc stdin unavailable"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| bad_request("pi rpc stdout unavailable"))?;
    let stderr = child.stderr.take();

    let state_for_events = state.clone();
    let command_tx = state.command_tx.clone();
    let attach_session_id = session_id.clone();

    if let Some(stderr_stream) = stderr {
        let stderr_command_tx = command_tx.clone();
        let stderr_session_id = attach_session_id.clone();
        tokio::spawn(async move {
            let mut stderr_seq: u64 = 1;
            let mut err_lines = BufReader::new(stderr_stream).lines();
            while let Ok(Some(line)) = err_lines.next_line().await {
                let _ = stderr_command_tx
                    .send(Action::IngestContinuousTransportEvent {
                        sequence: stderr_seq,
                        kind: "stderr_line".to_string(),
                        session_id: Some(stderr_session_id.clone()),
                        turn_id: None,
                        summary: Some(line),
                    })
                    .await;
                stderr_seq = stderr_seq.saturating_add(1);
            }
        });
    }

    tokio::spawn(async move {
        let _ = command_tx
            .send(Action::AttachContinuousTransportSession {
                adapter: "pi-rpc".to_string(),
                session_id: attach_session_id.clone(),
            })
            .await;
        let mut seq: u64 = 1;
        let mut last_assistant_output = String::new();
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let parsed: Value = serde_json::from_str(&line)
                .unwrap_or_else(|_| json!({"type":"raw","summary":line}));
            let kind = parsed
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            if kind == "turn_start" || kind == "agent_start" {
                last_assistant_output.clear();
            }
            if kind == "message_update"
                && let Some(delta) = parsed
                    .get("assistantMessageEvent")
                    .and_then(|v| v.get("delta"))
                    .and_then(Value::as_str)
            {
                last_assistant_output.push_str(delta);
            }
            if (kind == "turn_end" || kind == "agent_end") && last_assistant_output.is_empty() {
                if let Some(text) = parsed.get("message").and_then(extract_assistant_text) {
                    last_assistant_output = text;
                } else if let Some(text) = parsed
                    .get("messages")
                    .and_then(Value::as_array)
                    .and_then(|msgs| msgs.iter().rev().find_map(extract_assistant_text))
                {
                    last_assistant_output = text;
                }
            }
            let summary = parsed
                .get("message")
                .and_then(|m| m.get("role").and_then(Value::as_str).or_else(|| m.as_str()))
                .map(|s| s.to_string())
                .or_else(|| {
                    parsed
                        .get("assistantMessageEvent")
                        .and_then(|v| v.get("type"))
                        .and_then(Value::as_str)
                        .map(|s| s.to_string())
                })
                .or_else(|| {
                    parsed
                        .get("command")
                        .and_then(Value::as_str)
                        .map(|s| format!("response:{s}"))
                })
                .or_else(|| Some(kind.clone()));
            let _ = command_tx
                .send(Action::IngestContinuousTransportEvent {
                    sequence: seq,
                    kind: kind.clone(),
                    session_id: Some(attach_session_id.clone()),
                    turn_id: None,
                    summary,
                })
                .await;
            if matches!(
                kind.as_str(),
                "session_compact" | "compaction_end" | "session_compact_end"
            ) {
                let _ = maybe_dispatch_continuous_turn_prompt(
                    &state_for_events,
                    "pi rpc compaction completed; dispatching automatic continuation prompt",
                )
                .await;
            }
            if kind == "turn_end" || kind == "agent_end" {
                let current_task = {
                    let focusa = state_for_events.focusa.read().await;
                    focusa.work_loop.current_task.clone()
                };
                if let Some(task) = current_task {
                    let assistant_output = last_assistant_output.trim();
                    let has_assistant_output = !assistant_output.is_empty();
                    if has_assistant_output {
                        let assistant_excerpt =
                            assistant_output.chars().take(220).collect::<String>();
                        let spec_conformant = !assistant_output.contains("BLOCKER:")
                            && !assistant_output.contains("ESCALATE:");
                        let _ = command_tx
                            .send(Action::ObserveContinuousTurnOutcome {
                                task_run_id: None,
                                work_item_id: Some(task.work_item_id.clone()),
                                summary: format!(
                                    "{kind} for {}: {assistant_excerpt}",
                                    task.work_item_id
                                ),
                                continue_reason: Some(format!(
                                    "{kind} observed from pi rpc stream: {assistant_excerpt}"
                                )),
                                verification_satisfied: true,
                                spec_conformant,
                            })
                            .await;
                        let _ = maybe_dispatch_continuous_turn_prompt(
                            &state_for_events,
                            "pi rpc turn_end/agent_end observed and ready work remains",
                        )
                        .await;
                    } else {
                        let _ = maybe_dispatch_continuous_turn_prompt(
                            &state_for_events,
                            "pi rpc turn ended without assistant output (compaction/housekeeping); auto-retrying",
                        )
                        .await;
                    }
                }
                last_assistant_output.clear();
            }
            seq += 1;
        }

        let _ = command_tx
            .send(Action::IngestContinuousTransportEvent {
                sequence: seq,
                kind: "stream_closed".to_string(),
                session_id: Some(attach_session_id.clone()),
                turn_id: None,
                summary: Some("pi rpc stdout stream closed".to_string()),
            })
            .await;
        let _ = command_tx
            .send(Action::MarkContinuousLoopTransportDegraded {
                reason: "pi rpc stdout stream closed; restart required".to_string(),
            })
            .await;
    });

    *guard = Some(crate::server::PiRpcSession {
        child,
        stdin,
        session_id: session_id.clone(),
        cwd: payload.cwd.clone(),
        started_at: std::time::Instant::now(),
    });

    Ok(Json(
        json!({"status":"accepted","adapter":"pi-rpc","session_id":session_id}),
    ))
}

async fn prompt_pi_driver(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<PiDriverPromptRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    let mut guard = state.pi_rpc_session.lock().await;
    let Some(session) = guard.as_mut() else {
        return Err(bad_request("pi rpc driver not active"));
    };
    let msg = if let Some(streaming_behavior) = payload.streaming_behavior.as_deref() {
        json!({"id": format!("prompt-{}", Uuid::now_v7()), "type":"prompt", "message": payload.message, "streamingBehavior": streaming_behavior})
    } else {
        json!({"id": format!("prompt-{}", Uuid::now_v7()), "type":"prompt", "message": payload.message})
    };
    session
        .stdin
        .write_all(msg.to_string().as_bytes())
        .await
        .map_err(|e| bad_request(format!("failed writing prompt: {e}")))?;
    session
        .stdin
        .write_all(b"\n")
        .await
        .map_err(|e| bad_request(format!("failed writing newline: {e}")))?;
    Ok(Json(
        json!({"status":"accepted","session_id":session.session_id}),
    ))
}

async fn abort_pi_driver(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    let mut guard = state.pi_rpc_session.lock().await;
    let Some(session) = guard.as_mut() else {
        return Err(bad_request("pi rpc driver not active"));
    };
    let msg = json!({"type":"abort"}).to_string() + "\n";
    session
        .stdin
        .write_all(msg.as_bytes())
        .await
        .map_err(|e| bad_request(format!("failed writing abort: {e}")))?;
    Ok(Json(
        json!({"status":"accepted","session_id":session.session_id}),
    ))
}

async fn stop_pi_driver(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    let mut guard = state.pi_rpc_session.lock().await;
    let Some(mut session) = guard.take() else {
        return Err(bad_request("pi rpc driver not active"));
    };
    let _ = session.child.kill().await;
    Ok(Json(
        json!({"status":"accepted","session_id":session.session_id}),
    ))
}

async fn attach_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SessionAttachRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_id = ensure_writer_claim(&state, &headers).await?;
    state
        .command_tx
        .send(Action::AttachContinuousTransportSession {
            adapter: payload.adapter,
            session_id: payload.session_id,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn abort_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ReasonRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_id = ensure_writer_claim(&state, &headers).await?;
    state
        .command_tx
        .send(Action::AbortContinuousTransportSession {
            reason: payload
                .reason
                .unwrap_or_else(|| "abort requested".to_string()),
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn ingest_transport_event(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<TransportEventRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_id = ensure_writer_claim(&state, &headers).await?;
    let _guard = state.write_serial_lock.lock().await;
    state
        .command_tx
        .send(Action::IngestContinuousTransportEvent {
            sequence: payload.sequence,
            kind: payload.kind,
            session_id: payload.session_id,
            turn_id: payload.turn_id,
            summary: payload.summary,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn set_pause_flags(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<PauseFlagsRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_id = ensure_writer_claim(&state, &headers).await?;
    state
        .command_tx
        .send(Action::SetContinuousPauseFlags {
            destructive_confirmation_required: payload.destructive_confirmation_required,
            governance_decision_pending: payload.governance_decision_pending,
            operator_override_active: payload.operator_override_active,
            reason: payload.reason,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn delegate_authorship(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<DelegationRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    require_approval(
        &headers,
        "delegated authorship changes authority state and requires explicit approval",
    )?;
    let writer_id = ensure_writer_claim(&state, &headers).await?;
    state
        .command_tx
        .send(Action::SetDelegatedContinuousAuthorship {
            delegate_id: Some(payload.delegate_id),
            scope: Some(payload.scope),
            amendment_summary: payload.amendment_summary,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn clear_delegated_authorship(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ReasonRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    require_approval(
        &headers,
        "clearing delegated authorship changes authority state and requires explicit approval",
    )?;
    let writer_id = ensure_writer_claim(&state, &headers).await?;
    state
        .command_tx
        .send(Action::SetDelegatedContinuousAuthorship {
            delegate_id: None,
            scope: None,
            amendment_summary: payload.reason,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn transport_degraded(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ReasonRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_id = ensure_writer_claim(&state, &headers).await?;
    state
        .command_tx
        .send(Action::MarkContinuousLoopTransportDegraded {
            reason: payload
                .reason
                .unwrap_or_else(|| "transport degraded".to_string()),
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn checkpoints(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let focusa = state.focusa.read().await;
    let wl = &focusa.work_loop;
    Ok(Json(json!({
        "last_checkpoint_id": wl.run.last_checkpoint_id,
        "resume_payload": resume_payload_for_status(wl),
        "last_safe_reentry_prompt_basis": wl.last_safe_reentry_prompt_basis,
        "restored_context_summary": wl.restored_context_summary,
        "last_continue_reason": wl.last_continue_reason,
        "last_blocker_reason": wl.last_blocker_reason,
    })))
}

async fn heartbeat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_id = ensure_writer_claim(&state, &headers).await?;
    let dispatched =
        maybe_dispatch_continuous_turn_prompt(&state, "daemon heartbeat supervisor tick").await?;

    Ok(Json(json!({
        "ok": true,
        "writer_id": writer_id,
        "dispatched": dispatched,
    })))
}

async fn checkpoint(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CheckpointRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let writer_id = ensure_writer_claim(&state, &headers).await?;
    state
        .command_tx
        .send(Action::CheckpointContinuousLoop {
            checkpoint_id: payload.checkpoint_id.unwrap_or_else(Uuid::now_v7),
            summary: payload.summary,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn stop(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ReasonRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let released_writer = release_writer_claim(&state, &headers).await?;
    state
        .command_tx
        .send(Action::StopContinuousWork {
            reason: payload.reason.unwrap_or_default(),
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;

    Ok(Json(
        json!({ "ok": true, "released_writer": released_writer }),
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/work-loop", get(status))
        .route("/v1/work-loop/status", get(status))
        .route(
            "/v1/work-loop/replay/closure-evidence",
            get(closure_replay_evidence),
        )
        .route(
            "/v1/work-loop/replay/closure-bundle",
            get(closure_replay_bundle),
        )
        .route("/v1/work-loop/enable", post(enable))
        .route("/v1/work-loop/pause", post(pause))
        .route("/v1/work-loop/resume", post(resume))
        .route("/v1/work-loop/select-next", post(select_next))
        .route("/v1/work-loop/context", post(set_decision_context))
        .route("/v1/work-loop/driver/start", post(start_pi_driver))
        .route("/v1/work-loop/driver/prompt", post(prompt_pi_driver))
        .route("/v1/work-loop/driver/abort", post(abort_pi_driver))
        .route("/v1/work-loop/driver/stop", post(stop_pi_driver))
        .route("/v1/work-loop/session/attach", post(attach_session))
        .route("/v1/work-loop/session/abort", post(abort_session))
        .route("/v1/work-loop/events", post(ingest_transport_event))
        .route("/v1/work-loop/pause-flags", post(set_pause_flags))
        .route("/v1/work-loop/delegation/enable", post(delegate_authorship))
        .route(
            "/v1/work-loop/delegation/clear",
            post(clear_delegated_authorship),
        )
        .route("/v1/work-loop/degraded", post(transport_degraded))
        .route("/v1/work-loop/checkpoints", get(checkpoints))
        .route("/v1/work-loop/checkpoint", post(checkpoint))
        .route("/v1/work-loop/heartbeat", post(heartbeat))
        .route("/v1/work-loop/stop", post(stop))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ledger_entry(
        proposal_id: &str,
        promotion_status: &str,
        trace_id: &str,
    ) -> focusa_core::types::SecondaryLoopLedgerEntry {
        focusa_core::types::SecondaryLoopLedgerEntry {
            proposal_id: proposal_id.to_string(),
            source_function: "Action::ObserveContinuousTurnOutcome".to_string(),
            actor_instance_id: None,
            role_profile_id: "daemon.work_loop.secondary_cognition".to_string(),
            current_ask_id: Some("implement spec78".to_string()),
            query_scope_id: Some("mission_carryover".to_string()),
            input_window_ref: Some("pi-turn-7001".to_string()),
            evidence_refs: vec![format!("trace://{}", trace_id)],
            proposed_delta: "secondary loop delta".to_string(),
            verification_status: if promotion_status == "promoted" {
                "verified".to_string()
            } else {
                "unverified".to_string()
            },
            promotion_status: promotion_status.to_string(),
            confidence: 0.8,
            impact_metrics: json!({
                "loop_quality": if promotion_status == "promoted" { "useful" } else { "low_quality" },
                "latency_ms_since_turn_request": 12,
            }),
            failure_class: if promotion_status == "promoted" {
                None
            } else {
                Some("verification".to_string())
            },
            description: "continuous outcome quality artifact".to_string(),
            trace_id: trace_id.to_string(),
            correlation_id: Some("task-run-1".to_string()),
            created_at: Utc::now(),
        }
    }

    fn sample_current_task(work_item_id: &str) -> SpecLinkedTaskPacket {
        SpecLinkedTaskPacket {
            work_item_id: work_item_id.to_string(),
            title: "doc78 bounded secondary cognition".to_string(),
            task_class: TaskClass::Code,
            linked_spec_refs: vec![
                "docs/78-bounded-secondary-cognition-and-persistent-autonomy.md#15.2".to_string(),
            ],
            acceptance_criteria: vec![
                "emit replay/eval bundle dimensions".to_string(),
                "persist proposal advancement ledger".to_string(),
            ],
            required_verification_tier: Some("code-task-verification".to_string()),
            allowed_scope: vec!["mission_carryover".to_string()],
            dependencies: vec![],
            tranche_id: None,
            blocker_class: None,
            checkpoint_summary: None,
        }
    }

    fn sample_secondary_quality_trace(
        continuation_decision: &str,
        loop_quality: &str,
        subject_hijack_occurred: bool,
    ) -> Value {
        json!({
            "event_type": "verification_result",
            "payload": {
                "verification_kind": "secondary_loop_quality",
                "loop_quality": loop_quality,
                "continuation_decision": continuation_decision,
                "subject_hijack_occurred": subject_hijack_occurred,
            }
        })
    }

    #[test]
    fn secondary_loop_quality_metrics_include_rate_surfaces() {
        let mut state = focusa_core::types::FocusaState::default();
        state.work_loop.turn_count = 8;
        state.telemetry.verification_result_events = 4;
        state.telemetry.decision_consult_events = 2;
        state.telemetry.scope_contamination_events = 1;
        state.telemetry.subject_hijack_prevented_events = 3;
        state.telemetry.subject_hijack_occurred_events = 1;
        state.telemetry.secondary_loop_useful_events = 3;
        state.telemetry.secondary_loop_low_quality_events = 1;
        state.telemetry.secondary_loop_archived_events = 5;

        let metrics = secondary_loop_quality_metrics_for_status(&state, &state.work_loop);

        assert_eq!(
            metrics.get("decision_consult_rate").and_then(Value::as_f64),
            Some(0.5)
        );
        assert_eq!(
            metrics
                .get("scope_contamination_rate")
                .and_then(Value::as_f64),
            Some(0.25)
        );
        assert_eq!(
            metrics
                .get("verification_coverage_rate")
                .and_then(Value::as_f64),
            Some(0.5)
        );
        assert_eq!(
            metrics.get("subject_hijack_rate").and_then(Value::as_f64),
            Some(0.25)
        );
        assert_eq!(
            metrics
                .get("subject_hijack_occurred_events")
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            metrics.get("archived_events").and_then(Value::as_u64),
            Some(5)
        );
    }

    #[test]
    fn secondary_loop_quality_metrics_handle_zero_denominators() {
        let state = focusa_core::types::FocusaState::default();
        let metrics = secondary_loop_quality_metrics_for_status(&state, &state.work_loop);

        assert_eq!(
            metrics.get("decision_consult_rate").and_then(Value::as_f64),
            None
        );
        assert_eq!(
            metrics
                .get("scope_contamination_rate")
                .and_then(Value::as_f64),
            None
        );
        assert_eq!(
            metrics
                .get("verification_coverage_rate")
                .and_then(Value::as_f64),
            None
        );
        assert_eq!(
            metrics.get("subject_hijack_rate").and_then(Value::as_f64),
            None
        );
    }

    #[test]
    fn secondary_loop_eval_bundle_surfaces_doc78_audit_dimensions() {
        let mut state = focusa_core::types::FocusaState::default();
        let scenario_id = Uuid::now_v7();
        state.work_loop.run.task_run_id = Some(scenario_id);
        state.work_loop.last_completed_task_id = Some("focusa-o8vn".to_string());

        state.telemetry.total_prompt_tokens = 1200;
        state.telemetry.total_completion_tokens = 420;
        state.telemetry.verification_result_events = 2;
        state.telemetry.secondary_loop_useful_events = 1;
        state.telemetry.secondary_loop_low_quality_events = 1;
        state.telemetry.secondary_loop_archived_events = 3;
        state.telemetry.secondary_loop_ledger = vec![
            sample_ledger_entry("proposal-1", "promoted", "trace-1"),
            sample_ledger_entry("proposal-2", "rejected", "trace-2"),
        ];

        let bundle = secondary_loop_eval_bundle_for_status(&state, &state.work_loop);

        assert_eq!(
            bundle.get("task_id").and_then(Value::as_str),
            Some("focusa-o8vn")
        );
        let scenario_id_str = scenario_id.to_string();
        assert_eq!(
            bundle.get("scenario_id").and_then(Value::as_str),
            Some(scenario_id_str.as_str())
        );
        assert_eq!(
            bundle
                .get("secondary_loop_kind_invoked")
                .and_then(Value::as_str),
            Some("continuous_turn_outcome_quality")
        );

        let trace_handles = bundle
            .get("trace_handles")
            .and_then(Value::as_array)
            .expect("trace handles");
        assert_eq!(trace_handles.len(), 2);
        assert!(
            trace_handles
                .iter()
                .any(|value| value.as_str() == Some("trace://trace-1"))
        );
        assert!(
            trace_handles
                .iter()
                .any(|value| value.as_str() == Some("trace://trace-2"))
        );

        let promoted = bundle
            .get("promotion_rejection_archival_result")
            .and_then(|v| v.get("promoted"))
            .and_then(Value::as_u64);
        let rejected = bundle
            .get("promotion_rejection_archival_result")
            .and_then(|v| v.get("rejected"))
            .and_then(Value::as_u64);
        let archived = bundle
            .get("promotion_rejection_archival_result")
            .and_then(|v| v.get("archived"))
            .and_then(Value::as_u64);
        assert_eq!(promoted, Some(1));
        assert_eq!(rejected, Some(1));
        assert_eq!(archived, Some(3));

        assert_eq!(
            bundle
                .get("latency_token_cost_impact")
                .and_then(|v| v.get("total_prompt_tokens"))
                .and_then(Value::as_u64),
            Some(1200)
        );
        assert_eq!(
            bundle
                .get("latency_token_cost_impact")
                .and_then(|v| v.get("total_completion_tokens"))
                .and_then(Value::as_u64),
            Some(420)
        );

        let ledger_refs = bundle
            .get("ledger_refs")
            .and_then(Value::as_array)
            .expect("ledger refs");
        assert_eq!(ledger_refs.len(), 2);
        assert!(
            ledger_refs
                .iter()
                .any(|value| value.as_str() == Some("proposal-1"))
        );
        assert!(
            ledger_refs
                .iter()
                .any(|value| value.as_str() == Some("proposal-2"))
        );
    }

    #[test]
    fn secondary_loop_eval_bundle_tracks_extended_outcome_classes() {
        let mut state = focusa_core::types::FocusaState::default();
        state.telemetry.secondary_loop_archived_events = 2;
        state.telemetry.secondary_loop_ledger = vec![
            sample_ledger_entry("proposal-1", "deferred_for_review", "trace-1"),
            sample_ledger_entry("proposal-2", "archived_failed_attempt", "trace-2"),
        ];

        let bundle = secondary_loop_eval_bundle_for_status(&state, &state.work_loop);

        let outcome = bundle
            .get("promotion_rejection_archival_result")
            .expect("outcome summary");
        assert_eq!(
            outcome.get("deferred_for_review").and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            outcome
                .get("archived_failed_attempt")
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(outcome.get("archived").and_then(Value::as_u64), Some(3));
    }

    #[test]
    fn secondary_loop_acceptance_hooks_surface_controlled_run_proofs() {
        let mut state = focusa_core::types::FocusaState::default();
        state.telemetry.subject_hijack_occurred_events = 1;
        state.telemetry.secondary_loop_archived_events = 1;
        state.telemetry.trace_events = vec![
            sample_secondary_quality_trace("continue", "useful", false),
            sample_secondary_quality_trace("continue", "useful", false),
            sample_secondary_quality_trace("suppress", "low_quality", true),
        ];
        state.telemetry.secondary_loop_ledger = vec![
            sample_ledger_entry("proposal-1", "promoted", "trace-1"),
            sample_ledger_entry("proposal-2", "deferred_for_review", "trace-2"),
            sample_ledger_entry("proposal-3", "archived_failed_attempt", "trace-3"),
        ];

        let hooks = secondary_loop_acceptance_hooks_for_status(&state);
        let evidence_counts = hooks.get("evidence_counts").expect("evidence counts");

        assert_eq!(
            hooks
                .get("bounded_improvement_over_no_secondary_baseline")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            hooks
                .get("irrelevant_secondary_suggestion_suppressed")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            hooks
                .get("verification_rejection_observed")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            hooks
                .get("decay_or_archival_observed")
                .and_then(Value::as_bool),
            Some(true)
        );

        assert_eq!(
            evidence_counts
                .get("quality_trace_events")
                .and_then(Value::as_u64),
            Some(3)
        );
        assert_eq!(
            evidence_counts
                .get("suppressed_irrelevant_suggestions")
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            evidence_counts
                .get("rejected_or_deferred_outcomes")
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            evidence_counts
                .get("archived_outcomes")
                .and_then(Value::as_u64),
            Some(2)
        );
        assert_eq!(
            evidence_counts
                .get("comparative_improvement_pairs")
                .and_then(Value::as_u64),
            Some(1)
        );
    }

    #[test]
    fn secondary_loop_acceptance_hooks_default_to_false_without_evidence() {
        let state = focusa_core::types::FocusaState::default();
        let hooks = secondary_loop_acceptance_hooks_for_status(&state);

        assert_eq!(
            hooks
                .get("bounded_improvement_over_no_secondary_baseline")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            hooks
                .get("irrelevant_secondary_suggestion_suppressed")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            hooks
                .get("verification_rejection_observed")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            hooks
                .get("decay_or_archival_observed")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            hooks
                .get("evidence_counts")
                .and_then(|value| value.get("comparative_improvement_pairs"))
                .and_then(Value::as_u64),
            Some(0)
        );
    }

    #[test]
    fn secondary_loop_closure_replay_evidence_surfaces_current_task_pair() {
        let mut state = focusa_core::types::FocusaState::default();
        let task_run_id = Uuid::now_v7();
        state.work_loop.run.task_run_id = Some(task_run_id);
        state.work_loop.current_task = Some(sample_current_task("focusa-live"));
        state.work_loop.last_completed_task_id = Some("focusa-prev".to_string());

        let summary = focusa_core::replay::SecondaryLoopComparativeReplaySummary {
            replay_events_scanned: 22,
            secondary_loop_outcome_events: 5,
            promoted_outcomes: 2,
            rejected_outcomes: 2,
            deferred_for_review_outcomes: 1,
            archived_failed_attempt_outcomes: 0,
            comparative_improvement_pairs: 1,
            task_pairs: vec![focusa_core::replay::SecondaryLoopComparativePair {
                correlation_id: task_run_id.to_string(),
                promoted_outcomes: 1,
                non_promoted_outcomes: 1,
                comparative_improvement_observed: true,
            }],
        };

        let evidence =
            secondary_loop_closure_replay_evidence_for_status(&state.work_loop, &summary);

        let task_run_id_str = task_run_id.to_string();
        assert_eq!(
            evidence.get("current_task_pair_id").and_then(Value::as_str),
            Some(task_run_id_str.as_str())
        );
        assert_eq!(
            evidence
                .get("current_task_pair_observed")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            evidence
                .get("current_task_pair_promoted_outcomes")
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            evidence
                .get("current_task_pair_non_promoted_outcomes")
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            evidence
                .get("correlation_candidates")
                .and_then(Value::as_array)
                .map(|entries| entries.len()),
            Some(3)
        );
    }

    #[test]
    fn secondary_loop_closure_replay_evidence_defaults_fail_closed_without_match() {
        let mut state = focusa_core::types::FocusaState::default();
        state.work_loop.current_task = Some(sample_current_task("focusa-live"));
        state.work_loop.last_completed_task_id = Some("focusa-prev".to_string());

        let summary = focusa_core::replay::SecondaryLoopComparativeReplaySummary {
            replay_events_scanned: 9,
            secondary_loop_outcome_events: 1,
            promoted_outcomes: 1,
            rejected_outcomes: 0,
            deferred_for_review_outcomes: 0,
            archived_failed_attempt_outcomes: 0,
            comparative_improvement_pairs: 0,
            task_pairs: vec![focusa_core::replay::SecondaryLoopComparativePair {
                correlation_id: "unrelated".to_string(),
                promoted_outcomes: 1,
                non_promoted_outcomes: 0,
                comparative_improvement_observed: false,
            }],
        };

        let evidence =
            secondary_loop_closure_replay_evidence_for_status(&state.work_loop, &summary);

        assert_eq!(
            evidence
                .get("current_task_pair_observed")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(evidence.get("current_task_pair_id"), Some(&Value::Null));
        assert_eq!(
            evidence
                .get("correlation_candidates")
                .and_then(Value::as_array)
                .map(|entries| entries.len()),
            Some(2)
        );
    }

    #[test]
    fn secondary_loop_replay_consumer_payload_surfaces_ok_state() {
        let mut state = focusa_core::types::FocusaState::default();
        let task_run_id = Uuid::now_v7();
        state.work_loop.run.task_run_id = Some(task_run_id);
        state.work_loop.current_task = Some(sample_current_task("focusa-live"));

        let replay_summary = Ok(focusa_core::replay::SecondaryLoopComparativeReplaySummary {
            replay_events_scanned: 22,
            secondary_loop_outcome_events: 5,
            promoted_outcomes: 2,
            rejected_outcomes: 2,
            deferred_for_review_outcomes: 1,
            archived_failed_attempt_outcomes: 0,
            comparative_improvement_pairs: 1,
            task_pairs: vec![focusa_core::replay::SecondaryLoopComparativePair {
                correlation_id: task_run_id.to_string(),
                promoted_outcomes: 1,
                non_promoted_outcomes: 1,
                comparative_improvement_observed: true,
            }],
        });

        let payload =
            secondary_loop_replay_consumer_payload_for_status(&state.work_loop, &replay_summary);

        let task_run_id_str = task_run_id.to_string();
        assert_eq!(payload.get("status").and_then(Value::as_str), Some("ok"));
        assert_eq!(
            payload
                .get("secondary_loop_replay_comparative")
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str),
            Some("ok")
        );
        assert_eq!(
            payload
                .get("secondary_loop_replay_comparative")
                .and_then(|value| value.get("summary"))
                .and_then(|value| value.get("comparative_improvement_pairs"))
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            payload
                .get("secondary_loop_closure_replay_evidence")
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str),
            Some("ok")
        );
        assert_eq!(
            payload
                .get("secondary_loop_closure_replay_evidence")
                .and_then(|value| value.get("evidence"))
                .and_then(|value| value.get("current_task_pair_observed"))
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            payload
                .get("secondary_loop_closure_replay_evidence")
                .and_then(|value| value.get("evidence"))
                .and_then(|value| value.get("current_task_pair_id"))
                .and_then(Value::as_str),
            Some(task_run_id_str.as_str())
        );
    }

    #[test]
    fn secondary_loop_replay_consumer_payload_surfaces_error_state_fail_closed() {
        let mut state = focusa_core::types::FocusaState::default();
        state.work_loop.current_task = Some(sample_current_task("focusa-live"));

        let replay_summary: Result<
            focusa_core::replay::SecondaryLoopComparativeReplaySummary,
            String,
        > = Err("replay unavailable".to_string());

        let payload =
            secondary_loop_replay_consumer_payload_for_status(&state.work_loop, &replay_summary);

        assert_eq!(payload.get("status").and_then(Value::as_str), Some("error"));
        assert_eq!(
            payload.get("error").and_then(Value::as_str),
            Some("replay unavailable")
        );
        assert_eq!(
            payload
                .get("secondary_loop_replay_comparative")
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str),
            Some("error")
        );
        assert_eq!(
            payload
                .get("secondary_loop_replay_comparative")
                .and_then(|value| value.get("error"))
                .and_then(Value::as_str),
            Some("replay unavailable")
        );
        assert_eq!(
            payload
                .get("secondary_loop_closure_replay_evidence")
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str),
            Some("error")
        );
        assert_eq!(
            payload
                .get("secondary_loop_closure_replay_evidence")
                .and_then(|value| value.get("error"))
                .and_then(Value::as_str),
            Some("replay unavailable")
        );
        assert_eq!(
            payload
                .get("secondary_loop_closure_replay_evidence")
                .and_then(|value| value.get("evidence")),
            None
        );
    }

    #[test]
    fn secondary_loop_continuity_gate_surfaces_open_state_when_replay_ok() {
        let mut state = focusa_core::types::FocusaState::default();
        let task_run_id = Uuid::now_v7();
        state.work_loop.run.task_run_id = Some(task_run_id);
        state.work_loop.current_task = Some(sample_current_task("focusa-live"));

        let replay_summary = Ok(focusa_core::replay::SecondaryLoopComparativeReplaySummary {
            replay_events_scanned: 22,
            secondary_loop_outcome_events: 5,
            promoted_outcomes: 2,
            rejected_outcomes: 2,
            deferred_for_review_outcomes: 1,
            archived_failed_attempt_outcomes: 0,
            comparative_improvement_pairs: 1,
            task_pairs: vec![focusa_core::replay::SecondaryLoopComparativePair {
                correlation_id: task_run_id.to_string(),
                promoted_outcomes: 1,
                non_promoted_outcomes: 1,
                comparative_improvement_observed: true,
            }],
        });

        let replay_consumer =
            secondary_loop_replay_consumer_payload_for_status(&state.work_loop, &replay_summary);
        let gate = secondary_loop_continuity_gate_for_status(&replay_summary, &replay_consumer);

        assert_eq!(gate.get("state").and_then(Value::as_str), Some("open"));
        assert_eq!(
            gate.get("fail_closed").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            gate.get("current_task_pair_observed")
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn secondary_loop_continuity_gate_surfaces_fail_closed_when_replay_error() {
        let state = focusa_core::types::FocusaState::default();
        let replay_summary: Result<
            focusa_core::replay::SecondaryLoopComparativeReplaySummary,
            String,
        > = Err("replay unavailable".to_string());

        let replay_consumer =
            secondary_loop_replay_consumer_payload_for_status(&state.work_loop, &replay_summary);
        let gate = secondary_loop_continuity_gate_for_status(&replay_summary, &replay_consumer);

        assert_eq!(
            gate.get("state").and_then(Value::as_str),
            Some("fail-closed")
        );
        assert_eq!(gate.get("fail_closed").and_then(Value::as_bool), Some(true));
        assert_eq!(
            gate.get("reason").and_then(Value::as_str),
            Some("replay_consumer_error")
        );
    }

    #[test]
    fn secondary_loop_closure_bundle_surfaces_replay_gate_contract() {
        let state = focusa_core::types::FocusaState::default();
        let replay_summary: Result<
            focusa_core::replay::SecondaryLoopComparativeReplaySummary,
            String,
        > = Err("replay unavailable".to_string());

        let bundle =
            secondary_loop_closure_bundle_for_status(&state, &state.work_loop, &replay_summary);

        assert_eq!(bundle.get("status").and_then(Value::as_str), Some("ok"));
        assert_eq!(bundle.get("doc").and_then(Value::as_str), Some("78"));
        assert_eq!(
            bundle
                .get("secondary_loop_replay_consumer")
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str),
            Some("error")
        );
        assert_eq!(
            bundle
                .get("secondary_loop_continuity_gate")
                .and_then(|value| value.get("state"))
                .and_then(Value::as_str),
            Some("fail-closed")
        );
        assert_eq!(
            bundle
                .get("evidence_contract")
                .and_then(|value| value.get("replay_consumer_route"))
                .and_then(Value::as_str),
            Some("/v1/work-loop/replay/closure-evidence")
        );
    }

    #[test]
    fn secondary_loop_eval_bundle_prefers_current_task_when_bound() {
        let mut state = focusa_core::types::FocusaState::default();
        state.work_loop.last_completed_task_id = Some("focusa-old".to_string());
        state.work_loop.current_task = Some(sample_current_task("focusa-live"));

        let bundle = secondary_loop_eval_bundle_for_status(&state, &state.work_loop);

        assert_eq!(
            bundle.get("task_id").and_then(Value::as_str),
            Some("focusa-live")
        );
    }
}
