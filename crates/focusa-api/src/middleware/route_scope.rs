//! Route-scope authorization middleware.
//!
//! Global bearer auth proves caller identity; this layer limits what an
//! authenticated token may do when `FOCUSA_AUTH_TOKEN` is configured.
//! Local-first no-token mode remains unrestricted on loopback deployments.
//!
//! # Spec104 MW-04: Route-family scope enforcement
//!
//! Every API route is classified into a route-family scope:
//! - `public:health` — health probe, no auth needed
//! - `public:pairing` — pre-auth pairing flows
//! - `state:read`/`state:write` — focus/state mutations
//! - `workpoint:read`/`workpoint:write` — workpoint lifecycle
//! - `trajectory:read`/`trajectory:write` — trajectory operations
//! - `metacog:read`/`metacog:write` — metacognition store
//! - `prediction:read`/`prediction:write` — predictions
//! - `work_loop:read`/`work_loop:control` — continuous work loop
//! - `telemetry:write` — telemetry traces
//! - `project:read` — project identity
//!
//! Host/project scope context (`ScopeContext`) is preserved end-to-end via
//! the `FromRequestParts` extractor in `crate::scope::ScopeContext`. The
//! typed scope flows through headers/query params and is captured into
//! the workpoint packet envelope.

use std::sync::Arc;

use crate::{
    middleware::{entitlement::EntitlementGateAccepted, principal::request_principal},
    routes::permissions::requested_scopes,
    server::AppState,
};
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use focusa_core::capability_authorization::{
    CapabilityContext, CapabilityEffect, CapabilityRisk, GroundedCapability, can,
};

/// V2 P2 #12: enumerate the pre-auth pairing routes so route_scope_layer
/// can explicitly bypass them, rather than relying on layer-ordering
/// (which is brittle and easy to drift if a future refactor changes the
/// order). The auth_layer still gates these routes when FOCUSA_AUTH_TOKEN
/// is set (admin-token mode).
///
/// Pre-auth pairing routes include the V2 self-host Bridge Room flow:
///   - /v1/connect/start, /v1/connect/status, /v1/connect/approve
///   - /v1/connect/rooms, /v1/connect/room/create,
///     /v1/connect/room/{id}/join, /v1/connect/room/{id}/scan
///   - /v1/device/pair/start, /v1/device/pair/complete,
///     /v1/device/pair/status, /v1/device/pair/qr
///   - /v1/device/pair/list, /v1/device/pair/revoke
///     (list and revoke are admin-token-only when FOCUSA_AUTH_TOKEN set)
///   - /v1/connect/room/join, /v1/connect/room/approve
fn is_preauth_pairing_route(method: &Method, path: &str) -> bool {
    if path.starts_with("/v1/connect/") {
        return true;
    }
    if path.starts_with("/v1/device/pair/") {
        // list and revoke are still admin-gated via auth_layer when token enabled.
        return !path.starts_with("/v1/device/pair/list")
            && !path.starts_with("/v1/device/pair/revoke");
    }
    if method == Method::GET && path.starts_with("/connect/room/") {
        // PWA /scan page is pre-auth (no Bearer header in phone browser).
        return true;
    }
    let _ = method; // method unused for the connect/device paths above
    false
}

