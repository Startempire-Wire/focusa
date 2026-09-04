//! Visual workflow evidence routes.
//!
//! POST /v1/visual-workflow/evidence/store — persist a visual evidence artifact via reducer action channel
//! GET  /v1/visual-workflow/evidence       — list visual evidence handles (optionally filtered)

use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::reference::store::ReferenceStore;
use focusa_core::reference::{DEFAULT_HOT_HANDLE_LIMIT, retain_hot_handles};
use focusa_core::types::{HandleKind, SessionStatus};
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

type VisualResult<T = Json<serde_json::Value>> = Result<T, (StatusCode, Json<serde_json::Value>)>;

fn expand_visual_data_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

fn visual_ecs_root(data_dir: &str) -> PathBuf {
    expand_visual_data_path(data_dir).join("ecs")
}

fn visual_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    let error = error.into();
    let why = why.into();
    (
        http_status,
        Json(json!({
            "status": "blocked", "canonical": false, "degraded": true,
            "error": error, "failure_class": failure_class, "why": why,
            "recovery_hint": recovery_hint, "misuse_hint": misuse_hint,
            "next_tools": ["focusa_tool_doctor", "focusa_evidence_capture"],
            "details": {"tool_result_v1": {"ok": false, "status": "blocked", "canonical": false, "degraded": true, "failure_class": failure_class, "summary": why, "retry": {"safe": failure_class != "validation_rejected", "posture": if failure_class == "validation_rejected" { "do_not_retry_unchanged" } else { "safe_retry" }, "reason": failure_class}, "recovery_hint": recovery_hint, "misuse_hint": misuse_hint, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_tool_doctor", "focusa_evidence_capture"], "error": {"code": failure_class, "message": error}}}
        })),
    )
}

fn visual_content_rejected(reason: &str) -> (StatusCode, Json<serde_json::Value>) {
    visual_failure(
        StatusCode::BAD_REQUEST,
        format!("invalid visual evidence content: {reason}"),
        "validation_rejected",
        "Visual evidence store requires valid content_b64 or plain content.",
        "Send valid base64 content_b64 or non-empty content before retrying unchanged.",
        "Likely missing visual artifact content or malformed base64 from browser/vision capture.",
    )
}

#[derive(Debug, Clone, Deserialize)]
struct StoreVisualEvidenceBody {
    run_id: String,
    phase: String,
    evidence_kind: String,
    label: String,
    kind: HandleKind,
    /// Base64-encoded content.
    content_b64: Option<String>,
    /// Plain text content (alternative to base64).
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    project_root: Option<String>,
    #[serde(default)]
    continuity_id: Option<String>,
    #[serde(default)]
    workpoint_id: Option<String>,
}

impl StoreVisualEvidenceBody {
    fn resolve_content(&self) -> VisualResult<Vec<u8>> {
        if let Some(ref b64) = self.content_b64 {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(b64)
                .map_err(|_| visual_content_rejected("content_b64 is not valid base64"))
        } else if let Some(ref txt) = self.content {
            Ok(txt.as_bytes().to_vec())
        } else {
            Err(visual_content_rejected(
                "content_b64 and content are both missing",
            ))
        }
    }

    fn to_artifact_label(&self) -> String {
        format!(
            "visual:{}:{}:{}:{}",
            self.run_id, self.phase, self.evidence_kind, self.label
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
struct EvidenceQuery {
    #[serde(default)]
    run_id: Option<String>,
    #[serde(default)]
    phase: Option<String>,
    #[serde(default)]
    evidence_kind: Option<String>,
}

async fn store_visual_evidence(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StoreVisualEvidenceBody>,
) -> VisualResult {
    let content = body.resolve_content()?;
    let label = body.to_artifact_label();

    let handle_id = uuid::Uuid::now_v7();
    let session = state.focusa.read().await.session.clone();
    let session_id = session.as_ref().map(|s| s.session_id);
    let project_root = body
        .project_root
        .clone()
        .or_else(|| session.as_ref().and_then(|s| s.project_root.clone()));
    let continuity_id = body
        .continuity_id
        .clone()
        .or_else(|| session.as_ref().and_then(|s| s.continuity_id.clone()));
    let store = ReferenceStore::new(visual_ecs_root(&state.config.data_dir)).map_err(|error| {
        visual_failure(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to open ECS reference store: {error}"),
            "storage_unavailable",
            "visual workflow evidence store could not be opened",
            "check data_dir/ecs permissions and retry after storage health is clear",
            "likely data_dir permission, disk, or path expansion issue",
        )
    })?;
    let mut handle = store
        .store(
            body.kind,
            label.clone(),
            &content,
            session_id,
            Some(handle_id),
            project_root,
            continuity_id,
        )
        .map_err(|error| {
            visual_failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to store visual evidence artifact: {error}"),
                "storage_unavailable",
                "visual workflow evidence content or metadata could not be written",
                "check data_dir/ecs permissions and retry after storage health is clear",
                "likely data_dir permission, disk, or path expansion issue",
            )
        })?;
    let _guard = state.write_serial_lock.lock().await;
    let mut focusa = state.focusa.write().await;
    handle.trajectory = focusa.trajectory_ladder_context_for_scope(
        handle.project_root.as_deref(),
        handle.continuity_id.as_deref(),
    );
    store.persist_metadata(&handle).map_err(|error| {
        visual_failure(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to bind visual ECS metadata: {error}"),
            "storage_unavailable",
            "visual evidence was stored but its scoped trajectory metadata could not be committed",
            "check data_dir/ecs permissions; retry safely with a new evidence id",
            "likely metadata directory permission, disk, or durability failure",
        )
    })?;
    if !focusa
        .reference_index
        .handles
        .iter()
        .any(|h| h.id == handle.id)
    {
        focusa.reference_index.handles.push(handle.clone());
        let active_session_id = session
            .as_ref()
            .filter(|session| session.status == SessionStatus::Active)
            .map(|session| session.session_id);
        retain_hot_handles(
            &mut focusa.reference_index,
            active_session_id,
            DEFAULT_HOT_HANDLE_LIMIT,
        );
        state.mark_external_mutation();
    }
    drop(focusa);

    Ok(Json(json!({
        "id": handle.id,
        "handle": handle,
        "status": "accepted",
        "run_id": body.run_id,
        "phase": body.phase,
        "evidence_kind": body.evidence_kind,
        "label": body.label,
        "scope": {
            "project_root": body.project_root,
            "continuity_id": body.continuity_id,
            "workpoint_id": body.workpoint_id,
        },
        "tool_result_v1": {
            "ok": true,
            "status": "accepted",
            "canonical": true,
            "degraded": false,
            "failure_class": null,
            "summary": "visual workflow evidence stored with exact handle",
            "evidence_refs": [format!("focusa-handle:{}", handle.id)],
            "next_tools": ["focusa_evidence_capture", "focusa_workpoint_link_evidence"],
            "side_effects": ["reference_store_write"]
        }
    })))
}

async fn list_visual_evidence(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EvidenceQuery>,
) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;

    let mut items = Vec::new();
    for handle in focusa
        .reference_index
        .handles
        .iter()
        .filter(|h| h.label.starts_with("visual:"))
    {
        let mut parts = handle.label.splitn(5, ':');
        let prefix = parts.next().unwrap_or_default();
        let run_id = parts.next().unwrap_or_default();
        let phase = parts.next().unwrap_or_default();
        let evidence_kind = parts.next().unwrap_or_default();
        let label = parts.next().unwrap_or_default();

        if prefix != "visual" {
            continue;
        }
        if query.run_id.as_deref().is_some_and(|q| q != run_id) {
            continue;
        }
        if query.phase.as_deref().is_some_and(|q| q != phase) {
            continue;
        }
        if query
            .evidence_kind
            .as_deref()
            .is_some_and(|q| q != evidence_kind)
        {
            continue;
        }

        items.push(json!({
            "run_id": run_id,
            "phase": phase,
            "evidence_kind": evidence_kind,
            "label": label,
            "handle": handle,
        }));
    }

    Json(json!({
        "evidence": items,
        "count": items.len(),
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/v1/visual-workflow/evidence/store",
            post(store_visual_evidence),
        )
        .route("/v1/visual-workflow/evidence", get(list_visual_evidence))
}
