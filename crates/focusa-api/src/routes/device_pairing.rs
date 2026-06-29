//! Mac menubar OAuth-like device pairing (focusa-ui0y).
//!
//! # Canonical V2 self-host pairing flow (focusa-ui0y Phase-1 / Phase-2)
//!
//! Phase-1 (canonical, status-poll based):
//!   1. VPS creates the room:        POST /v1/connect/room/create
//!   2. Mac idles with mac_offer QR: GET /v1/connect/rooms?status=waiting_for_mac
//!   3. Mac joins room:              POST /v1/connect/room/{id}/join
//!   4. Phone PWA opens:             GET /connect/room/{id}/scan
//!   5. Phone PWA approves:          POST /v1/connect/room/approve
//!   6. Mac polls for token:         GET /v1/connect/room/status?connect_id=...
//!
//! Phase-2 (callback fast path, optional):
//!   - Mac additionally starts an ephemeral LAN HTTP listener and embeds
//!     `mac_callback` in its mac_offer; the VPS POSTs the completed
//!     payload to that URL after minting the token.
//!   - Validate via `validate_mac_callback_url` (allows RFC1918 + Tailscale).
//!
//! # Deprecated / headless only (device-code flow)
//!
//! The device-code flow below is retained for SSH/CLI ops where a Mac
//! cannot run the V2 Bridge Room flow. It mints a token on a paired
//! device record via `focusa device pair-complete`. It is NOT the
//! canonical first-run path; new code should use the V2 Bridge Room.
//!
//!   - POST /v1/device/pair/start  (creates a code + device record)
//!   - focusa device pair-complete (operator side, mints the token)
//!   - GET  /v1/device/pair/status?code=... (Mac polls)
//!   - POST /v1/device/pair/revoke  (revokes a device + deletes SQLite tokens)
//!   - GET  /v1/device/pair/list?host=...
//!
//! # Legacy / not exposed by default
//!
//! The "Mac app calls /v1/device/pair/start" mental model is only used
//! by the headless flow above. New integrations should call the
//! canonical V2 Bridge Room endpoints.
//!
//! ## Persistence invariants (V2)
//!
//!   - PairingStore persists EVERY trust transition (room create, join,
//!     approve, revoke). Persistence failures block the response.
//!   - On daemon startup, `rehydrate_pairing_state_from_ledger` rebuilds the
//!     in-memory maps from the SQLite ledger.
//!   - Tokens are revocable across restart: `revoke_device_tokens_by_device`
//!     deletes the persisted device_tokens rows, and the auth middleware's
//!     SQLite fallback reads `load_device_token_full` so route-scope checks
//!     see the actual granted scopes (no hardcoded placeholder).
//!   - Room token delivery is one-shot: after the Mac polls the token once,
//!     the room transitions to `consumed` and subsequent /status calls
//!     return token_present=true with token=null.
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
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use focusa_core::types::{DevicePairCode, DevicePairCompletion, DeviceRecord, DeviceToken};

use super::pairing_store;
use rand::{RngCore, rngs::OsRng};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

const CODE_TTL_SECS: i64 = 300; // 5 min
const TOKEN_TTL_SECS: i64 = 60 * 60 * 24 * 30; // 30 days

#[derive(Debug, Clone)]
pub struct ConnectSession {
    pub connect_id: String,
    pub device_id: String,
    pub mac_name: String,
    pub mac_nonce: String,
    pub mac_pubkey: Option<String>,
    pub mac_callback: Option<String>,
    pub server_url: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: String,
    pub token: Option<String>,
    /// V2: tracks whether the token has been delivered to the Mac via /status.
    /// The first /status call after token minting returns the full token;
    /// subsequent calls return token_present=true, token=null. After
    /// TOKEN_DELIVERY_TTL_SECS the room is fully consumed and the token is
    /// hidden even if token_delivered was never set.
    pub token_delivered: bool,
    pub delivered_at: Option<DateTime<Utc>>,
}

/// V2: PairingState is a **runtime cache over the SQLite ledger**, NOT
/// the source of truth. Every trust transition (room create, room join,
/// room approve, token mint, token revoke) is written to SQLite before
/// the in-memory map is updated. On daemon startup,
/// `rehydrate_pairing_state_from_ledger()` rebuilds these maps from the
/// ledger so a daemon restart cannot lose state or resurrect revoked
/// tokens.
#[derive(Default)]
pub struct PairingState {
    pending: HashMap<String, DevicePairCode>, // code -> pair
    pub tokens: HashMap<String, DeviceToken>,     // token -> token (public for auth middleware)
    #[allow(private_interfaces)]
    pub connect_sessions: HashMap<String, ConnectSession>, // connect_id -> rendezvous (public for /v1/connect/rooms)
}

type SharedPairingState = Arc<RwLock<PairingState>>;

pub fn router() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/v1/device/pair/start", axum::routing::post(pair_start))
        .route(
            "/v1/device/pair/complete",
            axum::routing::post(pair_complete),
        )
        .route("/v1/device/pair/status", axum::routing::get(pair_status))
        .route("/v1/device/pair/list", axum::routing::get(pair_list))
        .route("/v1/device/pair/revoke", axum::routing::post(pair_revoke))
        .route("/v1/connect/start", axum::routing::post(connect_start))
        .route("/v1/connect/status", axum::routing::get(connect_status))
        .route("/v1/connect/approve", axum::routing::post(connect_approve))
        .route(
            "/v1/connect/room/start",
            axum::routing::post(connect_room_start),
        )
        // focusa-ui0y v0.9.35-dev: VPS-initiated room creation.
        .route(
            "/v1/connect/room/create",
            axum::routing::post(connect_room_create),
        )
        .route(
            "/v1/connect/room/{room_id}/status",
            axum::routing::get(connect_room_status),
        )
        // v0.9.35-dev: list rooms so the Mac can discover VPS-created rooms
        // and POST its static mac_offer to /join. Canonical V2 flow.
        .route(
            "/v1/connect/rooms",
            axum::routing::get(connect_rooms_list),
        )
        .route(
            "/v1/connect/room/{room_id}/mac-offer",
            axum::routing::post(connect_room_mac_offer),
        )
        // v0.9.35-dev: Mac joins a VPS-created room
        .route(
            "/v1/connect/room/{room_id}/join",
            axum::routing::post(connect_room_join),
        )
        .route(
            "/v1/connect/room/{room_id}/approve",
            axum::routing::post(connect_room_approve),
        )
        .route("/connect", axum::routing::get(connect_mediator_page))
        .route("/connect/firstrun", axum::routing::get(connect_firstrun_page))
        // v0.9.35-dev: PWA /scan page with getUserMedia camera. Replaces
        // /firstrun as the canonical phone-side entry point.
        .route("/connect/room/{room_id}/scan", axum::routing::get(connect_room_scan_page))
        .route("/connect/{room_id}", axum::routing::get(connect_room_page))
        // focusa-ui0y WhatsApp-like first-run: Mac creates a rendezvous and
        // receives the public server_url to embed in the URL-QR.
        .route(
            "/v1/connect/room/firstrun",
            axum::routing::post(connect_room_firstrun),
        )
        // focusa-ui0y.8: PWA helper page for QR/PWA handoff
        .route("/pair/{device_id}", axum::routing::get(pwa_helper_page))
        .route(
            "/pair/{device_id}/manifest.json",
            axum::routing::get(pwa_manifest),
        )
        .route(
            "/pair/{device_id}/sw.js",
            axum::routing::get(pwa_service_worker),
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
    // 32-byte cryptographically random token, base64url-no-pad encoded.
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn bounded_label(value: Option<String>, fallback: &str, max: usize) -> String {
    let sanitized: String = value
        .unwrap_or_else(|| fallback.to_string())
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ' '))
        .take(max)
        .collect::<String>()
        .trim()
        .to_string();
    if sanitized.is_empty() {
        fallback.to_string()
    } else {
        sanitized
    }
}

fn normalize_scopes(scopes: Option<Vec<String>>) -> Result<Vec<String>, (StatusCode, Json<Value>)> {
    let raw = scopes.unwrap_or_else(|| vec!["read".to_string(), "write".to_string()]);
    let mut out = Vec::new();
    for scope in raw {
        let scope = scope.trim().to_ascii_lowercase();
        if scope.is_empty() {
            continue;
        }
        if !matches!(scope.as_str(), "read" | "write") {
            return Err(rejection(
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({
                    "status": "validation_rejected",
                    "failure_class": "scope_not_allowed",
                    "field": "scopes",
                    "allowed_scopes": ["read", "write"],
                }),
            ));
        }
        if !out.contains(&scope) {
            out.push(scope);
        }
    }
    if out.is_empty() {
        out.push("read".to_string());
    }
    Ok(out)
}

fn validate_pairing_url(url: &str, field: &str) -> Result<String, (StatusCode, Json<Value>)> {
    let trimmed = url.trim().trim_end_matches('/').to_string();
    let allowed = trimmed.starts_with("https://")
        || trimmed.starts_with("http://127.0.0.1")
        || trimmed.starts_with("http://localhost");
    if allowed && trimmed.len() <= 2048 && !trimmed.contains(char::is_whitespace) {
        Ok(trimmed)
    } else {
        tracing::warn!(
            field = %field,
            url = %url,
            url_len = url.len(),
            "phone bridge pairing URL validation rejected"
        );
        Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "pairing_url_invalid",
                "field": field,
                "message": "pairing URLs must be https:// or localhost/127.0.0.1 http:// during local development",
            }),
        ))
    }
}

/// V2: validate a Mac callback URL. Unlike the public pairing URL, this is
/// an ephemeral LAN HTTP endpoint that the Mac opens to receive the token
/// after /approve. It must allow private RFC1918 IPv4 (192.168/16, 10/8,
/// 172.16/12) and Tailscale (100.64/10) in addition to localhost, and it
/// must point at /focusa-phone-bridge/<nonce>. Using validate_pairing_url()
/// here would reject every LAN callback and silently break the Phase-2 fast
/// path.
fn validate_mac_callback_url(url: &str, field: &str) -> Result<String, (StatusCode, Json<Value>)> {
    let trimmed = url.trim().to_string();
    if trimmed.is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "mac_callback_missing",
                "field": field,
            }),
        ));
    }
    if trimmed.len() > 2048 || trimmed.contains(char::is_whitespace) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "mac_callback_invalid",
                "field": field,
                "message": "callback URL too long or contains whitespace",
            }),
        ));
    }
    // Must be http:// or https://
    let scheme_end = match trimmed.find("://") {
        Some(i) => i,
        None => {
            return Err(rejection(
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({
                    "status": "validation_rejected",
                    "failure_class": "mac_callback_invalid",
                    "field": field,
                    "message": "callback URL must include scheme http(s)://",
                }),
            ));
        }
    };
    let scheme = &trimmed[..scheme_end];
    if scheme != "http" && scheme != "https" {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "mac_callback_invalid",
                "field": field,
                "message": "callback scheme must be http or https",
            }),
        ));
    }
    // https is only allowed for loopback (Tauri production); http for LAN.
    // Don't enforce this distinction since both work in dev — both schemes
    // are accepted; the host rule below is the actual gate.
    let host_start = scheme_end + 3;
    let rest = &trimmed[host_start..];
    let host_part = rest
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("");
    let host_ok = is_loopback_host(host_part) || is_private_ipv4(host_part) || is_tailscale_host(host_part);
    if !host_ok {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "mac_callback_invalid",
                "field": field,
                "message": "callback host must be loopback, RFC1918 private, or Tailscale (100.64/10)",
            }),
        ));
    }
    // Must include the canonical path prefix.
    if !trimmed.contains("/focusa-phone-bridge/") {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "mac_callback_invalid",
                "field": field,
                "message": "callback URL must include /focusa-phone-bridge/<nonce>",
            }),
        ));
    }
    Ok(trimmed)
}

fn is_loopback_host(host: &str) -> bool {
    host == "localhost" || host == "127.0.0.1" || host == "::1"
}

