//! Mac menubar OAuth-like device pairing (focusa-ui0y).
//!
//! Pairing model (operator-facing, "mac-like + dumb simple"):
//! 1. The Mac app calls `POST /v1/device/pair/start` with a device
//!    name + platform + scopes. The daemon returns an 8-char code
//!    + a `device_id` + the daemon's base URL.
//! 2. The operator goes to their VPS (ssh, web UI, whatever) and runs
//!    `focusa device pair-complete <code> --host <host> --scopes ...`.
//!    The daemon matches the code, generates a long-lived token,
//!    returns it as the pair-completion response, and stores the
//!    DeviceRecord in the append-only JSONL ledger.
//! 3. The Mac app polls `GET /v1/device/pair/status?code=...` until
//!    the daemon reports `Completed`. It then stores the token in
//!    the macOS Keychain (Tauri) and uses it for subsequent calls.
//! 4. The operator can list paired devices with
//!    `GET /v1/device/pair/list?host=...` and revoke with
//!    `POST /v1/device/pair/revoke`.
//!
//! Codes expire in 5 minutes. Tokens expire in 30 days. The ledger
//! is append-only; revocation is a new entry with `revoked=true`.

use crate::server::AppState;
use axum::{Json, extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use focusa_core::types::{DevicePairCode, DevicePairCompletion, DeviceRecord, DeviceToken};
use uuid::Uuid;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const CODE_TTL_SECS: i64 = 300;        // 5 min
const TOKEN_TTL_SECS: i64 = 60 * 60 * 24 * 30;  // 30 days

#[derive(Default)]
struct PairingState {
    pending: HashMap<String, DevicePairCode>, // code -> pair
    tokens: HashMap<String, DeviceToken>,      // token -> token
}

pub type SharedPairingState = Arc<RwLock<PairingState>>;

pub fn router() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/v1/device/pair/start", axum::routing::post(pair_start))
        .route(
            "/v1/device/pair/complete",
            axum::routing::post(pair_complete),
        )
        .route(
            "/v1/device/pair/status",
            axum::routing::get(pair_status),
        )
        .route(
            "/v1/device/pair/list",
            axum::routing::get(pair_list),
        )
        .route(
            "/v1/device/pair/revoke",
            axum::routing::post(pair_revoke),
        )
}

fn rejection(status: StatusCode, body: Value) -> (StatusCode, Json<Value>) {
    (status, Json(body))
}

fn generate_code() -> String {
    // Format: FOCUS-XXXX-XXXX where XXXX = first 4 + last 4 hex chars
    // of a UUID v7. Avoids confusing chars (0/O, 1/I/L) by using
    // uppercase hex which is unambiguous in monospace fonts.
    let id = Uuid::now_v7().simple().to_string(); // 32 hex chars
    let upper: String = id.chars().take(8).collect::<String>().to_uppercase();
    let lower_tail: String = id.chars().skip(24).take(4).collect();
    format!("FOCUS-{}-{}", upper, lower_tail.to_uppercase())
}

fn generate_token() -> String {
    // 32-char hex token (UUID v7 stripped of dashes).
    Uuid::now_v7().simple().to_string()
}

fn shared_state() -> SharedPairingState {
    use std::sync::OnceLock;
    static STATE: OnceLock<SharedPairingState> = OnceLock::new();
    STATE
        .get_or_init(|| Arc::new(RwLock::new(PairingState::default())))
        .clone()
}

fn is_unsafe_agent_runtime_path_inline(path: &str) -> bool {
    const BLOCKED: &[&str] = &[
        "/root/pi-mono", "/root/.pi", "/root/.claude", "/root/.opencode", "/root/.letta",
    ];
    BLOCKED.iter().any(|p| path == *p || path.starts_with(&format!("{}/", p)))
}

#[derive(Debug, Deserialize)]
pub struct PairStartRequest {
    pub device_name: Option<String>,
    pub platform: Option<String>,
    pub daemon_base_url: Option<String>,
    pub scopes: Option<Vec<String>>,
}

