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

fn recovery_command(status: StatusCode) -> &'static str {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            "focusa doctor && check FOCUSA_AUTH_TOKEN/scopes"
        }
        StatusCode::NOT_FOUND => "focusa doctor && focusa docs status",
        StatusCode::SERVICE_UNAVAILABLE | StatusCode::BAD_GATEWAY | StatusCode::GATEWAY_TIMEOUT => {
            "systemctl status focusa-daemon --no-pager && journalctl -u focusa-daemon -n 80 --no-pager"
        }
        _ if status.is_server_error() => {
            "focusa doctor && journalctl -u focusa-daemon -n 80 --no-pager"
        }
        _ => "check request body/route, then retry with --json for details",
    }
}

fn severity(status: StatusCode) -> &'static str {
    if status.is_server_error() {
        "blocked"
    } else if status == StatusCode::NOT_FOUND {
        "watch"
    } else {
        "degraded"
    }
}

fn failure_class(status: StatusCode) -> &'static str {
    match status {
        StatusCode::NOT_FOUND => "not_found",
        StatusCode::METHOD_NOT_ALLOWED => "validation_rejected",
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => "permission_denied",
        StatusCode::SERVICE_UNAVAILABLE | StatusCode::BAD_GATEWAY | StatusCode::GATEWAY_TIMEOUT => "daemon_unavailable",
        StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => "validation_rejected",
        _ if status.is_server_error() => "daemon_unavailable",
        _ => "unknown_ambiguous_completion",
    }
}

fn recovery_hint(status: StatusCode) -> &'static str {
    match status {
        StatusCode::NOT_FOUND => "Check the route path against docs/current/API_REFERENCE_CURRENT.md; for model recovery run focusa_tool_doctor, then use the nearest project/trajectory/workpoint route.",
        StatusCode::METHOD_NOT_ALLOWED => "Use the documented HTTP method for this route; if uncertain run focusa_tool_doctor or inspect API_REFERENCE_CURRENT.md.",
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => "Verify Focusa auth token/scopes before retrying; do not retry the same credentials unchanged.",
        StatusCode::SERVICE_UNAVAILABLE | StatusCode::BAD_GATEWAY | StatusCode::GATEWAY_TIMEOUT => "Check daemon health/resource mode; continue from operator/repo context until /v1/health is ok.",
        StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => "Fix request body/query shape from the docs; do not retry unchanged.",
        _ if status.is_server_error() => "Run focusa_tool_doctor and inspect daemon logs before retrying.",
        _ => "Inspect status/code/details, then choose the safe next route from next_tools.",
    }
}

fn misuse_hint(status: StatusCode) -> &'static str {
    match status {
        StatusCode::NOT_FOUND => "Likely wrong endpoint such as /health instead of /v1/health, stale docs, or out-of-order route guessing.",
        StatusCode::METHOD_NOT_ALLOWED => "Likely correct route with wrong HTTP verb.",
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => "Likely missing/invalid credentials or wrong execution identity.",
        StatusCode::SERVICE_UNAVAILABLE | StatusCode::BAD_GATEWAY | StatusCode::GATEWAY_TIMEOUT => "Likely daemon/resource pressure, restart window, or cold route timeout.",
        StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => "Likely malformed JSON, missing project_root/continuity_id, or schema mismatch.",
        _ => "Likely improper route/order or ambiguous infrastructure state.",
    }
}

fn next_tools(status: StatusCode) -> Vec<&'static str> {
    match status {
        StatusCode::NOT_FOUND | StatusCode::METHOD_NOT_ALLOWED => vec!["focusa_tool_doctor", "focusa_project_identity", "focusa_trajectory_view", "focusa_workpoint_resume"],
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => vec!["focusa_tool_doctor"],
        StatusCode::SERVICE_UNAVAILABLE | StatusCode::BAD_GATEWAY | StatusCode::GATEWAY_TIMEOUT => vec!["focusa_tool_doctor", "focusa_resource_mode"],
        StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => vec!["focusa_tool_doctor", "focusa_project_identity"],
        _ => vec!["focusa_tool_doctor"],
    }
}

pub async fn error_envelope_layer(req: Request, next: Next) -> Response {
    let request_method = req.method().as_str().to_string();
    let request_path = req.uri().path().to_string();
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

    let failure_class = failure_class(status);
    let next_tools = next_tools(status);
    let recovery_hint = recovery_hint(status);
    let misuse_hint = misuse_hint(status);
    let envelope = json!({
        "status": "blocked",
        "code": status_code_key(status),
        "failure_class": failure_class,
        "message": status_message(status),
        "what_failed": status_message(status),
        "likely_why": status.canonical_reason().unwrap_or("unknown"),
        "request": {"method": request_method, "path": request_path},
        "safe_recovery": recovery_command(status),
        "recovery_hint": recovery_hint,
        "misuse_hint": misuse_hint,
        "next_tools": next_tools,
        "command": recovery_command(status),
        "fallback": "focusa doctor",
        "docs": ["docs/current/ERROR_EMPTY_STATES.md", "docs/current/TROUBLESHOOTING_CURRENT.md", "docs/current/API_REFERENCE_CURRENT.md"],
        "evidence_refs": [],
        "severity": severity(status),
        "details": {
            "http_status": status.as_u16(),
            "reason": status.canonical_reason().unwrap_or("unknown"),
            "request_method": request_method,
            "request_path": request_path,
            "tool_result_v1": {
                "ok": false,
                "status": "blocked",
                "failure_class": failure_class,
                "canonical": false,
                "degraded": true,
                "summary": status_message(status),
                "retry": { "safe": false, "posture": "do_not_retry_unchanged", "reason": failure_class },
                "recovery_hint": recovery_hint,
                "misuse_hint": misuse_hint,
                "side_effects": [],
                "evidence_refs": [],
                "next_tools": next_tools,
                "error": { "code": status_code_key(status), "message": status_message(status) }
            }
        },
        "correlation_id": correlation_id,
    });

    let mut wrapped = (status, Json(envelope)).into_response();
    if let Ok(hv) = HeaderValue::from_str(&correlation_id) {
        wrapped.headers_mut().insert("x-correlation-id", hv);
    }
    wrapped
}