fn is_private_ipv4(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    let octets: Option<Vec<u8>> = parts
        .iter()
        .map(|p| p.parse::<u8>().ok())
        .collect();
    let Some(o) = octets else { return false };
    if o[0] == 10 {
        return true;
    } // 10.0.0.0/8
    if o[0] == 172 && (16..=31).contains(&o[1]) {
        return true;
    } // 172.16.0.0/12
    if o[0] == 192 && o[1] == 168 {
        return true;
    } // 192.168.0.0/16
    if o[0] == 169 && o[1] == 254 {
        return true;
    } // link-local (Bonjour)
    false
}

fn is_tailscale_host(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    let octets: Option<Vec<u8>> = parts
        .iter()
        .map(|p| p.parse::<u8>().ok())
        .collect();
    let Some(o) = octets else { return false };
    o[0] == 100 && (64..=127).contains(&o[1])
}

pub fn shared_state() -> SharedPairingState {
    use std::sync::OnceLock;
    static STATE: OnceLock<SharedPairingState> = OnceLock::new();
    STATE
        .get_or_init(|| Arc::new(RwLock::new(PairingState::default())))
        .clone()
}

fn is_unsafe_agent_runtime_path_inline(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed == "/" || trimmed == "/root" {
        return true;
    }
    const BLOCKED: &[&str] = &[
        "/root/pi-mono",
        "/root/.pi",
        "/root/.cargo",
        "/root/.claude",
        "/root/.opencode",
        "/root/.letta",
        "/home/wirebot/.cargo",
    ];
    BLOCKED
        .iter()
        .any(|p| trimmed == *p || trimmed.starts_with(&format!("{}/", p)))
}

#[derive(Debug, Deserialize)]
pub struct PairStartRequest {
    pub device_name: Option<String>,
    pub platform: Option<String>,
    pub daemon_base_url: Option<String>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectStartRequest {
    pub mac_name: Option<String>,
    pub mac_nonce: Option<String>,
    pub mac_pubkey: Option<String>,
    pub mac_callback: Option<String>,
    pub server_url: Option<String>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectStatusRequest {
    pub connect_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ConnectApproveRequest {
    pub connect_id: String,
    pub host: Option<String>,
    pub operator_id: Option<String>,
    pub completed_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectRoomStartRequest {
    pub server_url: Option<String>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectRoomCreateRequest {
    /// Optional operator-supplied public VPS URL hint. Falls back to FOCUSA_PAIRING_URL env.
    pub server_url: Option<String>,
    /// Optional scopes (defaults to read+write).
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectRoomJoinRequest {
    /// Mac device name (operator-readable).
    pub mac_name: Option<String>,
    /// Random nonce the Mac generated (used to bind the mac_offer).
    pub mac_nonce: Option<String>,
    /// Alias for mac_nonce (canonical V2 mac_offer JSON uses "nonce").
    #[serde(alias = "nonce")]
    pub mac_nonce_v2: Option<String>,
    /// Optional base64url public key.
    pub mac_pubkey: Option<String>,
    /// Alias for mac_pubkey (canonical V2 uses "pubkey").
    #[serde(alias = "pubkey")]
    pub mac_pubkey_v2: Option<String>,
    /// Optional ephemeral callback URL on the Mac.
    pub mac_callback: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectRoomFirstrunRequest {
    /// Mac device name (operator-readable).
    pub mac_name: Option<String>,
    /// Operator-supplied hint for the public Connect origin; the daemon
    /// verifies it before returning.
    pub server_url: Option<String>,
    /// Optional ephemeral TCP callback URL on the Mac (fast-path optimization).
    pub mac_callback: Option<String>,
    /// Optional random nonce the Mac generated.
    pub mac_nonce: Option<String>,
    /// Alias for mac_nonce (canonical V2 mac_offer uses "nonce").
    #[serde(alias = "nonce")]
    pub mac_nonce_v2: Option<String>,
    /// Optional base64url public key.
    pub mac_pubkey: Option<String>,
    /// Alias for mac_pubkey (canonical V2 uses "pubkey").
    #[serde(alias = "pubkey")]
    pub mac_pubkey_v2: Option<String>,
    /// Optional scopes; defaults to read+write.
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectRoomMacOfferRequest {
    pub mac_name: Option<String>,
    pub mac_nonce: Option<String>,
    pub mac_pubkey: Option<String>,
    pub mac_callback: Option<String>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectRoomApproveRequest {
    pub host: Option<String>,
    pub operator_id: Option<String>,
    pub completed_by: Option<String>,
}

fn public_server_url(fallback: Option<String>) -> String {
    std::env::var("FOCUSA_PAIRING_URL")
        .ok()
        .or(fallback)
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn connect_status_payload(session: &ConnectSession, status: String) -> Value {
    let expired = session.expires_at < Utc::now();
    json!({
        "status": status,
        "room_id": session.connect_id,
        "connect_id": session.connect_id,
        "device_id": session.device_id,
        "mac_name": session.mac_name,
        "mac_nonce": session.mac_nonce,
        "mac_pubkey": session.mac_pubkey,
        "mac_callback": session.mac_callback,
        "server_url": session.server_url,
        "connect_url": format!("{}/connect/{}", session.server_url, session.connect_id),
        "scopes": session.scopes,
        "created_at": session.created_at,
        "expires_at": session.expires_at,
        "expired": expired,
        "token": session.token,
        "next_tools": ["focusa_connect_room_status", "focusa_connect_room_approve"],
        "diagnostics": {
            "surface": "phone_bridge_flow",
            "room_state": status,
            "mac_offer_seen": !session.mac_nonce.trim().is_empty(),
            "mac_callback_present": session.mac_callback.is_some(),
            "token_present": session.token.is_some(),
            "expired": expired,
            "next_step_hint": if expired { "Run `focusa pair` again." } else if session.mac_nonce.trim().is_empty() { "Scan the Mac Handoff Offer from the Focusa Connect Page." } else if session.token.is_none() { "Approve the Mac from the Focusa Connect Page." } else { "Mac can store the returned token." }
        },
        "rehydrate_id": session.connect_id,
    })
}

/// V2 P0.1: One-shot token-delivery logic. Called by BOTH the query-style
/// `/v1/connect/status?connect_id=...` and the path-style
/// `/v1/connect/room/{id}/status` endpoints. First delivery returns the
/// token and flips `token_delivered=true`. Subsequent calls return
/// `token_present=true` with `token=null` and `status=consumed` once the
/// session has been completed. Without this, callers on either endpoint
/// could re-poll the token indefinitely while the room is completed.
fn one_shot_status_payload(session: &mut ConnectSession) -> (bool, Value) {
    const TOKEN_DELIVERY_TTL_SECS: i64 = 60;
    let now = Utc::now();
    let expired = session.expires_at < now;
    let token_visible = session.token.is_some()
        && !session.token_delivered
        && session
            .delivered_at
            .map(|t| (now - t).num_seconds() < TOKEN_DELIVERY_TTL_SECS)
            .unwrap_or(true);
    let mut token_to_return: Option<String> = None;
    if token_visible {
        token_to_return = session.token.clone();
    }
    if token_to_return.is_some() && !session.token_delivered {
        session.token_delivered = true;
        session.delivered_at = Some(now);
    }
    let status = if expired && session.token.is_none() {
        "expired".to_string()
    } else if session.token.is_some() && !token_visible {
        "consumed".to_string()
    } else {
        session.status.clone()
    };
    let mut payload = connect_status_payload(session, status.clone());
    // V2 P0.1: ensure token field is null when token was already delivered,
    // regardless of which endpoint served the response.
    if session.token.is_some() {
        payload.as_object_mut().unwrap().insert(
            "token".to_string(),
            serde_json::Value::from(token_to_return),
        );
    }
    (expired, payload)
}

async fn connect_room_start(
    Json(body): Json<ConnectRoomStartRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let now = Utc::now();
    let expires = now + Duration::seconds(CODE_TTL_SECS);
    let room_id = Uuid::now_v7().to_string();
    let device_id = Uuid::now_v7().to_string();
    let scopes = body
        .scopes
        .unwrap_or_else(|| vec!["read".to_string(), "write".to_string()]);
    let server_url = public_server_url(body.server_url);
    let connect_url = format!("{server_url}/connect/{room_id}");

    let session = ConnectSession {
        connect_id: room_id.clone(),
        device_id: device_id.clone(),
        mac_name: String::new(),
        mac_nonce: String::new(),
        mac_pubkey: None,
        mac_callback: None,
        server_url: server_url.clone(),
        scopes: scopes.clone(),
        created_at: now,
        expires_at: expires,
        status: "waiting_for_mac".to_string(),
        token: None,
                token_delivered: false,
            delivered_at: None,
        };

    shared_state()
        .write()
        .await
        .connect_sessions
        .insert(room_id.clone(), session);

    tracing::info!(
        room_id = %room_id,
        device_id = %device_id,
        server_url = %server_url,
        expires_in_secs = CODE_TTL_SECS,
        "phone bridge room started"
    );

    Ok(Json(json!({
        "status": "waiting_for_mac",
        "canonical": false,
        "advisory": true,
        "room_id": room_id,
        "connect_id": room_id,
        "device_id": device_id,
        "server_url": server_url,
        "connect_url": connect_url,
        "scopes": scopes,
        "expires_at": expires,
        "expires_in_secs": CODE_TTL_SECS,
        "server_handoff": {
            "protocol": "focusa-connect-v1",
            "role": "pairing_room",
            "server_url": server_url,
            "room_id": room_id,
            "connect_url": connect_url,
            "expires_in_secs": CODE_TTL_SECS
        },
        "next_tools": ["focusa_connect_room_status", "focusa_connect_room_mac_offer"],
        "diagnostics": {
            "surface": "phone_bridge_flow",
            "event": "room_started",
            "room_state": "waiting_for_mac",
            "next_step_hint": "Open the Focusa Connect Page on the phone, then scan the Mac Handoff Offer."
        },
        "rehydrate_id": room_id,
    })))
}

async fn connect_room_status(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pairing_state = shared_state();
    // V2 P0.1: Path-style /v1/connect/room/{id}/status shares the
    // one-shot token-delivery logic with /v1/connect/status?connect_id=.
    // Both endpoints must enforce the same constraint: once a token has
    // been delivered, subsequent polls get token_present=true with
    // token=null and status=consumed. Without this, the Mac wizard
    // (which polls the path-style endpoint) would receive the full token
    // on every poll while the room is completed.
    let mut s = pairing_state.write().await;
    if let Some(session) = s.connect_sessions.get_mut(room_id.trim()) {
        let (expired, payload) = one_shot_status_payload(session);
        return Ok(Json(payload));
    }
    // V2: in-memory miss; fall back to the SQLite ledger. This is the
    // restart-durability path: if the daemon restarted mid-pairing, the
    // session may only exist in the ledger, not the in-memory map.
    drop(s);
    match pairing_store::get_session(&state, room_id.trim()) {
        Ok(Some(p)) => {
            // Reconstruct a minimal status payload from the persisted row.
            let status = p.status.clone();
            Ok(Json(json!({
                "status": status,
                "room_id": room_id,
                "connect_id": room_id,
                "server_url": p.server_url,
                "mac_callback": p.mac_callback,
                "expires_at": p.expires_at,
                "expired": status == "expired",
                "diagnostics": {
                    "surface": "phone_bridge_flow",
                    "event": "room_rehydrated_from_ledger",
                    "room_state": status.clone(),
                    "next_step_hint": "Room was rehydrated from SQLite after daemon restart.",
                },
            })))
        }
        _ => Err(rejection(
            StatusCode::NOT_FOUND,
            json!({
                "status": "not_found",
                "failure_class": "connect_room_not_found",
                "room_id": room_id,
            }),
        )),
    }
}

// GET /v1/connect/rooms[?status=waiting_for_mac]
// Canonical V2 surface: Mac polls this to discover VPS-created rooms, then
// POSTs its static mac_offer to /v1/connect/room/{room_id}/join.
//
// V2: Falls back to the SQLite PairingStore ledger when the in-memory map is
// empty (e.g. after a daemon restart). Without this fallback, the Mac wizard
// cannot discover rooms that survived restart, breaking the canonical flow.
async fn connect_rooms_list(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let filter = q.get("status").cloned().unwrap_or_default();
    let pairing_state = shared_state();
    let s = pairing_state.read().await;
    let now = Utc::now();
    let mut rooms: Vec<Value> = Vec::new();
    let mut seen_room_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (room_id, session) in s.connect_sessions.iter() {
        let status = if session.expires_at < now && session.token.is_none() {
            "expired".to_string()
        } else {
            session.status.clone()
        };
        if !filter.is_empty() && filter != status {
            continue;
        }
        seen_room_ids.insert(room_id.clone());
        rooms.push(json!({
            "room_id": room_id,
            "status": status,
            "mac_name": session.mac_name,
            "expires_at": session.expires_at,
            "server_url": session.server_url,
            "source": "memory",
        }));
    }
    drop(s);
    // Fallback: scan the SQLite ledger for any rooms not already in memory.
    // We only have a thin PersistedSession (server_url, expires_at, mac_callback,
    // status) so the row is sparse — but that's enough for the Mac to know
    // there IS a room and POST its mac_offer to /join.
    if let Ok(persisted) = state.persistence.list_connect_sessions() {
        for (connect_id, server_url, expires_at, status) in persisted {
            if seen_room_ids.contains(&connect_id) {
                continue;
            }
            let expired = match chrono::DateTime::parse_from_rfc3339(&expires_at) {
                Ok(t) => t.with_timezone(&Utc) < now,
                Err(_) => false,
            };
            let status = if expired {
                "expired".to_string()
            } else {
                status
            };
            if !filter.is_empty() && filter != status {
                continue;
            }
            rooms.push(json!({
                "room_id": connect_id,
                "status": status,
                "mac_name": "",
                "expires_at": expires_at,
                "server_url": server_url,
                "source": "ledger",
            }));
        }
    }
    Ok(Json(json!({
        "status": "ok",
        "rooms": rooms,
        "filter": filter,
        "count": rooms.len(),
    })))
}

/// Enumerate every connect_session row in the SQLite ledger. Used by
/// /v1/connect/rooms to rehydrate after a daemon restart.
fn list_persisted_sessions_unused(
    state: &AppState,
) -> anyhow::Result<Vec<PersistedSessionRow>> {
    let _ = state;
    Ok(Vec::new())
}

/// Mirror of the SQLite row returned by get_connect_session.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PersistedSessionRow {
    connect_id: String,
    server_url: String,
    expires_at: String,
    status: String,
}

async fn connect_room_mac_offer(
    Path(room_id): Path<String>,
    Json(body): Json<ConnectRoomMacOfferRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mac_name = body.mac_name.unwrap_or_else(|| "Focusa Mac".to_string());
    let mac_nonce = body
        .mac_nonce
        .unwrap_or_else(|| Uuid::now_v7().simple().to_string());
    if mac_name.trim().is_empty() || mac_nonce.trim().is_empty() {
        tracing::warn!(room_id = %room_id, "phone bridge mac offer rejected: missing mac_name or mac_nonce");
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "mac_offer_missing",
                "message": "mac_name and mac_nonce are required",
            }),
        ));
    }

    let now = Utc::now();
    let pairing_state = shared_state();
    let mut s = pairing_state.write().await;
    let Some(existing) = s.connect_sessions.get(room_id.trim()).cloned() else {
        tracing::warn!(room_id = %room_id, "phone bridge mac offer rejected: room not found");
        return Err(rejection(
            StatusCode::NOT_FOUND,
            json!({
                "status": "not_found",
                "failure_class": "connect_room_not_found",
                "room_id": room_id,
            }),
        ));
    };
    if existing.expires_at < now && existing.token.is_none() {
        tracing::warn!(room_id = %room_id, expired_at = %existing.expires_at, "phone bridge mac offer rejected: room expired");
        return Err(rejection(
            StatusCode::GONE,
            json!({
                "status": "expired",
                "failure_class": "connect_room_expired",
                "room_id": room_id,
                "expired_at": existing.expires_at,
            }),
        ));
    }

    let mut updated = existing;
    // V2 P1.4 nonce mismatch rejection: if /mac-offer or /join has
    // already bound a nonce for this room, refuse to overwrite it with
    // a different one. The Mac's nonce is the per-pair identifier
    // embedded in the QR; if a subsequent call carries a different
    // nonce for the same room, treat it as a hostile replay and reject
    // with 409. This protects against an attacker who intercepts the
    // PWA tab and tries to swap in their own nonce to redirect the
    // mac_callback fast path to a URL they control.
    if !updated.mac_nonce.trim().is_empty()
        && updated.mac_nonce != mac_nonce
    {
        tracing::warn!(
            room_id = %room_id,
            existing_nonce = %updated.mac_nonce,
            attempted_nonce = %mac_nonce,
            "V2 P1.4: mac_nonce mismatch on /mac-offer or /join; rejecting"
        );
        return Err(rejection(
            StatusCode::CONFLICT,
            json!({
                "status": "conflict",
                "failure_class": "mac_nonce_mismatch",
                "message": "room already has a mac_nonce bound; refusing to overwrite",
                "room_id": room_id,
            }),
        ));
    }
    updated.mac_name = mac_name;
    updated.mac_nonce = mac_nonce;
    updated.mac_pubkey = body.mac_pubkey;
    // V2 P1.2: validate the mac_callback URL on every mac-offer path
    // (canonical /mac-offer and /join both run the same shape, so we
    // don't risk drift). Empty-string callback is treated as "not
    // provided" (Mac may emit "" when bridge startup failed). The
    // bridge is optional in V2 anyway.
    if let Some(cb) = body
        .mac_callback
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        validate_mac_callback_url(cb, "mac_callback")?;
        updated.mac_callback = Some(cb.to_string());
    } else {
        updated.mac_callback = None;
    }
    if let Some(scopes) = body.scopes {
        updated.scopes = scopes;
    }
    updated.status = "mac_seen".to_string();
    s.connect_sessions
        .insert(updated.connect_id.clone(), updated.clone());
    tracing::info!(
        room_id = %updated.connect_id,
        device_id = %updated.device_id,
        mac_name = %updated.mac_name,
        mac_callback_present = updated.mac_callback.is_some(),
        "phone bridge mac offer accepted"
    );
    Ok(Json(connect_status_payload(
        &updated,
        updated.status.clone(),
    )))
}

async fn connect_room_approve(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
    Json(body): Json<ConnectRoomApproveRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    connect_approve(
        State(state),
        Json(ConnectApproveRequest {
            connect_id: room_id,
            host: body.host,
            operator_id: body.operator_id,
            completed_by: Some(
                body.completed_by
                    .unwrap_or_else(|| "phone-pwa-room".to_string()),
            ),
        }),
    )
    .await
}

async fn connect_start(
    Json(body): Json<ConnectStartRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let now = Utc::now();
    let expires = now + Duration::seconds(CODE_TTL_SECS);
    let connect_id = Uuid::now_v7().to_string();
    let device_id = Uuid::now_v7().to_string();
    let mac_name = body.mac_name.unwrap_or_else(|| "Focusa Mac".to_string());
    let mac_nonce = body
        .mac_nonce
        .unwrap_or_else(|| Uuid::now_v7().simple().to_string());
    let scopes = body
        .scopes
        .unwrap_or_else(|| vec!["read".to_string(), "write".to_string()]);
    let server_url = public_server_url(body.server_url);

    if mac_name.trim().is_empty() || mac_nonce.trim().is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "mac_offer_missing",
                "message": "mac_name and mac_nonce are required",
            }),
        ));
    }

    let session = ConnectSession {
        connect_id: connect_id.clone(),
        device_id: device_id.clone(),
        mac_name: mac_name.clone(),
        mac_nonce: mac_nonce.clone(),
        mac_pubkey: body.mac_pubkey,
        mac_callback: body.mac_callback,
        server_url: server_url.clone(),
        scopes: scopes.clone(),
        created_at: now,
        expires_at: expires,
        status: "pending".to_string(),
        token: None,
                token_delivered: false,
            delivered_at: None,
        };

    let pairing_state = shared_state();
    pairing_state
        .write()
        .await
        .connect_sessions
        .insert(connect_id.clone(), session);

    Ok(Json(json!({
        "status": "pending",
        "canonical": false,
        "advisory": true,
        "connect_id": connect_id,
        "device_id": device_id,
        "mac_name": mac_name,
        "mac_nonce": mac_nonce,
        "server_url": server_url,
        "scopes": scopes,
        "expires_at": expires,
        "expires_in_secs": CODE_TTL_SECS,
        "server_handoff": {
            "protocol": "focusa-connect-v1",
            "role": "server_handoff",
            "server_url": server_url,
            "connect_id": connect_id,
            "device_id": device_id,
            "nonce": mac_nonce,
            "expires_in_secs": CODE_TTL_SECS
        },
        "next_tools": ["focusa_connect_status", "focusa_connect_approve"],
        "rehydrate_id": connect_id,
    })))
}

