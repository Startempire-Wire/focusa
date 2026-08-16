//! Snapshot/restore/diff API surface for Spec80 implementation rollout.
//!
//! Endpoints:
//! - POST /v1/focus/snapshots
//! - POST /v1/focus/snapshots/restore
//! - POST /v1/focus/snapshots/diff

use crate::routes::bounded::{
    BoundedReadOptions, bounded_metadata, budgeted_default_limit, budgeted_hard_limit,
    budgeted_requested_limit,
};
use crate::routes::clt::scoped_clt_state;
use crate::routes::permissions::{forbid, permission_context};
use crate::scope::ScopeContext;
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::scoped_state::WorkstreamKey;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SnapshotRecord {
    snapshot_id: String,
    clt_node_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    accessed_at: chrono::DateTime<chrono::Utc>,
    checksum: String,
    state_version: u64,
    lineage_head: Option<String>,
    storage_path: String,
    #[serde(default)]
    project_root: String,
    #[serde(default)]
    continuity_id: String,
    #[serde(default)]
    session_id: String,
}

fn require_snapshot_session(scope: &ScopeContext) -> Result<String, (StatusCode, Json<Value>)> {
    scope
        .session_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "global")
        .map(str::to_string)
        .ok_or_else(|| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "status": "blocked",
                    "code": "SESSION_SCOPE_REQUIRED",
                    "reason": "snapshot lineage requires an exact non-global native session scope"
                })),
            )
        })
}

fn snapshot_scope_matches(
    record: &SnapshotRecord,
    scope: &WorkstreamKey,
    session_id: &str,
) -> bool {
    record.project_root == scope.root_scope.root_path.display().to_string()
        && record.continuity_id == scope.continuity_id
        && record.session_id == session_id
}

fn snapshot_quarantined_response(record: &SnapshotRecord) -> (StatusCode, Json<Value>) {
    (
        StatusCode::CONFLICT,
        Json(json!({
            "status": "quarantined",
            "code": "SNAPSHOT_SCOPE_QUARANTINED",
            "snapshot_id": record.snapshot_id,
            "reason": "snapshot lacks exact project, continuity, and native-session provenance or belongs to another scope",
            "restored": false,
            "recovery": "create a fresh snapshot from verified current scope; contaminated history remains immutable"
        })),
    )
}

#[derive(Debug, Clone, Copy)]
struct SnapshotStoreConfig {
    max_snapshots: usize,
    ttl_minutes: i64,
}

fn snapshot_store_config() -> SnapshotStoreConfig {
    let max_snapshots = std::env::var("FOCUSA_SNAPSHOT_MAX")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(512)
        .max(1);

    let ttl_minutes = std::env::var("FOCUSA_SNAPSHOT_TTL_MINUTES")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(24 * 60)
        .max(1);

    SnapshotStoreConfig {
        max_snapshots,
        ttl_minutes,
    }
}

fn prune_snapshot_store(
    store: &mut HashMap<String, SnapshotRecord>,
    now: chrono::DateTime<chrono::Utc>,
    cfg: SnapshotStoreConfig,
) {
    let ttl_cutoff = now - chrono::Duration::minutes(cfg.ttl_minutes);
    store.retain(|_, rec| rec.created_at >= ttl_cutoff);

    if store.len() <= cfg.max_snapshots {
        return;
    }

    let mut by_lru = store
        .iter()
        .map(|(k, v)| (k.clone(), v.accessed_at))
        .collect::<Vec<_>>();
    by_lru.sort_by_key(|(_, ts)| *ts);

    let to_remove = store.len().saturating_sub(cfg.max_snapshots);
    for (key, _) in by_lru.into_iter().take(to_remove) {
        store.remove(&key);
    }
}

fn scope_required_response(reason: String) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "status": "failed", "failure_class": "operation_failed", "retry_posture": "safe_retry", "safe_recovery": "retry the operation or inspect the recovery graph",
            "code": "SCOPE_REQUIRED",
            "reason": reason,
        })),
    )
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

fn snapshot_dir(state: &AppState, scope: &focusa_core::scoped_state::WorkstreamKey) -> PathBuf {
    Path::new(&state.config.data_dir)
        .join("runtime")
        .join("snapshots")
        .join(scope.storage_key())
}

