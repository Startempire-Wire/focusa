//! ECS routes.
//!
//! GET  /v1/ecs/handles              — list all handles
//! POST /v1/ecs/store                — store an artifact
//! GET  /v1/ecs/resolve/:handle_id   — resolve a handle (metadata)
//! GET  /v1/ecs/content/:handle_id   — get artifact content
//! POST /v1/ecs/rehydrate/:handle_id — rehydrate with token limit

use crate::routes::bounded::{
    BoundedReadOptions, bounded_metadata, env_limit, full_payload_blocked_by_pressure,
    pressure_status, record_json_response_size,
};
use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::{Action, HandleKind, HandleRef};
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

const DEFAULT_HANDLES_LIMIT: usize = 100;
const MAX_HANDLES_LIMIT: usize = 512;

type EcsResult<T = Json<serde_json::Value>> = Result<T, (StatusCode, Json<serde_json::Value>)>;

fn ecs_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
    next_tools: Vec<&'static str>,
) -> (StatusCode, Json<serde_json::Value>) {
    let error = error.into();
    let why = why.into();
    let next_tools_value = json!(next_tools);
    let retry_safe = !matches!(failure_class, "validation_rejected" | "not_found");
    let retry_posture = if retry_safe {
        "safe_retry"
    } else {
        "do_not_retry_unchanged"
    };
    (
        http_status,
        Json(json!({
            "status": "blocked",
            "canonical": false,
            "degraded": true,
            "error": error,
            "failure_class": failure_class,
            "why": why,
            "recovery_hint": recovery_hint,
            "misuse_hint": misuse_hint,
            "next_tools": next_tools_value.clone(),
            "details": {
                "tool_result_v1": {
                    "ok": false,
                    "status": "blocked",
                    "canonical": false,
                    "degraded": true,
                    "failure_class": failure_class,
                    "summary": why,
                    "retry": {"safe": retry_safe, "posture": retry_posture, "reason": failure_class},
                    "recovery_hint": recovery_hint,
                    "misuse_hint": misuse_hint,
                    "side_effects": [],
                    "evidence_refs": [],
                    "next_tools": next_tools_value,
                    "error": {"code": failure_class, "message": error}
                }
            }
        })),
    )
}

fn ecs_validation_rejected(why: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    let why = why.into();
    ecs_failure(
        StatusCode::BAD_REQUEST,
        why.clone(),
        "validation_rejected",
        why,
        "Provide content_b64 or content with valid encoding before retrying unchanged.",
        "Likely missing ECS content or invalid base64 content_b64 field.",
        vec!["focusa_tool_doctor", "focusa_active_object_resolve"],
    )
}

fn ecs_dispatch_failed(error: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    ecs_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("failed to dispatch ECS artifact store: {error}"),
        "daemon_unavailable",
        "ECS artifact store event could not be dispatched to daemon command channel.",
        "Check daemon health and retry after command channel recovery is clear.",
        "Likely daemon command channel closed, runtime shutdown, or writer/transport ownership issue.",
        vec!["focusa_tool_doctor", "focusa_work_loop_status"],
    )
}

fn ecs_handle_not_found(handle_id: uuid::Uuid) -> (StatusCode, Json<serde_json::Value>) {
    ecs_failure(
        StatusCode::NOT_FOUND,
        "handle not found",
        "not_found",
        format!("ECS handle {handle_id} is not present in the reference index"),
        "Use /v1/ecs/handles or focusa_traverse to discover valid handle ids before resolving content.",
        "Likely stale handle id, wrong daemon instance, or artifact not materialized yet.",
        vec!["focusa_traverse", "focusa_tool_doctor"],
    )
}

fn ecs_blob_not_found(handle_id: uuid::Uuid) -> (StatusCode, Json<serde_json::Value>) {
    ecs_failure(
        StatusCode::NOT_FOUND,
        "artifact content not found",
        "not_found",
        format!("ECS blob for handle {handle_id} is missing from object storage"),
        "Verify the handle via /v1/ecs/resolve and check data_dir/object storage before relying on content.",
        "Likely missing blob file, wrong data_dir, artifact cleanup, or persistence issue.",
        vec!["focusa_tool_doctor", "focusa_traverse"],
    )
}

