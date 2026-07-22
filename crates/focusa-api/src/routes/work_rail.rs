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
    types::{Action, FocusaEvent, WorkRailRecord, WorkRailStatus},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs::OpenOptions, io::Write, path::PathBuf, sync::Arc, time::Duration};
use uuid::Uuid;
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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RailAction {
    Bind,
    Activate,
    VerifyClose,
    Cancel,
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
    #[serde(default)]
    work_rail_id: Option<String>,
    workpoint_id: Uuid,
    provider_item_id: String,
    #[serde(default)]
    title: Option<String>,
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
        row,
        evidence_ref: evidence,
        receipt_ref: receipt,
        tool_result: result,
    }
}
fn close_bead(root: &str, item_id: &str, claim: &str) -> Result<(), String> {
    let root = PathBuf::from(root);
    if !root.join(".git").is_dir() {
        return Err("provider closure requires canonical parent Git root".into());
    }
    let ledger = root.join(".beads/issues.jsonl");
    let body =
        std::fs::read_to_string(&ledger).map_err(|e| format!("cannot read Beads ledger: {e}"))?;
    let now = Utc::now();
    let mut found = false;
    let mut out = String::new();
    for line in body.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut value: Value =
            serde_json::from_str(line).map_err(|e| format!("invalid Beads JSONL: {e}"))?;
        if value.get("id").and_then(Value::as_str) == Some(item_id) {
            found = true;
            value["status"] = json!("closed");
            value["closed_at"] = json!(now);
            value["updated_at"] = json!(now);
            value["close_reason"] = json!(format!("Focusa verified closure: {claim}"));
        }
        out.push_str(&serde_json::to_string(&value).map_err(|e| e.to_string())?);
        out.push('\n');
    }
    if !found {
        return Err(format!("Beads item not found: {item_id}"));
    }
    let temp = ledger.with_extension("jsonl.focusa.tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    file.write_all(out.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    std::fs::rename(temp, ledger).map_err(|e| e.to_string())
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
    let s = state.focusa.read().await;
    if let Some(existing) = s.work_rail_records.iter().find(|x| {
        scoped(
            x,
            &r.project_root,
            &r.working_subpath_id,
            &r.continuity_id,
            &r.attachment_id,
        ) && x.idempotency_key == r.idempotency_key
    }) {
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
            dependencies: vec![],
            blockers: vec![],
            evidence_refs: vec![],
            artifact_refs: vec![],
            receipt_ref: None,
            closure_claim_ref: None,
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
        x.updated_at = now;
        x
    };
    match r.action {
        RailAction::Bind => {}
        RailAction::Activate => row.focusa_status = WorkRailStatus::Active,
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
            close_bead(&r.project_root, &r.provider_item_id, &claim).map_err(|m| {
                fail(
                    StatusCode::BAD_GATEWAY,
                    ToolStatus::Blocked,
                    FailureClass::ProcessControlFailed,
                    m,
                )
            })?;
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