fn snapshot_record_path(
    state: &AppState,
    scope: &focusa_core::scoped_state::WorkstreamKey,
    snapshot_id: &str,
) -> PathBuf {
    snapshot_dir(state, scope).join(format!("{snapshot_id}.json"))
}

fn snapshot_index_path(
    state: &AppState,
    scope: &focusa_core::scoped_state::WorkstreamKey,
) -> PathBuf {
    snapshot_dir(state, scope).join("snapshot-index.json")
}

fn persist_snapshot_index(
    state: &AppState,
    scope: &focusa_core::scoped_state::WorkstreamKey,
    records: impl IntoIterator<Item = SnapshotRecord>,
) {
    let mut items = records.into_iter().collect::<Vec<_>>();
    items.sort_by_key(|rec| std::cmp::Reverse(rec.created_at));
    items.truncate(snapshot_store_config().max_snapshots.min(2048));
    let path = snapshot_index_path(state, scope);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(bytes) = serde_json::to_vec_pretty(&items) {
        let _ = fs::write(path, bytes);
    }
}

fn load_snapshot_index(
    state: &AppState,
    scope: &focusa_core::scoped_state::WorkstreamKey,
) -> Vec<SnapshotRecord> {
    fs::read(snapshot_index_path(state, scope))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Vec<SnapshotRecord>>(&bytes).ok())
        .unwrap_or_default()
}

fn persist_snapshot_record(
    state: &AppState,
    scope: &focusa_core::scoped_state::WorkstreamKey,
    rec: &SnapshotRecord,
) {
    let dir = snapshot_dir(state, scope);
    let _ = fs::create_dir_all(&dir);
    let path = snapshot_record_path(state, scope, &rec.snapshot_id);
    let payload = json!({
        "snapshot_id": rec.snapshot_id,
        "clt_node_id": rec.clt_node_id,
        "created_at": rec.created_at,
        "accessed_at": rec.accessed_at,
        "checksum": rec.checksum,
        "state_version": rec.state_version,
        "lineage_head": rec.lineage_head,
        "storage_path": rec.storage_path,
        "project_root": rec.project_root,
        "continuity_id": rec.continuity_id,
        "session_id": rec.session_id,
    });
    if let Ok(bytes) = serde_json::to_vec_pretty(&payload) {
        let _ = fs::write(path, bytes);
    }
}

fn load_snapshot_record(
    state: &AppState,
    scope: &focusa_core::scoped_state::WorkstreamKey,
    snapshot_id: &str,
) -> Option<SnapshotRecord> {
    let path = snapshot_record_path(state, scope, snapshot_id);
    let bytes = fs::read(path).ok()?;
    let v: Value = serde_json::from_slice(&bytes).ok()?;
    Some(SnapshotRecord {
        snapshot_id: v.get("snapshot_id")?.as_str()?.to_string(),
        clt_node_id: v.get("clt_node_id")?.as_str()?.to_string(),
        created_at: chrono::DateTime::parse_from_rfc3339(v.get("created_at")?.as_str()?)
            .ok()?
            .with_timezone(&Utc),
        accessed_at: chrono::DateTime::parse_from_rfc3339(v.get("accessed_at")?.as_str()?)
            .ok()?
            .with_timezone(&Utc),
        checksum: v.get("checksum")?.as_str()?.to_string(),
        state_version: v.get("state_version")?.as_u64()?,
        lineage_head: v
            .get("lineage_head")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        storage_path: v
            .get("storage_path")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string(),
        project_root: v
            .get("project_root")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string(),
        continuity_id: v
            .get("continuity_id")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string(),
        session_id: v
            .get("session_id")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string(),
    })
}

