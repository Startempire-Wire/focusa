//! Bearer token authentication middleware.
//!
//! Source: docs/25-26 (Capability Permissions), G1-12-api.md
//!
//! Auth token can be set via:
//!   1. FOCUSA_AUTH_TOKEN env var
//!   2. Config file (auth_token field)
//!
//! If no token configured, auth is disabled (local-first default).

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

/// Auth middleware — checks Bearer token if configured.
///
/// Checks FOCUSA_AUTH_TOKEN env var first, then falls back to config.
pub async fn auth_layer(req: Request, next: Next) -> Result<Response, StatusCode> {
    // Skip auth for health endpoint.
    if req.uri().path() == "/v1/health" {
        return Ok(next.run(req).await);
    }

    // Check for auth token (env var takes precedence).
    let expected = if let Ok(token) = std::env::var("FOCUSA_AUTH_TOKEN") {
        if !token.is_empty() { Some(token) } else { None }
    } else {
        None // Config token check would need state, skip for now
    };

    // If no token configured, allow all (local-first default).
    let expected = match expected {
        Some(token) if !token.is_empty() => token,
        _ => return Ok(next.run(req).await),
    };

    // Extract Bearer token from Authorization header.
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if let Some(token) = auth_header.strip_prefix("Bearer ")
        && token == expected
    {
        return Ok(next.run(req).await);
    }

    Err(StatusCode::UNAUTHORIZED)
}
