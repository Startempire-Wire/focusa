//! Bearer token authentication middleware.
//!
//! Source: docs/25-26 (Capability Permissions), G1-12-api.md
//!
//! V2 auth model and operating modes:
//!
//! ## Mode A — loopback dev (no FOCUSA_AUTH_TOKEN set, daemon on
//! 127.0.0.1):
//!   - No Bearer required for the local-first fallback.
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
    PRE_AUTH_PATHS
        .iter()
        .any(|p| path.starts_with(p) || path == p.trim_end_matches('/'))
}

enum AuthenticatedGrants {
    Owner,
    Device(crate::routes::permissions::PermissionContext),
}

/// Resolve identity and original grants together; a boolean loses authority.
async fn authenticated_grants(auth_header: &str) -> Option<AuthenticatedGrants> {
    let token = match auth_header.strip_prefix("Bearer ") {
        Some(t) if !t.is_empty() => t,
        _ => return None,
    };

    // 1. Admin token check
    if let Ok(expected) = std::env::var("FOCUSA_AUTH_TOKEN") {
        if !expected.is_empty() && token == expected {
            return Some(AuthenticatedGrants::Owner);
        }
    }

    // 2. Device pairing token check (look up in shared pairing state)
    let pairing_state = crate::routes::device_pairing::shared_state();
    let s = pairing_state.read().await;
    if let Some(stored) = s.tokens.get(token) {
        return (stored.expires_at > chrono::Utc::now()).then(|| {
            AuthenticatedGrants::Device(
                crate::routes::permissions::PermissionContext::from_device_grants(&stored.scopes),
            )
        });
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
        match state.persistence.load_device_token_full(token) {
            Ok(Some(stored)) => {
                if stored.expires_at <= chrono::Utc::now() {
                    return None;
                }
                let grants = crate::routes::permissions::PermissionContext::from_device_grants(
                    &stored.scopes,
                );
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
                return Some(AuthenticatedGrants::Device(grants));
            }
            Ok(None) => {}
            Err(error) => tracing::warn!("device grant lookup rejected: {error}"),
        }
    }

    None
}

/// Auth middleware — admin token or device pairing token.
pub async fn auth_layer(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let path = req.uri().path();

    // Shutdown has a dedicated per-start bearer credential and exact process
    // identity check in its route. Do not widen that credential to other APIs.
    if path == "/v1/shutdown" {
        return Ok(next.run(req).await);
    }

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
    if let Some(grants) = authenticated_grants(auth_header).await {
        if let AuthenticatedGrants::Device(permissions) = grants {
            permissions.bind_device_request(req.headers_mut())?;
        }
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
    use axum::{
        Router,
        body::Body,
        routing::{get, post},
    };
    use chrono::{Duration, Utc};
    use tower::ServiceExt;

    #[tokio::test]
    async fn device_permissions_cannot_exceed_verified_grants() {
        const CHILD: &str = "FOCUSA_ISOLATED_PERMISSION_PROBE";
        const OWNER: &str = "synthetic-owner-permission-probe";
        if std::env::var_os(CHILD).is_none() {
            // Isolate environment and pairing globals from every other test.
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .env_clear()
                .env(CHILD, "1")
                .env("FOCUSA_AUTH_TOKEN", OWNER)
                .args([
                    "device_permissions_cannot_exceed_verified_grants",
                    "--nocapture",
                    "--test-threads=1",
                ])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "isolated permission probe failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        let pairing = crate::routes::device_pairing::shared_state();
        let now = Utc::now();
        for (token, scopes) in [
            ("synthetic-read-device", vec!["read"]),
            ("synthetic-write-device", vec!["read", "write"]),
            ("synthetic-expired-device", vec!["read"]),
        ] {
            pairing.write().await.tokens.insert(
                token.to_string(),
                focusa_core::types::DeviceToken {
                    device_id: token.to_string(),
                    token: token.to_string(),
                    scopes: scopes.into_iter().map(str::to_string).collect(),
                    issued_at: now,
                    expires_at: if token == "synthetic-expired-device" {
                        now - Duration::seconds(1)
                    } else {
                        now + Duration::hours(1)
                    },
                    last_used_at: None,
                    issued_to: "isolated-test".to_string(),
                },
            );
        }

        // Exercise the real middleware without a listener, application state,
        // entitlement provider, persistence, or mutating route handlers.
        let app = Router::new()
            .route(
                "/v1/state/permission-probe",
                get(|| async { StatusCode::NO_CONTENT }).post(|| async { StatusCode::NO_CONTENT }),
            )
            .route(
                "/v1/commands/permission-probe",
                post(|| async { StatusCode::NO_CONTENT }),
            )
            .layer(axum::middleware::from_fn(
                crate::middleware::route_scope::route_scope_layer,
            ))
            .layer(axum::middleware::from_fn(auth_layer));

        let cases = [
            (
                "read grant",
                "synthetic-read-device",
                "GET",
                "/v1/state/permission-probe",
                "state:read",
                StatusCode::NO_CONTENT,
            ),
            (
                "read cannot claim write",
                "synthetic-read-device",
                "POST",
                "/v1/state/permission-probe",
                "state:write",
                StatusCode::FORBIDDEN,
            ),
            (
                "read cannot claim admin",
                "synthetic-read-device",
                "POST",
                "/v1/commands/permission-probe",
                "admin:*",
                StatusCode::FORBIDDEN,
            ),
            (
                "write grant",
                "synthetic-write-device",
                "POST",
                "/v1/state/permission-probe",
                "state:write",
                StatusCode::NO_CONTENT,
            ),
            (
                "owner grant",
                OWNER,
                "POST",
                "/v1/commands/permission-probe",
                "admin:*",
                StatusCode::NO_CONTENT,
            ),
            (
                "unissued token",
                "synthetic-unissued-device",
                "GET",
                "/v1/state/permission-probe",
                "state:read",
                StatusCode::UNAUTHORIZED,
            ),
            (
                "expired token",
                "synthetic-expired-device",
                "GET",
                "/v1/state/permission-probe",
                "state:read",
                StatusCode::UNAUTHORIZED,
            ),
            (
                "mixed over-request",
                "synthetic-read-device",
                "GET",
                "/v1/state/permission-probe",
                "state:read,admin:*",
                StatusCode::FORBIDDEN,
            ),
            (
                "write is not admin",
                "synthetic-write-device",
                "POST",
                "/v1/commands/permission-probe",
                "admin:*",
                StatusCode::FORBIDDEN,
            ),
            (
                "empty request grants nothing",
                "synthetic-read-device",
                "GET",
                "/v1/state/permission-probe",
                "",
                StatusCode::FORBIDDEN,
            ),
        ];
        let mut outcomes = Vec::new();
        for (label, token, method, path, requested, expected) in cases {
            let request = Request::builder()
                .method(method)
                .uri(path)
                .header("authorization", format!("Bearer {token}"))
                .header("x-focusa-permissions", requested)
                .body(Body::empty())
                .unwrap();
            let actual = app.clone().oneshot(request).await.unwrap().status();
            outcomes.push((label, actual, expected));
        }
        for token in [
            "synthetic-read-device",
            "synthetic-write-device",
            "synthetic-expired-device",
        ] {
            pairing.write().await.tokens.remove(token);
        }
        assert!(
            outcomes
                .iter()
                .all(|(_, actual, expected)| actual == expected),
            "permission outcomes (case, actual, expected): {outcomes:?}"
        );
    }
}
