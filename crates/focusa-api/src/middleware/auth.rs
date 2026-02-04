//! Bearer token authentication middleware.
//!
//! Source: docs/25-26 (Capability Permissions)
//!
//! If FOCUSA_AUTH_TOKEN is set, all API requests must include:
//!   Authorization: Bearer <token>
//!
//! If FOCUSA_AUTH_TOKEN is unset, auth is disabled (local-first default).

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

/// Auth middleware — checks Bearer token if FOCUSA_AUTH_TOKEN is set.
pub async fn auth_layer(req: Request, next: Next) -> Result<Response, StatusCode> {
    // Skip auth for health endpoint.
    if req.uri().path() == "/v1/health" {
        return Ok(next.run(req).await);
    }

    // If no token configured, allow all (local-first).
    let expected = match std::env::var("FOCUSA_AUTH_TOKEN") {
        Ok(token) if !token.is_empty() => token,
        _ => return Ok(next.run(req).await),
    };

    // Extract Bearer token from Authorization header.
    let auth_header = req.headers().get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if let Some(token) = auth_header.strip_prefix("Bearer ")
        && token == expected
    {
        return Ok(next.run(req).await);
    }

    Err(StatusCode::UNAUTHORIZED)
}
