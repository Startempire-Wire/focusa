//! Bearer token authentication middleware.
//!
//! Source: docs/25-26 (Capability Permissions), G1-12-api.md
//!
//! Auth token enforcement currently uses `FOCUSA_AUTH_TOKEN` only.
//!
//! If no env token is configured, auth is disabled (local-first loopback default).
//! Non-loopback startup is rejected unless `FOCUSA_AUTH_TOKEN` is present.

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

/// Auth middleware — checks Bearer token if configured.
///
/// Checks FOCUSA_AUTH_TOKEN env var.
pub async fn auth_layer(req: Request, next: Next) -> Result<Response, StatusCode> {
    // Skip auth for health endpoint.
    if req.uri().path() == "/v1/health" {
        return Ok(next.run(req).await);
    }

    // Check for enforced auth token.
    let expected = if let Ok(token) = std::env::var("FOCUSA_AUTH_TOKEN") {
        if !token.is_empty() { Some(token) } else { None }
    } else {
        None
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
