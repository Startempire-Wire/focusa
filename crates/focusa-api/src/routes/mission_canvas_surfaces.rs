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
        Action, FocusaEvent, MissionCanvasBindingKind, MissionCanvasBrowserIsolationClass,
        MissionCanvasStateRecord, MissionCanvasSurfaceBindingRecord, MissionCanvasSurfaceStatus,
        MissionCanvasWorkSurfaceRecord,
    },
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
#[derive(Debug, Deserialize)]
pub struct BindingQuery {
    project_root: String,
    continuity_id: String,
    attachment_id: String,
    work_surface_id: String,
    #[serde(default)]
    binding_id: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingAction {
    Bind,
    Unbind,
}
#[derive(Debug, Deserialize)]
pub struct BindingRequest {
    project_root: String,
    continuity_id: String,
    attachment_id: String,
    work_surface_id: String,
    idempotency_key: String,
    expected_state_version: u64,
    expected_binding_revision: u64,
    action: BindingAction,
    #[serde(default)]
    binding_id: Option<String>,
    #[serde(default)]
    binding_kind: Option<MissionCanvasBindingKind>,
    #[serde(default)]
    target_ref: Option<String>,
    #[serde(default)]
    access_mode: Option<String>,
    #[serde(default)]
    browser_isolation_class: Option<MissionCanvasBrowserIsolationClass>,
    #[serde(default)]
    authentication_sharing: Option<String>,
    #[serde(default)]
    retention_policy: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct BindingList {
    schema: &'static str,
    state_version: u64,
    bindings: Vec<MissionCanvasSurfaceBindingRecord>,
}
#[derive(Debug, Serialize)]
pub struct BindingResult {
    schema: &'static str,
    state_version: u64,
    replayed: bool,
    binding: MissionCanvasSurfaceBindingRecord,
    evidence_ref: String,
    receipt_ref: String,
    tool_result: ToolResultV1,
}
fn binding_scoped(x: &MissionCanvasSurfaceBindingRecord, q: &BindingQuery) -> bool {
    x.project_root == q.project_root
        && x.continuity_id == q.continuity_id
        && x.attachment_id == q.attachment_id
        && x.work_surface_id == q.work_surface_id
}
fn binding_fail(
    code: StatusCode,
    status: ToolStatus,
    class: FailureClass,
    message: impl Into<String>,
) -> ApiError {
    let mut x = ToolResultV1::failure(status, class, message.into());
    x.tool = Some("focusa_mission_canvas_binding_mutate".into());
    x.family = Some("mission_canvas".into());
    x.endpoint = Some("/v1/mission-canvas/surface-bindings/mutate".into());
    (code, Json(Box::new(x)))
}
fn binding_response(
    binding: MissionCanvasSurfaceBindingRecord,
    version: u64,
    replayed: bool,
) -> BindingResult {
    let evidence = format!(
        "evidence:surface-binding:{}:r{}",
        binding.binding_id, binding.state_revision
    );
    let receipt = format!(
        "receipt:surface-binding:{}:{}",
        binding.binding_id, binding.idempotency_key
    );
    let mut result = ToolResultV1::success(
        ToolStatus::Completed,
        if replayed {
            "Surface binding replayed idempotently"
        } else {
            "Exact attachment binding revision committed"
        },
    );
    result.tool = Some("focusa_mission_canvas_binding_mutate".into());
    result.family = Some("mission_canvas".into());
    result.endpoint = Some("/v1/mission-canvas/surface-bindings/mutate".into());
    result.evidence_refs = vec![evidence.clone(), receipt.clone()];
    BindingResult {
        schema: "focusa.mission_canvas_surface_binding_mutation_result.v1",
        state_version: version,
        replayed,
        binding,
        evidence_ref: evidence,
        receipt_ref: receipt,
        tool_result: result,
    }
}
pub async fn list_bindings(
    State(state): State<Arc<AppState>>,
    Query(q): Query<BindingQuery>,
) -> Result<Json<BindingList>, ApiError> {
    if [
        &q.project_root,
        &q.continuity_id,
        &q.attachment_id,
        &q.work_surface_id,
    ]
    .iter()
    .any(|x| x.trim().is_empty())
    {
        return Err(binding_fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ScopeMismatch,
            "exact project/workstream/attachment/surface scope required",
        ));
    }
    let state = state.focusa.read().await;
    let bindings = state
        .mission_canvas_surface_bindings
        .iter()
        .filter(|binding| {
            binding_scoped(binding, &q)
                && q.binding_id
                    .as_ref()
                    .is_none_or(|id| id == &binding.binding_id)
        })
        .cloned()
        .collect();
    Ok(Json(BindingList {
        schema: "focusa.mission_canvas_surface_binding_list.v1",
        state_version: state.version,
        bindings,
    }))
}
pub async fn mutate_binding(
    State(state): State<Arc<AppState>>,
    Json(r): Json<BindingRequest>,
) -> Result<Json<BindingResult>, ApiError> {
    if [
        &r.project_root,
        &r.continuity_id,
        &r.attachment_id,
        &r.work_surface_id,
        &r.idempotency_key,
    ]
    .iter()
    .any(|x| x.trim().is_empty())
    {
        return Err(binding_fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ScopeMismatch,
            "exact attachment/surface scope and idempotency required",
        ));
    }
    let snapshot = state.focusa.read().await;
    if let Some(existing) = snapshot
        .mission_canvas_surface_bindings
        .iter()
        .find(|binding| {
            binding.project_root == r.project_root
                && binding.continuity_id == r.continuity_id
                && binding.attachment_id == r.attachment_id
                && binding.work_surface_id == r.work_surface_id
                && binding.idempotency_key == r.idempotency_key
        })
    {
        return Ok(Json(binding_response(
            existing.clone(),
            snapshot.version,
            true,
        )));
    }
    if snapshot.version != r.expected_state_version {
        return Err(binding_fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "stale canonical state version",
        ));
    }
    let surface_exists = snapshot.mission_canvas_surfaces.iter().any(|surface| {
        surface.work_surface_id == r.work_surface_id
            && surface.project_root == r.project_root
            && surface.continuity_id == r.continuity_id
            && surface.attachment_id == r.attachment_id
    });
    if !surface_exists {
        return Err(binding_fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ScopeMismatch,
            "Cross-surface binding mutation denied: target Work Surface is absent from exact attachment scope",
        ));
    }
    let latest = r
        .binding_id
        .as_ref()
        .and_then(|id| {
            snapshot
                .mission_canvas_surface_bindings
                .iter()
                .filter(|binding| {
                    binding.binding_id == *id
                        && binding.project_root == r.project_root
                        && binding.continuity_id == r.continuity_id
                        && binding.attachment_id == r.attachment_id
                        && binding.work_surface_id == r.work_surface_id
                })
                .max_by_key(|binding| binding.state_revision)
        })
        .cloned();
    if latest.as_ref().map_or(0, |binding| binding.state_revision) != r.expected_binding_revision {
        return Err(binding_fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "stale surface binding revision",
        ));
    }
    let now = Utc::now();
    let binding = match r.action {
        BindingAction::Bind => {
            if latest.is_some() {
                return Err(binding_fail(
                    StatusCode::CONFLICT,
                    ToolStatus::Blocked,
                    FailureClass::WriterConflict,
                    "binding already exists",
                ));
            }
            let kind = r.binding_kind.ok_or_else(|| {
                binding_fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "binding_kind required",
                )
            })?;
            let target = r
                .target_ref
                .filter(|x| !x.trim().is_empty())
                .ok_or_else(|| {
                    binding_fail(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ToolStatus::ValidationRejected,
                        FailureClass::ValidationRejected,
                        "target_ref required",
                    )
                })?;
            let mode = r
                .access_mode
                .filter(|x| matches!(x.as_str(), "read" | "write" | "invoke"))
                .ok_or_else(|| {
                    binding_fail(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ToolStatus::ValidationRejected,
                        FailureClass::PermissionDenied,
                        "read, write, or invoke access_mode required",
                    )
                })?;
            let (browser_isolation_class, authentication_sharing, retention_policy) = if kind
                == MissionCanvasBindingKind::BrowserContext
            {
                let owns_session = snapshot
                    .mission_canvas_surface_bindings
                    .iter()
                    .any(|binding| {
                        binding.active
                            && binding.binding_kind == MissionCanvasBindingKind::Session
                            && binding.project_root == r.project_root
                            && binding.continuity_id == r.continuity_id
                            && binding.attachment_id == r.attachment_id
                            && binding.work_surface_id == r.work_surface_id
                    });
                if !owns_session {
                    return Err(binding_fail(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ToolStatus::ValidationRejected,
                        FailureClass::ScopeMismatch,
                        "Browser context requires an active UIAI session binding in the exact Work Surface attachment scope",
                    ));
                }
                let isolation = r.browser_isolation_class.ok_or_else(|| {
                    binding_fail(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ToolStatus::ValidationRejected,
                        FailureClass::ValidationRejected,
                        "browser_isolation_class required for browser contexts",
                    )
                })?;
                let expected_sharing =
                    if isolation == MissionCanvasBrowserIsolationClass::SharedAuthenticated {
                        "shared_explicit"
                    } else {
                        "isolated"
                    };
                let sharing = r
                    .authentication_sharing
                    .filter(|value| value == expected_sharing)
                    .ok_or_else(|| {
                        binding_fail(
                            StatusCode::UNPROCESSABLE_ENTITY,
                            ToolStatus::ValidationRejected,
                            FailureClass::PermissionDenied,
                            format!(
                                "browser context requires authentication_sharing={expected_sharing}"
                            ),
                        )
                    })?;
                let retention = r
                        .retention_policy
                        .filter(|value| {
                            matches!(
                                value.as_str(),
                                "persistent" | "dispose_on_close" | "manual"
                            )
                        })
                        .ok_or_else(|| {
                            binding_fail(
                                StatusCode::UNPROCESSABLE_ENTITY,
                                ToolStatus::ValidationRejected,
                                FailureClass::ValidationRejected,
                                "browser context retention_policy must be persistent, dispose_on_close, or manual",
                            )
                        })?;
                if let Some(existing) = snapshot
                    .mission_canvas_surface_bindings
                    .iter()
                    .filter(|binding| binding.active)
                    .find(|binding| {
                        binding.binding_kind == MissionCanvasBindingKind::BrowserContext
                            && binding.target_ref == target
                            && (binding.project_root != r.project_root
                                || binding.continuity_id != r.continuity_id
                                || binding.attachment_id != r.attachment_id
                                || binding.work_surface_id != r.work_surface_id)
                    })
                {
                    let cross_project = existing.project_root != r.project_root
                        || existing.continuity_id != r.continuity_id;
                    let explicitly_shared = !cross_project
                        && isolation == MissionCanvasBrowserIsolationClass::SharedAuthenticated
                        && existing.browser_isolation_class
                            == Some(MissionCanvasBrowserIsolationClass::SharedAuthenticated)
                        && existing.authentication_sharing.as_deref() == Some("shared_explicit");
                    if !explicitly_shared {
                        return Err(binding_fail(
                            StatusCode::CONFLICT,
                            ToolStatus::ValidationRejected,
                            FailureClass::ScopeMismatch,
                            "Browser context reuse denied: exact attachment ownership or explicit same-project shared authentication is required",
                        ));
                    }
                }
                (Some(isolation), Some(sharing), Some(retention))
            } else {
                if kind == MissionCanvasBindingKind::BrowserTarget {
                    let owns_context =
                        snapshot
                            .mission_canvas_surface_bindings
                            .iter()
                            .any(|binding| {
                                binding.active
                                    && binding.binding_kind
                                        == MissionCanvasBindingKind::BrowserContext
                                    && binding.project_root == r.project_root
                                    && binding.continuity_id == r.continuity_id
                                    && binding.attachment_id == r.attachment_id
                                    && binding.work_surface_id == r.work_surface_id
                            });
                    if !owns_context {
                        return Err(binding_fail(
                            StatusCode::UNPROCESSABLE_ENTITY,
                            ToolStatus::ValidationRejected,
                            FailureClass::ScopeMismatch,
                            "Browser target requires an active browser context owned by the exact Work Surface attachment",
                        ));
                    }
                }
                if r.browser_isolation_class.is_some()
                    || r.authentication_sharing.is_some()
                    || r.retention_policy.is_some()
                {
                    return Err(binding_fail(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ToolStatus::ValidationRejected,
                        FailureClass::ValidationRejected,
                        "browser isolation metadata is valid only for browser_context bindings",
                    ));
                }
                (None, None, None)
            };
            MissionCanvasSurfaceBindingRecord {
                binding_id: r.binding_id.unwrap_or_else(|| {
                    stable(&[
                        &r.project_root,
                        &r.continuity_id,
                        &r.attachment_id,
                        &r.work_surface_id,
                        &target,
                    ])
                }),
                state_revision: 1,
                project_root: r.project_root.clone(),
                continuity_id: r.continuity_id.clone(),
                attachment_id: r.attachment_id.clone(),
                work_surface_id: r.work_surface_id.clone(),
                binding_kind: kind,
                target_ref: target,
                access_mode: mode,
                browser_isolation_class,
                authentication_sharing,
                retention_policy,
                active: true,
                idempotency_key: r.idempotency_key.clone(),
                created_at: now,
                updated_at: now,
            }
        }
        BindingAction::Unbind => {
            let mut x = latest.ok_or_else(|| {
                binding_fail(
                    StatusCode::NOT_FOUND,
                    ToolStatus::Blocked,
                    FailureClass::NotFound,
                    "surface binding missing",
                )
            })?;
            x.state_revision += 1;
            x.active = false;
            x.idempotency_key = r.idempotency_key.clone();
            x.updated_at = now;
            x
        }
    };
    let id = binding.binding_id.clone();
    let key = binding.idempotency_key.clone();
    drop(snapshot);
    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::MissionCanvasSurfaceBindingRevised { binding },
        })
        .await
        .map_err(|_| {
            binding_fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "surface binding command channel unavailable",
            )
        })?;
    for _ in 0..100 {
        let current = state.focusa.read().await;
        if let Some(saved) = current
            .mission_canvas_surface_bindings
            .iter()
            .find(|binding| binding.binding_id == id && binding.idempotency_key == key)
        {
            return Ok(Json(binding_response(
                saved.clone(),
                current.version,
                false,
            )));
        }
        drop(current);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(binding_fail(
        StatusCode::SERVICE_UNAVAILABLE,
        ToolStatus::Degraded,
        FailureClass::ReadModelLag,
        "surface binding revision not visible",
    ))
}
const CANVAS_STATE_ENDPOINT: &str = "/v1/mission-canvas/state/mutate";

