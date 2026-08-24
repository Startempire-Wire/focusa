//! Bearer token authentication middleware.
//!
//! Source: docs/25-26 (Capability Permissions), G1-12-api.md
//!
//! V2 auth model and operating modes:
//!
//! ## Mode A — loopback dev (no FOCUSA_AUTH_TOKEN set, daemon on
//! 127.0.0.1):
//!   - No Bearer required. `is_authorized` short-circuits to true.
//!   - Pre-auth pairing routes (Mac join, phone PWA approve, /status,
//!     /scan page) work without any setup.
//!   - Pairing mints a device token that subsequent calls use as Bearer.
//!   - Intended for local development and integration tests.
//!
//! ## Mode B — non-loopback self-host (daemon on Tailscale or
//! 0.0.0.0, FOCUSA_AUTH_TOKEN set):
//!   - All protected routes require Bearer <FOCUSA_AUTH_TOKEN> OR
//!     Bearer <paired-device-token>.
//!   - Pre-auth pairing routes are still reachable without auth so a
//!     Mac can join, but `/v1/device/pair/{list,revoke}` require the
//!     admin token to prevent post-pair enumeration.
//!   - On daemon startup, the bind loopback guard at main.rs:130
//!     refuses to bind non-loopback if FOCUSA_AUTH_TOKEN is unset.
//!
//! ## Mode C — public deployment (Tailscale HTTPS proxy in front):
//!   - Same as Mode B, plus the operator should treat FOCUSA_AUTH_TOKEN
//!     as a secret and rotate it on revocation.
//!   - Pre-auth pairing routes are attack surface; recommend Tailscale
//!     ACLs or per-IP rate limiting at the proxy layer.
//!
//! Token types:
//!   - `FOCUSA_AUTH_TOKEN` env: admin/service token (full access).
//!   - DeviceToken: per-device pairing token (issued at pair-completion,
//!     lives in-memory + SQLite ledger).
//!   - Bearer <device_token> from a paired Mac is accepted on protected
//!     routes when an admin token is also configured.
//!
//! Pre-auth routes (Mac join / phone PWA approve / room status / connect
//! pages) skip auth entirely in all modes. Scope enforcement for those
//! routes is handled by route_scope_layer via the `public:pairing` token.

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

const PRE_AUTH_PATHS: &[&str] = &[
    "/v1/health",
    "/v1/connect/room/", // /join, /approve, /status, /mac-offer
    "/v1/connect/rooms", // list rooms (Mac polls this)
    "/v1/connect/",      // /connect/start, /connect/status, /connect/approve
    "/connect/",         // /connect/room/<id>/scan, /connect/firstrun, /connect/mediator, etc.
    "/static/",
    "/pair/", // legacy /pair/<device_id>
];

fn is_pre_auth(path: &str) -> bool {
    if matches!(
        path,
        "/v1/device/pair/start" | "/v1/device/pair/status" | "/v1/device/pair/qr"
    ) {
        return true;
    }
    PRE_AUTH_PATHS
        .iter()
        .any(|p| path.starts_with(p) || path == p.trim_end_matches('/'))
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
    drop(s);

    // V2: Fall back to the SQLite device_tokens ledger. This is the
    // daemon-restart-survival path: a token minted by a previous daemon
    // instance is still valid in the new instance because it was
    // persisted at mint time. We re-hydrate the FULL record (including
    // the original scopes), not a hardcoded ["read","write"] placeholder,
    // so route-scope checks see the actual granted permissions.
    let req_state = crate::server::app_state_for_token_lookup();
    if let Some(state) = req_state {
        if let Ok(Some(stored)) = state.persistence.load_device_token_full(token) {
            let pairing_state = crate::routes::device_pairing::shared_state();
            let mut s = pairing_state.write().await;
            if !s.tokens.contains_key(token) {
                s.tokens.insert(
                    token.to_string(),
                    focusa_core::types::DeviceToken {
                        token: token.to_string(),
                        device_id: stored.device_id.clone(),
                        scopes: stored.scopes.clone(),
                        issued_at: stored.issued_at,
                        expires_at: stored.expires_at,
                        last_used_at: None,
                        issued_to: stored.issued_to.clone(),
                    },
                );
            }
            return true;
        }
    }

    false
}

/// Auth middleware — admin token or device pairing token.
pub async fn auth_layer(req: Request, next: Next) -> Result<Response, StatusCode> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_pair_bootstrap_is_public_but_completion_and_admin_are_not() {
        for path in [
            "/v1/device/pair/start",
            "/v1/device/pair/status",
            "/v1/device/pair/qr",
        ] {
            assert!(is_pre_auth(path), "{path}");
        }
        for path in [
            "/v1/device/pair/complete",
            "/v1/device/pair/list",
            "/v1/device/pair/revoke",
        ] {
            assert!(!is_pre_auth(path), "{path}");
        }
    }
}
