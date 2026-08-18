//! Canonical Spec 131 work-item lifecycle routes required by Spec 137.

use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, sync::Arc};
use uuid::Uuid;

type ApiError = (StatusCode, Json<Value>);
type ApiResult = Result<Json<Value>, ApiError>;

#[derive(Clone, Debug, Deserialize)]
struct ScopeQuery {
    project_root: String,
    continuity_id: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Mutation {
    project_root: String,
    continuity_id: String,
    #[serde(default)]
    item_id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    estimate_ms: Option<u64>,
    #[serde(default)]
    expected_revision: Option<u64>,
    #[serde(default)]
    evidence_refs: Vec<String>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    idempotency_key: Option<String>,
    #[serde(default)]
    confirm: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WorkItem {
    item_id: String,
    project_root: String,
    continuity_id: String,
    title: String,
    status: String,
    revision: u64,
    estimate_ms: Option<u64>,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    paused_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    evidence_refs: Vec<String>,
    last_reason: Option<String>,
    idempotency_keys: Vec<String>,
}

fn error(status: StatusCode, code: &str, message: impl Into<String>) -> ApiError {
    (
        status,
        Json(json!({"status":"blocked","canonical":true,"error":code,"message":message.into()})),
    )
}

fn scope_key(project_root: &str, continuity_id: &str) -> String {
    hex::encode(Sha256::digest(
        format!("{project_root}\0{continuity_id}").as_bytes(),
    ))
}

fn store_path(state: &AppState, project_root: &str, continuity_id: &str) -> PathBuf {
    PathBuf::from(&state.config.data_dir)
        .join("temporal-work-items")
        .join(format!("{}.json", scope_key(project_root, continuity_id)))
}

fn read_items(
    state: &AppState,
    project_root: &str,
    continuity_id: &str,
) -> Result<Vec<WorkItem>, ApiError> {
    let path = store_path(state, project_root, continuity_id);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let body = std::fs::read(&path).map_err(|e| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "work_item_read_failed",
            e.to_string(),
        )
    })?;
    serde_json::from_slice(&body).map_err(|e| {
        error(
            StatusCode::CONFLICT,
            "work_item_store_invalid",
            e.to_string(),
        )
    })
}

fn write_items(
    state: &AppState,
    project_root: &str,
    continuity_id: &str,
    items: &[WorkItem],
) -> Result<(), ApiError> {
    let path = store_path(state, project_root, continuity_id);
    let parent = path.parent().ok_or_else(|| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "work_item_path_invalid",
            "store has no parent",
        )
    })?;
    std::fs::create_dir_all(parent).map_err(|e| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "work_item_store_create_failed",
            e.to_string(),
        )
    })?;
    let temp = path.with_extension(format!("{}.tmp", Uuid::now_v7()));
    let body = serde_json::to_vec_pretty(items).map_err(|e| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "work_item_encode_failed",
            e.to_string(),
        )
    })?;
    std::fs::write(&temp, body).map_err(|e| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "work_item_write_failed",
            e.to_string(),
        )
    })?;
    std::fs::rename(&temp, &path).map_err(|e| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "work_item_activate_failed",
            e.to_string(),
        )
    })
}

fn require_mutation(request: &Mutation) -> Result<&str, ApiError> {
    if request.project_root.trim().is_empty() || request.continuity_id.trim().is_empty() {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "scope_required",
            "project_root and continuity_id are required",
        ));
    }
    if !request.confirm {
        return Err(error(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "confirm=true is required",
        ));
    }
    request
        .idempotency_key
        .as_deref()
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| {
            error(
                StatusCode::PRECONDITION_REQUIRED,
                "idempotency_key_required",
                "idempotency_key is required",
            )
        })
}

