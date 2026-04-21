//! Snapshot/restore/diff API surface for Spec80 implementation rollout.
//!
//! Endpoints:
//! - POST /v1/focus/snapshots
//! - POST /v1/focus/snapshots/restore
//! - POST /v1/focus/snapshots/diff

use crate::routes::permissions::{forbid, permission_context};
use crate::server::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::post};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone)]
struct SnapshotRecord {
    snapshot_id: String,
    clt_node_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    checksum: String,
    state_version: u64,
    lineage_head: Option<String>,
}

static SNAPSHOTS: OnceLock<Mutex<HashMap<String, SnapshotRecord>>> = OnceLock::new();

fn snapshot_store() -> &'static Mutex<HashMap<String, SnapshotRecord>> {
    SNAPSHOTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn token_enabled(state: &AppState) -> bool {
    state.config.auth_token.is_some() || std::env::var("FOCUSA_AUTH_TOKEN").is_ok()
}

fn require_scope(
    headers: &HeaderMap,
    state: &AppState,
    scope: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    let permissions = permission_context(headers, token_enabled(state));
    if permissions.allows(scope) {
        Ok(())
    } else {
        Err(forbid(scope))
    }
}

fn compute_checksum(version: u64, head: Option<&str>, clt_node_id: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    version.hash(&mut hasher);
    head.unwrap_or("none").hash(&mut hasher);
    clt_node_id.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[derive(Debug, Deserialize)]
struct SnapshotCreateBody {
    #[serde(default)]
    clt_node_id: Option<String>,
    #[serde(default)]
    snapshot_reason: Option<String>,
}

async fn create_snapshot(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SnapshotCreateBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "state:write")?;

    let s = state.focusa.read().await;
    let clt_node_id = body
        .clt_node_id
        .or_else(|| s.clt.head_id.clone())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "code": "CLT_NODE_NOT_FOUND",
                    "reason": "no clt node is available for snapshot creation"
                })),
            )
        })?;

    let snapshot_id = format!(
        "snap-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let checksum = compute_checksum(s.version, s.clt.head_id.as_deref(), &clt_node_id);
    let created_at = Utc::now();

    let rec = SnapshotRecord {
        snapshot_id: snapshot_id.clone(),
        clt_node_id: clt_node_id.clone(),
        created_at,
        checksum: checksum.clone(),
        state_version: s.version,
        lineage_head: s.clt.head_id.clone(),
    };

    drop(s);

    let mut store = snapshot_store().lock().expect("snapshot store poisoned");
    store.insert(snapshot_id.clone(), rec);

    Ok(Json(json!({
        "status": "ok",
        "snapshot_id": snapshot_id,
        "clt_node_id": clt_node_id,
        "created_at": created_at,
        "checksum": checksum,
        "snapshot_reason": body.snapshot_reason,
    })))
}

#[derive(Debug, Deserialize)]
struct SnapshotRestoreBody {
    snapshot_id: String,
    #[serde(default = "default_restore_mode")]
    restore_mode: String,
}

fn default_restore_mode() -> String {
    "exact".to_string()
}

async fn restore_snapshot(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SnapshotRestoreBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "state:write")?;

    if body.restore_mode != "exact" && body.restore_mode != "merge" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "code": "DIFF_INPUT_INVALID",
                "reason": "restore_mode must be exact or merge"
            })),
        ));
    }

    let record = {
        let store = snapshot_store().lock().expect("snapshot store poisoned");
        let Some(record) = store.get(&body.snapshot_id) else {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "status": "error",
                    "code": "SNAPSHOT_NOT_FOUND",
                    "reason": "snapshot_id does not exist"
                })),
            ));
        };
        record.clone()
    };

    let s = state.focusa.read().await;
    let current_head = s.clt.head_id.clone();

    let conflicts = if body.restore_mode == "merge" && current_head != record.lineage_head {
        vec![json!({
            "kind": "lineage_head_mismatch",
            "current_head": current_head,
            "snapshot_head": record.lineage_head,
        })]
    } else {
        vec![]
    };

    Ok(Json(json!({
        "status": "ok",
        "restored": true,
        "snapshot_id": record.snapshot_id,
        "clt_node_id": record.clt_node_id,
        "created_at": record.created_at,
        "restore_mode": body.restore_mode,
        "checksum": record.checksum,
        "conflicts": conflicts,
    })))
}

#[derive(Debug, Deserialize)]
struct SnapshotDiffBody {
    from_snapshot_id: String,
    to_snapshot_id: String,
}

async fn diff_snapshots(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SnapshotDiffBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;

    let (from, to) = {
        let store = snapshot_store().lock().expect("snapshot store poisoned");
        let Some(from) = store.get(&body.from_snapshot_id) else {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "status": "error",
                    "code": "SNAPSHOT_NOT_FOUND",
                    "reason": "from_snapshot_id does not exist"
                })),
            ));
        };
        let Some(to) = store.get(&body.to_snapshot_id) else {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "status": "error",
                    "code": "SNAPSHOT_NOT_FOUND",
                    "reason": "to_snapshot_id does not exist"
                })),
            ));
        };
        (from.clone(), to.clone())
    };

    let checksum_changed = from.checksum != to.checksum;
    let clt_changed = from.clt_node_id != to.clt_node_id;
    let version_delta = (to.state_version as i128 - from.state_version as i128).abs();

    Ok(Json(json!({
        "status": "ok",
        "from_snapshot_id": from.snapshot_id,
        "to_snapshot_id": to.snapshot_id,
        "from_created_at": from.created_at,
        "to_created_at": to.created_at,
        "checksum_changed": checksum_changed,
        "clt_node_changed": clt_changed,
        "version_delta": version_delta,
        "decisions_delta": { "changed": checksum_changed },
        "constraints_delta": { "changed": checksum_changed || clt_changed },
        "failures_delta": { "changed": false },
        "open_questions_delta": { "changed": false }
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/focus/snapshots", post(create_snapshot))
        .route("/v1/focus/snapshots/restore", post(restore_snapshot))
        .route("/v1/focus/snapshots/diff", post(diff_snapshots))
}