fn snapshot_authority_posture(rec: &SnapshotRecord) -> Value {
    let exact_scope = !rec.project_root.is_empty()
        && !rec.continuity_id.is_empty()
        && !rec.session_id.is_empty()
        && rec.session_id != "global";
    json!({
        "migration_class": if exact_scope { "typed_snapshot_lineage" } else { "old_snapshots_and_clt_nodes" },
        "read_behavior": if exact_scope { "exact_scope_only" } else { "quarantined_history_only" },
        "authority_status": "lineage_not_current_action_authority",
        "migration_warnings": if exact_scope { Vec::<String>::new() } else { vec!["clt_snapshot_authority_unscoped".to_string()] },
        "scope": {
            "project_root": if exact_scope { Some(rec.project_root.as_str()) } else { None },
            "continuity_id": if exact_scope { Some(rec.continuity_id.as_str()) } else { None },
            "session_id": if exact_scope { Some(rec.session_id.as_str()) } else { None },
            "scope_status": if exact_scope { "exact" } else { "quarantined" },
            "scope_source": if exact_scope { "typed_snapshot_record" } else { "legacy_snapshot_or_lineage_record" },
        },
        "promotion_path": ["focusa_workpoint_resume", "focusa_workpoint_checkpoint"],
        "snapshot_id": rec.snapshot_id,
    })
}

// Snapshot records are loaded from the persistent index file.
// Directory scan removed per Spec94 E1: index must be pre-populated.

#[derive(Debug, Deserialize)]
struct SnapshotCreateBody {
    #[serde(default)]
    clt_node_id: Option<String>,
    #[serde(default)]
    snapshot_reason: Option<String>,
}

async fn create_snapshot(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
    Json(body): Json<SnapshotCreateBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "state:write")?;
    let session_id = require_snapshot_session(&scope_context)?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;

    let s = state.focusa.read().await;
    let scoped_clt = scoped_clt_state(&s.clt, &scope_context);
    let clt_node_id = body
        .clt_node_id
        .or_else(|| scoped_clt.head_id.clone())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "failed", "failure_class": "operation_failed", "retry_posture": "safe_retry", "safe_recovery": "retry the operation or inspect the recovery graph",
                    "code": "CLT_NODE_NOT_FOUND",
                    "reason": "no clt node is available for snapshot creation"
                })),
            )
        })?;

    if !scoped_clt
        .nodes
        .iter()
        .any(|node| node.node_id == clt_node_id)
    {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "blocked",
                "code": "CLT_NODE_SCOPE_MISMATCH",
                "reason": "clt_node_id is not present in the exact project, continuity, and native-session lineage"
            })),
        ));
    }

    let snapshot_id = format!(
        "snap-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let checksum = compute_checksum(s.version, scoped_clt.head_id.as_deref(), &clt_node_id);
    let created_at = Utc::now();

    let storage_path = snapshot_record_path(&state, &scope, &snapshot_id)
        .display()
        .to_string();
    let rec = SnapshotRecord {
        snapshot_id: snapshot_id.clone(),
        clt_node_id: clt_node_id.clone(),
        created_at,
        accessed_at: created_at,
        checksum: checksum.clone(),
        state_version: s.version,
        lineage_head: scoped_clt.head_id.clone(),
        storage_path: storage_path.clone(),
        project_root: scope.root_scope.root_path.display().to_string(),
        continuity_id: scope.continuity_id.clone(),
        session_id: session_id.clone(),
    };

    drop(s);

    persist_snapshot_record(&state, &scope, &rec);

    let mut stores = state.snapshots_by_scope.write().await;
    let store = stores.entry(scope.clone()).or_insert_with(HashMap::new);
    store.insert(snapshot_id.clone(), rec.clone());
    prune_snapshot_store(store, Utc::now(), snapshot_store_config());
    persist_snapshot_index(&state, &scope, store.values().cloned());

    Ok(Json(json!({
        "status": "ok",
        "snapshot_id": snapshot_id,
        "clt_node_id": clt_node_id,
        "created_at": created_at,
        "checksum": checksum,
        "snapshot_reason": body.snapshot_reason,
        "storage_path": storage_path,
        "authority_posture": snapshot_authority_posture(&rec),
        "scope_provenance": {
            "project_root": rec.project_root,
            "continuity_id": rec.continuity_id,
            "session_id": rec.session_id,
            "global_fallback": false
        },
        "migration_warnings": [],
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
    scope_context: ScopeContext,
    headers: HeaderMap,
    Json(body): Json<SnapshotRestoreBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "state:write")?;
    let session_id = require_snapshot_session(&scope_context)?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;

    if body.restore_mode != "exact" && body.restore_mode != "merge" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "failed", "failure_class": "operation_failed", "retry_posture": "safe_retry", "safe_recovery": "retry the operation or inspect the recovery graph",
                "code": "DIFF_INPUT_INVALID",
                "reason": "restore_mode must be exact or merge"
            })),
        ));
    }

    let record = {
        let mut stores = state.snapshots_by_scope.write().await;
        let store = stores.entry(scope.clone()).or_insert_with(HashMap::new);
        if !store.contains_key(&body.snapshot_id)
            && let Some(mut disk_record) = load_snapshot_record(&state, &scope, &body.snapshot_id)
        {
            disk_record.accessed_at = Utc::now();
            store.insert(body.snapshot_id.clone(), disk_record);
        }

        let Some(record) = store.get_mut(&body.snapshot_id) else {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "status": "failed", "failure_class": "operation_failed", "retry_posture": "safe_retry", "safe_recovery": "retry the operation or inspect the recovery graph",
                    "code": "SNAPSHOT_NOT_FOUND",
                    "reason": "snapshot_id does not exist"
                })),
            ));
        };
        record.accessed_at = Utc::now();
        persist_snapshot_record(&state, &scope, record);
        let cloned = record.clone();
        persist_snapshot_index(&state, &scope, store.values().cloned());
        cloned
    };

    if !snapshot_scope_matches(&record, &scope, &session_id) {
        return Err(snapshot_quarantined_response(&record));
    }

    let s = state.focusa.read().await;
    let current_head = scoped_clt_state(&s.clt, &scope_context).head_id;

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
        "authority_posture": snapshot_authority_posture(&record),
        "scope_provenance": {
            "project_root": record.project_root,
            "continuity_id": record.continuity_id,
            "session_id": record.session_id,
            "global_fallback": false
        },
        "migration_warnings": [],
    })))
}

