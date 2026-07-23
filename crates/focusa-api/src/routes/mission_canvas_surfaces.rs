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
    types::{Action, FocusaEvent, MissionCanvasSurfaceStatus, MissionCanvasWorkSurfaceRecord},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Duration};
type ApiError = (StatusCode, Json<Box<ToolResultV1>>);
const ENDPOINT: &str = "/v1/mission-canvas/surfaces/mutate";
#[derive(Debug, Deserialize)]
pub struct SurfaceQuery {
    project_root: String,
    continuity_id: String,
    attachment_id: String,
    #[serde(default)]
    work_surface_id: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceAction {
    Create,
    Arrange,
    Suspend,
    Resume,
    CloseView,
}
#[derive(Debug, Deserialize)]
pub struct SurfaceRequest {
    project_root: String,
    continuity_id: String,
    attachment_id: String,
    idempotency_key: String,
    expected_state_version: u64,
    expected_surface_revision: u64,
    action: SurfaceAction,
    #[serde(default)]
    work_surface_id: Option<String>,
    #[serde(default)]
    instance_id: Option<String>,
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    workpoint_id: Option<String>,
    #[serde(default)]
    mission_ref: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    surface_kind: Option<String>,
    #[serde(default)]
    pane_id: Option<String>,
    #[serde(default)]
    tab_index: Option<u32>,
    #[serde(default)]
    pinned: Option<bool>,
    #[serde(default)]
    unread: Option<bool>,
    #[serde(default)]
    canonical_state_refs: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct SurfaceList {
    schema: &'static str,
    state_version: u64,
    surfaces: Vec<MissionCanvasWorkSurfaceRecord>,
}
#[derive(Debug, Serialize)]
pub struct SurfaceResult {
    schema: &'static str,
    state_version: u64,
    replayed: bool,
    surface: MissionCanvasWorkSurfaceRecord,
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
    x.tool = Some("focusa_mission_canvas_surface_mutate".into());
    x.family = Some("mission_canvas".into());
    x.endpoint = Some(ENDPOINT.into());
    (code, Json(Box::new(x)))
}
fn stable(parts: &[&str]) -> String {
    let mut h = Sha256::new();
    for part in parts {
        h.update(part.as_bytes());
        h.update([0]);
    }
    format!("work-surface:{}", &hex::encode(h.finalize())[..24])
}
fn scoped(x: &MissionCanvasWorkSurfaceRecord, p: &str, c: &str, a: &str) -> bool {
    x.project_root == p && x.continuity_id == c && x.attachment_id == a
}
fn response(
    surface: MissionCanvasWorkSurfaceRecord,
    version: u64,
    replayed: bool,
) -> SurfaceResult {
    let evidence = format!(
        "evidence:mission-canvas-surface:{}:r{}",
        surface.work_surface_id, surface.state_revision
    );
    let receipt = format!(
        "receipt:mission-canvas-surface:{}:{}",
        surface.work_surface_id, surface.idempotency_key
    );
    let mut result = ToolResultV1::success(
        ToolStatus::Completed,
        if replayed {
            "Mission Canvas surface mutation replayed idempotently"
        } else {
            "Mission Canvas surface revision committed"
        },
    );
    result.tool = Some("focusa_mission_canvas_surface_mutate".into());
    result.family = Some("mission_canvas".into());
    result.endpoint = Some(ENDPOINT.into());
    result.evidence_refs = vec![evidence.clone(), receipt.clone()];
    SurfaceResult {
        schema: "focusa.mission_canvas_surface_mutation_result.v1",
        state_version: version,
        replayed,
        surface,
        evidence_ref: evidence,
        receipt_ref: receipt,
        tool_result: result,
    }
}
pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SurfaceQuery>,
) -> Result<Json<SurfaceList>, ApiError> {
    if [&q.project_root, &q.continuity_id, &q.attachment_id]
        .iter()
        .any(|x| x.trim().is_empty())
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ScopeMismatch,
            "exact project/workstream/attachment scope required",
        ));
    }
    let state = state.focusa.read().await;
    let surfaces = state
        .mission_canvas_surfaces
        .iter()
        .filter(|surface| {
            scoped(surface, &q.project_root, &q.continuity_id, &q.attachment_id)
                && q.work_surface_id
                    .as_ref()
                    .is_none_or(|id| id == &surface.work_surface_id)
        })
        .cloned()
        .collect();
    Ok(Json(SurfaceList {
        schema: "focusa.mission_canvas_surface_list.v1",
        state_version: state.version,
        surfaces,
    }))
}
pub async fn mutate(
    State(state): State<Arc<AppState>>,
    Json(r): Json<SurfaceRequest>,
) -> Result<Json<SurfaceResult>, ApiError> {
    if [
        &r.project_root,
        &r.continuity_id,
        &r.attachment_id,
        &r.idempotency_key,
    ]
    .iter()
    .any(|x| x.trim().is_empty())
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ScopeMismatch,
            "exact scope and idempotency required",
        ));
    }
    let snapshot = state.focusa.read().await;
    if let Some(existing) = snapshot.mission_canvas_surfaces.iter().find(|surface| {
        scoped(surface, &r.project_root, &r.continuity_id, &r.attachment_id)
            && surface.idempotency_key == r.idempotency_key
    }) {
        return Ok(Json(response(existing.clone(), snapshot.version, true)));
    }
    if snapshot.version != r.expected_state_version {
        return Err(fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "stale canonical state version",
        ));
    }
    let latest = r
        .work_surface_id
        .as_ref()
        .and_then(|id| {
            snapshot
                .mission_canvas_surfaces
                .iter()
                .filter(|surface| {
                    surface.work_surface_id == *id
                        && scoped(surface, &r.project_root, &r.continuity_id, &r.attachment_id)
                })
                .max_by_key(|surface| surface.state_revision)
        })
        .cloned();
    if latest.as_ref().map_or(0, |surface| surface.state_revision) != r.expected_surface_revision {
        return Err(fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "stale Work Surface revision",
        ));
    }
    let now = Utc::now();
    let mut surface = if matches!(r.action, SurfaceAction::Create) {
        if latest.is_some() {
            return Err(fail(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::WriterConflict,
                "Work Surface already exists",
            ));
        }
        let instance = r
            .instance_id
            .filter(|x| !x.trim().is_empty())
            .ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "instance_id required",
                )
            })?;
        let mission = r
            .mission_ref
            .filter(|x| !x.trim().is_empty())
            .ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "mission_ref required",
                )
            })?;
        let title = r.title.filter(|x| !x.trim().is_empty()).ok_or_else(|| {
            fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                "title required",
            )
        })?;
        let kind = r
            .surface_kind
            .filter(|x| !x.trim().is_empty())
            .ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "surface_kind required",
                )
            })?;
        if r.canonical_state_refs.is_empty()
            || r.canonical_state_refs
                .iter()
                .any(|x| x.trim().is_empty() || x.len() > 512)
        {
            return Err(fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                "bounded canonical_state_refs required; inline canonical payloads prohibited",
            ));
        }
        MissionCanvasWorkSurfaceRecord {
            work_surface_id: r.work_surface_id.clone().unwrap_or_else(|| {
                stable(&[
                    &r.project_root,
                    &r.continuity_id,
                    &r.attachment_id,
                    &instance,
                ])
            }),
            state_revision: 1,
            project_root: r.project_root.clone(),
            continuity_id: r.continuity_id.clone(),
            attachment_id: r.attachment_id.clone(),
            instance_id: instance,
            session_id: r.session_id,
            workpoint_id: r.workpoint_id,
            mission_ref: mission,
            title,
            surface_kind: kind,
            status: MissionCanvasSurfaceStatus::Active,
            pane_id: r
                .pane_id
                .clone()
                .filter(|x| !x.trim().is_empty())
                .unwrap_or_else(|| "primary".into()),
            tab_index: r.tab_index.unwrap_or(0),
            pinned: r.pinned.unwrap_or(false),
            unread: r.unread.unwrap_or(false),
            canonical_state_refs: r.canonical_state_refs,
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
                "Work Surface missing",
            )
        })?;
        x.state_revision += 1;
        x.idempotency_key = r.idempotency_key.clone();
        x.updated_at = now;
        x
    };
    match r.action {
        SurfaceAction::Create => {}
        SurfaceAction::Arrange => {
            if let Some(pane) = r.pane_id.filter(|x| !x.trim().is_empty()) {
                surface.pane_id = pane
            }
            if let Some(index) = r.tab_index {
                surface.tab_index = index
            }
            if let Some(pinned) = r.pinned {
                surface.pinned = pinned
            }
            if let Some(unread) = r.unread {
                surface.unread = unread
            }
        }
        SurfaceAction::Suspend => surface.status = MissionCanvasSurfaceStatus::Suspended,
        SurfaceAction::Resume => surface.status = MissionCanvasSurfaceStatus::Active,
        SurfaceAction::CloseView => surface.status = MissionCanvasSurfaceStatus::ViewClosed,
    }
    let id = surface.work_surface_id.clone();
    let key = surface.idempotency_key.clone();
    drop(snapshot);
    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::MissionCanvasSurfaceRevised { surface },
        })
        .await
        .map_err(|_| {
            fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "Mission Canvas surface command channel unavailable",
            )
        })?;
    for _ in 0..100 {
        let current = state.focusa.read().await;
        if let Some(saved) = current
            .mission_canvas_surfaces
            .iter()
            .find(|surface| surface.work_surface_id == id && surface.idempotency_key == key)
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
        "Mission Canvas surface revision not visible",
    ))
}
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/mission-canvas/surfaces", get(list))
        .route(ENDPOINT, post(mutate))
}