fn item_index(items: &[WorkItem], request: &Mutation) -> Result<usize, ApiError> {
    let id = request.item_id.as_deref().ok_or_else(|| {
        error(
            StatusCode::BAD_REQUEST,
            "item_id_required",
            "item_id is required",
        )
    })?;
    items
        .iter()
        .position(|item| item.item_id == id)
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "work_item_not_found",
                "item does not exist in this exact scope",
            )
        })
}

fn completed(schema: &str, item: &WorkItem) -> Json<Value> {
    Json(json!({"schema":schema,"status":"completed","canonical":true,"work_item":item}))
}

async fn create(State(state): State<Arc<AppState>>, Json(request): Json<Mutation>) -> ApiResult {
    let key = require_mutation(&request)?.to_string();
    let title = request
        .title
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| {
            error(
                StatusCode::BAD_REQUEST,
                "title_required",
                "title is required",
            )
        })?;
    let mut items = read_items(&state, &request.project_root, &request.continuity_id)?;
    if let Some(item) = items
        .iter()
        .find(|item| item.idempotency_keys.iter().any(|seen| seen == &key))
    {
        return Ok(completed("focusa.workpoint_item_create.v1", item));
    }
    let item = WorkItem {
        item_id: request
            .item_id
            .unwrap_or_else(|| format!("item:{}", Uuid::now_v7())),
        project_root: request.project_root.clone(),
        continuity_id: request.continuity_id.clone(),
        title: title.to_string(),
        status: "planned".into(),
        revision: 1,
        estimate_ms: request.estimate_ms,
        created_at: Utc::now(),
        started_at: None,
        paused_at: None,
        completed_at: None,
        evidence_refs: request.evidence_refs,
        last_reason: request.reason,
        idempotency_keys: vec![key],
    };
    items.push(item.clone());
    write_items(
        &state,
        &request.project_root,
        &request.continuity_id,
        &items,
    )?;
    Ok(completed("focusa.workpoint_item_create.v1", &item))
}

async fn list(State(state): State<Arc<AppState>>, Query(scope): Query<ScopeQuery>) -> ApiResult {
    let items = read_items(&state, &scope.project_root, &scope.continuity_id)?;
    Ok(Json(
        json!({"schema":"focusa.workpoint_items.v1","status":"completed","canonical":true,"items":items}),
    ))
}

async fn transition(
    state: &AppState,
    request: Mutation,
    from: &[&str],
    to: &str,
    schema: &str,
    evidence_required: bool,
) -> ApiResult {
    let key = require_mutation(&request)?.to_string();
    let mut items = read_items(state, &request.project_root, &request.continuity_id)?;
    let index = item_index(&items, &request)?;
    if items[index]
        .idempotency_keys
        .iter()
        .any(|seen| seen == &key)
    {
        return Ok(completed(schema, &items[index]));
    }
    if request.expected_revision != Some(items[index].revision) {
        return Err(error(
            StatusCode::CONFLICT,
            "revision_conflict",
            format!("expected revision {}", items[index].revision),
        ));
    }
    if !from.contains(&items[index].status.as_str()) {
        return Err(error(
            StatusCode::CONFLICT,
            "invalid_work_item_transition",
            format!("cannot transition {} to {to}", items[index].status),
        ));
    }
    if evidence_required && request.evidence_refs.is_empty() {
        return Err(error(
            StatusCode::PRECONDITION_FAILED,
            "completion_evidence_required",
            "completion requires evidence_refs",
        ));
    }
    let now = Utc::now();
    items[index].status = to.into();
    items[index].revision += 1;
    items[index].idempotency_keys.push(key);
    items[index].last_reason = request.reason;
    items[index].evidence_refs.extend(request.evidence_refs);
    match to {
        "in_progress" if items[index].started_at.is_none() => items[index].started_at = Some(now),
        "paused" => items[index].paused_at = Some(now),
        "completed" => items[index].completed_at = Some(now),
        _ => {}
    }
    let item = items[index].clone();
    write_items(state, &request.project_root, &request.continuity_id, &items)?;
    Ok(completed(schema, &item))
}

