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
        Action, FocusaEvent, MaterializedTaskRef, ProviderNeutralTaskPlanRecord,
        ProviderNeutralTaskRecord, SpecWorkbenchStatus, TaskMaterializationRecord, TaskPlanStatus,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

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
    provider_capabilities: Vec<TaskProviderCapabilityTruth>,
}

#[derive(Debug, Serialize)]
pub struct TaskProviderCapabilityTruth {
    provider: &'static str,
    status: &'static str,
    read_write_posture: &'static str,
    configured: bool,
    credential_reference_present: bool,
    mutation_approval_required: bool,
    adapter_ref: &'static str,
    recovery_action: String,
}

fn provider_capabilities(project_root: &str) -> Vec<TaskProviderCapabilityTruth> {
    let root = Path::new(project_root);
    let beads_ledger = root.join(".beads/issues.jsonl");
    let beads_status = if root.join(".git").is_dir() && beads_ledger.is_file() {
        "configured and operational"
    } else if beads_ledger.is_file() {
        "read-only"
    } else {
        "adapter unavailable"
    };
    let external = |provider: &'static str,
                    credential_env: &str,
                    adapter_ref: &'static str,
                    recovery_without_credentials: &str| {
        let credential_reference_present = env::var(credential_env)
            .ok()
            .is_some_and(|value| !value.trim().is_empty());
        TaskProviderCapabilityTruth {
            provider,
            status: if credential_reference_present {
                "schema-only support"
            } else {
                "credentials missing"
            },
            read_write_posture: "read-only",
            configured: credential_reference_present,
            credential_reference_present,
            mutation_approval_required: true,
            adapter_ref,
            recovery_action: if credential_reference_present {
                "install and verify the provider adapter before mutation".to_string()
            } else {
                recovery_without_credentials.to_string()
            },
        }
    };
    vec![
        TaskProviderCapabilityTruth {
            provider: "beads",
            status: beads_status,
            read_write_posture: if beads_status == "configured and operational" {
                "read-write"
            } else {
                "read-only"
            },
            configured: beads_ledger.is_file(),
            credential_reference_present: true,
            mutation_approval_required: true,
            adapter_ref: "focusa.work_item.adapters.bd.v1",
            recovery_action: if beads_ledger.is_file() {
                "use the canonical parent Git root and explicit permission grant".to_string()
            } else {
                "initialize canonical parent Beads before materialization".to_string()
            },
        },
        external(
            "github_issues",
            "FOCUSA_GITHUB_CREDENTIAL_REF",
            "focusa.task_provider.github_issues.schema.v1",
            "configure a GitHub credential reference with minimum issue scopes",
        ),
        external(
            "linear",
            "FOCUSA_LINEAR_CREDENTIAL_REF",
            "focusa.task_provider.linear.schema.v1",
            "configure a Linear credential reference with minimum issue scopes",
        ),
        external(
            "asana",
            "FOCUSA_ASANA_CREDENTIAL_REF",
            "focusa.task_provider.asana.schema.v1",
            "configure an Asana credential reference with minimum task scopes",
        ),
        TaskProviderCapabilityTruth {
            provider: "markdown_checklist",
            status: "schema-only support",
            read_write_posture: "read-only",
            configured: true,
            credential_reference_present: true,
            mutation_approval_required: true,
            adapter_ref: "focusa.task_provider.markdown_checklist.schema.v1",
            recovery_action: "install and verify the Markdown Checklist mutation adapter"
                .to_string(),
        },
    ]
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
    format!("{prefix}:{}", &hex::encode(h.finalize())[..24])
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
        provider_capabilities: provider_capabilities(&q.project_root),
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
#[derive(Debug, Deserialize)]
pub struct MaterializeRequest {
    project_root: String,
    continuity_id: String,
    attachment_id: String,
    task_plan_id: String,
    expected_state_version: u64,
    expected_plan_revision: u64,
    worktree_prefix: String,
    permission_grant_ref: String,
    idempotency_key: String,
}
#[derive(Debug, Serialize)]
pub struct MaterializeResponse {
    schema: &'static str,
    state_version: u64,
    replayed: bool,
    materialization: TaskMaterializationRecord,
    evidence_ref: String,
    receipt_ref: String,
    tool_result: ToolResultV1,
}
fn materialize_response(
    record: TaskMaterializationRecord,
    version: u64,
    replayed: bool,
) -> MaterializeResponse {
    let mut result = ToolResultV1::success(
        ToolStatus::Completed,
        if replayed {
            "Beads task materialization replayed idempotently"
        } else {
            "Approved task DAG materialized into canonical parent Beads"
        },
    );
    result.tool = Some("focusa_task_plan_materialize_beads".into());
    result.family = Some("provider_neutral_task_plan".into());
    result.endpoint = Some("/v1/task-plans/materialize/beads".into());
    result.evidence_refs = vec![record.evidence_ref.clone(), record.receipt_ref.clone()];
    MaterializeResponse {
        schema: "focusa.task_plan_beads_materialization_result.v1",
        state_version: version,
        replayed,
        evidence_ref: record.evidence_ref.clone(),
        receipt_ref: record.receipt_ref.clone(),
        materialization: record,
        tool_result: result,
    }
}
fn materialize_fail(
    code: StatusCode,
    status: ToolStatus,
    class: FailureClass,
    message: impl Into<String>,
) -> ApiError {
    let mut result = ToolResultV1::failure(status, class, message.into());
    result.tool = Some("focusa_task_plan_materialize_beads".into());
    result.family = Some("provider_neutral_task_plan".into());
    result.endpoint = Some("/v1/task-plans/materialize/beads".into());
    (code, Json(Box::new(result)))
}
fn read_ledger(path: &Path) -> Result<BTreeMap<String, serde_json::Value>, String> {
    let body = std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read canonical Beads ledger: {error}"))?;
    let mut entries = BTreeMap::new();
    for (index, line) in body.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)
            .map_err(|error| format!("invalid Beads JSONL line {}: {error}", index + 1))?;
        let id = value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("Beads JSONL line {} has no id", index + 1))?;
        entries.insert(id.to_string(), value);
    }
    Ok(entries)
}
fn has_local_database(beads: &Path) -> bool {
    std::fs::read_dir(beads)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "db")
        })
}
struct MaterializationLock(PathBuf);
impl Drop for MaterializationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn acquire_materialization_lock(beads: &Path) -> Result<MaterializationLock, std::io::Error> {
    let path = beads.join(".focusa-materialize.lock");
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    Ok(MaterializationLock(path))
}
fn provider_id(prefix: &str, plan_id: &str, task_id: &str) -> String {
    let digest = hash("", &[plan_id, task_id]);
    format!("{prefix}-{}", digest.trim_start_matches(':'))
}
pub async fn materialize_beads(
    State(state): State<Arc<AppState>>,
    Json(request): Json<MaterializeRequest>,
) -> Result<Json<MaterializeResponse>, ApiError> {
    if request.project_root.trim().is_empty()
        || request.continuity_id.trim().is_empty()
        || request.attachment_id.trim().is_empty()
        || request.task_plan_id.trim().is_empty()
        || request.permission_grant_ref.trim().is_empty()
        || request.idempotency_key.trim().is_empty()
        || request.worktree_prefix.trim().is_empty()
        || !request.worktree_prefix.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        return Err(materialize_fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "exact scope, approved plan, lowercase worktree prefix, permission, and idempotency are required",
        ));
    }
    let root = PathBuf::from(&request.project_root);
    let beads = root.join(".beads");
    let ledger = beads.join("issues.jsonl");
    if !root.join(".git").is_dir() || !ledger.is_file() {
        return Err(materialize_fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ScopeMismatch,
            "materialization target must be the canonical parent Git root with .beads/issues.jsonl; worktree-local targets are prohibited",
        ));
    }
    if has_local_database(&beads) {
        return Err(materialize_fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::ValidationRejected,
            "canonical Beads target must remain JSONL-only; local database files are prohibited",
        ));
    }
    let snapshot = state.focusa.read().await;
    if let Some(existing) = snapshot.task_materializations.iter().find(|existing| {
        existing.project_root == request.project_root
            && existing.continuity_id == request.continuity_id
            && existing.attachment_id == request.attachment_id
            && existing.idempotency_key == request.idempotency_key
    }) {
        return Ok(Json(materialize_response(
            existing.clone(),
            snapshot.version,
            true,
        )));
    }
    if snapshot.version != request.expected_state_version {
        return Err(materialize_fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "stale canonical state version",
        ));
    }
    let plan = snapshot
        .provider_neutral_task_plans
        .iter()
        .find(|plan| {
            plan.task_plan_id == request.task_plan_id
                && plan.state_revision == request.expected_plan_revision
                && plan.project_root == request.project_root
                && plan.continuity_id == request.continuity_id
                && plan.attachment_id == request.attachment_id
                && matches!(plan.status, TaskPlanStatus::Approved)
        })
        .cloned()
        .ok_or_else(|| {
            materialize_fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ApprovalRequired,
                "exact-scoped approved task plan revision required",
            )
        })?;
    if snapshot.task_materializations.iter().any(|existing| {
        existing.task_plan_id == plan.task_plan_id
            && existing.task_plan_revision == plan.state_revision
    }) {
        return Err(materialize_fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "approved task plan revision is already materialized",
        ));
    }
    let before = read_ledger(&ledger).map_err(|message| {
        materialize_fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            message,
        )
    })?;
    let mapping: BTreeMap<_, _> = plan
        .tasks
        .iter()
        .map(|task| {
            (
                task.provider_neutral_id.clone(),
                provider_id(
                    &request.worktree_prefix,
                    &plan.task_plan_id,
                    &task.provider_neutral_id,
                ),
            )
        })
        .collect();
    for task in &plan.tasks {
        let id = &mapping[&task.provider_neutral_id];
        let external = format!(
            "focusa-task-plan:{}:{}",
            plan.task_plan_id, task.provider_neutral_id
        );
        if let Some(existing) = before.get(id) {
            if existing
                .get("external_ref")
                .and_then(serde_json::Value::as_str)
                != Some(external.as_str())
            {
                return Err(materialize_fail(
                    StatusCode::CONFLICT,
                    ToolStatus::Blocked,
                    FailureClass::WriterConflict,
                    format!("stable provider ID collision: {id}"),
                ));
            }
        }
    }
    drop(snapshot);
    let _lock = acquire_materialization_lock(&beads).map_err(|error| {
        materialize_fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            format!("canonical Beads materialization is busy: {error}"),
        )
    })?;
    let mut after = before.clone();
    let created_at = Utc::now();
    let mut append = std::fs::OpenOptions::new()
        .append(true)
        .open(&ledger)
        .map_err(|error| {
            materialize_fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::ProcessControlFailed,
                format!("cannot append canonical Beads ledger: {error}"),
            )
        })?;
    for task in &plan.tasks {
        let id = &mapping[&task.provider_neutral_id];
        if after.contains_key(id) {
            continue;
        }
        let external = format!(
            "focusa-task-plan:{}:{}",
            plan.task_plan_id, task.provider_neutral_id
        );
        let description = format!(
            "{}\n\nTask plan: {}@{}\nProvider-neutral ID: {}\nRequirements: {}\nSpec sections: {}\nEvidence: {}\nVerification policy: {}",
            task.description,
            plan.task_plan_id,
            plan.state_revision,
            task.provider_neutral_id,
            task.requirement_refs.join(", "),
            task.linked_spec_sections.join(", "),
            task.evidence_requirements.join(", "),
            task.verification_policy_ref
        );
        let dependencies: Vec<_> = task.dependencies.iter().map(|dependency| serde_json::json!({"issue_id": id, "depends_on_id": mapping[dependency], "type": "blocks", "created_at": created_at, "created_by": "focusa"})).collect();
        let entry = serde_json::json!({"id":id,"title":task.title,"description":description,"acceptance_criteria":task.acceptance_criteria.join("\n"),"status":"open","priority":2,"issue_type":"task","created_at":created_at,"updated_at":created_at,"external_ref":external,"labels":["spec135","generated-task"],"dependencies":dependencies});
        writeln!(
            append,
            "{}",
            serde_json::to_string(&entry).expect("Beads entry serializes")
        )
        .map_err(|error| {
            materialize_fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::ProcessControlFailed,
                format!("cannot write canonical Beads task {id}: {error}"),
            )
        })?;
        after.insert(id.clone(), entry);
    }
    append.flush().map_err(|error| {
        materialize_fail(
            StatusCode::SERVICE_UNAVAILABLE,
            ToolStatus::Offline,
            FailureClass::ProcessControlFailed,
            format!("cannot flush canonical Beads ledger: {error}"),
        )
    })?;
    if has_local_database(&beads) {
        return Err(materialize_fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            ToolStatus::Blocked,
            FailureClass::ValidationRejected,
            "Beads adapter created a prohibited local database",
        ));
    }
    let tasks: Vec<_> = plan
        .tasks
        .iter()
        .map(|task| {
            let provider = mapping[&task.provider_neutral_id].clone();
            let external = format!(
                "focusa-task-plan:{}:{}",
                plan.task_plan_id, task.provider_neutral_id
            );
            let entry = after.get(&provider).ok_or_else(|| {
                materialize_fail(
                    StatusCode::BAD_GATEWAY,
                    ToolStatus::Blocked,
                    FailureClass::ProcessControlFailed,
                    format!("Beads task not visible after create: {provider}"),
                )
            })?;
            if entry
                .get("external_ref")
                .and_then(serde_json::Value::as_str)
                != Some(external.as_str())
            {
                return Err(materialize_fail(
                    StatusCode::CONFLICT,
                    ToolStatus::Blocked,
                    FailureClass::WriterConflict,
                    format!("Beads parity mismatch: {provider}"),
                ));
            }
            Ok(MaterializedTaskRef {
                provider_neutral_id: task.provider_neutral_id.clone(),
                provider_id: provider,
                provider_dependency_ids: task
                    .dependencies
                    .iter()
                    .map(|dependency| mapping[dependency].clone())
                    .collect(),
                external_ref: external,
            })
        })
        .collect::<Result<_, ApiError>>()?;
    let materialization_id = hash(
        "task-materialization",
        &[
            &plan.task_plan_id,
            &plan.state_revision.to_string(),
            &request.worktree_prefix,
        ],
    );
    let record = TaskMaterializationRecord {
        materialization_id: materialization_id.clone(),
        task_plan_id: plan.task_plan_id.clone(),
        task_plan_revision: plan.state_revision,
        project_root: request.project_root.clone(),
        continuity_id: request.continuity_id.clone(),
        attachment_id: request.attachment_id.clone(),
        provider: "work_item.bd".into(),
        worktree_prefix: request.worktree_prefix,
        target_ledger_ref: ledger.to_string_lossy().to_string(),
        tasks,
        permission_grant_ref: request.permission_grant_ref,
        idempotency_key: request.idempotency_key.clone(),
        evidence_ref: format!("evidence:task-materialization:{materialization_id}"),
        receipt_ref: format!(
            "receipt:task-materialization:{materialization_id}:{}",
            request.idempotency_key
        ),
        created_at: Utc::now(),
    };
    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::TaskPlanMaterialized {
                materialization: record,
            },
        })
        .await
        .map_err(|_| {
            materialize_fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "materialization event channel unavailable",
            )
        })?;
    for _ in 0..100 {
        let current = state.focusa.read().await;
        if let Some(saved) = current
            .task_materializations
            .iter()
            .find(|saved| saved.materialization_id == materialization_id)
        {
            return Ok(Json(materialize_response(
                saved.clone(),
                current.version,
                false,
            )));
        }
        drop(current);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(materialize_fail(
        StatusCode::SERVICE_UNAVAILABLE,
        ToolStatus::Degraded,
        FailureClass::ReadModelLag,
        "task materialization not visible",
    ))
}
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/task-plans", get(list))
        .route(ENDPOINT, post(mutate))
        .route("/v1/task-plans/materialize/beads", post(materialize_beads))
}