fn ecs_root(data_dir: &str) -> PathBuf {
    expand_data_path(data_dir).join("ecs")
}

fn handle_metadata_path(data_dir: &str, handle_id: uuid::Uuid) -> PathBuf {
    ecs_root(data_dir)
        .join("handles")
        .join(format!("{handle_id}.json"))
}

fn handle_blob_path(data_dir: &str, handle: &HandleRef) -> PathBuf {
    ecs_root(data_dir).join("objects").join(&handle.sha256)
}

fn load_handle_metadata_from_disk(data_dir: &str, handle_id: uuid::Uuid) -> Option<HandleRef> {
    let meta = std::fs::read_to_string(handle_metadata_path(data_dir, handle_id)).ok()?;
    serde_json::from_str(&meta).ok()
}

fn resolve_handle_with_disk_fallback(
    data_dir: &str,
    handles: &[HandleRef],
    handle_id: uuid::Uuid,
) -> Option<HandleRef> {
    handles
        .iter()
        .find(|h| h.id == handle_id)
        .cloned()
        .filter(|handle| !handle.sha256.trim().is_empty())
        .or_else(|| load_handle_metadata_from_disk(data_dir, handle_id))
}

#[derive(Debug, Clone, Deserialize)]
struct StoreBody {
    kind: HandleKind,
    label: String,
    /// Base64-encoded content.
    content_b64: Option<String>,
    /// Plain text content (alternative to base64).
    #[serde(default)]
    content: Option<String>,
}

impl StoreBody {
    fn resolve_content(&self) -> Result<Vec<u8>, (StatusCode, Json<serde_json::Value>)> {
        if let Some(ref b64) = self.content_b64 {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(b64)
                .map_err(|_| ecs_validation_rejected("invalid content_b64"))
        } else if let Some(ref txt) = self.content {
            Ok(txt.as_bytes().to_vec())
        } else {
            Err(ecs_validation_rejected("missing content"))
        }
    }
}