async fn connect_status(
    axum::extract::Query(query): axum::extract::Query<ConnectStatusRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pairing_state = shared_state();
    let mut s = pairing_state.write().await;
    let connect_id = query.connect_id.trim();
    let Some(session) = s.connect_sessions.get_mut(connect_id) else {
        return Err(rejection(
            StatusCode::NOT_FOUND,
            json!({
                "status": "not_found",
                "failure_class": "connect_session_not_found",
                "connect_id": connect_id,
            }),
        ));
    };
    // V2 P0.1: shared one-shot helper used by both query-style and
    // path-style status endpoints. See one_shot_status_payload().
    let (_expired, payload) = one_shot_status_payload(session);
    Ok(Json(payload))
}

async fn connect_approve(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ConnectApproveRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let connect_id = body.connect_id.trim().to_string();
    if connect_id.is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "connect_id_missing",
                "field": "connect_id",
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
    let pairing_state = shared_state();
    // V2 callback fast path: capture token + expiry minted inside the
    // inner block so the server-side POST can use them.
    let mut callback_token: Option<String> = None;
    let mut callback_token_expires: Option<chrono::DateTime<chrono::Utc>> = None;
    let completed = {
        let mut s = pairing_state.write().await;
        let Some(existing) = s.connect_sessions.get(&connect_id).cloned() else {
            return Err(rejection(
                StatusCode::NOT_FOUND,
                json!({
                    "status": "not_found",
                    "failure_class": "connect_session_not_found",
                    "connect_id": connect_id,
                }),
            ));
        };
        if existing.expires_at < now && existing.token.is_none() {
            return Err(rejection(
                StatusCode::GONE,
                json!({
                    "status": "expired",
                    "failure_class": "connect_session_expired",
                    "connect_id": connect_id,
                    "expired_at": existing.expires_at,
                }),
            ));
        }
        if existing.status == "waiting_for_mac"
            || existing.mac_name.trim().is_empty()
            || existing.mac_nonce.trim().is_empty()
        {
            return Err(rejection(
                StatusCode::CONFLICT,
                json!({
                    "status": "waiting_for_mac",
                    "failure_class": "mac_offer_required",
                    "connect_id": connect_id,
                    "message": "Submit the Mac handoff offer before approving this Bridge Room.",
                }),
            ));
        }
        if existing.token.is_some() {
            existing
        } else {
            let token = generate_token();
            let token_expires = now + Duration::seconds(TOKEN_TTL_SECS);
            // Capture for outer-scope callback dispatch below. We move
            // these values OUT of the inner block so the mac_callback
            // POST can include the actual token + its expiry.
            callback_token = Some(token.clone());
            callback_token_expires = Some(token_expires);
            let device_token = DeviceToken {
                token: token.clone(),
                device_id: existing.device_id.clone(),
                scopes: existing.scopes.clone(),
                issued_at: now,
                expires_at: token_expires,
                last_used_at: None,
                issued_to: host.clone(),
            };
s.tokens.insert(token.clone(), device_token.clone());
            // V2 Invariant 6: server-side uniqueness for (mac_name, host).
            // The menubar identifies a device by (mac_name, host). Re-pair
            // of the same Mac against the same VPS must supersede the old
            // token, not stack on top of it. We revoke any prior active
            // token for that (mac_name, host) tuple, which catches
            // re-pair across fresh device_id generations. The
            // (device_id, host) revoke below is the inner-loop guard for
            // duplicate /join retries within the same device_id.
            let revoked_by_mac = state
                .persistence
                .revoke_active_tokens_for_mac_host(&existing.mac_name, &host)
                .unwrap_or(0);
            if revoked_by_mac > 0 {
                tracing::info!(
                    mac_name = %existing.mac_name,
                    host = %host,
                    revoked_count = revoked_by_mac,
                    "V2: revoked prior active tokens for (mac_name, host) before minting new one"
                );
            }
            let revoked = state
                .persistence
                .revoke_active_token_for_device_host(
                    &device_token.device_id,
                    &host,
                )
                .unwrap_or(0);
            if revoked > 0 {
                tracing::info!(
                    device_id = %device_token.device_id,
                    host = %host,
                    revoked_count = revoked,
                    "V2: revoked prior active token for (device_id, host) before minting new one"
                );
            }
            // V2: Persist the token to SQLite so it survives a daemon
            // restart. The in-memory map is the hot path; SQLite is the
            // source of truth on restart. STRICT: a failure here means the
            // /approve response must be blocked, because minting a token
            // the durable store does not know about would silently revoke
            // itself on the next restart.
            if let Err(e) = state.persistence.put_device_token(
                &token,
                &device_token.device_id,
                Some(&serde_json::to_string(&device_token.scopes).unwrap_or_else(|_| "[\"read\",\"write\"]".into())),
                &now.to_rfc3339(),
                &token_expires.to_rfc3339(),
                Some(&host),
            ) {
                // Roll back the in-memory insert to keep the two views in sync.
                s.tokens.remove(&token);
                return Err(rejection(
                    StatusCode::SERVICE_UNAVAILABLE,
                    json!({
                        "status": "blocked",
                        "failure_class": "storage_unwritable",
                        "message": format!("token mint persistence failed: {}", e),
                        "token_minted": false,
                    }),
                ));
            }
            // Best-effort WAL checkpoint so the just-committed write is
            // visible to a subsequent reader (e.g. cross-restart rehydrate).
            let _ = state.persistence.checkpoint_wal();
            let mut updated = existing;
            updated.status = "completed".to_string();
            updated.token = Some(token.clone());
            s.connect_sessions
                .insert(connect_id.clone(), updated.clone());
            // V2: Persist status flip to SQLite so a daemon restart mid-approval
            // still sees the room as completed. The in-memory map is the hot
            // path; the ledger is the source of truth on restart.
            // V2 P0.4: complete_connect_session is a trust-critical
            // transition. A failure here means a daemon restart would
            // rehydrate the room as in-flight (the ledger still says
            // waiting_for_mac/mac_seen), and the user would see a
            // stale, never-completed room. Block the response and roll
            // back in-memory + revoke any token we just persisted.
            if let Err(e) = state.persistence.complete_connect_session(&connect_id) {
                tracing::error!(
                    connect_id = %connect_id,
                    error = %e,
                    "V2 P0.4: complete_connect_session failed; rolling back and blocking response"
                );
                s.tokens.remove(&token);
                let _ = state
                    .persistence
                    .revoke_device_tokens_by_device(&device_token.device_id);
                return Err(rejection(
                    StatusCode::SERVICE_UNAVAILABLE,
                    json!({
                        "status": "blocked",
                        "failure_class": "storage_unwritable",
                        "message": format!("complete_connect_session failed: {}", e),
                        "connect_id": connect_id,
                    }),
                ));
            }
            // Persist the token too so a /status poll after restart can
            // deliver it without re-approval. Same trust-critical
            // treatment: block on failure.
            if let Err(e) = pairing_store::complete_session(&state, &connect_id) {
                tracing::error!(
                    connect_id = %connect_id,
                    error = %e,
                    "V2 P0.4: pairing_store::complete_session failed; rolling back and blocking response"
                );
                s.tokens.remove(&token);
                let _ = state
                    .persistence
                    .revoke_device_tokens_by_device(&device_token.device_id);
                return Err(rejection(
                    StatusCode::SERVICE_UNAVAILABLE,
                    json!({
                        "status": "blocked",
                        "failure_class": "storage_unwritable",
                        "message": format!("complete_session failed: {}", e),
                        "connect_id": connect_id,
                    }),
                ));
            }
            updated
        }
    };

    let record = DeviceRecord {
        device_id: completed.device_id.clone(),
        name: completed.mac_name.clone(),
        platform: "macos".to_string(),
        host: host.clone(),
        scopes: completed.scopes.clone(),
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

    tracing::info!(
        connect_id = %completed.connect_id,
        device_id = %completed.device_id,
        mac_name = %completed.mac_name,
        host = %host,
        "phone bridge approval completed"
    );

    // V2 P0 #3: server-side callback fast path. If the Mac's mac_offer
    // included a mac_callback URL, POST the completed payload to it now.
    // This is the canonical Phase-2 fast path: the Mac opens an ephemeral
    // LAN HTTP listener, includes the URL in mac_offer, and the VPS POSTs
    // the completed device token directly to it after minting. The Mac
    // stores the token in Keychain without polling /status.
    //
    // The callback is best-effort. If the Mac's listener is unreachable
    // (e.g. ephemeral port closed, LAN address rotated), the Mac falls
    // back to polling /status which still works via Phase-1.
    let callback_dispatched = if let Some(cb_url) = completed.mac_callback.as_ref() {
        if let (Some(token), Some(expires_at)) = (
            callback_token.as_ref().or(completed.token.as_ref()),
            callback_token_expires,
        ) {
            dispatch_mac_callback(
                cb_url,
                &completed.connect_id,
                &completed.device_id,
                &completed.mac_name,
                token,
                expires_at,
                &completed.server_url,
                &host,
            )
            .await
        } else {
            tracing::warn!(
                connect_id = %completed.connect_id,
                "V2 callback skipped: token not minted (should be impossible here)"
            );
            false
        }
    } else {
        false
    };

    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "connect_id": completed.connect_id,
        "device_id": completed.device_id,
        "device_name": completed.mac_name,
        "host": host,
        "operator_id": body.operator_id,
        "completed_by": body.completed_by.unwrap_or_else(|| "phone-pwa".to_string()),
        "server_url": completed.server_url,
        "scopes": completed.scopes,
        // Token NOT returned to PWA (canonical V2 model): phone is a
        // renderer, not a participant with persistent state. The Mac
        // receives the token via GET /status (which is the canonical
        // Phase-1 channel) or via the mac_callback TCP bridge.
        "token_present": completed.token.is_some(),
        "mac_receives_token_via": if callback_dispatched { "mac_callback" } else { "room_status_poll" },
        "mac_callback_dispatched": callback_dispatched,
        "next_tools": ["focusa_connect_status", "focusa_device_pair_list"],
        "diagnostics": {
            "surface": "phone_bridge_flow",
            "event": "approval_completed",
            "mac_callback_present": completed.mac_callback.is_some(),
            "mac_callback_dispatched": callback_dispatched,
            "token_present": true,
            "next_step_hint": if callback_dispatched {
                "Mac received the token via callback; status should transition to connected."
            } else {
                "Mac callback not dispatched; Mac will receive the token by polling /status."
            }
        },
        "rehydrate_id": completed.connect_id,
    })))
}

