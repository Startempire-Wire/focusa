//! JSON shape guard for mutation payloads.
//!
//! Body-size limits stop raw byte abuse; this guard bounds parsed JSON shape for
//! mutation-style requests so excessive nesting or giant arrays are rejected
//! before route handlers persist or further process the payload.

use axum::body::{Body, Bytes, to_bytes};
use axum::extract::Request;
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;
use serde_json::Value;

fn max_body_bytes_for_guard() -> usize {
    crate::routes::bounded::env_limit("FOCUSA_API_MAX_BODY_BYTES", 1_048_576)
}

fn max_json_depth() -> usize {
    crate::routes::bounded::env_limit("FOCUSA_API_JSON_MAX_DEPTH", 64)
}

fn max_json_array_items() -> usize {
    crate::routes::bounded::env_limit("FOCUSA_API_JSON_MAX_ARRAY_ITEMS", 2_048)
}

fn max_json_object_fields() -> usize {
    crate::routes::bounded::env_limit("FOCUSA_API_JSON_MAX_OBJECT_FIELDS", 2_048)
}

fn is_mutation_request(method: &Method, path: &str) -> bool {
    !matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS) && path != "/v1/health"
}

fn is_json_content_type(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            let content_type = value.to_ascii_lowercase();
            content_type.contains("application/json") || content_type.contains("+json")
        })
        .unwrap_or(false)
}

fn validate_json_shape(value: &Value) -> Result<(), &'static str> {
    validate_json_shape_inner(value, 0)
}

fn validate_json_shape_inner(value: &Value, depth: usize) -> Result<(), &'static str> {
    if depth > max_json_depth() {
        return Err("json_depth_exceeded");
    }
    match value {
        Value::Array(items) => {
            if items.len() > max_json_array_items() {
                return Err("json_array_items_exceeded");
            }
            for item in items {
                validate_json_shape_inner(item, depth.saturating_add(1))?;
            }
        }
        Value::Object(map) => {
            if map.len() > max_json_object_fields() {
                return Err("json_object_fields_exceeded");
            }
            for item in map.values() {
                validate_json_shape_inner(item, depth.saturating_add(1))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn rebuild_request(parts: axum::http::request::Parts, bytes: Bytes) -> Request {
    Request::from_parts(parts, Body::from(bytes))
}

pub async fn mutation_json_guard_layer(req: Request, next: Next) -> Result<Response, StatusCode> {
    if !is_mutation_request(req.method(), req.uri().path()) || !is_json_content_type(req.headers())
    {
        return Ok(next.run(req).await);
    }

    let (parts, body) = req.into_parts();
    let bytes = to_bytes(body, max_body_bytes_for_guard())
        .await
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
    if bytes.is_empty() {
        return Ok(next.run(rebuild_request(parts, bytes)).await);
    }

    let value: Value = serde_json::from_slice(&bytes).map_err(|_| StatusCode::BAD_REQUEST)?;
    validate_json_shape(&value).map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(next.run(rebuild_request(parts, bytes)).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn content_type_accepts_json_suffix() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            "application/vnd.focusa+json".parse().unwrap(),
        );
        assert!(is_json_content_type(&headers));
    }

    #[test]
    fn shape_validation_accepts_small_payloads() {
        let value = json!({"mission":"test","refs":["a","b"]});
        assert!(validate_json_shape(&value).is_ok());
    }
}