#[derive(Debug, Deserialize)]
pub struct CanvasStateQuery {
    project_root: String,
    continuity_id: String,
    client_instance_id: String,
    user_id: String,
    device_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CanvasStateRequest {
    project_root: String,
    continuity_id: String,
    client_instance_id: String,
    user_id: String,
    device_id: String,
    idempotency_key: String,
    expected_state_version: u64,
    expected_canvas_revision: u64,
    #[serde(default)]
    open_work_surface_ids: Vec<String>,
    #[serde(default)]
    focused_work_surface_id: Option<String>,
    #[serde(default)]
    secondary_focused_surface_id: Option<String>,
    #[serde(default)]
    split_layout_ref: Option<String>,
    #[serde(default)]
    group_order: Vec<String>,
    #[serde(default)]
    aggregate_project_roots: Vec<String>,
    #[serde(default)]
    aggregate_continuity_ids: Vec<String>,
    #[serde(default)]
    aggregate_surface_kinds: Vec<String>,
    #[serde(default)]
    aggregate_surface_states: Vec<String>,
    #[serde(default)]
    selected_context_refs: Vec<String>,
    #[serde(default)]
    unread_event_cursor: Option<u64>,
    session_projection_revision: u64,
}

#[derive(Debug, Serialize)]
pub struct CanvasStateView {
    schema: &'static str,
    state_version: u64,
    canvas: MissionCanvasStateRecord,
    surfaces: Vec<MissionCanvasWorkSurfaceRecord>,
    recovery_actions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CanvasStateResult {
    schema: &'static str,
    state_version: u64,
    replayed: bool,
    canvas: MissionCanvasStateRecord,
    evidence_ref: String,
    receipt_ref: String,
    tool_result: ToolResultV1,
}

fn canvas_fail(
    code: StatusCode,
    status: ToolStatus,
    class: FailureClass,
    message: impl Into<String>,
) -> ApiError {
    let mut result = ToolResultV1::failure(status, class, message.into());
    result.tool = Some("focusa_mission_canvas_state_mutate".into());
    result.family = Some("mission_canvas".into());
    result.endpoint = Some(CANVAS_STATE_ENDPOINT.into());
    (code, Json(Box::new(result)))
}

fn canvas_id(request: &CanvasStateRequest) -> String {
    stable(&[
        "canvas",
        &request.project_root,
        &request.continuity_id,
        &request.client_instance_id,
        &request.user_id,
        &request.device_id,
    ])
    .replacen("work-surface:", "mission-canvas:", 1)
}

fn canvas_result(
    canvas: MissionCanvasStateRecord,
    state_version: u64,
    replayed: bool,
) -> CanvasStateResult {
    let evidence_ref = format!(
        "evidence:mission-canvas-state:{}:r{}",
        canvas.canvas_id, canvas.state_revision
    );
    let receipt_ref = format!(
        "receipt:mission-canvas-state:{}:{}",
        canvas.canvas_id, canvas.idempotency_key
    );
    let mut tool_result = ToolResultV1::success(
        ToolStatus::Completed,
        if replayed {
            "Mission Canvas restoration state replayed idempotently"
        } else {
            "Mission Canvas restoration state persisted"
        },
    );
    tool_result.tool = Some("focusa_mission_canvas_state_mutate".into());
    tool_result.family = Some("mission_canvas".into());
    tool_result.endpoint = Some(CANVAS_STATE_ENDPOINT.into());
    tool_result.evidence_refs = vec![evidence_ref.clone(), receipt_ref.clone()];
    CanvasStateResult {
        schema: "focusa.mission_canvas_state_mutation_result.v1",
        state_version,
        replayed,
        canvas,
        evidence_ref,
        receipt_ref,
        tool_result,
    }
}

pub async fn get_canvas_state(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CanvasStateQuery>,
) -> Result<Json<CanvasStateView>, ApiError> {
    if [
        &query.project_root,
        &query.continuity_id,
        &query.client_instance_id,
        &query.user_id,
        &query.device_id,
    ]
    .iter()
    .any(|value| value.trim().is_empty())
    {
        return Err(canvas_fail(
            StatusCode::BAD_REQUEST,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "project, continuity, client, user, and device scope are required",
        ));
    }
    let snapshot = state.focusa.read().await;
    let canvas = snapshot
        .mission_canvas_states
        .iter()
        .filter(|canvas| {
            canvas.project_root == query.project_root
                && canvas.continuity_id == query.continuity_id
                && canvas.client_instance_id == query.client_instance_id
                && canvas.user_id == query.user_id
                && canvas.device_id == query.device_id
        })
        .max_by_key(|canvas| canvas.state_revision)
        .cloned()
        .ok_or_else(|| {
            canvas_fail(
                StatusCode::NOT_FOUND,
                ToolStatus::Blocked,
                FailureClass::NotFound,
                "Mission Canvas restoration state is absent; refusing to manufacture a replacement session or project",
            )
        })?;
    let mut surfaces = Vec::new();
    let mut recovery_actions = Vec::new();
    for surface_id in &canvas.open_work_surface_ids {
        let surface = snapshot
            .mission_canvas_surfaces
            .iter()
            .filter(|surface| {
                surface.work_surface_id == *surface_id
                    && surface.project_root == canvas.project_root
                    && surface.continuity_id == canvas.continuity_id
            })
            .max_by_key(|surface| surface.state_revision)
            .cloned();
        match surface {
            Some(surface) => {
                match surface.status {
                    MissionCanvasSurfaceStatus::Suspended => {
                        recovery_actions.push(format!("resume_surface:{surface_id}"));
                    }
                    MissionCanvasSurfaceStatus::ViewClosed => {
                        recovery_actions.push(format!("reopen_view:{surface_id}"));
                    }
                    MissionCanvasSurfaceStatus::Active => {}
                }
                surfaces.push(surface);
            }
            None => recovery_actions.push(format!("remove_missing_surface:{surface_id}")),
        }
    }
    Ok(Json(CanvasStateView {
        schema: "focusa.mission_canvas_state.v1",
        state_version: snapshot.version,
        canvas,
        surfaces,
        recovery_actions,
    }))
}

pub async fn mutate_canvas_state(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CanvasStateRequest>,
) -> Result<Json<CanvasStateResult>, ApiError> {
    if [
        &request.project_root,
        &request.continuity_id,
        &request.client_instance_id,
        &request.user_id,
        &request.device_id,
        &request.idempotency_key,
    ]
    .iter()
    .any(|value| value.trim().is_empty())
    {
        return Err(canvas_fail(
            StatusCode::BAD_REQUEST,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "project, continuity, client, user, device, and idempotency are required",
        ));
    }
    let snapshot = state.focusa.read().await;
    if let Some(existing) = snapshot
        .mission_canvas_states
        .iter()
        .find(|canvas| {
            canvas.project_root == request.project_root
                && canvas.continuity_id == request.continuity_id
                && canvas.client_instance_id == request.client_instance_id
                && canvas.user_id == request.user_id
                && canvas.device_id == request.device_id
                && canvas.idempotency_key == request.idempotency_key
        })
        .cloned()
    {
        return Ok(Json(canvas_result(existing, snapshot.version, true)));
    }
    if snapshot.version != request.expected_state_version {
        return Err(canvas_fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "stale canonical state version",
        ));
    }
    let canvas_id = canvas_id(&request);
    let latest = snapshot
        .mission_canvas_states
        .iter()
        .filter(|canvas| canvas.canvas_id == canvas_id)
        .max_by_key(|canvas| canvas.state_revision);
    if latest.map_or(0, |canvas| canvas.state_revision) != request.expected_canvas_revision {
        return Err(canvas_fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "stale Mission Canvas restoration revision",
        ));
    }
    let exact_surfaces = request.open_work_surface_ids.iter().all(|surface_id| {
        snapshot.mission_canvas_surfaces.iter().any(|surface| {
            surface.work_surface_id == *surface_id
                && surface.project_root == request.project_root
                && surface.continuity_id == request.continuity_id
        })
    });
    if !exact_surfaces {
        return Err(canvas_fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ScopeMismatch,
            "Mission Canvas topology contains a Work Surface outside exact project and continuity scope",
        ));
    }
    let now = Utc::now();
    let canvas = MissionCanvasStateRecord {
        canvas_id,
        state_revision: request.expected_canvas_revision + 1,
        project_root: request.project_root,
        continuity_id: request.continuity_id,
        client_instance_id: request.client_instance_id,
        user_id: request.user_id,
        device_id: request.device_id,
        open_work_surface_ids: request.open_work_surface_ids,
        focused_work_surface_id: request.focused_work_surface_id,
        secondary_focused_surface_id: request.secondary_focused_surface_id,
        split_layout_ref: request.split_layout_ref,
        group_order: request.group_order,
        aggregate_project_roots: request.aggregate_project_roots,
        aggregate_continuity_ids: request.aggregate_continuity_ids,
        aggregate_surface_kinds: request.aggregate_surface_kinds,
        aggregate_surface_states: request.aggregate_surface_states,
        selected_context_refs: request.selected_context_refs,
        unread_event_cursor: request.unread_event_cursor,
        session_projection_revision: request.session_projection_revision,
        idempotency_key: request.idempotency_key,
        created_at: latest.map_or(now, |canvas| canvas.created_at),
        updated_at: now,
    };
    let id = canvas.canvas_id.clone();
    let key = canvas.idempotency_key.clone();
    drop(snapshot);
    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::MissionCanvasStateRevised { canvas },
        })
        .await
        .map_err(|_| {
            canvas_fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "Mission Canvas state command channel unavailable",
            )
        })?;
    for _ in 0..100 {
        let current = state.focusa.read().await;
        if let Some(saved) = current
            .mission_canvas_states
            .iter()
            .find(|canvas| canvas.canvas_id == id && canvas.idempotency_key == key)
        {
            return Ok(Json(canvas_result(saved.clone(), current.version, false)));
        }
        drop(current);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(canvas_fail(
        StatusCode::SERVICE_UNAVAILABLE,
        ToolStatus::Degraded,
        FailureClass::ReadModelLag,
        "Mission Canvas restoration revision not visible",
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/mission-canvas/surfaces", get(list))
        .route(ENDPOINT, post(mutate))
        .route("/v1/mission-canvas/surface-bindings", get(list_bindings))
        .route(
            "/v1/mission-canvas/surface-bindings/mutate",
            post(mutate_binding),
        )
        .route("/v1/mission-canvas/state", get(get_canvas_state))
        .route(CANVAS_STATE_ENDPOINT, post(mutate_canvas_state))
}