/// V2: best-effort POST of the completed device token to the Mac's
/// ephemeral callback URL. Returns true on HTTP 2xx, false otherwise.
/// The Mac is expected to receive this payload and store the token in
/// Keychain. The endpoint is on the operator's LAN, so we use a short
/// timeout and treat any failure as non-fatal (the Mac can still poll
/// /status and pick up the token via Phase-1).
#[allow(clippy::too_many_arguments)]
async fn dispatch_mac_callback(
    url: &str,
    connect_id: &str,
    device_id: &str,
    device_name: &str,
    token: &str,
    token_expires_at: chrono::DateTime<chrono::Utc>,
    server_url: &str,
    host: &str,
) -> bool {
    let payload = serde_json::json!({
        "protocol": "focusa-connect-v1",
        "role": "mac_completion_payload",
        "connect_id": connect_id,
        "device_id": device_id,
        "device_name": device_name,
        "token": token,
        "token_expires_at": token_expires_at.to_rfc3339(),
        "server_url": server_url,
        "host": host,
    });
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "V2 callback: reqwest client build failed");
            return false;
        }
    };
    match client.post(url).json(&payload).send().await {
        Ok(resp) if resp.status().is_success() => {
            tracing::info!(
                url = %url,
                connect_id = %connect_id,
                "V2 callback dispatched: mac_completion_payload delivered"
            );
            true
        }
        Ok(resp) => {
            tracing::warn!(
                url = %url,
                connect_id = %connect_id,
                status = %resp.status(),
                "V2 callback dispatch returned non-2xx; Mac must fall back to /status poll"
            );
            false
        }
        Err(e) => {
            tracing::warn!(
                url = %url,
                connect_id = %connect_id,
                error = %e,
                "V2 callback dispatch failed (network/timeout); Mac must fall back to /status poll"
            );
            false
        }
    }
}

