use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::{
    tool_result::{FailureClass, ToolResultV1, ToolStatus},
    types::{
        Action, FocusaEvent, ProviderNeutralTaskPlanRecord, ProviderNeutralTaskRecord,
        SpecWorkbenchStatus, TaskPlanStatus,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, sync::Arc, time::Duration};

type ApiError = (StatusCode, Json<Box<ToolResultV1>>);
const ENDPOINT: &str = "/v1/task-plans/mutate";
const TOOL: &str = "focusa_task_plan_mutate";
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    project_root: String,
    continuity_id: String,
    attachment_id: String,
    #[serde(default)]
    task_plan_id: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationAction {
    Open,
    UpsertTask,
    RemoveTask,
    Preview,
    Approve,
}
#[derive(Debug, Deserialize)]
pub struct MutationRequest {
    project_root: String,
    continuity_id: String,
    attachment_id: String,
    idempotency_key: String,
    expected_state_version: u64,
    expected_plan_revision: u64,
    action: MutationAction,
    #[serde(default)]
    task_plan_id: Option<String>,
    #[serde(default)]
    workbench_session_id: Option<String>,
    #[serde(default)]
    task: Option<ProviderNeutralTaskRecord>,
    #[serde(default)]
    task_id: Option<String>,
    #[serde(default)]
    preview_token: Option<String>,
    #[serde(default)]
    approved_by: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct ListResponse {
    schema: &'static str,
    state_version: u64,
    task_plans: Vec<ProviderNeutralTaskPlanRecord>,
}
#[derive(Debug, Serialize)]
pub struct MutationResponse {
    schema: &'static str,
    state_version: u64,
    replayed: bool,
    materialization_allowed: bool,
    task_plan: ProviderNeutralTaskPlanRecord,
    evidence_ref: String,
    receipt_ref: String,
    tool_result: ToolResultV1,
}
fn scoped(x: &ProviderNeutralTaskPlanRecord, p: &str, c: &str, a: &str) -> bool {
    x.project_root == p && x.continuity_id == c && x.attachment_id == a
}
fn hash(prefix: &str, parts: &[&str]) -> String {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p.as_bytes());
        h.update([0]);
    }
    format!("{prefix}:{}", hex::encode(h.finalize())[..24].to_string())
}
fn fail(
    code: StatusCode,
    status: ToolStatus,
    class: FailureClass,
    msg: impl Into<String>,
) -> ApiError {
    let mut result = ToolResultV1::failure(status, class, msg.into());
    result.tool = Some(TOOL.into());
    result.family = Some("provider_neutral_task_plan".into());
    result.endpoint = Some(ENDPOINT.into());
    (code, Json(Box::new(result)))
}
fn response(x: ProviderNeutralTaskPlanRecord, v: u64, replayed: bool) -> MutationResponse {
    let receipt = x.receipt_refs.last().cloned().unwrap_or_default();
    MutationResponse {
        schema: "focusa.provider_neutral_task_plan_mutation_result.v1",
        state_version: v,
        replayed,
        materialization_allowed: matches!(x.status, TaskPlanStatus::Approved),
        evidence_ref: format!("evidence:task-plan:{}:{}", x.task_plan_id, x.state_revision),
        receipt_ref: receipt,
        task_plan: x,
        tool_result: {
            let mut result = ToolResultV1::success(
                ToolStatus::Completed,
                "Canonical provider-neutral task plan revision committed.",
            );
            result.tool = Some(TOOL.into());
            result.family = Some("provider_neutral_task_plan".into());
            result.endpoint = Some(ENDPOINT.into());
            result
        },
    }
}
fn validate(tasks: &[ProviderNeutralTaskRecord]) -> Result<(), String> {
    let ids: BTreeSet<_> = tasks
        .iter()
        .map(|x| x.provider_neutral_id.as_str())
        .collect();
    if ids.len() != tasks.len() {
        return Err("task IDs must be unique".into());
    }
    if tasks.iter().any(|x| {
        x.provider_neutral_id.trim().is_empty()
            || x.title.trim().is_empty()
            || x.description.trim().is_empty()
            || x.linked_spec_sections.is_empty()
            || x.requirement_refs.is_empty()
            || x.acceptance_criteria.is_empty()
            || x.evidence_requirements.is_empty()
            || x.verification_policy_ref.trim().is_empty()
            || x.allowed_scope.is_empty()
            || x.task_class.trim().is_empty()
            || x.closure_kind.trim().is_empty()
            || x.closure_policy_ref.trim().is_empty()
            || x.dependencies
                .iter()
                .any(|d| d == &x.provider_neutral_id || !ids.contains(d.as_str()))
    }) {
        return Err("tasks require identity, Spec/requirement/proof links, policy, scope, and valid dependencies".into());
    }
    let mut done = BTreeSet::new();
    loop {
        let n = done.len();
        for x in tasks {
            if x.dependencies.iter().all(|d| done.contains(d.as_str())) {
                done.insert(x.provider_neutral_id.as_str());
            }
        }
        if done.len() == n {
            break;
        }
    }
    if done.len() != tasks.len() {
        return Err("task dependency graph must be acyclic".into());
    }
    Ok(())
}
pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ListResponse>, ApiError> {
    if q.project_root.trim().is_empty()
        || q.continuity_id.trim().is_empty()
        || q.attachment_id.trim().is_empty()
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ScopeMismatch,
            "exact scope required",
        ));
    }
    let s = state.focusa.read().await;
    let plans = s
        .provider_neutral_task_plans
        .iter()
        .filter(|x| {
            scoped(x, &q.project_root, &q.continuity_id, &q.attachment_id)
                && q.task_plan_id
                    .as_ref()
                    .is_none_or(|id| id == &x.task_plan_id)
        })
        .cloned()
        .collect();
    Ok(Json(ListResponse {
        schema: "focusa.provider_neutral_task_plan_list.v1",
        state_version: s.version,
        task_plans: plans,
    }))
}
pub async fn mutate(
    State(state): State<Arc<AppState>>,
    Json(r): Json<MutationRequest>,
) -> Result<Json<MutationResponse>, ApiError> {
    if r.project_root.trim().is_empty()
        || r.continuity_id.trim().is_empty()
        || r.attachment_id.trim().is_empty()
        || r.idempotency_key.trim().is_empty()
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ScopeMismatch,
            "exact scope and idempotency required",
        ));
    }
    let snap = state.focusa.read().await;
    if snap.version != r.expected_state_version {
        return Err(fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "state version conflict",
        ));
    }
    if let Some(x) = snap.provider_neutral_task_plans.iter().find(|x| {
        scoped(x, &r.project_root, &r.continuity_id, &r.attachment_id)
            && x.idempotency_key == r.idempotency_key
    }) {
        return Ok(Json(response(x.clone(), snap.version, true)));
    }
    let latest = r
        .task_plan_id
        .as_ref()
        .and_then(|id| {
            snap.provider_neutral_task_plans
                .iter()
                .filter(|x| {
                    x.task_plan_id == *id
                        && scoped(x, &r.project_root, &r.continuity_id, &r.attachment_id)
                })
                .max_by_key(|x| x.state_revision)
        })
        .cloned();
    if latest.as_ref().map_or(0, |x| x.state_revision) != r.expected_plan_revision {
        return Err(fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "task plan revision conflict",
        ));
    }
    let now = Utc::now();
    let mut p = if matches!(r.action, MutationAction::Open) {
        if latest.is_some() || r.expected_plan_revision != 0 {
            return Err(fail(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::WriterConflict,
                "task plan already exists",
            ));
        }
        let wid = r.workbench_session_id.as_deref().ok_or_else(|| {
            fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                "workbench_session_id required",
            )
        })?;
        let source = snap
            .spec_workbench_sessions
            .iter()
            .filter(|x| {
                x.workbench_session_id == wid
                    && x.project_root == r.project_root
                    && x.continuity_id == r.continuity_id
                    && x.attachment_id == r.attachment_id
                    && matches!(x.status, SpecWorkbenchStatus::FinalApproved)
            })
            .max_by_key(|x| x.state_revision)
            .ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ApprovalRequired,
                    "exact-scoped final-approved Spec required",
                )
            })?;
        let final_id = source.final_spec_id.clone().ok_or_else(|| {
            fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ApprovalRequired,
                "final_spec_id required",
            )
        })?;
        ProviderNeutralTaskPlanRecord {
            task_plan_id: r
                .task_plan_id
                .clone()
                .unwrap_or_else(|| hash("task-plan", &[wid, &final_id])),
            project_root: r.project_root.clone(),
            continuity_id: r.continuity_id.clone(),
            attachment_id: r.attachment_id.clone(),
            workbench_session_id: wid.into(),
            final_spec_id: final_id,
            state_revision: 1,
            status: TaskPlanStatus::Draft,
            tasks: vec![],
            preview_token: None,
            previewed_revision: None,
            approved_revision: None,
            approved_by: None,
            receipt_refs: vec![],
            materialized: false,
            idempotency_key: r.idempotency_key.clone(),
            created_at: now,
            updated_at: now,
        }
    } else {
        let mut x = latest.ok_or_else(|| {
            fail(
                StatusCode::NOT_FOUND,
                ToolStatus::Blocked,
                FailureClass::NotFound,
                "task plan missing",
            )
        })?;
        if matches!(x.status, TaskPlanStatus::Approved) {
            return Err(fail(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::ApprovalRequired,
                "approved task plan is immutable; create an explicit revision workflow",
            ));
        }
        x.state_revision += 1;
        x.updated_at = now;
        x.idempotency_key = r.idempotency_key.clone();
        x
    };
    match r.action {
        MutationAction::Open => {}
        MutationAction::UpsertTask => {
            let task = r.task.ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "task required",
                )
            })?;
            if let Some(i) = p
                .tasks
                .iter()
                .position(|x| x.provider_neutral_id == task.provider_neutral_id)
            {
                p.tasks[i] = task
            } else {
                p.tasks.push(task)
            }
            p.tasks.sort_by_key(|x| x.order_index);
            p.status = TaskPlanStatus::Draft;
            p.preview_token = None;
            p.previewed_revision = None;
        }
        MutationAction::RemoveTask => {
            let id = r.task_id.ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "task_id required",
                )
            })?;
            if p.tasks.iter().any(|x| x.dependencies.contains(&id)) {
                return Err(fail(
                    StatusCode::CONFLICT,
                    ToolStatus::Blocked,
                    FailureClass::ValidationRejected,
                    "dependent tasks must be edited before removal",
                ));
            }
            p.tasks.retain(|x| x.provider_neutral_id != id);
            p.status = TaskPlanStatus::Draft;
            p.preview_token = None;
            p.previewed_revision = None;
        }
        MutationAction::Preview => {
            validate(&p.tasks).map_err(|m| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    m,
                )
            })?;
            p.status = TaskPlanStatus::PendingOperator;
            p.previewed_revision = Some(p.state_revision);
            p.preview_token = Some(hash(
                "task-preview",
                &[
                    &p.task_plan_id,
                    &p.state_revision.to_string(),
                    &p.idempotency_key,
                ],
            ));
        }
        MutationAction::Approve => {
            if !matches!(p.status, TaskPlanStatus::PendingOperator)
                || p.preview_token.as_deref() != r.preview_token.as_deref()
                || p.previewed_revision != Some(p.state_revision - 1)
            {
                return Err(fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ApprovalRequired,
                    "matching prior preview token required",
                ));
            }
            let operator = r
                .approved_by
                .filter(|x| !x.trim().is_empty())
                .ok_or_else(|| {
                    fail(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ToolStatus::ValidationRejected,
                        FailureClass::PermissionDenied,
                        "explicit operator approval required",
                    )
                })?;
            validate(&p.tasks).map_err(|m| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    m,
                )
            })?;
            p.status = TaskPlanStatus::Approved;
            p.approved_revision = Some(p.state_revision);
            p.approved_by = Some(operator);
        }
    }
    if !p.tasks.is_empty() {
        validate(&p.tasks).map_err(|message| {
            fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                message,
            )
        })?;
    }
    let pid = p.task_plan_id.clone();
    let key = p.idempotency_key.clone();
    let receipt = format!("receipt:task-plan:{pid}:{key}");
    p.receipt_refs.push(receipt);
    drop(snap);
    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::ProviderNeutralTaskPlanRevised { task_plan: p },
        })
        .await
        .map_err(|_| {
            fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "task plan command channel unavailable",
            )
        })?;
    for _ in 0..100 {
        let s = state.focusa.read().await;
        if let Some(x) = s
            .provider_neutral_task_plans
            .iter()
            .find(|x| x.task_plan_id == pid && x.idempotency_key == key)
        {
            return Ok(Json(response(x.clone(), s.version, false)));
        }
        drop(s);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(fail(
        StatusCode::SERVICE_UNAVAILABLE,
        ToolStatus::Degraded,
        FailureClass::ReadModelLag,
        "task plan revision not visible",
    ))
}
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/task-plans", get(list))
        .route(ENDPOINT, post(mutate))
}
