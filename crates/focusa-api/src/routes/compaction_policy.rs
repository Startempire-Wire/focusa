//! Scoped durable compaction-policy status and operator overrides.

use crate::{scope::ScopeContext, server::AppState};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};
use uuid::Uuid;

const SCHEMA: &str = "focusa.compaction_policy_status.v1";
const STORE_SCHEMA: &str = "focusa.compaction_policy_store.v1";
const ROUTES: &[&str] = &[
    "no_op",
    "curate_context",
    "checkpoint",
    "summarize",
    "native_compact",
    "rollover",
];
static STORE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScopeKey {
    project_root: String,
    continuity_id: String,
    fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OperatorOverride {
    receipt_id: String,
    route: String,
    reason: String,
    actor_ref: String,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PolicyStatus {
    schema: String,
    status: String,
    scope: ScopeKey,
    pressure_percent: Option<f64>,
    selected_route: Option<String>,
    reason: Option<String>,
    #[serde(default)]
    evidence_refs: Vec<String>,
    rollback_route: Option<String>,
    operator_override: Option<OperatorOverride>,
    updated_at: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Store {
    schema: String,
    #[serde(default)]
    policies: Vec<PolicyStatus>,
    #[serde(default)]
    override_receipts: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct ReportRequest {
    pressure_percent: Option<f64>,
    selected_route: String,
    reason: String,
    #[serde(default)]
    evidence_refs: Vec<String>,
    rollback_route: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OverrideRequest {
    action: String,
    route: Option<String>,
    reason: String,
    actor_ref: Option<String>,
}

fn scope_key(scope: &ScopeContext) -> Result<ScopeKey, String> {
    let workstream = scope.require_workstream_key()?;
    let project_root = workstream
        .root_scope
        .root_path
        .to_string_lossy()
        .to_string();
    let continuity_id = workstream.continuity_id;
    let fingerprint = format!(
        "sha256:{}",
        hex::encode(Sha256::digest(
            format!("{project_root}\0{continuity_id}").as_bytes()
        ))
    );
    Ok(ScopeKey {
        project_root,
        continuity_id,
        fingerprint,
    })
}

fn bounded(value: &str, max: usize) -> String {
    value.trim().chars().take(max).collect()
}

fn valid_route(value: &str) -> bool {
    ROUTES.contains(&value)
}

fn store_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("compaction-policy-v1.json")
}

fn load(path: &Path) -> Store {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_else(|| Store {
            schema: STORE_SCHEMA.into(),
            ..Store::default()
        })
}

fn save(path: &Path, store: &Store) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("json.tmp-{}", Uuid::now_v7()));
    fs::write(&temporary, serde_json::to_vec_pretty(store)?)?;
    fs::rename(temporary, path)
}

fn unavailable(scope: ScopeKey) -> PolicyStatus {
    PolicyStatus {
        schema: SCHEMA.into(),
        status: "unavailable".into(),
        scope,
        pressure_percent: None,
        selected_route: None,
        reason: Some("no_policy_report".into()),
        evidence_refs: Vec::new(),
        rollback_route: None,
        operator_override: None,
        updated_at: Utc::now().to_rfc3339(),
    }
}

fn mutate_store<T>(
    path: &Path,
    mutation: impl FnOnce(&mut Store) -> Result<T, String>,
) -> Result<T, String> {
    let _guard = STORE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| "policy_store_lock_poisoned".to_string())?;
    let mut store = load(path);
    let result = mutation(&mut store)?;
    save(path, &store).map_err(|error| format!("policy_store_write_failed:{error}"))?;
    Ok(result)
}

async fn status(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let key = scope_key(&scope).map_err(bad_scope)?;
    let path = store_path(Path::new(&state.config.data_dir));
    let fingerprint = key.fingerprint.clone();
    let policy = tokio::task::spawn_blocking(move || {
        load(&path)
            .policies
            .into_iter()
            .find(|item| item.scope.fingerprint == fingerprint)
    })
    .await
    .map_err(|_| internal("policy_reader_join_failed"))?
    .unwrap_or_else(|| unavailable(key));
    Ok(Json(serde_json::to_value(policy).unwrap_or_else(
        |_| json!({"schema":SCHEMA,"status":"degraded"}),
    )))
}

async fn report(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(req): Json<ReportRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let key = scope_key(&scope).map_err(bad_scope)?;
    if !valid_route(&req.selected_route)
        || req
            .rollback_route
            .as_deref()
            .is_some_and(|route| !valid_route(route))
    {
        return Err(bad_request("invalid_compaction_route"));
    }
    if req
        .pressure_percent
        .is_some_and(|value| !value.is_finite() || !(0.0..=100.0).contains(&value))
    {
        return Err(bad_request("invalid_pressure_percent"));
    }
    let path = store_path(Path::new(&state.config.data_dir));
    let policy = tokio::task::spawn_blocking(move || {
        mutate_store(&path, |store| {
            let existing_override = store
                .policies
                .iter()
                .find(|item| item.scope.fingerprint == key.fingerprint)
                .and_then(|item| item.operator_override.clone());
            let policy = PolicyStatus {
                schema: SCHEMA.into(),
                status: "observed".into(),
                scope: key,
                pressure_percent: req.pressure_percent,
                selected_route: Some(req.selected_route),
                reason: Some(bounded(&req.reason, 240)),
                evidence_refs: req
                    .evidence_refs
                    .into_iter()
                    .map(|value| bounded(&value, 512))
                    .filter(|value| !value.is_empty())
                    .take(32)
                    .collect(),
                rollback_route: req.rollback_route,
                operator_override: existing_override,
                updated_at: Utc::now().to_rfc3339(),
            };
            store
                .policies
                .retain(|item| item.scope.fingerprint != policy.scope.fingerprint);
            store.policies.push(policy.clone());
            Ok(policy)
        })
    })
    .await
    .map_err(|_| internal("policy_writer_join_failed"))?
    .map_err(|error| internal(&error))?;
    Ok(Json(serde_json::to_value(policy).unwrap()))
}

fn apply_override(store: &mut Store, key: ScopeKey, req: OverrideRequest) -> Value {
    let receipt_id = Uuid::now_v7().to_string();
    let now = Utc::now().to_rfc3339();
    let actor_ref = bounded(req.actor_ref.as_deref().unwrap_or("operator"), 160);
    let reason = bounded(&req.reason, 240);
    let index = store
        .policies
        .iter()
        .position(|item| item.scope.fingerprint == key.fingerprint);
    let mut policy = index
        .map(|i| store.policies.remove(i))
        .unwrap_or_else(|| unavailable(key.clone()));
    policy.operator_override = if req.action == "set" {
        Some(OperatorOverride {
            receipt_id: receipt_id.clone(),
            route: req.route.clone().unwrap(),
            reason: reason.clone(),
            actor_ref: actor_ref.clone(),
            created_at: now.clone(),
        })
    } else {
        None
    };
    policy.status = if policy.operator_override.is_some() {
        "overridden".into()
    } else if policy.selected_route.is_some() {
        "observed".into()
    } else {
        "unavailable".into()
    };
    policy.updated_at = now.clone();
    let receipt = json!({"schema":"focusa.compaction_policy_override_receipt.v1","receipt_id":receipt_id,"action":req.action,"route":req.route,"reason":reason,"actor_ref":actor_ref,"scope":key,"created_at":now,"reversible":true});
    store.policies.push(policy.clone());
    store.override_receipts.push(receipt.clone());
    if store.override_receipts.len() > 100 {
        store
            .override_receipts
            .drain(..store.override_receipts.len() - 100);
    }
    json!({"schema":SCHEMA,"status":policy.status,"policy":policy,"receipt":receipt})
}

async fn override_policy(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(req): Json<OverrideRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let key = scope_key(&scope).map_err(bad_scope)?;
    if !matches!(req.action.as_str(), "set" | "clear")
        || req.action == "set" && !req.route.as_deref().is_some_and(valid_route)
    {
        return Err(bad_request("invalid_override_action_or_route"));
    }
    if req.reason.trim().is_empty() {
        return Err(bad_request("override_reason_required"));
    }
    let path = store_path(Path::new(&state.config.data_dir));
    let result = tokio::task::spawn_blocking(move || {
        mutate_store(&path, |store| Ok(apply_override(store, key, req)))
    })
    .await
    .map_err(|_| internal("policy_override_writer_join_failed"))?
    .map_err(|error| internal(&error))?;
    Ok(Json(result))
}

fn bad_scope(error: String) -> (StatusCode, Json<Value>) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({"schema":SCHEMA,"status":"blocked","error":error})),
    )
}
fn bad_request(error: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"schema":SCHEMA,"status":"blocked","error":error})),
    )
}
fn internal(error: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"schema":SCHEMA,"status":"degraded","error":bounded(error,240)})),
    )
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/compaction/policy", get(status))
        .route("/v1/compaction/policy/report", post(report))
        .route("/v1/compaction/policy/override", post(override_policy))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_is_scoped_durable_and_reversible() {
        let dir = std::env::temp_dir().join(format!("focusa-policy-test-{}", Uuid::now_v7()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = store_path(&dir);
        let key = ScopeKey {
            project_root: "/p".into(),
            continuity_id: "c".into(),
            fingerprint: "f".into(),
        };
        let foreign = ScopeKey {
            project_root: "/other".into(),
            continuity_id: "other".into(),
            fingerprint: "foreign".into(),
        };
        mutate_store(&path, |store| {
            store.policies.push(unavailable(foreign));
            Ok(apply_override(
                store,
                key.clone(),
                OverrideRequest {
                    action: "set".into(),
                    route: Some("checkpoint".into()),
                    reason: "operator safety".into(),
                    actor_ref: Some("test-operator".into()),
                },
            ))
        })
        .unwrap();
        let durable = load(&path);
        assert_eq!(durable.policies.len(), 2);
        let scoped = durable
            .policies
            .iter()
            .find(|policy| policy.scope.fingerprint == "f")
            .unwrap();
        assert_eq!(
            scoped.operator_override.as_ref().unwrap().route,
            "checkpoint"
        );
        assert!(
            durable
                .policies
                .iter()
                .find(|policy| policy.scope.fingerprint == "foreign")
                .unwrap()
                .operator_override
                .is_none()
        );
        mutate_store(&path, |store| {
            Ok(apply_override(
                store,
                key,
                OverrideRequest {
                    action: "clear".into(),
                    route: None,
                    reason: "restore adaptive authority".into(),
                    actor_ref: Some("test-operator".into()),
                },
            ))
        })
        .unwrap();
        let cleared = load(&path);
        assert!(
            cleared
                .policies
                .iter()
                .find(|policy| policy.scope.fingerprint == "f")
                .unwrap()
                .operator_override
                .is_none()
        );
        assert_eq!(cleared.override_receipts.len(), 2);
        assert!(
            cleared
                .override_receipts
                .iter()
                .all(|receipt| receipt["reversible"] == true)
        );
        assert!(valid_route("checkpoint"));
        assert!(!valid_route("unsafe"));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