async fn pair_start(
    Json(body): Json<PairStartRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let device_name = body
        .device_name
        .unwrap_or_else(|| "operator-device".to_string());
    let platform = body.platform.unwrap_or_else(|| "macos".to_string());
    let daemon_base_url = body
        .daemon_base_url
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string());
    let scopes = body.scopes.unwrap_or_else(|| vec!["read".to_string(), "write".to_string()]);

    if device_name.trim().is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "device_name_missing",
                "field": "device_name",
            }),
        ));
    }

    let now = Utc::now();
    let expires = now + Duration::seconds(CODE_TTL_SECS);
    let code = generate_code();
    let device_id = uuid::Uuid::now_v7().to_string();

    let pair = DevicePairCode {
        code: code.clone(),
        device_id: device_id.clone(),
        device_name: device_name.clone(),
        platform: platform.clone(),
        daemon_base_url: daemon_base_url.clone(),
        scopes: scopes.clone(),
        created_at: now,
        expires_at: expires,
        status: "Pending".to_string(),
    };

    let pairing_state = shared_state();
    {
        let mut s = pairing_state.write().await;
        // If the same code is already pending, replace it (idempotent for
        // re-tries from a flaky network).
        s.pending.insert(code.clone(), pair.clone());
    }

    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "device_id": device_id,
        "code": code,
        "device_name": device_name,
        "platform": platform,
        "scopes": scopes,
        "daemon_base_url": daemon_base_url,
        "expires_at": expires,
        "expires_in_secs": CODE_TTL_SECS,
        "operator_handoff": {
            "command": format!("focusa device pair-complete {} --host <host> --operator-id <id>", code),
            "on_your_vps_run": format!("focusa device pair-complete {} --host <host>", code),
            "scopes": scopes,
        },
        "next_tools": [
            "focusa_device_pair_status",
            "focusa_device_pair_list"
        ],
        "rehydrate_id": code,
    })))
}

#[derive(Debug, Deserialize)]
pub struct PairCompleteRequest {
    pub code: String,
    pub host: Option<String>,
    pub operator_id: Option<String>,
    pub completed_by: Option<String>,
}