async fn store_artifact(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StoreBody>,
) -> EcsResult {
    let content = body.resolve_content()?;

    state
        .command_tx
        .send(Action::StoreArtifact {
            kind: body.kind,
            label: body.label.clone(),
            content,
        })
        .await
        .map_err(ecs_dispatch_failed)?;

    // Poll for the new handle by label (last-written wins for duplicate labels).
    // §33.3: Return handle.id so the extension can show [ECS: HANDLE:uuid] reference.
    let handle_id = loop {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let focusa = state.focusa.read().await;
        if let Some(h) = focusa
            .reference_index
            .handles
            .iter()
            .find(|h| h.label == body.label)
        {
            break h.id;
        }
    };

    Ok(Json(json!({
        "id": handle_id,
        "status": "accepted",
        "trajectory": state.focusa.read().await.trajectory_ladder_context(),
    })))
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ListHandlesQuery {
    limit: Option<usize>,
    cursor: Option<usize>,
    #[serde(default = "default_true")]
    summary_only: bool,
    #[serde(default)]
    include_full_payload: bool,
    #[serde(default)]
    force_full_payload: bool,
}

fn default_true() -> bool {
    true
}

fn handles_default_limit() -> usize {
    env_limit("FOCUSA_ECS_HANDLES_DEFAULT_LIMIT", DEFAULT_HANDLES_LIMIT)
}

fn handles_full_limit() -> usize {
    env_limit("FOCUSA_ECS_HANDLES_FULL_LIMIT", MAX_HANDLES_LIMIT).max(handles_default_limit())
}

fn limit_handles(
    handles: &[HandleRef],
    cursor: usize,
    limit: usize,
) -> (Vec<HandleRef>, Option<String>) {
    let total = handles.len();
    let start = cursor.min(total);
    let end = (start + limit).min(total);
    let out = handles
        .iter()
        .rev()
        .skip(start)
        .take(end.saturating_sub(start))
        .cloned()
        .collect::<Vec<_>>();
    let next_cursor = (end < total).then(|| end.to_string());
    (out, next_cursor)
}

fn handle_summaries(handles: &[HandleRef]) -> Vec<serde_json::Value> {
    handles
        .iter()
        .map(|handle| {
            json!({
                "id": handle.id,
                "kind": handle.kind,
                "label": handle.label,
                "created_at": handle.created_at,
                "pinned": handle.pinned,
                "trajectory": handle.trajectory,
            })
        })
        .collect()
}

/// GET /v1/ecs/handles — list handles with optional summary/limit shaping.
async fn list_handles(
    Query(query): Query<ListHandlesQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    let total = focusa.reference_index.handles.len();
    let default_limit = handles_default_limit();
    let full_limit = handles_full_limit();
    let full_payload_blocked =
        full_payload_blocked_by_pressure(query.include_full_payload, query.force_full_payload);
    let effective_include_full_payload = query.include_full_payload && !full_payload_blocked;
    let effective_summary_only =
        (query.summary_only && !effective_include_full_payload) || full_payload_blocked;
    let pressure = pressure_status();
    let mut options = BoundedReadOptions {
        requested_limit: query.limit,
        include_full_payload: effective_include_full_payload,
        summary_only: effective_summary_only,
        cursor: query.cursor.map(|v| v.to_string()),
        next_cursor: None,
        default_limit,
        full_limit,
    };
    let resolved_limit = options.resolved_limit();
    let (handles, next_cursor) = limit_handles(
        &focusa.reference_index.handles,
        query.cursor.unwrap_or(0),
        resolved_limit,
    );
    options.next_cursor = next_cursor;
    let bounds = bounded_metadata(total, handles.len(), options);
    let payload = json!({
        "handles": if effective_summary_only { json!(handle_summaries(&handles)) } else { json!(handles) },
        "count": total,
        "bounds": bounds,
        "pressure": pressure,
        "degraded": full_payload_blocked,
        "full_payload_blocked_by_pressure": full_payload_blocked,
    });
    record_json_response_size("/v1/ecs/handles", &payload);
    Json(payload)
}

async fn resolve_handle(
    State(state): State<Arc<AppState>>,
    Path(handle_id): Path<uuid::Uuid>,
) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    match resolve_handle_with_disk_fallback(
        &state.config.data_dir,
        &focusa.reference_index.handles,
        handle_id,
    ) {
        Some(handle) => Json(json!({"handle": handle})),
        None => Json(ecs_handle_not_found(handle_id).1.0),
    }
}

/// GET /v1/ecs/content/:handle_id — get artifact content.
async fn get_content(
    State(state): State<Arc<AppState>>,
    Path(handle_id): Path<uuid::Uuid>,
) -> EcsResult {
    use base64::Engine;
    let focusa = state.focusa.read().await;

    // Resolve metadata from live state, with disk metadata fallback for legacy lossy state snapshots.
    let handle = resolve_handle_with_disk_fallback(
        &state.config.data_dir,
        &focusa.reference_index.handles,
        handle_id,
    )
    .ok_or_else(|| ecs_handle_not_found(handle_id))?;

    // Get content from canonical content-addressed object storage.
    let blob_path = handle_blob_path(&state.config.data_dir, &handle);
    let content = std::fs::read(&blob_path).map_err(|_| ecs_blob_not_found(handle_id))?;

    Ok(Json(json!({
        "handle_id": handle_id,
        "content_b64": base64::engine::general_purpose::STANDARD.encode(&content),
        "size": content.len(),
        "trajectory": handle.trajectory,
    })))
}

/// Expand ~ in path.
fn expand_data_path(path: &str) -> std::path::PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return std::path::PathBuf::from(home).join(rest);
    }
    std::path::PathBuf::from(path)
}

/// POST /v1/ecs/rehydrate/:handle_id — rehydrate with token limit.
#[derive(Deserialize)]
struct RehydrateQuery {
    #[serde(default = "default_max_tokens")]
    max_tokens: u32,
}

fn default_max_tokens() -> u32 {
    300
}

