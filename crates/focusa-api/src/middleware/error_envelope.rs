use axum::Json;
use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use uuid::Uuid;

fn status_code_key(status: StatusCode) -> &'static str {
    match status {
        StatusCode::BAD_REQUEST => "bad_request",
        StatusCode::UNAUTHORIZED => "unauthorized",
        StatusCode::FORBIDDEN => "forbidden",
        StatusCode::NOT_FOUND => "not_found",
        StatusCode::METHOD_NOT_ALLOWED => "method_not_allowed",
        StatusCode::UNSUPPORTED_MEDIA_TYPE => "unsupported_media_type",
        StatusCode::UNPROCESSABLE_ENTITY => "validation_error",
        StatusCode::TOO_MANY_REQUESTS => "rate_limited",
        StatusCode::INTERNAL_SERVER_ERROR => "internal_error",
        StatusCode::BAD_GATEWAY => "bad_gateway",
        StatusCode::SERVICE_UNAVAILABLE => "service_unavailable",
        StatusCode::GATEWAY_TIMEOUT => "gateway_timeout",
        _ if status.is_server_error() => "server_error",
        _ if status.is_client_error() => "client_error",
        _ => "error",
    }
}

fn status_message(status: StatusCode) -> &'static str {
    match status {
        StatusCode::BAD_REQUEST => "Request body or query parameters are invalid",
        StatusCode::UNAUTHORIZED => "Authentication required or token invalid",
        StatusCode::FORBIDDEN => "Request is not permitted",
        StatusCode::NOT_FOUND => "Route or resource not found",
        StatusCode::METHOD_NOT_ALLOWED => "HTTP method not allowed for this route",
        StatusCode::UNSUPPORTED_MEDIA_TYPE => "Unsupported content type",
        StatusCode::UNPROCESSABLE_ENTITY => "Request schema validation failed",
        StatusCode::TOO_MANY_REQUESTS => "Too many requests",
        StatusCode::INTERNAL_SERVER_ERROR => "Internal server error",
        StatusCode::BAD_GATEWAY => "Bad gateway",
        StatusCode::SERVICE_UNAVAILABLE => "Service unavailable",
        StatusCode::GATEWAY_TIMEOUT => "Gateway timeout",
        _ if status.is_server_error() => "Server error",
        _ if status.is_client_error() => "Client error",
        _ => "Request failed",
    }
}

pub async fn error_envelope_layer(req: Request, next: Next) -> Response {
    let incoming_corr = req
        .headers()
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let correlation_id = incoming_corr.unwrap_or_else(|| Uuid::now_v7().to_string());

    let mut response = next.run(req).await;
    if let Ok(hv) = HeaderValue::from_str(&correlation_id) {
        response.headers_mut().insert("x-correlation-id", hv);
    }

    let status = response.status();
    if !(status.is_client_error() || status.is_server_error()) {
        return response;
    }

    let is_json = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_ascii_lowercase().starts_with("application/json"))
        .unwrap_or(false);

    if is_json {
        return response;
    }

    let envelope = json!({
        "code": status_code_key(status),
        "message": status_message(status),
        "details": {
            "http_status": status.as_u16(),
            "reason": status.canonical_reason().unwrap_or("unknown"),
        },
        "correlation_id": correlation_id,
    });

    let mut wrapped = (status, Json(envelope)).into_response();
    if let Ok(hv) = HeaderValue::from_str(&correlation_id) {
        wrapped.headers_mut().insert("x-correlation-id", hv);
    }
    wrapped
}