#[derive(Debug, Deserialize)]
struct RecentSnapshotsQuery {
    limit: Option<usize>,
    cursor: Option<usize>,
}

async fn recent_snapshots(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
    Query(query): Query<RecentSnapshotsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;
    let session_id = require_snapshot_session(&scope_context)?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;

    let mut by_id: HashMap<String, SnapshotRecord> = HashMap::new();
    {
        let stores = state.snapshots_by_scope.read().await;
        if let Some(store) = stores.get(&scope) {
            for rec in store.values() {
                by_id.insert(rec.snapshot_id.clone(), rec.clone());
            }
        }
    }
    for rec in load_snapshot_index(&state, &scope) {
        by_id.entry(rec.snapshot_id.clone()).or_insert(rec);
    }

    let quarantined_count = by_id
        .values()
        .filter(|record| !snapshot_scope_matches(record, &scope, &session_id))
        .count();
    by_id.retain(|_, record| snapshot_scope_matches(record, &scope, &session_id));

    let default_limit = budgeted_default_limit("FOCUSA_SNAPSHOT_RECENT_DEFAULT_LIMIT", 5);
    let hard_limit = budgeted_hard_limit("FOCUSA_SNAPSHOT_RECENT_HARD_LIMIT", 20, default_limit);
    let limit = budgeted_requested_limit(query.limit, default_limit, hard_limit);
    let total = by_id.len();
    let cursor = query.cursor.unwrap_or(0).min(total);
    let mut items = by_id.into_values().collect::<Vec<_>>();
    items.sort_by_key(|r| std::cmp::Reverse(r.created_at));
    let window = items
        .into_iter()
        .skip(cursor)
        .take(limit)
        .collect::<Vec<_>>();
    let next_cursor = (cursor + window.len() < total).then(|| (cursor + window.len()).to_string());

    Ok(Json(json!({
        "status": "ok",
        "source": "snapshot_hot_index",
        "total": total,
        "returned": window.len(),
        "limit": limit,
        "cursor": cursor,
        "next_cursor": next_cursor,
        "truncated": next_cursor.is_some() || cursor > 0,
        "metadata": bounded_metadata(total, window.len(), BoundedReadOptions {
            requested_limit: query.limit,
            include_full_payload: false,
            summary_only: true,
            cursor: query.cursor.map(|value| value.to_string()),
            next_cursor: next_cursor.clone(),
            default_limit,
            full_limit: hard_limit,
        }),
        "cold_full_payload_opt_in": false,
        "quarantined_count": quarantined_count,
        "scope_provenance": {
            "project_root": scope.root_scope.root_path,
            "continuity_id": scope.continuity_id,
            "session_id": session_id,
            "global_fallback": false
        },
        "snapshots": window.into_iter().map(|rec| json!({
            "snapshot_id": rec.snapshot_id,
            "clt_node_id": rec.clt_node_id,
            "created_at": rec.created_at,
            "checksum": rec.checksum,
            "state_version": rec.state_version,
            "lineage_head": rec.lineage_head,
            "authority_posture": snapshot_authority_posture(&rec),
            "scope_provenance": {
                "project_root": rec.project_root,
                "continuity_id": rec.continuity_id,
                "session_id": rec.session_id,
                "global_fallback": false
            },
            "migration_warnings": [],
        })).collect::<Vec<_>>()
    })))
}