fn route_scope(method: &Method, path: &str) -> &'static str {
    if method == Method::GET && path == "/v1/health" {
        return "public:health";
    }
    // V2 P2 #12: pre-auth pairing routes bypass scope check. They are
    // needed before a device token exists, so requiring any scope would
    // be a chicken-and-egg. Auth-layer still gates them when
    // FOCUSA_AUTH_TOKEN is configured (admin-token mode).
    if is_preauth_pairing_route(method, path) {
        return "public:pairing";
    }
    if path.starts_with("/v1/harnesses") || path.starts_with("/v1/providers") {
        return if method == Method::GET {
            "silent_sessions:read"
        } else {
            "silent_sessions:create"
        };
    }
    if path.starts_with("/v1/silent-sessions") {
        if path.contains("/config") {
            return "silent_sessions:config";
        }
        if method == Method::GET {
            return if path.contains("/forensics") || path.contains("/raw-output") {
                "silent_sessions:forensics"
            } else if path.ends_with("/events") || path.ends_with("/output") {
                "silent_sessions:stream"
            } else {
                "silent_sessions:read"
            };
        }
        if path.ends_with("/adopt") {
            return "silent_sessions:admin";
        }
        if path == "/v1/silent-sessions"
            || path == "/v1/silent-sessions/preflight"
            || path.ends_with("/start")
        {
            return "silent_sessions:create";
        }
        return "silent_sessions:control";
    }
    if path.starts_with("/v1/workpoint/") {
        return if method == Method::GET || path.ends_with("/resume") || path.ends_with("/status") {
            "workpoint:read"
        } else {
            "workpoint:write"
        };
    }
    if path.starts_with("/v1/trajectory/") {
        return match (method, path) {
            (&Method::GET, _) => "trajectory:read",
            (_, p) if p.ends_with("/view") || p.ends_with("/assess") || p.ends_with("/resume") => {
                "trajectory:read"
            }
            _ => "trajectory:write",
        };
    }
    if path.starts_with("/v1/metacognition/") || path.starts_with("/v1/metacog/") {
        return if method == Method::GET
            || path.contains("/retrieve")
            || path.contains("/recent")
            || path.contains("/doctor")
        {
            "metacog:read"
        } else {
            "metacog:write"
        };
    }
    if path.starts_with("/v1/predictions") {
        return if method == Method::GET || path.ends_with("/recent") || path.ends_with("/stats") {
            "prediction:read"
        } else {
            "prediction:write"
        };
    }
    if path.starts_with("/v1/work-loop/") || path.starts_with("/v1/work_loop/") {
        return if method == Method::GET || path.ends_with("/status") || path.ends_with("/writer") {
            "work_loop:read"
        } else {
            "work_loop:control"
        };
    }
    if path == "/v1/silent-sessions"
        || path.starts_with("/v1/silent-sessions/")
        || path == "/v1/silent_sessions"
        || path.starts_with("/v1/silent_sessions/")
        || path == "/v1/harnesses"
        || path.starts_with("/v1/harnesses/")
        || path == "/v1/providers"
        || path.starts_with("/v1/providers/")
    {
        if path.contains("/forensics") || path.contains("/raw-output") {
            return "silent_sessions:forensics";
        }
        if path.contains("/admin") || path.ends_with("/adopt") || path.ends_with("/force-kill") {
            return "silent_sessions:admin";
        }
        if path.contains("/events")
            || path.contains("/output")
            || path.ends_with("/follow")
            || path.ends_with("/stream")
        {
            return "silent_sessions:stream";
        }
        if path.contains("/config") {
            return "silent_sessions:config";
        }
        if method == Method::POST
            && (path.ends_with("/preflight")
                || path.ends_with("/start")
                || path == "/v1/silent-sessions"
                || path == "/v1/silent_sessions")
        {
            return "silent_sessions:create";
        }
        return if method == Method::GET {
            "silent_sessions:read"
        } else {
            "silent_sessions:control"
        };
    }
    if path.starts_with("/v1/focus")
        || path.starts_with("/v1/ascc")
        || path.starts_with("/v1/state")
    {
        return if method == Method::GET {
            "state:read"
        } else {
            "state:write"
        };
    }
    if path.starts_with("/v1/project/") {
        return "project:read";
    }
    if path.starts_with("/v1/telemetry/") {
        return if method == Method::GET {
            "telemetry:read"
        } else {
            "telemetry:write"
        };
    }
    if path.starts_with("/v1/events") {
        return "events:read";
    }
    if path.starts_with("/v1/attachments/") {
        return if method == Method::GET {
            "read:*"
        } else {
            "attachments:write"
        };
    }
    if path.starts_with("/v1/sync") || path.starts_with("/v1/tokens") {
        return "sync:admin";
    }
    if path.starts_with("/proxy") || path.starts_with("/v1/proxy") {
        return "proxy:invoke";
    }
    if path.starts_with("/v1/ontology")
        || path.starts_with("/v1/traverse")
        || path.starts_with("/v1/reflex")
    {
        return "ontology:read";
    }
    if path.starts_with("/v1/release") || path.starts_with("/v1/commands") {
        return "admin:service";
    }
    if method == Method::GET {
        "read:*"
    } else {
        "admin:service"
    }
}

pub async fn route_scope_layer(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let required = route_scope(req.method(), req.uri().path());
    if required.starts_with("public:") {
        return Ok(next.run(req).await);
    }

    let resolved = request_principal(req.headers())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let effect = capability_effect(req.method(), required);
    let risk = capability_risk(effect, required);
    let context = CapabilityContext {
        request_id: header(req.headers(), "x-request-id")
            .unwrap_or_else(|| format!("request:{}", uuid::Uuid::now_v7())),
        workstream_key: header(req.headers(), "x-focusa-workstream-key"),
        workset_id: header(req.headers(), "x-focusa-workset-id"),
        work_item_id: header(req.headers(), "x-focusa-work-item-id"),
        frame_id: header(req.headers(), "x-focusa-frame-id"),
        risk,
        // The entitlement layer is outside this layer. Reaching this point means
        // its exact route policy accepted or classified the request.
        entitlement_satisfied: req.extensions().get::<EntitlementGateAccepted>().is_some(),
        requested_scopes: requested_scopes(req.headers()),
    };
    let capability = GroundedCapability {
        name: format!("http:{}:{}", req.method(), req.uri().path()),
        required_scope: required.into(),
        effect,
        assignable: true,
    };
    let principal = resolved.canonical_capability_principal();
    let decision = can(&principal, &capability, &context);
    state
        .persistence
        .append_capability_authorization_audit(&decision)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !decision.allowed {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(req).await)
}

