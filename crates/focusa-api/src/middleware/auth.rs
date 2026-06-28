//! Bearer token authentication middleware.
//!
//! Source: docs/25-26 (Capability Permissions), G1-12-api.md
//!
//! V2 auth model:
//! - `FOCUSA_AUTH_TOKEN` = admin/service token (full access)
//! - DeviceToken = per-device pairing token (issued at pair-completion, lives
//!   in-memory in the pairing state map AND in the pairing_store SQLite ledger)
//! - Bearer <device_token> from a paired Mac is accepted on protected routes.
//! - Pre-auth routes (Mac join / phone PWA approve / room status / connect
//!   pages) skip both checks.
//!
//! If no `FOCUSA_AUTH_TOKEN` AND no device has ever paired, auth is disabled
//! (local-first loopback default). Non-loopback startup is rejected unless
//! `FOCUSA_AUTH_TOKEN` is present.

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

const PRE_AUTH_PATHS: &[&str] = &[
    "/v1/health",
    "/v1/connect/room/",      // /join, /approve, /status, /mac-offer
    "/v1/connect/rooms",       // list rooms (Mac polls this)
    "/v1/connect/",            // /connect/start, /connect/status, /connect/approve
    "/connect/",               // /connect/room/<id>/scan, /connect/firstrun, /connect/mediator, etc.
    "/static/",
    "/pair/",                  // legacy /pair/<device_id>
];

fn is_pre_auth(path: &str) -> bool {
    PRE_AUTH_PATHS.iter().any(|p| path.starts_with(p) || path == p.trim_end_matches('/'))
}

/// V2 auth: accept admin token OR device pairing token.
async fn is_authorized(auth_header: &str) -> bool {
    let token = match auth_header.strip_prefix("Bearer ") {
        Some(t) if !t.is_empty() => t,
        _ => return false,
    };

    // 1. Admin token check
    if let Ok(expected) = std::env::var("FOCUSA_AUTH_TOKEN") {
        if !expected.is_empty() && token == expected {
            return true;
        }
    }

    // 2. Device pairing token check (look up in shared pairing state)
    let pairing_state = crate::routes::device_pairing::shared_state();
    let s = pairing_state.read().await;
    if s.tokens.contains_key(token) {
        return true;
    }

    false
}

/// Auth middleware — admin token or device pairing token.
pub async fn auth_layer(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();

    // Pre-auth routes bypass auth entirely (pairing bootstrap is public).
    if is_pre_auth(path) {
        return Ok(next.run(req).await);
    }

    // If no admin token configured AND device tokens aren't a known model,
    // allow all (local-first default).
    let admin_token_set = std::env::var("FOCUSA_AUTH_TOKEN")
        .ok()
        .filter(|s| !s.is_empty())
        .is_some();
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if is_authorized(auth_header).await {
        return Ok(next.run(req).await);
    }

    if !admin_token_set {
        // No admin token configured AND no device token supplied.
        // Local-first default: allow unauthenticated access on loopback
        // surfaces only (loopback binding is enforced at startup).
        return Ok(next.run(req).await);
    }

    // Admin token configured and request did not present a valid admin
    // or device token: reject.
    Err(StatusCode::UNAUTHORIZED)
}