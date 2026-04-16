//! Continuous work loop control/status routes.

use crate::routes::permissions::{forbid, permission_context};
use crate::server::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::{get, post}};
use chrono::Utc;
use focusa_core::types::{Action, ProjectRunId, SpecLinkedTaskPacket, WorkLoopPolicy, WorkLoopPolicyOverrides, WorkLoopPreset, WorkLoopStatus};
use serde::Deserialize;
use serde_json::{Value, json};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

const WRITER_HEADER: &str = "x-focusa-writer-id";
const APPROVAL_HEADER: &str = "x-focusa-approval";

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

fn conflict(message: impl Into<String>, active_writer: Option<String>) -> (StatusCode, Json<Value>) {
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
        Some(existing) if existing != writer_id => {
            Err(conflict("continuous work loop already claimed by another writer", guard.clone()))
        }
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
        Some(existing) if existing != writer_id => {
            Err(conflict("continuous work loop claimed by another writer", guard.clone()))
        }
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
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .await
    {
        Ok(top) if top.status.success() => top,
        Ok(_) => {
            return json!({
                "git_available": true,
                "in_worktree": false,
                "clean": false,
            })
        }
        Err(e) => {
            return json!({
                "git_available": false,
                "in_worktree": false,
                "clean": false,
                "error": e.to_string(),
            })
        }
    };

    let repo_root = String::from_utf8_lossy(&top.stdout).trim().to_string();
    let status = match Command::new("git")
        .args(["status", "--porcelain"])
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
            })
        }
        Err(e) => {
            return json!({
                "git_available": true,
                "in_worktree": true,
                "clean": false,
                "repo_root": repo_root,
                "error": e.to_string(),
            })
        }
    };

    let dirty = String::from_utf8_lossy(&status.stdout)
        .lines()
        .take(10)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let diff_stat = Command::new("git")
        .args(["diff", "--stat"])
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