async fn rehydrate(
    State(state): State<Arc<AppState>>,
    Path(handle_id): Path<uuid::Uuid>,
    Query(query): Query<RehydrateQuery>,
) -> EcsResult {
    let focusa = state.focusa.read().await;

    // Resolve metadata from live state, with disk metadata fallback for legacy lossy state snapshots.
    let handle = resolve_handle_with_disk_fallback(
        &state.config.data_dir,
        &focusa.reference_index.handles,
        handle_id,
    )
    .ok_or_else(|| ecs_handle_not_found(handle_id))?;

    // Get content from canonical content-addressed object storage.
    let blob_path = handle_blob_path(&state.config.data_dir, &handle);
    let content = std::fs::read(&blob_path).map_err(|_| ecs_blob_not_found(handle_id))?;

    // Convert to string if possible.
    let text = String::from_utf8_lossy(&content);

    // Estimate chars per token (rough: 4 chars = 1 token).
    let max_chars = (query.max_tokens * 4) as usize;
    let truncated = if text.len() > max_chars {
        // UTF-8 safe truncation.
        let boundary = text
            .char_indices()
            .take_while(|(i, _)| *i < max_chars)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!("{}…", &text[..boundary])
    } else {
        text.to_string()
    };

    Ok(Json(json!({
        "handle_id": handle_id,
        "content": truncated,
        "truncated": text.len() > max_chars,
        "original_size": content.len(),
        "trajectory": handle.trajectory,
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/ecs/handles", get(list_handles))
        .route("/v1/ecs/store", post(store_artifact))
        .route("/v1/ecs/resolve/{handle_id}", get(resolve_handle))
        .route("/v1/ecs/content/{handle_id}", get(get_content))
        .route("/v1/ecs/rehydrate/{handle_id}", post(rehydrate))
}

#[cfg(test)]
mod tests {
    use super::{handle_summaries, limit_handles, resolve_handle_with_disk_fallback};
    use chrono::Utc;
    use focusa_core::types::{HandleKind, HandleRef};
    use uuid::Uuid;

    fn handle(label: &str, kind: HandleKind, pinned: bool) -> HandleRef {
        HandleRef {
            id: Uuid::now_v7(),
            kind,
            label: label.to_string(),
            size: 123,
            sha256: "deadbeef".to_string(),
            created_at: Utc::now(),
            session_id: None,
            pinned,
            trajectory: None,
        }
    }

    #[test]
    fn limit_handles_returns_cursor_window() {
        let items = vec![
            handle("old", HandleKind::Log, false),
            handle("mid", HandleKind::Diff, true),
            handle("new", HandleKind::Text, false),
        ];
        let (limited, next_cursor) = limit_handles(&items, 1, 2);
        assert_eq!(limited.len(), 2);
        assert_eq!(limited[0].label, "mid");
        assert_eq!(limited[1].label, "old");
        assert_eq!(next_cursor, None);
    }

    #[test]
    fn handle_summaries_strip_blob_metadata() {
        let items = vec![handle("artifact", HandleKind::Text, true)];
        let summary = handle_summaries(&items);
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0]["label"], "artifact");
        assert_eq!(summary[0]["kind"], "text");
        assert_eq!(summary[0]["pinned"], true);
        assert!(summary[0].get("sha256").is_none());
        assert!(summary[0].get("size").is_none());
    }

    #[test]
    fn resolve_handle_uses_disk_metadata_when_state_handle_is_lossy() {
        let temp = std::env::temp_dir().join(format!("focusa-ecs-test-{}", Uuid::now_v7()));
        let handles_dir = temp.join("ecs/handles");
        std::fs::create_dir_all(&handles_dir).expect("handles dir");

        let full = handle("artifact", HandleKind::Text, false);
        let mut lossy = full.clone();
        lossy.sha256 = "".to_string();
        lossy.size = 0;
        std::fs::write(
            handles_dir.join(format!("{}.json", full.id)),
            serde_json::to_string(&full).expect("serialize handle"),
        )
        .expect("write metadata");

        let resolved = resolve_handle_with_disk_fallback(
            temp.to_str().expect("utf8 temp path"),
            &[lossy],
            full.id,
        )
        .expect("disk fallback handle");

        assert_eq!(resolved.sha256, full.sha256);
        assert_eq!(resolved.size, full.size);
        let _ = std::fs::remove_dir_all(temp);
    }
}