#[derive(Debug, Deserialize)]
struct SnapshotDiffBody {
    from_snapshot_id: String,
    to_snapshot_id: String,
}

async fn diff_snapshots(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
    Json(body): Json<SnapshotDiffBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;
    let session_id = require_snapshot_session(&scope_context)?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;

    let (from, to) = {
        let mut stores = state.snapshots_by_scope.write().await;
        let store = stores.entry(scope.clone()).or_insert_with(HashMap::new);

        if !store.contains_key(&body.from_snapshot_id)
            && let Some(mut rec) = load_snapshot_record(&state, &scope, &body.from_snapshot_id)
        {
            rec.accessed_at = Utc::now();
            store.insert(body.from_snapshot_id.clone(), rec);
        }
        if !store.contains_key(&body.to_snapshot_id)
            && let Some(mut rec) = load_snapshot_record(&state, &scope, &body.to_snapshot_id)
        {
            rec.accessed_at = Utc::now();
            store.insert(body.to_snapshot_id.clone(), rec);
        }

        let Some(from) = store.get_mut(&body.from_snapshot_id) else {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "status": "failed", "failure_class": "operation_failed", "retry_posture": "safe_retry", "safe_recovery": "retry the operation or inspect the recovery graph",
                    "code": "SNAPSHOT_NOT_FOUND",
                    "reason": "from_snapshot_id does not exist"
                })),
            ));
        };
        from.accessed_at = Utc::now();
        persist_snapshot_record(&state, &scope, from);
        let from_cloned = from.clone();

        let Some(to) = store.get_mut(&body.to_snapshot_id) else {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "status": "failed", "failure_class": "operation_failed", "retry_posture": "safe_retry", "safe_recovery": "retry the operation or inspect the recovery graph",
                    "code": "SNAPSHOT_NOT_FOUND",
                    "reason": "to_snapshot_id does not exist"
                })),
            ));
        };
        to.accessed_at = Utc::now();
        persist_snapshot_record(&state, &scope, to);
        let to_cloned = to.clone();

        prune_snapshot_store(store, Utc::now(), snapshot_store_config());
        persist_snapshot_index(&state, &scope, store.values().cloned());
        (from_cloned, to_cloned)
    };

    if !snapshot_scope_matches(&from, &scope, &session_id) {
        return Err(snapshot_quarantined_response(&from));
    }
    if !snapshot_scope_matches(&to, &scope, &session_id) {
        return Err(snapshot_quarantined_response(&to));
    }

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
        "open_questions_delta": { "changed": false },
        "authority_posture": {
            "from": snapshot_authority_posture(&from),
            "to": snapshot_authority_posture(&to),
        },
        "scope_provenance": {
            "project_root": from.project_root,
            "continuity_id": from.continuity_id,
            "session_id": from.session_id,
            "global_fallback": false
        },
        "migration_warnings": []
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/focus/snapshots", post(create_snapshot))
        .route("/v1/focus/snapshots/recent", get(recent_snapshots))
        .route("/v1/focus/snapshots/restore", post(restore_snapshot))
        .route("/v1/focus/snapshots/diff", post(diff_snapshots))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn rec(
        id: &str,
        created_at: chrono::DateTime<chrono::Utc>,
        accessed_at: chrono::DateTime<chrono::Utc>,
    ) -> SnapshotRecord {
        SnapshotRecord {
            snapshot_id: id.to_string(),
            clt_node_id: format!("clt-{id}"),
            created_at,
            accessed_at,
            checksum: format!("chk-{id}"),
            state_version: 1,
            lineage_head: Some("head".to_string()),
            storage_path: format!("/tmp/{id}.json"),
            project_root: "/tmp/focusa-snapshot-tests".to_string(),
            continuity_id: "continuity-a".to_string(),
            session_id: "session-a".to_string(),
        }
    }

    fn workstream() -> WorkstreamKey {
        WorkstreamKey::new(
            focusa_core::scoped_state::ScopeRef::project(
                "project-snapshot-tests",
                "/tmp/focusa-snapshot-tests",
                "focusa-snapshot-tests",
                "fingerprint-snapshot-tests",
            )
            .unwrap(),
            "continuity-a",
        )
        .unwrap()
    }

    #[test]
    fn snapshot_scope_requires_exact_non_global_session() {
        let missing = ScopeContext {
            project_root: Some("/tmp/focusa-snapshot-tests".to_string()),
            continuity_id: Some("continuity-a".to_string()),
            ..ScopeContext::default()
        };
        assert_eq!(
            require_snapshot_session(&missing).unwrap_err().0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        let global = ScopeContext {
            session_id: Some("global".to_string()),
            ..missing.clone()
        };
        assert_eq!(
            require_snapshot_session(&global).unwrap_err().0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn legacy_or_foreign_snapshot_is_quarantined() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 4, 21, 20, 0, 0).unwrap();
        let current = rec("current", now, now);
        assert!(snapshot_scope_matches(&current, &workstream(), "session-a"));
        assert!(!snapshot_scope_matches(
            &current,
            &workstream(),
            "session-b"
        ));

        let legacy_json = serde_json::json!({
            "snapshot_id": "legacy",
            "clt_node_id": "clt-legacy",
            "created_at": now,
            "accessed_at": now,
            "checksum": "chk-legacy",
            "state_version": 1,
            "lineage_head": "head",
            "storage_path": "/tmp/legacy.json"
        });
        let legacy: SnapshotRecord = serde_json::from_value(legacy_json).unwrap();
        assert!(!snapshot_scope_matches(&legacy, &workstream(), "session-a"));
        assert_eq!(
            snapshot_quarantined_response(&legacy).0,
            StatusCode::CONFLICT
        );
    }

    #[test]
    fn prune_snapshot_store_removes_expired_entries() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 4, 21, 20, 0, 0).unwrap();
        let mut store = HashMap::new();
        store.insert(
            "old".to_string(),
            rec(
                "old",
                now - Duration::minutes(400),
                now - Duration::minutes(400),
            ),
        );
        store.insert(
            "new".to_string(),
            rec(
                "new",
                now - Duration::minutes(20),
                now - Duration::minutes(10),
            ),
        );

        prune_snapshot_store(
            &mut store,
            now,
            SnapshotStoreConfig {
                max_snapshots: 10,
                ttl_minutes: 60,
            },
        );

        assert!(store.contains_key("new"));
        assert!(!store.contains_key("old"));
    }

    #[test]
    fn prune_snapshot_store_uses_lru_when_over_capacity() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 4, 21, 20, 0, 0).unwrap();
        let mut store = HashMap::new();
        store.insert(
            "a".to_string(),
            rec(
                "a",
                now - Duration::minutes(20),
                now - Duration::minutes(20),
            ),
        );
        store.insert(
            "b".to_string(),
            rec("b", now - Duration::minutes(20), now - Duration::minutes(5)),
        );
        store.insert(
            "c".to_string(),
            rec("c", now - Duration::minutes(20), now - Duration::minutes(1)),
        );

        prune_snapshot_store(
            &mut store,
            now,
            SnapshotStoreConfig {
                max_snapshots: 2,
                ttl_minutes: 60,
            },
        );

        assert_eq!(store.len(), 2);
        assert!(!store.contains_key("a"));
        assert!(store.contains_key("b"));
        assert!(store.contains_key("c"));
    }
}