async fn pair_complete(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PairCompleteRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let code = body.code.trim().to_uppercase();
    if code.is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "code_missing",
                "field": "code",
            }),
        ));
    }

    let now = Utc::now();
    let host = body.host.unwrap_or_else(|| "operator-vps".to_string());
    if is_unsafe_agent_runtime_path_inline(&host) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "scope_mismatch",
                "field": "host",
                "rejected_value": host,
            }),
        ));
    }

    let pairing_state = shared_state();

    // Look up the pending pair; reject if missing, expired, or already completed.
    let pair = {
        let mut s = pairing_state.write().await;
        let p = s.pending.get(&code).cloned();
        if let Some(p) = p {
            if p.expires_at < now {
                s.pending.remove(&code);
                return Err(rejection(
                    StatusCode::GONE,
                    json!({
                        "status": "expired",
                        "failure_class": "pair_code_expired",
                        "code": code,
                        "expired_at": p.expires_at,
                    }),
                ));
            }
            if p.status == "Completed" {
                return Err(rejection(
                    StatusCode::CONFLICT,
                    json!({
                        "status": "already_completed",
                        "failure_class": "pair_code_already_used",
                        "code": code,
                        "device_id": p.device_id,
                    }),
                ));
            }
            // Generate the token and mark the pair as completed.
            let token = generate_token();
            let token_expires = now + Duration::seconds(TOKEN_TTL_SECS);
            let device_token = DeviceToken {
                device_id: p.device_id.clone(),
                token: token.clone(),
                scopes: p.scopes.clone(),
                issued_at: now,
                expires_at: token_expires,
                last_used_at: None,
                issued_to: host.clone(),
            };
            s.tokens.insert(token.clone(), device_token);
            // Mark the pending pair as completed.
            let mut updated = p.clone();
            updated.status = "Completed".to_string();
            s.pending.insert(code.clone(), updated.clone());
            updated
        } else {
            return Err(rejection(
                StatusCode::NOT_FOUND,
                json!({
                    "status": "not_found",
                    "failure_class": "pair_code_not_found",
                    "code": code,
                }),
            ));
        }
    };

    // The token was already inserted into the in-memory pairing_state.tokens
    // above; look it up by device_id for the response.
    let token = pairing_state
        .read()
        .await
        .tokens
        .iter()
        .find(|(_, t)| t.device_id == pair.device_id)
        .map(|(_, t)| t.token.clone())
        .unwrap_or_default();

    let record = DeviceRecord {
        device_id: pair.device_id.clone(),
        name: pair.device_name.clone(),
        platform: pair.platform.clone(),
        host: host.clone(),
        scopes: pair.scopes.clone(),
        paired_at: now,
        last_seen_at: now,
        revoked: false,
        revoked_at: None,
    };
    if let Err(e) = state.persistence.append_device_record(&record) {
        return Err(rejection(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({
                "status": "blocked",
                "failure_class": "storage_unwritable",
                "message": format!("append failed: {}", e),
            }),
        ));
    }

    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "code": code,
        "device_id": pair.device_id,
        "device_name": pair.device_name,
        "platform": pair.platform,
        "host": host,
        "scopes": pair.scopes,
        "token": token,
        "token_expires_at": now + Duration::seconds(TOKEN_TTL_SECS),
        "token_ttl_secs": TOKEN_TTL_SECS,
        "operator_handoff": {
            "command": format!("# On your Mac app, store the token in Keychain and reconnect using the daemon URL"),
            "next_step": "mac app should poll /v1/device/pair/status?code=... to retrieve the token"
        },
        "next_tools": ["focusa_device_pair_status", "focusa_device_pair_list"],
        "rehydrate_id": pair.device_id,
        "_record": record,
    })))
}

#[derive(Debug, Deserialize)]
pub struct PairStatusRequest {
    pub code: Option<String>,
    pub device_id: Option<String>,
}

async fn pair_status(
    axum::extract::Query(query): axum::extract::Query<PairStatusRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pairing_state = shared_state();
    let s = pairing_state.read().await;
    if let Some(code) = query.code.as_deref() {
        let code = code.trim().to_uppercase();
        if let Some(pair) = s.pending.get(&code) {
            let now = Utc::now();
            let expired = pair.expires_at < now;
            let token = if pair.status == "Completed" {
                s.tokens
                    .iter()
                    .find(|(_, t)| t.device_id == pair.device_id)
                    .map(|(_, t)| t.token.clone())
            } else {
                None
            };
            let status_str: String = if expired {
                "expired".to_string()
            } else {
                pair.status.to_lowercase()
            };
            return Ok(Json(json!({
                "status": status_str,
                "code": code,
                "device_id": pair.device_id,
                "device_name": pair.device_name,
                "platform": pair.platform,
                "scopes": pair.scopes,
                "expires_at": pair.expires_at,
                "expired": expired,
                "token": token,
                "next_tools": ["focusa_device_pair_list"],
                "rehydrate_id": pair.device_id,
            })));
        }
        return Err(rejection(
            StatusCode::NOT_FOUND,
            json!({
                "status": "not_found",
                "failure_class": "pair_code_not_found",
                "code": code,
            }),
        ));
    }
    if let Some(device_id) = query.device_id.as_deref() {
        let now = Utc::now();
        let token = s
            .tokens
            .values()
            .find(|t| t.device_id == device_id)
            .map(|t| json!({
                "token": t.token,
                "scopes": t.scopes,
                "issued_at": t.issued_at,
                "expires_at": t.expires_at,
                "expired": t.expires_at < now,
            }));
        return Ok(Json(json!({
            "status": "completed",
            "device_id": device_id,
            "token": token,
            "next_tools": ["focusa_device_pair_list"],
            "rehydrate_id": device_id,
        })));
    }
    Err(rejection(
        StatusCode::UNPROCESSABLE_ENTITY,
        json!({
            "status": "validation_rejected",
            "failure_class": "query_missing",
            "message": "code or device_id required",
        }),
    ))
}