async fn start(State(state): State<Arc<AppState>>, Json(request): Json<Mutation>) -> ApiResult {
    transition(
        &state,
        request,
        &["planned"],
        "in_progress",
        "focusa.workpoint_item_start.v1",
        false,
    )
    .await
}
async fn pause(State(state): State<Arc<AppState>>, Json(request): Json<Mutation>) -> ApiResult {
    transition(
        &state,
        request,
        &["in_progress"],
        "paused",
        "focusa.workpoint_item_pause.v1",
        false,
    )
    .await
}
async fn resume(State(state): State<Arc<AppState>>, Json(request): Json<Mutation>) -> ApiResult {
    transition(
        &state,
        request,
        &["paused"],
        "in_progress",
        "focusa.workpoint_item_resume.v1",
        false,
    )
    .await
}
async fn complete(State(state): State<Arc<AppState>>, Json(request): Json<Mutation>) -> ApiResult {
    transition(
        &state,
        request,
        &["in_progress"],
        "completed",
        "focusa.workpoint_item_complete.v1",
        true,
    )
    .await
}

async fn closure_check(
    State(state): State<Arc<AppState>>,
    Json(request): Json<Mutation>,
) -> ApiResult {
    let items = read_items(&state, &request.project_root, &request.continuity_id)?;
    let item = &items[item_index(&items, &request)?];
    let ready = item.status == "completed" && !item.evidence_refs.is_empty();
    Ok(Json(
        json!({"schema":"focusa.task_closure_check.v1","status":"completed","canonical":true,"item_id":item.item_id,"ready_to_close":ready,"blockers":if ready { vec![] } else { vec!["completed_status_and_evidence_required"] }}),
    ))
}

async fn timing_status(
    State(state): State<Arc<AppState>>,
    Query(scope): Query<ScopeQuery>,
) -> ApiResult {
    let items = read_items(&state, &scope.project_root, &scope.continuity_id)?;
    let now = Utc::now();
    let rows = items.iter().map(|item| json!({
        "item_id":item.item_id,"status":item.status,"estimate_ms":item.estimate_ms,
        "elapsed_ms":item.started_at.map(|start| item.completed_at.unwrap_or(now).signed_duration_since(start).num_milliseconds().max(0) as u64),
        "revision":item.revision
    })).collect::<Vec<_>>();
    Ok(Json(
        json!({"schema":"focusa.work_timing_status.v1","status":"completed","canonical":true,"items":rows}),
    ))
}

async fn velocity(
    State(state): State<Arc<AppState>>,
    Query(scope): Query<ScopeQuery>,
) -> ApiResult {
    let items = read_items(&state, &scope.project_root, &scope.continuity_id)?;
    let durations = items
        .iter()
        .filter_map(|item| {
            Some(
                item.completed_at?
                    .signed_duration_since(item.started_at?)
                    .num_milliseconds()
                    .max(0) as u64,
            )
        })
        .collect::<Vec<_>>();
    let average_ms =
        (!durations.is_empty()).then(|| durations.iter().sum::<u64>() / durations.len() as u64);
    Ok(Json(
        json!({"schema":"focusa.work_velocity.v1","status":"completed","canonical":true,"completed_count":durations.len(),"average_completion_ms":average_ms,"sample_count":durations.len()}),
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/workpoint/item/create", post(create))
        .route("/v1/workpoint/items", get(list))
        .route("/v1/workpoint/item/start", post(start))
        .route("/v1/workpoint/item/pause", post(pause))
        .route("/v1/workpoint/item/resume", post(resume))
        .route("/v1/workpoint/item/complete", post(complete))
        .route("/v1/workpoint/item/close-check", post(closure_check))
        .route("/v1/work/timing/status", get(timing_status))
        .route("/v1/work/velocity", get(velocity))
        .route("/v1/task/closure/check", post(closure_check))
}