async fn alternate_ready_work_snapshot(current_task: Option<&focusa_core::types::SpecLinkedTaskPacket>) -> Value {
    let Some(task) = current_task else {
        return json!({ "exists": false });
    };
    let Some(parent_work_item_id) = task.dependencies.first() else {
        return json!({ "exists": false });
    };

    let output = match Command::new("bd")
        .args(["show", parent_work_item_id, "--json"])
        .output()
        .await
    {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            return json!({
                "exists": false,
                "error": String::from_utf8_lossy(&output.stderr).trim().to_string(),
            })
        }
        Err(e) => {
            return json!({ "exists": false, "error": e.to_string() })
        }
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
            && matches!(dep.get("status").and_then(Value::as_str), Some("open") | Some("in_progress"))
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
    let linked_spec_requirement = current_task
        .and_then(|task| task.linked_spec_refs.first().cloned());
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
        && !matches!(blocker_class, focusa_core::types::BlockerClass::Governance | focusa_core::types::BlockerClass::Permission);

    let (exact_operator_decision_needed, recommended_next_action) = if self_recovery_allowed {
        (
            "no immediate operator decision required unless retry budget is exhausted".to_string(),
            format!("retry self-recovery on the blocked task (remaining retry budget: {retries_remaining})"),
        )
    } else if wl.pause_flags.operator_override_active {
        (
            "confirm override intent and choose whether to resume, pause longer, or stop".to_string(),
            "honor operator override before any further autonomous step".to_string(),
        )
    } else if wl.pause_flags.destructive_confirmation_required {
        (
            "approve or reject the destructive/high-risk action".to_string(),
            "provide explicit approval or redirect to a safer path".to_string(),
        )
    } else if wl.pause_flags.governance_decision_pending || blocker_class == focusa_core::types::BlockerClass::Governance {
        (
            "resolve the governance-sensitive decision or amend policy/spec".to_string(),
            "choose the governing outcome, then resume with updated authority".to_string(),
        )
    } else if alternate_ready_work.get("exists").and_then(Value::as_bool) == Some(true) {
        (
            "decide whether to defer the blocked task and switch to alternate ready work".to_string(),
            "defer the blocked task and continue with the alternate ready item".to_string(),
        )
    } else {
        (
            "review blocker package because no valid ready work remains".to_string(),
            "escalate to the operator because retries and alternate ready work are exhausted".to_string(),
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

async fn defer_work_item_for_alternate_switch(work_item_id: &str, reason: &str) {
    let note = format!(
        "Deferred by continuous loop for alternate-ready work: {}",
        reason.chars().take(220).collect::<String>()
    );
    let _ = Command::new("bd")
        .args(["update", work_item_id, "--defer", "+1d", "--append-notes", &note])
        .output()
        .await;
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
                        .or_else(|| part.get("content").and_then(Value::as_str).map(|s| s.to_string()))
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

async fn dispatch_pi_prompt(state: &Arc<AppState>, message: String) -> Result<(), (StatusCode, Json<Value>)> {
    let mut guard = state.pi_rpc_session.lock().await;
    let Some(session) = guard.as_mut() else {
        return Err(bad_request("pi rpc driver not active"));
    };
    let msg = json!({"id": format!("prompt-{}", Uuid::now_v7()), "type":"prompt", "message": message}).to_string() + "\n";
    session.stdin.write_all(msg.as_bytes()).await.map_err(|e| bad_request(format!("failed writing prompt: {e}")))?;
    Ok(())
}

async fn maybe_auto_advance_from_blocked(state: &Arc<AppState>, reason: &str) -> Result<bool, (StatusCode, Json<Value>)> {
    let (enabled, status, current_task, blocker_reason) = {
        let focusa = state.focusa.read().await;
        (
            focusa.work_loop.enabled,
            focusa.work_loop.status,
            focusa.work_loop.current_task.clone(),
            focusa.work_loop
                .last_blocker_reason
                .clone()
                .unwrap_or_else(|| "blocked in continuous loop".to_string()),
        )
    };

    if !enabled || status != WorkLoopStatus::Blocked {
        return Ok(false);
    }

    let Some(task) = current_task else {
        return Ok(false);
    };

    let parent_work_item_id = task
        .tranche_id
        .clone()
        .unwrap_or_else(|| task.work_item_id.clone());

    defer_work_item_for_alternate_switch(&task.work_item_id, &blocker_reason).await;

    state
        .command_tx
        .send(Action::SelectNextContinuousSubtask { parent_work_item_id })
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

async fn maybe_select_global_ready_work_item(state: &Arc<AppState>) -> Result<bool, (StatusCode, Json<Value>)> {
    let output = Command::new("bd")
        .args(["ready"])
        .output()
        .await
        .map_err(|e| bad_request(format!("failed to run bd ready: {e}")))?;
    if !output.status.success() {
        return Ok(false);
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let selected = text
        .split_whitespace()
        .map(|token| token.trim_matches(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '.')))
        .find(|token| token.starts_with("focusa-") && token.len() > 7)
        .map(str::to_string);

    let Some(work_item_id) = selected else {
        return Ok(false);
    };

    let show_output = Command::new("bd")
        .args(["show", &work_item_id, "--json"])
        .output()
        .await
        .map_err(|e| bad_request(format!("failed to inspect ready work item: {e}")))?;

    let title = if show_output.status.success() {
        serde_json::from_slice::<Vec<Value>>(&show_output.stdout)
            .ok()
            .and_then(|v| v.first().cloned())
            .and_then(|v| v.get("title").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| "untitled work item".to_string())
    } else {
        "untitled work item".to_string()
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
                linked_spec_refs: vec!["docs/79-focusa-governed-continuous-work-loop.md".to_string()],
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

    let (enabled, status, task_run_id, current_task, mission, focus, last_checkpoint_id, last_turn_requested_at, status_heartbeat_ms) = {
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
            focusa.work_loop.run.last_checkpoint_id.map(|v| v.to_string()),
            focusa.work_loop.last_turn_requested_at,
            focusa.work_loop.policy.status_heartbeat_ms,
        )
    };

    if !enabled || !matches!(status, WorkLoopStatus::SelectingReadyWork | WorkLoopStatus::Idle | WorkLoopStatus::AwaitingHarnessTurn | WorkLoopStatus::AdvancingTask | WorkLoopStatus::EvaluatingOutcome) {
        return Ok(false);
    }

    if current_task.is_none() {
        if maybe_select_global_ready_work_item(state).await? {
            let refreshed_task = {
                let focusa = state.focusa.read().await;
                focusa.work_loop.current_task.clone()
            };
            if let Some(task) = refreshed_task {
                state.command_tx.send(Action::RequestNextContinuousTurn {
                    task_run_id,
                    work_item_id: Some(task.work_item_id.clone()),
                    reason: "re-selected work after unassigned turn state".to_string(),
                }).await.map_err(|e| (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("dispatch failed: {e}") })),
                ))?;

                let prompt = render_continuous_turn_prompt(&task, mission, focus, last_checkpoint_id);
                dispatch_pi_prompt(state, prompt).await?;
                return Ok(true);
            }
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

    state.command_tx.send(Action::RequestNextContinuousTurn {
        task_run_id,
        work_item_id: Some(task.work_item_id.clone()),
        reason: reason.to_string(),
    }).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": format!("dispatch failed: {e}") })),
    ))?;

    let prompt = render_continuous_turn_prompt(&task, mission, focus, last_checkpoint_id);
    dispatch_pi_prompt(state, prompt).await?;
    Ok(true)
}

fn budget_remaining_for_status(wl: &focusa_core::types::WorkLoopState) -> Value {
    let policy = &wl.policy;
    let elapsed_ms = wl.enabled_at.map(|ts| (Utc::now() - ts).num_milliseconds().max(0) as u64).unwrap_or(0);
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
    if task.allowed_scope.iter().any(|scope| scope.to_ascii_lowercase().contains("governance"))
        || matches!(wl.last_blocker_class, Some(focusa_core::types::BlockerClass::Governance | focusa_core::types::BlockerClass::Permission))
        || ["delete", "drop", "remove", "rename", "migrate", "rewrite", "destructive", "governance"].iter().any(|needle| title.contains(needle)) {
        "high"
    } else if matches!(task.task_class, focusa_core::types::TaskClass::Architecture | focusa_core::types::TaskClass::Integration) {
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
        guard.as_ref().map(|session| json!({
            "adapter": "pi-rpc",
            "session_id": session.session_id,
            "cwd": session.cwd,
            "uptime_ms": session.started_at.elapsed().as_millis(),
        }))
    };
    let worktree = worktree_status_snapshot().await;
    let alternate_ready_work = alternate_ready_work_snapshot(wl.current_task.as_ref()).await;
    let blocker_package = build_blocker_package(wl, alternate_ready_work.clone());
    let transport_health = transport_health_for_status(wl);
    let budget_remaining = budget_remaining_for_status(wl);
    let resume_payload = resume_payload_for_status(wl);
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
        "budget_remaining": budget_remaining,
        "resume_payload": resume_payload,
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
    let policy = WorkLoopPolicy::with_overrides(preset, payload.policy_overrides.unwrap_or_default());
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
        state.command_tx.send(Action::SelectNextContinuousSubtask {
            parent_work_item_id,
        }).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("dispatch failed: {e}") })),
            )
        })?;
        let _ = maybe_dispatch_continuous_turn_prompt(&state, "continuous work enabled with ready work selected").await;
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
        return Err(conflict("continuous work loop claimed by another writer", active_writer));
    }

    state.command_tx.send(Action::PauseContinuousWork {
        reason: payload.reason.unwrap_or_default(),
    }).await.map_err(|e| {
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

    state.command_tx.send(Action::ResumeContinuousWork {
        reason: payload.reason.unwrap_or_default(),
    }).await.map_err(|e| {
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

    let (current_task_id, blocked, blocker_reason) = {
        let s = state.focusa.read().await;
        (
            s.work_loop.current_task.as_ref().map(|t| t.work_item_id.clone()),
            s.work_loop.status == WorkLoopStatus::Blocked,
            s.work_loop.last_blocker_reason.clone().unwrap_or_else(|| "blocked in continuous loop".to_string()),
        )
    };
    if blocked {
        if let Some(work_item_id) = current_task_id.as_deref() {
            defer_work_item_for_alternate_switch(work_item_id, &blocker_reason).await;
        }
    }

    state.command_tx.send(Action::SelectNextContinuousSubtask {
        parent_work_item_id: payload.parent_work_item_id,
    }).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("dispatch failed: {e}") })),
        )
    })?;
    let _ = maybe_dispatch_continuous_turn_prompt(&state, "ready work selected for continuous execution").await;

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

    state.command_tx.send(Action::SetContinuousDecisionContext {
        current_ask: payload.current_ask,
        ask_kind: payload.ask_kind,
        scope_kind: payload.scope_kind,
        carryover_policy: payload.carryover_policy,
        excluded_context_reason: payload.excluded_context_reason,
        excluded_context_labels: payload.excluded_context_labels,
        source_turn_id: payload.source_turn_id,
        operator_steering_detected: payload.operator_steering_detected,
    }).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("dispatch failed: {e}") })))
    })?;

    Ok(Json(json!({ "status": "accepted", "writer_id": writer_id })))
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
    let mut cmd = Command::new("pi");
    cmd.args(["--mode", "rpc", "--no-session"])
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
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("failed to spawn pi rpc: {e}")})))
    })?;
    let stdin = child.stdin.take().ok_or_else(|| bad_request("pi rpc stdin unavailable"))?;
    let stdout = child.stdout.take().ok_or_else(|| bad_request("pi rpc stdout unavailable"))?;

    let state_for_events = state.clone();
    let command_tx = state.command_tx.clone();
    let attach_session_id = session_id.clone();
    tokio::spawn(async move {
        let _ = command_tx.send(Action::AttachContinuousTransportSession {
            adapter: "pi-rpc".to_string(),
            session_id: attach_session_id.clone(),
        }).await;
        let mut seq: u64 = 1;
        let mut last_assistant_output = String::new();
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let parsed: Value = serde_json::from_str(&line).unwrap_or_else(|_| json!({"type":"raw","summary":line}));
            let kind = parsed.get("type").and_then(Value::as_str).unwrap_or("unknown").to_string();
            if kind == "turn_start" || kind == "agent_start" {
                last_assistant_output.clear();
            }
            if kind == "message_update" {
                if let Some(delta) = parsed.get("assistantMessageEvent")
                    .and_then(|v| v.get("delta"))
                    .and_then(Value::as_str) {
                    last_assistant_output.push_str(delta);
                }
            }
            if (kind == "turn_end" || kind == "agent_end") && last_assistant_output.is_empty() {
                if let Some(text) = parsed.get("message").and_then(extract_assistant_text) {
                    last_assistant_output = text;
                } else if let Some(text) = parsed.get("messages")
                    .and_then(Value::as_array)
                    .and_then(|msgs| msgs.iter().rev().find_map(extract_assistant_text)) {
                    last_assistant_output = text;
                }
            }
            let summary = parsed.get("message")
                .and_then(|m| m.get("role").and_then(Value::as_str).or_else(|| m.as_str()))
                .map(|s| s.to_string())
                .or_else(|| parsed.get("assistantMessageEvent").and_then(|v| v.get("type")).and_then(Value::as_str).map(|s| s.to_string()))
                .or_else(|| parsed.get("command").and_then(Value::as_str).map(|s| format!("response:{s}")))
                .or_else(|| Some(kind.clone()));
            let _ = command_tx.send(Action::IngestContinuousTransportEvent {
                sequence: seq,
                kind: kind.clone(),
                session_id: Some(attach_session_id.clone()),
                turn_id: None,
                summary,
            }).await;
            if kind == "turn_end" || kind == "agent_end" {
                let current_task = {
                    let focusa = state_for_events.focusa.read().await;
                    focusa.work_loop.current_task.clone()
                };
                if let Some(task) = current_task {
                    let verification_satisfied = !last_assistant_output.trim().is_empty();
                    let _ = command_tx.send(Action::ObserveContinuousTurnOutcome {
                        task_run_id: None,
                        work_item_id: Some(task.work_item_id.clone()),
                        summary: if last_assistant_output.trim().is_empty() {
                            format!("{kind} without assistant output")
                        } else {
                            format!("{kind} for {}", task.work_item_id)
                        },
                        continue_reason: Some(format!("{kind} observed from pi rpc stream")),
                        verification_satisfied,
                        spec_conformant: true,
                    }).await;
                    let _ = maybe_dispatch_continuous_turn_prompt(&state_for_events, "pi rpc turn_end/agent_end observed and ready work remains").await;
                }
                last_assistant_output.clear();
            }
            seq += 1;
        }
    });

    *guard = Some(crate::server::PiRpcSession {
        child,
        stdin,
        session_id: session_id.clone(),
        cwd: payload.cwd.clone(),
        started_at: std::time::Instant::now(),
    });

    Ok(Json(json!({"status":"accepted","adapter":"pi-rpc","session_id":session_id})))
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
    session.stdin.write_all(msg.to_string().as_bytes()).await.map_err(|e| bad_request(format!("failed writing prompt: {e}")))?;
    session.stdin.write_all(b"\n").await.map_err(|e| bad_request(format!("failed writing newline: {e}")))?;
    Ok(Json(json!({"status":"accepted","session_id":session.session_id})))
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
    session.stdin.write_all(msg.as_bytes()).await.map_err(|e| bad_request(format!("failed writing abort: {e}")))?;
    Ok(Json(json!({"status":"accepted","session_id":session.session_id})))
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
    Ok(Json(json!({"status":"accepted","session_id":session.session_id})))
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
    state.command_tx.send(Action::AttachContinuousTransportSession {
        adapter: payload.adapter,
        session_id: payload.session_id,
    }).await.map_err(|e| {
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
    state.command_tx.send(Action::AbortContinuousTransportSession {
        reason: payload.reason.unwrap_or_else(|| "abort requested".to_string()),
    }).await.map_err(|e| {
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
    state.command_tx.send(Action::IngestContinuousTransportEvent {
        sequence: payload.sequence,
        kind: payload.kind,
        session_id: payload.session_id,
        turn_id: payload.turn_id,
        summary: payload.summary,
    }).await.map_err(|e| {
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
    state.command_tx.send(Action::SetContinuousPauseFlags {
        destructive_confirmation_required: payload.destructive_confirmation_required,
        governance_decision_pending: payload.governance_decision_pending,
        operator_override_active: payload.operator_override_active,
        reason: payload.reason,
    }).await.map_err(|e| {
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
    state.command_tx.send(Action::SetDelegatedContinuousAuthorship {
        delegate_id: Some(payload.delegate_id),
        scope: Some(payload.scope),
        amendment_summary: payload.amendment_summary,
    }).await.map_err(|e| {
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
    state.command_tx.send(Action::SetDelegatedContinuousAuthorship {
        delegate_id: None,
        scope: None,
        amendment_summary: payload.reason,
    }).await.map_err(|e| {
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
    state.command_tx.send(Action::MarkContinuousLoopTransportDegraded {
        reason: payload.reason.unwrap_or_else(|| "transport degraded".to_string()),
    }).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("dispatch failed: {e}") })),
        )
    })?;

    Ok(Json(json!({ "ok": true, "writer_id": writer_id })))
}

async fn checkpoints(State(state): State<Arc<AppState>>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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
    state.command_tx.send(Action::CheckpointContinuousLoop {
        checkpoint_id: payload.checkpoint_id.unwrap_or_else(Uuid::now_v7),
        summary: payload.summary,
    }).await.map_err(|e| {
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
    state.command_tx.send(Action::StopContinuousWork {
        reason: payload.reason.unwrap_or_default(),
    }).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("dispatch failed: {e}") })),
        )
    })?;

    Ok(Json(json!({ "ok": true, "released_writer": released_writer })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/work-loop", get(status))
        .route("/v1/work-loop/status", get(status))
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
        .route("/v1/work-loop/delegation/clear", post(clear_delegated_authorship))
        .route("/v1/work-loop/degraded", post(transport_degraded))
        .route("/v1/work-loop/checkpoints", get(checkpoints))
        .route("/v1/work-loop/checkpoint", post(checkpoint))
        .route("/v1/work-loop/stop", post(stop))
}