#[derive(Debug, Deserialize)]
pub struct PairListRequest {
    pub host: Option<String>,
    pub limit: Option<usize>,
}

async fn pair_list(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<PairListRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let host = query
        .host
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("operator-vps");
    let limit = query.limit.unwrap_or(50).min(200);
    let records = state
        .persistence
        .read_device_records(host, limit)
        .unwrap_or_default();
    let summary: Vec<Value> = records
        .iter()
        .map(|r| {
            json!({
                "device_id": r.device_id,
                "name": r.name,
                "platform": r.platform,
                "host": r.host,
                "scopes": r.scopes,
                "paired_at": r.paired_at,
                "last_seen_at": r.last_seen_at,
                "revoked": r.revoked,
                "revoked_at": r.revoked_at,
            })
        })
        .collect();
    Ok(Json(json!({
        "status": "completed",
        "host": host,
        "count": records.len(),
        "devices": summary,
        "next_tools": ["focusa_device_pair_revoke", "focusa_session_transfer"],
        "rehydrate_id": records.last().map(|r| r.device_id.clone()).unwrap_or_else(|| "no_devices".to_string()),
    })))
}

#[derive(Debug, Deserialize)]
pub struct PairRevokeRequest {
    pub device_id: String,
    pub host: Option<String>,
    pub reason: Option<String>,
}

async fn pair_revoke(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PairRevokeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let device_id = body.device_id.trim();
    if device_id.is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "device_id_missing",
                "field": "device_id",
            }),
        ));
    }
    let host = body.host.unwrap_or_else(|| "operator-vps".to_string());
    if is_unsafe_agent_runtime_path_inline(&host) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "scope_mismatch",
                "field": "host",
                "rejected_value": host,
            }),
        ));
    }
    let now = Utc::now();
    // Append a new DeviceRecord with revoked=true (append-only ledger).
    let record = DeviceRecord {
        device_id: device_id.to_string(),
        name: format!("revoked-{}", device_id),
        platform: "unknown".to_string(),
        host: host.clone(),
        scopes: Vec::new(),
        paired_at: now,
        last_seen_at: now,
        revoked: true,
        revoked_at: Some(now),
    };
    if let Err(e) = state.persistence.append_device_record(&record) {
        return Err(rejection(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({
                "status": "blocked",
                "failure_class": "storage_unwritable",
                "message": format!("append failed: {}", e),
            }),
        ));
    }
    // Invalidate the in-memory token too.
    {
        let pairing_state = shared_state();
        let mut st = pairing_state.write().await;
        st.tokens.retain(|_, t| t.device_id != device_id);
    }
    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "device_id": device_id,
        "host": host,
        "reason": body.reason,
        "ledger_appended": true,
        "next_tools": ["focusa_device_pair_list"],
        "rehydrate_id": device_id,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_format_is_FOCUS_DASH_8_DASH_4() {
        let code = generate_code();
        assert!(code.starts_with("FOCUS-"));
        // 4 hex + dash + 4 hex after the FOCUS- prefix.
        let suffix = &code[6..];
        let dash = suffix.chars().position(|c| c == '-').expect("dash");
        assert_eq!(dash, 8, "first 4 hex chars then dash");
        assert_eq!(suffix.len(), 8 + 1 + 4, "4+1+4 = 9 chars after FOCUS-");
    }

    #[test]
    fn token_is_32_hex() {
        let t = generate_token();
        assert_eq!(t.len(), 32);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn unsafe_paths_blocked() {
        assert!(is_unsafe_agent_runtime_path_inline("/root/pi-mono"));
        assert!(is_unsafe_agent_runtime_path_inline("/root/pi-mono/sub"));
        assert!(!is_unsafe_agent_runtime_path_inline("/home/wirebot/focusa"));
        assert!(!is_unsafe_agent_runtime_path_inline("/home/operator-vps"));
    }
}