fn header(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn capability_effect(method: &Method, required: &str) -> CapabilityEffect {
    if required.ends_with(":admin") || required == "admin:service" || required == "sync:admin" {
        CapabilityEffect::Admin
    } else if required.ends_with(":control") || required == "proxy:invoke" {
        CapabilityEffect::Control
    } else if method == Method::GET || required.ends_with(":read") || required == "read:*" {
        CapabilityEffect::Read
    } else {
        CapabilityEffect::Write
    }
}

fn capability_risk(effect: CapabilityEffect, required: &str) -> CapabilityRisk {
    if effect == CapabilityEffect::Admin || required.ends_with(":forensics") {
        CapabilityRisk::High
    } else if matches!(effect, CapabilityEffect::Write | CapabilityEffect::Control) {
        CapabilityRisk::Medium
    } else {
        CapabilityRisk::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_routes_require_write_or_control_scopes() {
        assert_eq!(
            route_scope(&Method::POST, "/v1/focus/update"),
            "state:write"
        );
        assert_eq!(
            route_scope(&Method::POST, "/v1/workpoint/checkpoint"),
            "workpoint:write"
        );
        assert_eq!(
            route_scope(&Method::POST, "/v1/work-loop/control"),
            "work_loop:control"
        );
        assert_eq!(
            route_scope(&Method::POST, "/v1/telemetry/trace"),
            "telemetry:write"
        );
    }

    #[test]
    fn silent_session_routes_use_exact_spec133_scopes() {
        assert_eq!(
            route_scope(&Method::GET, "/v1/silent-sessions"),
            "silent_sessions:read"
        );
        assert_eq!(
            route_scope(&Method::GET, "/v1/silent-sessions/id/events"),
            "silent_sessions:stream"
        );
        assert_eq!(
            route_scope(&Method::POST, "/v1/silent-sessions/preflight"),
            "silent_sessions:create"
        );
        assert_eq!(
            route_scope(&Method::POST, "/v1/silent-sessions/id/input"),
            "silent_sessions:control"
        );
        assert_eq!(
            route_scope(&Method::POST, "/v1/silent-sessions/id/config/revisions"),
            "silent_sessions:config"
        );
        assert_eq!(
            route_scope(&Method::POST, "/v1/silent-sessions/id/adopt"),
            "silent_sessions:admin"
        );
        assert_eq!(
            route_scope(&Method::GET, "/v1/harnesses/pi/capabilities"),
            "silent_sessions:read"
        );
        assert_eq!(
            route_scope(&Method::POST, "/v1/harnesses/pi/preflight"),
            "silent_sessions:create"
        );
        assert_eq!(
            route_scope(&Method::GET, "/v1/providers/provider/models"),
            "silent_sessions:read"
        );
        assert_eq!(
            route_scope(&Method::POST, "/v1/providers/provider/models/preflight"),
            "silent_sessions:create"
        );
    }

    #[test]
    fn read_routes_remain_read_scoped() {
        assert_eq!(route_scope(&Method::GET, "/v1/health"), "public:health");
        assert_eq!(
            route_scope(&Method::GET, "/v1/workpoint/status"),
            "workpoint:read"
        );
        assert_eq!(
            route_scope(&Method::GET, "/v1/project/identity"),
            "project:read"
        );
        assert_eq!(
            route_scope(&Method::GET, "/v1/events/recent"),
            "events:read"
        );
    }

    #[test]
    fn silent_session_routes_use_exact_spec_133_scopes() {
        let cases = [
            (Method::GET, "/v1/silent-sessions", "silent_sessions:read"),
            (
                Method::GET,
                "/v1/silent-sessions/session-1",
                "silent_sessions:read",
            ),
            (
                Method::GET,
                "/v1/silent-sessions/session-1/events",
                "silent_sessions:stream",
            ),
            (
                Method::POST,
                "/v1/silent-sessions",
                "silent_sessions:create",
            ),
            (
                Method::POST,
                "/v1/silent-sessions/session-1/start",
                "silent_sessions:create",
            ),
            (
                Method::POST,
                "/v1/silent-sessions/session-1/input",
                "silent_sessions:control",
            ),
            (
                Method::PUT,
                "/v1/silent-sessions/session-1/config",
                "silent_sessions:config",
            ),
            (
                Method::POST,
                "/v1/silent-sessions/session-1/adopt",
                "silent_sessions:admin",
            ),
            (
                Method::GET,
                "/v1/silent-sessions/session-1/raw-output",
                "silent_sessions:forensics",
            ),
            (Method::GET, "/v1/harnesses", "silent_sessions:read"),
            (
                Method::POST,
                "/v1/harnesses/pi/preflight",
                "silent_sessions:create",
            ),
            (Method::GET, "/v1/providers", "silent_sessions:read"),
            (
                Method::POST,
                "/v1/providers/openai/models/preflight",
                "silent_sessions:create",
            ),
        ];
        for (method, path, expected) in cases {
            assert_eq!(route_scope(&method, path), expected, "{method} {path}");
        }
    }
}
