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
    types::{Action, FocusaEvent, WorkRailInteractionRecord, WorkRailRecord, WorkRailStatus},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Duration};
use uuid::Uuid;
#[path = "work_rail_provider.rs"]
mod provider;
use provider::{close_bead, reopen_bead};

type ApiError = (StatusCode, Json<Box<ToolResultV1>>);
const ENDPOINT: &str = "/v1/work-rail/mutate";
#[derive(Debug, Deserialize)]
pub struct RailQuery {
    project_root: String,
    working_subpath_id: String,
    continuity_id: String,
    attachment_id: String,
    #[serde(default)]
    work_rail_id: Option<String>,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RailAction {
    Bind,
    Activate,
    VerifyClose,
    Cancel,
    Steer,
    Defer,
    RequestApproval,
    Reopen,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RailSideEffectPolicy {
    Preview,
    Commit,
}
#[derive(Debug, Deserialize)]
pub struct RailRequest {
    project_root: String,
    working_subpath_id: String,
    continuity_id: String,
    attachment_id: String,
    idempotency_key: String,
    expected_state_version: u64,
    expected_rail_revision: u64,
    action: RailAction,
    side_effect_policy: RailSideEffectPolicy,
    #[serde(default)]
    preview_token: Option<String>,
    #[serde(default)]
    actor_ref: Option<String>,
    #[serde(default)]
    interaction_reason: Option<String>,
    #[serde(default)]
    work_rail_id: Option<String>,
    workpoint_id: Uuid,
    provider_item_id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    instance_id: Option<String>,
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    work_surface_ids: Vec<String>,
    #[serde(default)]
    priority: Option<i64>,
    #[serde(default)]
    rank: Option<i64>,
    #[serde(default)]
    change_set_ref: Option<String>,
    #[serde(default)]
    evidence_refs: Vec<String>,
    #[serde(default)]
    artifact_refs: Vec<String>,
    #[serde(default)]
    closure_claim_ref: Option<String>,
    #[serde(default)]
    cancellation_reason: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct RailList {
    schema: &'static str,
    state_version: u64,
    rows: Vec<WorkRailRecord>,
}
#[derive(Debug, Serialize)]
pub struct RailResult {
    schema: &'static str,
    state_version: u64,
    replayed: bool,
    committed: bool,
    preview_token: String,
    row: WorkRailRecord,
    evidence_ref: String,
    receipt_ref: String,
    tool_result: ToolResultV1,
}
fn fail(
    code: StatusCode,
    status: ToolStatus,
    class: FailureClass,
    message: impl Into<String>,
) -> ApiError {
    let mut x = ToolResultV1::failure(status, class, message.into());
    x.tool = Some("focusa_work_rail_mutate".into());
    x.family = Some("work_rail".into());
    x.endpoint = Some(ENDPOINT.into());
    (code, Json(Box::new(x)))
}
fn stable(prefix: &str, parts: &[&str]) -> String {
    let mut h = Sha256::new();
    for part in parts {
        h.update(part.as_bytes());
        h.update([0]);
    }
    format!("{prefix}:{}", &hex::encode(h.finalize())[..24])
}
fn scoped(x: &WorkRailRecord, p: &str, w: &str, c: &str, a: &str) -> bool {
    x.project_root == p && x.working_subpath_id == w && x.continuity_id == c && x.attachment_id == a
}
fn action_name(action: RailAction) -> &'static str {
    match action {
        RailAction::Bind => "bind",
        RailAction::Activate => "activate",
        RailAction::VerifyClose => "verify_close",
        RailAction::Cancel => "cancel",
        RailAction::Steer => "steer",
        RailAction::Defer => "defer",
        RailAction::RequestApproval => "request_approval",
        RailAction::Reopen => "reopen",
    }
}
fn request_preview_token(request: &RailRequest) -> String {
    stable(
        "work-rail-preview",
        &[
            &request.project_root,
            &request.working_subpath_id,
            &request.continuity_id,
            &request.attachment_id,
            &request.workpoint_id.to_string(),
            &request.provider_item_id,
            action_name(request.action),
            &request.expected_state_version.to_string(),
            &request.expected_rail_revision.to_string(),
            &request.idempotency_key,
            request.actor_ref.as_deref().unwrap_or_default(),
            request.interaction_reason.as_deref().unwrap_or_default(),
        ],
    )
}
fn response(row: WorkRailRecord, version: u64, replayed: bool) -> RailResult {
    let evidence = format!(
        "evidence:work-rail:{}:r{}",
        row.work_rail_id, row.state_revision
    );
    let receipt = row.receipt_ref.clone().unwrap_or_else(|| {
        format!(
            "receipt:work-rail:{}:{}",
            row.work_rail_id, row.idempotency_key
        )
    });
    let mut result = ToolResultV1::success(
        ToolStatus::Completed,
        if replayed {
            "Work Rail mutation replayed idempotently"
        } else {
            "Canonical Work Rail revision committed"
        },
    );
    result.tool = Some("focusa_work_rail_mutate".into());
    result.family = Some("work_rail".into());
    result.endpoint = Some(ENDPOINT.into());
    result.evidence_refs = vec![evidence.clone(), receipt.clone()];
    RailResult {
        schema: "focusa.work_rail_mutation_result.v1",
        state_version: version,
        replayed,
        committed: true,
        preview_token: String::new(),
        row,
        evidence_ref: evidence,
        receipt_ref: receipt,
        tool_result: result,
    }
}
fn preview_response(row: WorkRailRecord, version: u64, preview_token: String) -> RailResult {
    let evidence = format!("evidence:work-rail-preview:{}", row.work_rail_id);
    let receipt = format!("receipt:work-rail-preview:{}", preview_token);
    let mut result = ToolResultV1::success(
        ToolStatus::NoOp,
        "Work Rail mutation previewed; canonical state is unchanged",
    );
    result.tool = Some("focusa_work_rail_mutate".into());
    result.family = Some("work_rail".into());
    result.endpoint = Some(ENDPOINT.into());
    result.evidence_refs = vec![evidence.clone(), receipt.clone()];
    RailResult {
        schema: "focusa.work_rail_mutation_result.v1",
        state_version: version,
        replayed: false,
        committed: false,
        preview_token,
        row,
        evidence_ref: evidence,
        receipt_ref: receipt,
        tool_result: result,
    }
}
pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RailQuery>,
) -> Result<Json<RailList>, ApiError> {
    if [
        &q.project_root,
        &q.working_subpath_id,
        &q.continuity_id,
        &q.attachment_id,
    ]
    .iter()
    .any(|x| x.trim().is_empty())
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ScopeMismatch,
            "exact project, working sub-path, continuity, and attachment scope required",
        ));
    }
    let s = state.focusa.read().await;
    let rows = s
        .work_rail_records
        .iter()
        .filter(|x| {
            scoped(
                x,
                &q.project_root,
                &q.working_subpath_id,
                &q.continuity_id,
                &q.attachment_id,
            ) && q
                .work_rail_id
                .as_ref()
                .is_none_or(|id| id == &x.work_rail_id)
        })
        .cloned()
        .collect();
    Ok(Json(RailList {
        schema: "focusa.work_rail_list.v1",
        state_version: s.version,
        rows,
    }))
}
pub async fn mutate(
    State(state): State<Arc<AppState>>,
    Json(r): Json<RailRequest>,
) -> Result<Json<RailResult>, ApiError> {
    if [
        &r.project_root,
        &r.working_subpath_id,
        &r.continuity_id,
        &r.attachment_id,
        &r.idempotency_key,
        &r.provider_item_id,
    ]
    .iter()
    .any(|x| x.trim().is_empty())
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ScopeMismatch,
            "exact authority scope, Bead, and idempotency required",
        ));
    }
    let expected_preview_token = request_preview_token(&r);
    if matches!(r.side_effect_policy, RailSideEffectPolicy::Commit) {
        if r.actor_ref
            .as_deref()
            .is_none_or(|actor| actor.trim().is_empty())
        {
            return Err(fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ApprovalRequired,
                "commit requires actor_ref",
            ));
        }
        if r.preview_token.as_deref() != Some(expected_preview_token.as_str()) {
            return Err(fail(
                StatusCode::PRECONDITION_REQUIRED,
                ToolStatus::Blocked,
                FailureClass::ApprovalRequired,
                "commit requires the exact typed Work Rail preview_token",
            ));
        }
    }
    let s = state.focusa.read().await;
    if matches!(r.side_effect_policy, RailSideEffectPolicy::Commit)
        && let Some(existing) = s.work_rail_records.iter().find(|x| {
            scoped(
                x,
                &r.project_root,
                &r.working_subpath_id,
                &r.continuity_id,
                &r.attachment_id,
            ) && x.idempotency_key == r.idempotency_key
        })
    {
        return Ok(Json(response(existing.clone(), s.version, true)));
    }
    if s.version != r.expected_state_version {
        return Err(fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "stale canonical state version",
        ));
    }
    let wp = s
        .workpoint
        .records
        .iter()
        .find(|x| {
            x.workpoint_id == r.workpoint_id
                && x.canonical
                && x.project_root.as_deref() == Some(r.project_root.as_str())
                && x.continuity_id.as_deref() == Some(r.continuity_id.as_str())
                && x.work_item_id.as_deref() == Some(r.provider_item_id.as_str())
                && x.session_identity
                    .as_ref()
                    .and_then(|i| i.working_subpath_id.as_deref())
                    == Some(r.working_subpath_id.as_str())
        })
        .cloned()
        .ok_or_else(|| {
            fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ScopeMismatch,
                "canonical Workpoint must match project, working sub-path, continuity, and Bead",
            )
        })?;
    let latest = r
        .work_rail_id
        .as_ref()
        .and_then(|id| {
            s.work_rail_records
                .iter()
                .filter(|x| {
                    x.work_rail_id == *id
                        && scoped(
                            x,
                            &r.project_root,
                            &r.working_subpath_id,
                            &r.continuity_id,
                            &r.attachment_id,
                        )
                })
                .max_by_key(|x| x.state_revision)
        })
        .cloned();
    if latest.as_ref().map_or(0, |x| x.state_revision) != r.expected_rail_revision {
        return Err(fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "stale Work Rail revision",
        ));
    }
    let now = Utc::now();
    let mut row = if matches!(r.action, RailAction::Bind) {
        if latest.is_some() {
            return Err(fail(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::WriterConflict,
                "Work Rail row already exists",
            ));
        }
        let materialized = s
            .task_materializations
            .iter()
            .flat_map(|m| m.tasks.iter())
            .any(|task| task.provider_id == r.provider_item_id);
        if !materialized {
            return Err(fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::NotFound,
                "Bead must originate from approved task materialization",
            ));
        }
        WorkRailRecord {
            work_rail_id: r.work_rail_id.clone().unwrap_or_else(|| {
                stable(
                    "work-rail",
                    &[&r.project_root, &r.continuity_id, &r.provider_item_id],
                )
            }),
            state_revision: 1,
            provider: "work_item.bd".into(),
            provider_item_id: r.provider_item_id.clone(),
            title: r
                .title
                .clone()
                .filter(|x| !x.trim().is_empty())
                .unwrap_or_else(|| r.provider_item_id.clone()),
            provider_status: "open".into(),
            focusa_status: WorkRailStatus::Ready,
            workpoint_id: r.workpoint_id,
            project_root: r.project_root.clone(),
            working_subpath_id: r.working_subpath_id.clone(),
            continuity_id: r.continuity_id.clone(),
            attachment_id: r.attachment_id.clone(),
            instance_id: r.instance_id.clone(),
            session_id: r.session_id.clone(),
            work_surface_ids: r.work_surface_ids.clone(),
            priority: r.priority,
            rank: r.rank,
            dependencies: vec![],
            blockers: vec![],
            evidence_refs: vec![],
            artifact_refs: vec![],
            change_set_ref: r.change_set_ref.clone(),
            receipt_ref: None,
            closure_claim_ref: None,
            interaction_history: vec![],
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
                "Work Rail row missing",
            )
        })?;
        x.state_revision += 1;
        x.idempotency_key = r.idempotency_key.clone();
        x.instance_id = r.instance_id.clone().or(x.instance_id);
        x.session_id = r.session_id.clone().or(x.session_id);
        if !r.work_surface_ids.is_empty() {
            x.work_surface_ids = r.work_surface_ids.clone();
        }
        x.priority = r.priority.or(x.priority);
        x.rank = r.rank.or(x.rank);
        x.change_set_ref = r.change_set_ref.clone().or(x.change_set_ref);
        x.updated_at = now;
        x
    };
    match r.action {
        RailAction::Bind => {}
        RailAction::Activate | RailAction::Steer => {
            row.focusa_status = WorkRailStatus::Active;
            row.blockers.clear();
        }
        RailAction::Defer => {
            let reason = r
                .interaction_reason
                .as_deref()
                .filter(|reason| !reason.trim().is_empty())
                .ok_or_else(|| {
                    fail(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ToolStatus::ValidationRejected,
                        FailureClass::ValidationRejected,
                        "defer requires interaction_reason",
                    )
                })?;
            row.focusa_status = WorkRailStatus::Ready;
            row.blockers = vec![format!("deferred:{reason}")];
        }
        RailAction::RequestApproval => {
            row.focusa_status = WorkRailStatus::ProofMissing;
            row.blockers = vec!["approval_requested".to_string()];
        }
        RailAction::Reopen => {
            if matches!(r.side_effect_policy, RailSideEffectPolicy::Commit) {
                reopen_bead(&r.project_root, &r.provider_item_id).map_err(|message| {
                    fail(
                        StatusCode::BAD_GATEWAY,
                        ToolStatus::Blocked,
                        FailureClass::ProcessControlFailed,
                        message,
                    )
                })?;
            }
            row.provider_status = "open".to_string();
            row.focusa_status = WorkRailStatus::Ready;
            row.blockers.clear();
            row.closure_claim_ref = None;
        }
        RailAction::Cancel => {
            row.focusa_status = WorkRailStatus::Cancelled;
            row.blockers = r.cancellation_reason.into_iter().collect()
        }
        RailAction::VerifyClose => {
            let claim = r
                .closure_claim_ref
                .filter(|x| !x.trim().is_empty())
                .ok_or_else(|| {
                    fail(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ToolStatus::ValidationRejected,
                        FailureClass::ApprovalRequired,
                        "closure_claim_ref required",
                    )
                })?;
            let linked: std::collections::BTreeSet<_> = wp
                .verification_records
                .iter()
                .filter_map(|v| v.evidence_ref.as_deref())
                .collect();
            if r.evidence_refs.is_empty()
                || r.evidence_refs.iter().any(|e| !linked.contains(e.as_str()))
            {
                return Err(fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "all closure proof must be linked to the Workpoint",
                ));
            }
            if matches!(r.side_effect_policy, RailSideEffectPolicy::Commit) {
                close_bead(&r.project_root, &r.provider_item_id, &claim).map_err(|m| {
                    fail(
                        StatusCode::BAD_GATEWAY,
                        ToolStatus::Blocked,
                        FailureClass::ProcessControlFailed,
                        m,
                    )
                })?;
            }
            row.provider_status = "closed".into();
            row.focusa_status = WorkRailStatus::VerifiedComplete;
            row.evidence_refs = r.evidence_refs;
            row.artifact_refs = r.artifact_refs;
            row.closure_claim_ref = Some(claim);
            row.receipt_ref = Some(format!(
                "receipt:work-rail-closure:{}:r{}",
                row.work_rail_id, row.state_revision
            ));
        }
    }
    if matches!(r.side_effect_policy, RailSideEffectPolicy::Preview) {
        return Ok(Json(preview_response(
            row,
            s.version,
            expected_preview_token,
        )));
    }
    let interaction_receipt = stable(
        "receipt:work-rail-interaction",
        &[
            &row.work_rail_id,
            &row.state_revision.to_string(),
            action_name(r.action),
            &r.idempotency_key,
        ],
    );
    row.interaction_history.push(WorkRailInteractionRecord {
        interaction_id: stable(
            "work-rail-interaction",
            &[&row.work_rail_id, &row.state_revision.to_string()],
        ),
        action: action_name(r.action).to_string(),
        actor_ref: r.actor_ref.unwrap_or_default(),
        reason: r.interaction_reason.unwrap_or_default(),
        receipt_ref: interaction_receipt.clone(),
        committed_at: now,
    });
    row.receipt_ref = Some(interaction_receipt);
    let id = row.work_rail_id.clone();
    let key = row.idempotency_key.clone();
    drop(s);
    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::WorkRailRevised { record: row },
        })
        .await
        .map_err(|_| {
            fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "Work Rail command channel unavailable",
            )
        })?;
    for _ in 0..100 {
        let current = state.focusa.read().await;
        if let Some(saved) = current
            .work_rail_records
            .iter()
            .find(|x| x.work_rail_id == id && x.idempotency_key == key)
        {
            return Ok(Json(response(saved.clone(), current.version, false)));
        }
        drop(current);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(fail(
        StatusCode::SERVICE_UNAVAILABLE,
        ToolStatus::Degraded,
        FailureClass::ReadModelLag,
        "Work Rail revision not visible",
    ))
}
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/work-rail", get(list))
        .route(ENDPOINT, post(mutate))
}