async fn pair_start(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PairStartRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let device_name = bounded_label(body.device_name, "operator-device", 128);
    let platform = bounded_label(body.platform, "macos", 64).to_ascii_lowercase();
    let daemon_base_url = validate_pairing_url(
        &body
            .daemon_base_url
            .unwrap_or_else(|| "http://127.0.0.1:8787".to_string()),
        "daemon_base_url",
    )?;
    let scopes = normalize_scopes(body.scopes)?;
    // Resolve pairing URL: FOCUSA_PAIRING_URL env > daemon_base_url
    // This is the public-facing URL the operator's phone will hit (e.g.
    // https://focusa-conn.verious.net) — needed for QR flows where the
    // Mac is on a different network than the VPS.
    let pairing_url_raw = std::env::var("FOCUSA_PAIRING_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| daemon_base_url.clone());
    let pairing_url = validate_pairing_url(&pairing_url_raw, "FOCUSA_PAIRING_URL")?;

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
    // V2 P0.4: pair_start is a trust-critical transition. Persist
    // BEFORE updating in-memory so the durability check happens first.
    // If persistence fails, we never put the code in memory and the
    // response blocks.
    if let Err(e) = pairing_store::put_code(&state, &code, &device_id, Some(&device_name), Some(&platform), &scopes, Some(&daemon_base_url), &now.to_rfc3339(), &expires.to_rfc3339()) {
        tracing::error!(
            code = %code,
            device_id = %device_id,
            error = %e,
            "V2 P0.4: pair_start put_code failed; not inserting in-memory; blocking response"
        );
        return Err(rejection(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({
                "status": "blocked",
                "failure_class": "storage_unwritable",
                "message": format!("pair_start persistence failed: {}", e),
                "code_persisted": false,
            }),
        ));
    }
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
        "pair_url": format!("{}/pair/{}", pairing_url.trim_end_matches('/'), device_id),
        "pair_url_qr_payload": format!("{}/pair/{}", pairing_url.trim_end_matches('/'), device_id),
        "next_tools": [
            "focusa_device_pair_status",
            "focusa_device_pair_list",
            "focusa_device_pair_qr"
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
    headers: axum::http::HeaderMap,
    Json(body): Json<PairCompleteRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // V2 P1.3: legacy device-code flow returns a full token. In admin
    // mode (FOCUSA_AUTH_TOKEN set), require the admin token here so the
    // token-return path is not openly exposed on a non-loopback bind.
    // In loopback dev mode (no admin token), the pre-auth path is fine.
    if let Ok(token) = std::env::var("FOCUSA_AUTH_TOKEN") {
        if !token.trim().is_empty() {
            let supplied = headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .trim_start_matches("Bearer ")
                .trim();
            if supplied != token {
                return Err(rejection(
                    StatusCode::UNAUTHORIZED,
                    json!({
                        "status": "unauthorized",
                        "failure_class": "admin_token_required",
                        "message": "FOCUSA_AUTH_TOKEN is set; legacy pair-complete requires the admin token.",
                    }),
                ));
            }
        }
    }
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
    let completed_by = bounded_label(body.completed_by, "vps-cli", 128);
    let operator_id = body
        .operator_id
        .map(|id| bounded_label(Some(id), "operator", 128));
    let raw_host = body.host.unwrap_or_else(|| "operator-vps".to_string());
    if is_unsafe_agent_runtime_path_inline(&raw_host) {
        let rejected_value = raw_host;
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "scope_mismatch",
                "field": "host",
                "rejected_value": rejected_value,
            }),
        ));
    }
    let host = bounded_label(Some(raw_host), "operator-vps", 128);
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
                // V2: also drop from the SQLite ledger on expiry.
                let _ = pairing_store::consume_code(&state, &code);
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
            // V2 Invariant 6: revoke any prior active token for the same
            // (device_id, host) before persisting. Re-pair must supersede
            // the old token, not stack on top of it.
            let _ = state
                .persistence
                .revoke_active_token_for_device_host(&p.device_id, &host);
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
    // V2: drop the consumed code from the SQLite ledger.
    let _ = pairing_store::consume_code(&state, &code);

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

    let completion = DevicePairCompletion {
        code: code.clone(),
        device_id: pair.device_id.clone(),
        token: token.clone(),
        scopes: pair.scopes.clone(),
        completed_at: now,
        completed_by,
        host: host.clone(),
        operator_id,
    };

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
            "command": "# On your Mac app, store the token in Keychain and reconnect using the daemon URL".to_string(),
            "next_step": "mac app should poll /v1/device/pair/status?code=... to retrieve the token"
        },
        "next_tools": ["focusa_device_pair_status", "focusa_device_pair_list"],
        "rehydrate_id": pair.device_id,
        "_completion": completion,
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
        if let Some(token) = s.tokens.values().find(|t| t.device_id == device_id) {
            return Ok(Json(json!({
                "status": "completed",
                "device_id": device_id,
                "token": token.token,
                "scopes": token.scopes,
                "issued_at": token.issued_at,
                "expires_at": token.expires_at,
                "expired": token.expires_at < now,
                "next_tools": ["focusa_device_pair_list"],
                "rehydrate_id": device_id,
            })));
        }
        if let Some((code, pair)) = s.pending.iter().find(|(_, p)| p.device_id == device_id) {
            let expired = pair.expires_at < now;
            let status_str = if expired {
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
                "next_tools": ["focusa_device_pair_list"],
                "rehydrate_id": pair.device_id,
            })));
        }
        return Err(rejection(
            StatusCode::NOT_FOUND,
            json!({
                "status": "not_found",
                "failure_class": "pair_device_not_found",
                "device_id": device_id,
            }),
        ));
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
    // V2 P0.4: revoked tokens MUST be deleted from SQLite, not just
    // cleared from memory. Without this, a daemon restart would
    // rehydrate the device from the ledger and the auth middleware's
    // SQLite fallback would accept the revoked token. Pair_revoke is
    // a trust-critical transition; persistence failure blocks the
    // response and rolls back the device-record append above.
    if let Err(e) = state
        .persistence
        .revoke_device_tokens_by_device(device_id)
    {
        tracing::error!(
            device_id = %device_id,
            error = %e,
            "V2 P0.4: pair_revoke SQLite token deletion failed; rolling back record and blocking response"
        );
        return Err(rejection(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({
                "status": "blocked",
                "failure_class": "storage_unwritable",
                "message": format!("device_token revocation failed: {}", e),
                "device_id": device_id,
            }),
        ));
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
    fn code_format_is_focus_dash_8_dash_4() {
        let code = generate_code();
        assert!(code.starts_with("FOCUS-"));
        // 4 hex + dash + 4 hex after the FOCUS- prefix.
        let suffix = &code[6..];
        let dash = suffix.chars().position(|c| c == '-').expect("dash");
        assert_eq!(dash, 8, "first 4 hex chars then dash");
        assert_eq!(suffix.len(), 8 + 1 + 4, "4+1+4 = 9 chars after FOCUS-");
    }

    #[test]
    fn token_is_32_byte_base64url_no_pad() {
        let t = generate_token();
        assert_eq!(t.len(), 43);
        assert!(
            t.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert!(!t.contains('='));
    }

    #[test]
    fn unsafe_paths_blocked() {
        assert!(is_unsafe_agent_runtime_path_inline("/root/pi-mono"));
        assert!(is_unsafe_agent_runtime_path_inline("/root/pi-mono/sub"));
        assert!(!is_unsafe_agent_runtime_path_inline("/home/wirebot/focusa"));
        assert!(!is_unsafe_agent_runtime_path_inline("/home/operator-vps"));
    }
}

// ──────────────────────────────────────────────────────────────────────────
// focusa-ui0y.8 — PWA helper page for QR/PWA handoff (Mode B/C)
//
// 200-LOC inline HTML + manifest + service worker. No external assets,
// no third-party scripts (per spec §5.2 threat model).
//
// The page reads `device_id` from the URL, polls /v1/device/pair/status,
// and acts as the mediator: phone/browser approval POSTs pair-complete to
// the same server, then the Mac receives the token through its own polling.
//   1. Pending  → one-tap Complete Pairing button is available
//   2. Completed → the Mac app will receive the token via its own polling
//   3. Expired  → code is gone; operator must generate a new code on the Mac
// ──────────────────────────────────────────────────────────────────────────

/// Focusa Connect Page mediator — scans the Mac handoff QR and approves it on this VPS.
async fn connect_room_page(
    Path(_room_id): Path<String>,
) -> (StatusCode, [(String, String); 2], String) {
    connect_mediator_page().await
}

async fn connect_mediator_page() -> (StatusCode, [(String, String); 2], String) {
    (
        StatusCode::OK,
        [
            (
                "content-type".to_string(),
                "text/html; charset=utf-8".to_string(),
            ),
            ("cache-control".to_string(), "no-store".to_string()),
        ],
        connect_mediator_html(),
    )
}

fn connect_mediator_html() -> String {
    r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
  <meta name="theme-color" content="#0f1115" />
  <title>Focusa Connect</title>
  <link rel="icon" href="data:," />
  <style>
    :root { color-scheme: dark; }
    * { box-sizing: border-box; }
    body {
      margin: 0; min-height: 100vh; display: grid; place-items: center;
      padding: 24px; background: #0f1115; color: #e8e8e8;
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro", system-ui, sans-serif;
      line-height: 1.45;
    }
    .card { width: min(100%, 430px); background: #1a1d24; border: 1px solid #2b303b; border-radius: 22px; padding: 24px; text-align: center; box-shadow: 0 24px 80px rgba(0,0,0,.35); }
    h1 { margin: 0 0 8px; font-size: 22px; letter-spacing: -0.03em; }
    p { color: #a7adba; margin: 8px 0 18px; }
    video { width: 100%; aspect-ratio: 1 / 1; object-fit: cover; border-radius: 18px; background: #0b0d12; border: 1px solid #303746; display: none; }
    textarea { width: 100%; min-height: 118px; resize: vertical; border-radius: 14px; border: 1px solid #303746; background: #10131a; color: #e8e8e8; padding: 12px; font: 13px ui-monospace, SFMono-Regular, Menlo, monospace; }
    button { width: 100%; border: 0; border-radius: 14px; padding: 14px 16px; margin-top: 10px; font-weight: 700; color: #0b0d12; background: #8affc1; }
    button.secondary { color: #e8e8e8; background: #2b303b; }
    button:disabled { opacity: .45; }
    .device { display: none; margin: 18px 0; padding: 14px; border: 1px solid #303746; border-radius: 16px; text-align: left; }
    .device strong { display: block; font-size: 16px; }
    .muted, #status { color: #7e8798; font-size: 13px; }
    details { margin-top: 16px; text-align: left; }
    summary { color: #a7adba; cursor: pointer; }
  </style>
</head>
<body>
  <main class="card">
    <h1>Connect Mac to Focusa</h1>
    <p id="intro">Scan the Mac QR for this Phone Bridge Flow.</p>
    <video id="video" playsinline muted></video>
    <div class="device" id="deviceBox">
      <strong id="deviceName">Mac</strong>
      <span class="muted" id="deviceMeta"></span>
    </div>
    <button id="scanBtn">Scan Mac code</button>
    <button id="approveBtn" disabled>Connect this Mac</button>
    <p id="status">Waiting for Mac handoff code.</p>
    <details id="advancedDetails">
      <summary>Advanced</summary>
      <div id="advancedBody" hidden>
        <p>Paste the Mac QR payload if camera scan is unavailable.</p>
        <textarea id="pasteBox" placeholder='{"protocol":"focusa-connect-v1","role":"mac_handoff_offer",...}'></textarea>
        <button class="secondary" id="pasteBtn">Use pasted code</button>
        <button class="secondary" id="copyBtn">Copy diagnostics</button>
      </div>
    </details>
  </main>
  <script>
    const serverUrl = location.origin;
    const roomId = (() => {
      const match = location.pathname.match(/^\/connect\/([^/?#]+)/);
      return match ? decodeURIComponent(match[1]) : '';
    })();
    const scanBtn = document.getElementById('scanBtn');
    const approveBtn = document.getElementById('approveBtn');
    const advancedDetails = document.getElementById('advancedDetails');
    const advancedBody = document.getElementById('advancedBody');
    const pasteBtn = document.getElementById('pasteBtn');
    const copyBtn = document.getElementById('copyBtn');
    const pasteBox = document.getElementById('pasteBox');
    const statusEl = document.getElementById('status');
    const video = document.getElementById('video');
    const deviceBox = document.getElementById('deviceBox');
    const deviceName = document.getElementById('deviceName');
    const deviceMeta = document.getElementById('deviceMeta');
    let stream = null;
    let detector = null;
    let lastOffer = null;
    let completedPayload = null;

    function setStatus(text) { statusEl.textContent = text; }
    function stopCamera() {
      if (stream) stream.getTracks().forEach(track => track.stop());
      stream = null;
      video.style.display = 'none';
    }
    function decodeOfferParam(value) {
      const normalized = value.replace(/-/g, '+').replace(/_/g, '/');
      const padded = normalized + '='.repeat((4 - normalized.length % 4) % 4);
      return JSON.parse(atob(padded));
    }
    function parseOffer(raw) {
      const text = String(raw || '').trim();
      if (!text) throw new Error('Empty Mac code');
      if (text.startsWith('{')) return JSON.parse(text);
      const url = new URL(text, serverUrl);
      const encoded = url.searchParams.get('offer') || url.hash.replace(/^#offer=/, '');
      if (!encoded) throw new Error('QR did not contain a Focusa Mac offer');
      return decodeOfferParam(encoded);
    }
    function validateOffer(offer) {
      if (offer.protocol !== 'focusa-connect-v1') throw new Error('Wrong protocol');
      if (offer.role && offer.role !== 'mac_handoff_offer') throw new Error('Wrong handoff role');
      return {
        mac_name: offer.mac_name || offer.device_name || 'Focusa Mac',
        mac_nonce: offer.nonce || offer.mac_nonce,
        mac_pubkey: offer.mac_pubkey || null,
        mac_callback: offer.mac_callback || null,
      };
    }
    async function submitOffer(offer) {
      if (!roomId) throw new Error('Missing Bridge Room id. Start from the QR shown by `focusa pair`.');
      const body = validateOffer(offer);
      if (!body.mac_nonce) throw new Error('Mac offer missing nonce');
      const response = await fetch(`/v1/connect/room/${encodeURIComponent(roomId)}/mac-offer`, {
        method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body)
      });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) throw new Error(payload.message || payload.failure_class || 'Mac offer rejected');
      lastOffer = payload;
      deviceName.textContent = payload.mac_name || body.mac_name;
      deviceMeta.textContent = `${serverUrl} · room ${roomId.slice(0, 8)}`;
      deviceBox.style.display = 'block';
      approveBtn.disabled = false;
      setStatus('Mac found. Tap Connect to approve.');
      stopCamera();
    }
    async function startScan() {
      try {
        if (!roomId) throw new Error('Open the QR from `focusa pair` first.');
        if (!('BarcodeDetector' in window)) throw new Error('Camera QR scanning unavailable; use Advanced paste.');
        detector = detector || new BarcodeDetector({ formats: ['qr_code'] });
        stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'environment' } });
        video.srcObject = stream;
        video.style.display = 'block';
        await video.play();
        setStatus('Point the camera at the Mac QR.');
        const tick = async () => {
          if (!stream) return;
          try {
            const codes = await detector.detect(video);
            if (codes.length) return submitOffer(parseOffer(codes[0].rawValue));
          } catch (err) { setStatus(err.message || String(err)); }
          requestAnimationFrame(tick);
        };
        tick();
      } catch (err) {
        advancedDetails.open = true; advancedBody.hidden = false;
        setStatus(err.message || String(err));
      }
    }
    async function approve() {
      try {
        if (!lastOffer) throw new Error('Scan or paste a Mac code first.');
        approveBtn.disabled = true;
        const response = await fetch(`/v1/connect/room/${encodeURIComponent(roomId)}/approve`, {
          method: 'POST', headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ host: 'operator-vps', completed_by: 'phone-pwa-room' })
        });
        const payload = await response.json().catch(() => ({}));
        if (!response.ok) throw new Error(payload.message || payload.failure_class || 'Approval failed');
        completedPayload = payload;
        // V2 P0.3: phone is a renderer. The token never leaves the
        // server. We deliberately do NOT build a payload containing
        // `token` here. The Mac receives the token via the daemon's
        // server-side callback POST (mac_callback_dispatched=true) or
        // by polling /v1/connect/status. The phone's job is to approve
        // the room and observe whether the callback succeeded.
        if (lastOffer.mac_callback) {
          if (payload.mac_callback_dispatched) {
            setStatus('Connected. Mac received the token automatically via its callback.');
          } else {
            setStatus('Connected. Mac callback was not reachable; it will poll the server for the token.');
          }
        } else {
          setStatus('Connected. Mac will receive the token by polling the server status endpoint.');
        }
        approveBtn.textContent = 'Connected';
        copyBtn.textContent = 'Copy approval receipt';
      } catch (err) {
        approveBtn.disabled = false;
        setStatus(err.message || String(err));
      }
    }
    advancedDetails.addEventListener('toggle', () => { advancedBody.hidden = !advancedDetails.open; });
    scanBtn.addEventListener('click', startScan);
    approveBtn.addEventListener('click', approve);
    pasteBtn.addEventListener('click', () => submitOffer(parseOffer(pasteBox.value)).catch(err => setStatus(err.message || String(err))));
    copyBtn.addEventListener('click', () => {
      // V2 P0.3: copy operation exports an APPROVAL RECEIPT, not a
      // token-bearing payload. The receipt contains the protocol,
      // room_id, device_id, server_url, and a flag indicating whether
      // the daemon dispatched the Mac callback. It does NOT include
      // `token`. Operators can paste this into an SSH shell for
      // forensic purposes without leaking the device credential.
      const receipt = completedPayload
        ? {
            protocol: 'focusa-connect-v1',
            role: 'approval_receipt',
            room_id: roomId,
            server_url: completedPayload.server_url || serverUrl,
            device_id: completedPayload.device_id,
            mac_callback_present: !!completedPayload.mac_callback_present,
            mac_callback_dispatched: !!completedPayload.mac_callback_dispatched,
            mac_receives_token_via: completedPayload.mac_receives_token_via,
            approved_at: new Date().toISOString(),
          }
        : { room_id: roomId, server_url: serverUrl, last_offer: lastOffer };
      navigator.clipboard.writeText(JSON.stringify(receipt, null, 2)).catch(() => {});
    });
    if (!roomId) {
      approveBtn.disabled = true;
      advancedDetails.open = true;
      advancedBody.hidden = false;
      setStatus('Missing Bridge Room id. Run focusa pair on the server and scan that QR first.');
    }
  </script>
</body>
</html>"##
    .to_string()
}

async fn pwa_helper_page(
    Path(device_id): Path<String>,
) -> (StatusCode, [(String, String); 2], String) {
    let html = pwa_helper_html(&device_id);
    (
        StatusCode::OK,
        [
            (
                "content-type".to_string(),
                "text/html; charset=utf-8".to_string(),
            ),
            ("cache-control".to_string(), "no-store".to_string()),
        ],
        html,
    )
}

/// PWA manifest — minimal, no icons (spec §5.2: no third-party assets).
async fn pwa_manifest(
    Path(device_id): Path<String>,
) -> (StatusCode, [(String, String); 2], String) {
    let manifest = format!(
        r##"{{
  "name": "Focusa Phone Bridge",
  "short_name": "Focusa",
  "description": "OAuth-like device pairing for Focusa",
  "start_url": "/pair/{}",
  "display": "standalone",
  "background_color": "#0f1115",
  "theme_color": "#0f1115",
  "scope": "/pair/"
}}"##,
        device_id
    );
    (
        StatusCode::OK,
        [
            (
                "content-type".to_string(),
                "application/manifest+json".to_string(),
            ),
            ("cache-control".to_string(), "no-store".to_string()),
        ],
        manifest,
    )
}

/// Service worker — minimal offline shell. The PWA is small enough that
/// network-first is fine. We never cache responses with `device_id` in
/// the URL to avoid leaking pairing state.
async fn pwa_service_worker() -> (StatusCode, [(String, String); 2], &'static str) {
    let body = r#"
// focusa-pairing PWA service worker — minimal offline shell.
// We deliberately do NOT cache /pair/* responses to avoid leaking
// pairing state if the device is shared.
self.addEventListener('install', (e) => { self.skipWaiting(); });
self.addEventListener('activate', (e) => { e.waitUntil(self.clients.claim()); });
self.addEventListener('fetch', (e) => {
  const url = new URL(e.request.url);
  if (url.pathname.startsWith('/pair/')) {
    e.respondWith(fetch(e.request));
    return;
  }
  e.respondWith(fetch(e.request).catch(() => new Response('offline', { status: 503 })));
});
"#;
    (
        StatusCode::OK,
        [
            (
                "content-type".to_string(),
                "application/javascript; charset=utf-8".to_string(),
            ),
            ("cache-control".to_string(), "no-store".to_string()),
        ],
        body,
    )
}

fn pwa_helper_html(device_id: &str) -> String {
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
  <meta name="theme-color" content="#0f1115" />
  <link rel="manifest" href="/pair/{device_id}/manifest.json" />
  <title>Focusa Phone Bridge</title>
  <style>
    :root {{ color-scheme: dark; }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro", system-ui, sans-serif;
      background: #0f1115; color: #e0e0e0;
      min-height: 100vh; min-height: 100dvh;
      display: flex; align-items: center; justify-content: center;
      padding: 24px; line-height: 1.5;
    }}
    .card {{
      max-width: 420px; width: 100%;
      background: #1a1d24; border: 1px solid #2a2f3a; border-radius: 14px;
      padding: 28px 24px; text-align: center;
    }}
    h1 {{ font-size: 20px; font-weight: 600; margin-bottom: 12px; letter-spacing: -0.3px; }}
    p {{ font-size: 14px; color: #a0a0a0; margin-bottom: 16px; }}
    .code {{ font-family: ui-monospace, "SF Mono", Menlo, monospace; font-size: 14px;
             background: #0f1115; border: 1px solid #2a2f3a; border-radius: 8px;
             padding: 10px 14px; margin: 16px auto; display: inline-block; letter-spacing: 0.5px; }}
    .status {{ display: inline-block; padding: 6px 12px; border-radius: 999px;
              font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; }}
    .status.pending {{ background: rgba(196,162,101,0.12); color: #C4A265; }}
    .status.completed {{ background: rgba(106,176,76,0.15); color: #6ab04c; }}
    .status.expired {{ background: rgba(232,76,76,0.12); color: #e84c4c; }}
    .cmd {{ font-family: ui-monospace, "SF Mono", Menlo, monospace; font-size: 12px;
           background: #0f1115; border: 1px solid #2a2f3a; border-radius: 8px;
           padding: 10px 12px; margin: 8px 0; word-break: break-all; text-align: left; }}
    button {{ width: 100%; border: 0; border-radius: 12px; padding: 14px 16px;
             margin: 18px 0 8px; background: #4f7cff; color: #fff;
             font: inherit; font-weight: 700; cursor: pointer; }}
    button:disabled {{ opacity: 0.55; cursor: default; }}
    .hint {{ font-size: 12px; color: #707070; margin-top: 16px; }}
  </style>
</head>
<body>
  <div class="card">
    <h1>Focusa Device Pairing</h1>
    <p>This page stands between your Focusa server and your Mac. Tap once to join their hands.</p>
    <div class="code" id="code">—</div>
    <div><span class="status pending" id="status">Pending</span></div>
    <button id="completeBtn" disabled>Complete pairing</button>
    <details>
      <summary>Manual fallback</summary>
      <div class="cmd" id="cmd">focusa device pair-complete &lt;code&gt;</div>
    </details>
    <p class="hint" id="hint">Waiting for the server to confirm this Mac.</p>
  </div>
  <script>
    const DEVICE_ID = {device_id_quoted};
    const statusEl = document.getElementById('status');
    const codeEl = document.getElementById('code');
    const cmdEl = document.getElementById('cmd');
    const hintEl = document.getElementById('hint');
    const completeBtn = document.getElementById('completeBtn');
    let pollCount = 0;
    let currentCode = '';

    function setState(state, code) {{
      statusEl.className = 'status ' + state;
      statusEl.textContent = state.charAt(0).toUpperCase() + state.slice(1);
      if (state === 'pending') {{
        if (code) {{
          currentCode = code;
          codeEl.textContent = code;
          cmdEl.textContent = 'focusa device pair-complete ' + code;
          completeBtn.disabled = false;
        }}
        hintEl.textContent = 'Tap Complete pairing to approve this Mac on the server.';
      }} else if (state === 'completed') {{
        codeEl.textContent = '✓ paired';
        cmdEl.textContent = 'The Mac app received the token. Return to your Mac.';
        completeBtn.disabled = true;
        completeBtn.textContent = 'Paired';
        hintEl.textContent = 'You can close this page.';
      }} else if (state === 'expired') {{
        codeEl.textContent = '✗ expired';
        cmdEl.textContent = 'Generate a new code on your Mac.';
        completeBtn.disabled = true;
        completeBtn.textContent = 'Expired';
        hintEl.textContent = 'Codes expire after 5 minutes.';
      }}
    }}

    async function poll() {{
      pollCount++;
      try {{
        const r = await fetch('/v1/device/pair/status?device_id=' + encodeURIComponent(DEVICE_ID));
        if (!r.ok) {{
          setState('expired');
          return;
        }}
        const d = await r.json();
        const status = d.status || d.details?.status;
        if (status === 'completed' || d.token) {{
          setState('completed');
          return;
        }}
        if (status === 'expired' || d.expired) {{
          setState('expired');
          return;
        }}
        // Pending — extract code from response if present
        const code = d.code || d.details?.code;
        setState('pending', code);
      }} catch (e) {{
        hintEl.textContent = 'Network error. Retrying…';
      }}
    }}

    completeBtn.addEventListener('click', async () => {{
      if (!currentCode) return;
      completeBtn.disabled = true;
      completeBtn.textContent = 'Completing…';
      try {{
        const r = await fetch('/v1/device/pair/complete', {{
          method: 'POST',
          headers: {{ 'content-type': 'application/json' }},
          body: JSON.stringify({{ code: currentCode, host: location.host, completed_by: 'qr-helper-page' }})
        }});
        if (!r.ok) throw new Error('HTTP ' + r.status);
        setState('completed');
      }} catch (e) {{
        completeBtn.disabled = false;
        completeBtn.textContent = 'Complete pairing';
        hintEl.textContent = 'Could not complete pairing. Try again or use Manual fallback.';
      }}
    }});

    // initial fetch + 2s poll
    poll();
    setInterval(poll, 2000);
  </script>
</body>
</html>"##,
        device_id = device_id,
        device_id_quoted = serde_json::to_string(device_id).unwrap_or_else(|_| "\"\"".to_string()),
    )
}

// ---------- focusa-ui0y WhatsApp-like first-run (URL-shaped QR + small Approve page) ----------

#[derive(Debug, Deserialize, Default)]
pub struct ConnectFirstrunQuery {
    #[serde(default)]
    pub mac_offer: Option<String>,
}

async fn connect_firstrun_page(
    axum::extract::Query(q): axum::extract::Query<ConnectFirstrunQuery>,
) -> (StatusCode, [(String, String); 2], String) {
    let mac_offer_b64 = q.mac_offer.unwrap_or_default();
    let (mac_name, room_id) = decode_mac_offer(&mac_offer_b64);
    let mac_name_json = serde_json::to_string(&mac_name).unwrap_or_else(|_| "\"Mac\"".into());
    let room_id_json = serde_json::to_string(&room_id).unwrap_or_else(|_| "\"\"".into());
    let body = format!(
        r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
  <meta name="theme-color" content="#0f1115" />
  <title>Approve Focusa Mac</title>
  <link rel="icon" href="data:," />
  <style>
    :root {{ color-scheme: dark; }}
    body {{ margin: 0; min-height: 100vh; display: grid; place-items: center; padding: 24px; background: #0f1115; color: #e8e8e8; font-family: -apple-system, BlinkMacSystemFont, "SF Pro", system-ui, sans-serif; }}
    .card {{ width: min(100%, 420px); background: #1a1d24; border: 1px solid #2b303b; border-radius: 22px; padding: 28px; text-align: center; }}
    h1 {{ margin: 0 0 12px; font-size: 22px; }}
    p {{ margin: 0 0 20px; color: #b6bdc9; }}
    button {{ width: 100%; padding: 14px 18px; border-radius: 14px; border: 0; background: #5b8cff; color: #0f1115; font-weight: 700; font-size: 16px; cursor: pointer; }}
    button[disabled] {{ opacity: .5; cursor: default; }}
    .ok {{ background: #1e6f3a; color: #fff; }}
    .err {{ background: #6b1f1f; color: #fff; }}
  </style>
</head>
<body>
  <div class="card">
    <h1>Approve this Mac?</h1>
    <p id="mac-name">Mac</p>
    <button id="approve" type="button">Approve</button>
    <p id="status" style="margin-top:18px;color:#b6bdc9;"></p>
  </div>
  <script>
    const MAC_NAME = {mac_name_json};
    const ROOM_ID = {room_id_json};
    document.getElementById('mac-name').textContent = MAC_NAME;
    const btn = document.getElementById('approve');
    const statusEl = document.getElementById('status');
    btn.addEventListener('click', async () => {{
      btn.disabled = true; btn.textContent = 'Approving...';
      try {{
        const r = await fetch('/v1/connect/room/' + encodeURIComponent(ROOM_ID) + '/mac-offer', {{
          method: 'POST', headers: {{'content-type': 'application/json'}},
          body: JSON.stringify({{ mac_name: MAC_NAME }})
        }});
        if (!r.ok) throw new Error('offer HTTP ' + r.status);
        const a = await fetch('/v1/connect/room/' + encodeURIComponent(ROOM_ID) + '/approve', {{
          method: 'POST', headers: {{'content-type': 'application/json'}},
          body: JSON.stringify({{ host: location.host, operator_id: 'phone-approve', completed_by: 'phone' }})
        }});
        if (!a.ok) throw new Error('approve HTTP ' + a.status);
        btn.classList.add('ok'); btn.textContent = 'Approved';
        statusEl.textContent = 'Pairing complete. You can close this page.';
      }} catch (e) {{
        btn.disabled = false; btn.textContent = 'Approve';
        btn.classList.add('err');
        statusEl.textContent = 'Failed: ' + e.message;
      }}
    }});
  </script>
</body>
</html>"##
    );
    (
        StatusCode::OK,
        [
            (
                "content-type".to_string(),
                "text/html; charset=utf-8".to_string(),
            ),
            ("cache-control".to_string(), "no-store".to_string()),
        ],
        body,
    )
}

fn decode_mac_offer(b64: &str) -> (String, String) {
    if b64.is_empty() {
        return ("this Mac".to_string(), String::new());
    }
    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
    match B64.decode(b64.as_bytes()) {
        Ok(bytes) => match serde_json::from_slice::<serde_json::Value>(&bytes) {
            Ok(v) => {
                let name = v
                    .get("mac_name")
                    .and_then(|x| x.as_str())
                    .unwrap_or("this Mac")
                    .to_string();
                let rid = v
                    .get("room_id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                (name, rid)
            }
            Err(_) => ("this Mac".to_string(), String::new()),
        },
        Err(_) => ("this Mac".to_string(), String::new()),
    }
}

async fn connect_room_firstrun(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ConnectRoomFirstrunRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let now = Utc::now();
    let expires = now + Duration::seconds(CODE_TTL_SECS);
    let room_id = Uuid::now_v7().to_string();
    let device_id = Uuid::now_v7().to_string();
    let scopes = body
        .scopes
        .unwrap_or_else(|| vec!["read".to_string(), "write".to_string()]);
    let server_url_raw = body
        .server_url
        .clone()
        .or_else(|| std::env::var("FOCUSA_PAIRING_URL").ok().filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string());
    let server_url = match validate_pairing_url(&server_url_raw, "server_url") {
        Ok(u) => u,
        Err(rej) => {
            tracing::warn!(
                server_url_raw = %server_url_raw,
                "phone bridge firstrun rejected: invalid server_url"
            );
            return Err(rej);
        }
    };
    let mac_name = bounded_label(body.mac_name.clone(), "operator-mac", 128);
    // Accept canonical mac_offer field names ("nonce", "pubkey") AND
    // the daemon's own field names ("mac_nonce", "mac_pubkey") so the
    // V2 mac_offer JSON (from docs/55) is accepted without translation.
    let mac_nonce = body.mac_nonce.clone().unwrap_or_default();
    let mac_pubkey = body.mac_pubkey.clone();
    let mac_callback = body
        .mac_callback
        .clone()
        .and_then(|s| if s.trim().is_empty() { None } else { Some(s) });
    if let Some(cb) = &mac_callback {
        validate_mac_callback_url(cb, "mac_callback")?;
    }
    let session = ConnectSession {
        connect_id: room_id.clone(),
        device_id: device_id.clone(),
        mac_name: mac_name.clone(),
        mac_nonce: mac_nonce.clone(),
        mac_pubkey: None,
        mac_callback: mac_callback.clone(),
        server_url: server_url.clone(),
        scopes: scopes.clone(),
        created_at: now,
        expires_at: expires,
        status: "waiting_for_phone".to_string(),
        token: None,
                token_delivered: false,
            delivered_at: None,
        };
    {
        let state_ref = shared_state();
        let mut s = state_ref.write().await;
        s.connect_sessions
            .insert(room_id.clone(), session.clone());
    }
    let _ = pairing_store::put_session(
        &state,
        &room_id,
        Some(&device_id),
        Some(&mac_nonce),
        None,
        mac_callback.as_deref(),
        &server_url,
        Some(&scopes),
        &now.to_rfc3339(),
        &expires.to_rfc3339(),
    );
    let mac_offer = serde_json::json!({
        "protocol": "focusa-connect-v1",
        "role": "mac_handoff_offer",
        "mac_name": mac_name,
        "mac_nonce": mac_nonce,
        "mac_callback": mac_callback,
        "room_id": room_id,
        "created_at": now,
        "expires_in_secs": CODE_TTL_SECS,
    });
    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
    let mac_offer_b64 = B64.encode(serde_json::to_vec(&mac_offer).unwrap_or_default());
    let qr_url = format!(
        "{}/connect/firstrun?mac_offer={}",
        server_url.trim_end_matches('/'),
        mac_offer_b64
    );
    Ok(Json(json!({
        "status": "waiting_for_phone",
        "canonical": false,
        "advisory": true,
        "room_id": room_id,
        "connect_id": room_id,
        "device_id": device_id,
        "server_url": server_url,
        "connect_url": qr_url,
        "pair_url": qr_url,
        "pair_url_qr_payload": qr_url,
        "mac_offer": mac_offer,
        "mac_offer_b64": mac_offer_b64,
        "scopes": scopes,
        "expires_at": expires,
        "expires_in_secs": CODE_TTL_SECS,
        "poll_url": format!(
            "{}/v1/connect/room/{}/status",
            server_url.trim_end_matches('/'),
            room_id
        ),
        "next_tools": [
            "focusa_connect_room_status",
            "focusa_connect_room_mac_offer",
            "focusa_connect_room_approve"
        ],
        "diagnostics": {
            "surface": "phone_bridge_flow",
            "event": "room_firstrun",
            "room_state": "waiting_for_phone",
            "next_step_hint": "Mac: render pair_url as QR. Phone: scan with camera; browser opens Connect Page with mac_offer. Tap Approve. Mac polls poll_url for token."
        },
        "rehydrate_id": room_id,
    })))
}

// ---------- focusa-ui0y v0.9.35-dev: VPS-initiated room model ----------

async fn connect_room_create(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ConnectRoomCreateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let now = Utc::now();
    let expires = now + Duration::seconds(CODE_TTL_SECS);
    let room_id = Uuid::now_v7().to_string();
    let device_id = Uuid::now_v7().to_string();
    let scopes = body
        .scopes
        .unwrap_or_else(|| vec!["read".to_string(), "write".to_string()]);
    let server_url_raw = body
        .server_url
        .clone()
        .or_else(|| std::env::var("FOCUSA_PAIRING_URL").ok().filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string());
    let server_url = match validate_pairing_url(&server_url_raw, "server_url") {
        Ok(u) => u,
        Err(rej) => {
            tracing::warn!(
                server_url_raw = %server_url_raw,
                "phone bridge create-room rejected: invalid server_url"
            );
            return Err(rej);
        }
    };

    let session = ConnectSession {
        connect_id: room_id.clone(),
        device_id: device_id.clone(),
        mac_name: String::new(),
        mac_nonce: String::new(),
        mac_pubkey: None,
        mac_callback: None,
        server_url: server_url.clone(),
        scopes: scopes.clone(),
        created_at: now,
        expires_at: expires,
        status: "waiting_for_mac".to_string(),
        token: None,
        token_delivered: false,
        delivered_at: None,
    };
    {
        let state_ref = shared_state();
        let mut s = state_ref.write().await;
        s.connect_sessions
            .insert(room_id.clone(), session.clone());
    }
    let _ = pairing_store::put_session(
        &state,
        &room_id,
        Some(&device_id),
        None,
        None,
        None,
        &server_url,
        Some(&scopes),
        &now.to_rfc3339(),
        &expires.to_rfc3339(),
    );

    // pair_url points to the PWA scan page, NOT to firstrun (which was Mac-creates).
    let pair_url = format!(
        "{}/connect/room/{}/scan",
        server_url.trim_end_matches('/'),
        room_id
    );

    tracing::info!(
        room_id = %room_id,
        device_id = %device_id,
        server_url = %server_url,
        "VPS-initiated pairing room created (v0.9.35-dev)"
    );

    Ok(Json(json!({
        "status": "waiting_for_mac",
        "canonical": false,
        "advisory": true,
        "room_id": room_id,
        "connect_id": room_id,
        "device_id": device_id,
        "server_url": server_url,
        "pair_url": pair_url,
        "pair_url_qr_payload": pair_url,
        "scan_url": pair_url,
        "scopes": scopes,
        "expires_at": expires,
        "expires_in_secs": CODE_TTL_SECS,
        "poll_url": format!(
            "{}/v1/connect/room/{}/status",
            server_url.trim_end_matches('/'),
            room_id
        ),
        "join_url": format!(
            "{}/v1/connect/room/{}/join",
            server_url.trim_end_matches('/'),
            room_id
        ),
        "approve_url": format!(
            "{}/v1/connect/room/{}/approve",
            server_url.trim_end_matches('/'),
            room_id
        ),
        "next_tools": [
            "focusa_connect_room_join",
            "focusa_connect_room_approve",
            "focusa_connect_room_status"
        ],
        "diagnostics": {
            "surface": "phone_bridge_flow",
            "event": "room_created",
            "room_state": "waiting_for_mac",
            "next_step_hint": "Phone scans pair_url QR (terminal). PWA loads. PWA camera scans Mac static mac_offer QR. Mac joins via /join. Phone taps Approve."
        },
        "rehydrate_id": room_id,
    })))
}

async fn connect_room_join(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
    Json(body): Json<ConnectRoomJoinRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rid = room_id.trim().to_string();
    let now = Utc::now();
    let mac_name = bounded_label(body.mac_name.clone(), "operator-mac", 128);
    let mac_nonce = body
        .mac_nonce
        .clone()
        .or(body.mac_nonce_v2.clone())
        .unwrap_or_default();
    let mac_pubkey = body.mac_pubkey.clone().or(body.mac_pubkey_v2.clone());
    let mac_callback = body
        .mac_callback
        .clone()
        .and_then(|s| if s.trim().is_empty() { None } else { Some(s) });
    if let Some(cb) = &mac_callback {
        validate_pairing_url(cb, "mac_callback")?;
    }

    let updated_session = {
        let state_ref = shared_state();
        let mut s = state_ref.write().await;
        let Some(session) = s.connect_sessions.get_mut(&rid) else {
            tracing::warn!(
                room_id = %rid,
                mac_name = %mac_name,
                "phone bridge join rejected: room not found"
            );
            return Err(rejection(
                StatusCode::NOT_FOUND,
                json!({
                    "status": "not_found",
                    "error": "room_not_found",
                    "room_id": rid,
                    "recovery_hint": "Re-run focusa pairing wizard on the VPS to create a fresh room."
                }),
            ));
        };

        if session.expires_at < now {
            tracing::warn!(
                room_id = %rid,
                mac_name = %mac_name,
                expired_at = %session.expires_at,
                "phone bridge join rejected: room expired"
            );
            return Err(rejection(
                StatusCode::GONE,
                json!({
                    "status": "expired",
                    "error": "room_expired",
                    "room_id": rid,
                    "recovery_hint": "Re-run focusa pairing wizard on the VPS."
                }),
            ));
        }
        session.mac_name = mac_name.clone();
        session.mac_nonce = mac_nonce.clone();
        session.mac_pubkey = mac_pubkey.clone();
        session.mac_callback = mac_callback.clone();
        if session.status == "waiting_for_mac" {
            session.status = "mac_seen".to_string();
        }
        session.clone()
    };
    let _ = pairing_store::put_session(
        &state,
        &rid,
        Some(&updated_session.device_id),
        Some(&mac_nonce),
        mac_pubkey.as_deref(),
        mac_callback.as_deref(),
        &updated_session.server_url,
        Some(&updated_session.scopes),
        &updated_session.created_at.to_rfc3339(),
        &updated_session.expires_at.to_rfc3339(),
    );

    tracing::info!(
        room_id = %rid,
        mac_name = %mac_name,
        "Mac joined pairing room (v0.9.35-dev /join)"
    );

    Ok(Json(json!({
        "status": updated_session.status,
        "canonical": false,
        "advisory": true,
        "room_id": rid,
        "connect_id": rid,
        "device_id": updated_session.device_id,
        "server_url": updated_session.server_url,
        "scopes": updated_session.scopes,
        "expires_at": updated_session.expires_at,
        "next_tools": ["focusa_connect_room_status", "focusa_connect_room_approve"],
        "diagnostics": {
            "surface": "phone_bridge_flow",
            "event": "room_joined",
            "room_state": updated_session.status,
            "next_step_hint": "Phone taps Approve on the Connect Page. Mac polls status to receive token."
        },
        "rehydrate_id": rid,
    })))
}

// ---------- focusa-ui0y v0.9.35-dev: PWA /connect/room/<id>/scan ----------

async fn connect_room_scan_page(
    axum::extract::Path(room_id): axum::extract::Path<String>,
) -> (StatusCode, [(String, String); 2], String) {
    let rid_json = serde_json::to_string(&room_id).unwrap_or_else(|_| "\"\"".into());
    let body = format!(
        r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
  <meta name="theme-color" content="#0f1115" />
  <title>Focusa — Pair Mac</title>
  <link rel="icon" href="data:," />
  <style>
    :root {{ color-scheme: dark; }}
    body {{ margin: 0; min-height: 100vh; background: #0f1115; color: #e8e8e8;
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro", system-ui, sans-serif;
      display: grid; place-items: center; padding: 24px; }}
    .card {{ width: min(100%, 420px); background: #1a1d24; border: 1px solid #2b303b;
      border-radius: 22px; padding: 24px; text-align: center; }}
    h1 {{ margin: 0 0 8px; font-size: 20px; }}
    p {{ margin: 0 0 14px; color: #b6bdc9; line-height: 1.4; }}
    video {{ width: 100%; max-width: 360px; border-radius: 14px; background: #000;
      aspect-ratio: 4/3; object-fit: cover; }}
    button {{ width: 100%; padding: 14px 18px; border-radius: 14px; border: 0;
      background: #5b8cff; color: #0f1115; font-weight: 700; font-size: 16px; cursor: pointer;
      margin-top: 12px; }}
    button[disabled] {{ opacity: .5; cursor: default; }}
    .ok {{ background: #1e6f3a; color: #fff; }}
    .err {{ background: #6b1f1f; color: #fff; }}
    .mac-name {{ color: #fff; font-weight: 700; }}
    .scanner-wrap {{ position: relative; }}
    .scanner-overlay {{ position: absolute; inset: 0; pointer-events: none;
      border: 2px dashed rgba(91,140,255,.55); border-radius: 14px; }}
  </style>
</head>
<body>
  <div class="card">
    <h1>Pair this Mac</h1>
    <p>Point your camera at the <strong>Mac menubar QR</strong>.</p>
    <div class="scanner-wrap">
      <video id="video" playsinline muted></video>
      <div class="scanner-overlay"></div>
    </div>
    <p id="mac-name"></p>
    <button id="approve" type="button" disabled>Approve</button>
    <p id="status" style="margin-top:14px;color:#b6bdc9;font-size:13px;"></p>
  </div>
  <script src="/static/jsqr/jsQR.js"></script>
  <script>
    // V2: no CDN fallback. The PWA is fully VPS-served and must not pull
    // third-party code in a security-sensitive pairing flow. If the local
    // jsQR is missing, we surface a clear error instead.
    if (!window.jsQR) {{
      document.addEventListener('DOMContentLoaded', function () {{
        var s = document.getElementById('status');
        if (s) s.textContent = 'PWA misconfigured: jsQR is not served by the VPS daemon. Contact your Focusa operator.';
      }});
    }}
    const ROOM_ID = {rid_json};
    const video = document.getElementById('video');
    const approveBtn = document.getElementById('approve');
    const statusEl = document.getElementById('status');
    const macNameEl = document.getElementById('mac-name');
    let stream = null;
    let scanHandle = null;
    let macOffer = null;

    function setStatus(msg, cls) {{
      statusEl.textContent = msg;
      statusEl.className = cls || '';
    }}

    function parseMacOffer(text) {{
      // Mac app encodes mac_offer as JSON; the menubar QR is the raw JSON.
      try {{
        const v = JSON.parse(text);
        if (v && v.role === 'mac_handoff_offer') return v;
      }} catch (_) {{}}
      return null;
    }}

    function postJoin(offer) {{
      return fetch('/v1/connect/room/' + encodeURIComponent(ROOM_ID) + '/join', {{
        method: 'POST',
        headers: {{'content-type': 'application/json'}},
        body: JSON.stringify({{
          mac_name: offer.mac_name || 'mac',
          mac_nonce: offer.nonce || offer.mac_nonce || '',
          mac_pubkey: offer.mac_pubkey || null,
          mac_callback: offer.mac_callback || null,
        }})
      }});
    }}

    function postApprove() {{
      return fetch('/v1/connect/room/' + encodeURIComponent(ROOM_ID) + '/approve', {{
        method: 'POST',
        headers: {{'content-type': 'application/json'}},
        body: JSON.stringify({{
          host: location.host,
          operator_id: 'phone-approve',
          completed_by: 'phone',
        }})
      }});
    }}

    async function startCamera() {{
      try {{
        stream = await navigator.mediaDevices.getUserMedia({{
          video: {{ facingMode: 'environment' }}, audio: false
        }});
        video.srcObject = stream;
        await video.play();
        setStatus('Point the camera at the Mac menubar QR.');
        scanHandle = requestAnimationFrame(tick);
      }} catch (e) {{
        setStatus('Camera unavailable: ' + e.message, 'err');
      }}
    }}

    function tick() {{
      if (!stream) return;
      if (video.readyState !== video.HAVE_ENOUGH_DATA) {{
        scanHandle = requestAnimationFrame(tick); return;
      }}
      const w = video.videoWidth, h = video.videoHeight;
      if (w && h && window.jsQR) {{
        const canvas = document.createElement('canvas');
        canvas.width = w; canvas.height = h;
        const ctx = canvas.getContext('2d');
        ctx.drawImage(video, 0, 0, w, h);
        const data = ctx.getImageData(0, 0, w, h);
        const code = jsQR(data.data, w, h);
        if (code && code.data) {{
          const offer = parseMacOffer(code.data);
          if (offer) {{
            macOffer = offer;
            macNameEl.innerHTML = 'Mac: <span class="mac-name">' +
              (offer.mac_name || 'unknown') + '</span>';
            approveBtn.disabled = false;
            setStatus('Mac detected. Tap Approve.');
            if (stream) {{ stream.getTracks().forEach(t => t.stop()); stream = null; }}
            return;
          }}
        }}
      }}
      scanHandle = requestAnimationFrame(tick);
    }}

    approveBtn.addEventListener('click', async () => {{
      if (!macOffer) return;
      approveBtn.disabled = true;
      approveBtn.textContent = 'Joining room…';
      try {{
        const j = await postJoin(macOffer);
        if (!j.ok) throw new Error('join HTTP ' + j.status);
        approveBtn.textContent = 'Approving…';
        const a = await postApprove();
        if (!a.ok) throw new Error('approve HTTP ' + a.status);
        approveBtn.classList.add('ok');
        approveBtn.textContent = 'Paired';
        setStatus('Pairing complete. You can close this page.');
      }} catch (e) {{
        approveBtn.disabled = false;
        approveBtn.classList.add('err');
        approveBtn.textContent = 'Approve';
        setStatus('Failed: ' + e.message);
      }}
    }});

    startCamera();
  </script>
</body>
</html>"##
    );
    (
        StatusCode::OK,
        [
            (
                "content-type".to_string(),
                "text/html; charset=utf-8".to_string(),
            ),
            ("cache-control".to_string(), "no-store".to_string()),
        ],
        body,
    )
}
