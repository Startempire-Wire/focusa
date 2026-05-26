//! Route-scope authorization middleware.
//!
//! Global bearer auth proves caller identity; this layer limits what an
//! authenticated token may do when `FOCUSA_AUTH_TOKEN` is configured.
//! Local-first no-token mode remains unrestricted on loopback deployments.

use crate::routes::permissions::permission_context;
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

fn token_enabled() -> bool {
    std::env::var("FOCUSA_AUTH_TOKEN")
        .map(|token| !token.trim().is_empty())
        .unwrap_or(false)
}

fn route_scope(method: &Method, path: &str) -> &'static str {
    if method == Method::GET && path == "/v1/health" {
        return "public:health";
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
    if path.starts_with("/v1/focus") || path.starts_with("/v1/ascc") || path.starts_with("/v1/state") {
        return if method == Method::GET { "state:read" } else { "state:write" };
    }
    if path.starts_with("/v1/project/") {
        return "project:read";
    }
    if path.starts_with("/v1/telemetry/") {
        return if method == Method::GET { "telemetry:read" } else { "telemetry:write" };
    }
    if path.starts_with("/v1/events") {
        return "events:read";
    }
    if path.starts_with("/v1/attachments/") {
        return if method == Method::GET { "read:*" } else { "attachments:write" };
    }
    if path.starts_with("/v1/sync") || path.starts_with("/v1/tokens") {
        return "sync:admin";
    }
    if path.starts_with("/proxy") || path.starts_with("/v1/proxy") {
        return "proxy:invoke";
    }
    if path.starts_with("/v1/ontology") || path.starts_with("/v1/traverse") || path.starts_with("/v1/reflex") {
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

pub async fn route_scope_layer(req: Request, next: Next) -> Result<Response, StatusCode> {
    if !token_enabled() {
        return Ok(next.run(req).await);
    }
    let required = route_scope(req.method(), req.uri().path());
    if required == "public:health" {
        return Ok(next.run(req).await);
    }
    let permissions = permission_context(req.headers(), true);
    if permissions.allows(required) {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_routes_require_write_or_control_scopes() {
        assert_eq!(route_scope(&Method::POST, "/v1/focus/update"), "state:write");
        assert_eq!(route_scope(&Method::POST, "/v1/workpoint/checkpoint"), "workpoint:write");
        assert_eq!(route_scope(&Method::POST, "/v1/work-loop/control"), "work_loop:control");
        assert_eq!(route_scope(&Method::POST, "/v1/telemetry/trace"), "telemetry:write");
    }

    #[test]
    fn read_routes_remain_read_scoped() {
        assert_eq!(route_scope(&Method::GET, "/v1/health"), "public:health");
        assert_eq!(route_scope(&Method::GET, "/v1/workpoint/status"), "workpoint:read");
        assert_eq!(route_scope(&Method::GET, "/v1/project/identity"), "project:read");
        assert_eq!(route_scope(&Method::GET, "/v1/events/recent"), "events:read");
    }
}
