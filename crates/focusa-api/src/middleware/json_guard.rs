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

fn is_scope_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    matches!(
        key.as_str(),
        "project_root"
            | "continuity_id"
            | "session_id"
            | "scope_kind"
            | "query_scope_kind"
            | "action_type"
            | "checkpoint_reason"
            | "work_item_id"
            | "workpoint_id"
    )
}

fn validate_scope_field_value(key: &str, value: &str) -> Result<(), &'static str> {
    if value.len() > 512 {
        return Err("scope_field_too_long");
    }
    if key == "scope_kind" || key == "query_scope_kind" {
        let valid = matches!(
            value,
            "project"
                | "host"
                | "fresh_question"
                | "mission_carryover"
                | "correction"
                | "meta"
                | "suppress_by_default"
                | "allow_if_relevant"
                | "prefer_reset"
        );
        if !valid {
            return Err("invalid_scope_kind");
        }
    }
    if key == "continuity_id" {
        let valid = value.len() <= 256
            && value
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':');
        if !valid {
            return Err("invalid_continuity_id");
        }
    }
    if key == "checkpoint_reason"
        && !matches!(
            value,
            "manual"
                | "session_start"
                | "operator_checkpoint"
                | "before_compact"
                | "after_compact"
                | "context_overflow"
                | "session_resume"
                | "model_switch"
                | "fork"
        )
    {
        return Err("invalid_checkpoint_reason");
    }
    Ok(())
}

fn validate_scope_fields(value: &Value) -> Result<(), &'static str> {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                if is_scope_key(key) {
                    if let Value::String(s) = val {
                        validate_scope_field_value(key, s)?;
                    }
                }
                validate_scope_fields(val)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                validate_scope_fields(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn is_path_like_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    matches!(
        key.as_str(),
        "project_root"
            | "cwd"
            | "root_dir"
            | "target_ref"
            | "evidence_ref"
            | "diagnostics_ref"
            | "artifact_ref"
            | "storage_path"
    ) || key.ends_with("_path")
        || key.ends_with("_paths")
        || key.ends_with("_ref")
        || key.ends_with("_refs")
        || key.ends_with("_root")
        || key.ends_with("_dir")
}

fn looks_like_path_traversal(value: &str) -> bool {
    let normalized = value.replace('\\', "/");
    if normalized.split('/').any(|segment| segment == "..") {
        return true;
    }
    let lower = normalized.to_ascii_lowercase();
    lower.contains("%2e%2e") || lower.contains("..%2f") || lower.contains("..%5c")
}

fn validate_json_path_safety(value: &Value) -> Result<(), &'static str> {
    validate_json_path_safety_inner(value, None)
}

fn validate_json_path_safety_inner(
    value: &Value,
    active_key: Option<&str>,
) -> Result<(), &'static str> {
    match value {
        Value::String(text) => {
            if active_key.is_some_and(is_path_like_key) && looks_like_path_traversal(text) {
                return Err("json_path_traversal");
            }
        }
        Value::Array(items) => {
            for item in items {
                validate_json_path_safety_inner(item, active_key)?;
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                validate_json_path_safety_inner(item, Some(key.as_str()))?;
            }
        }
        _ => {}
    }
    Ok(())
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
    validate_json_path_safety(&value).map_err(|_| StatusCode::BAD_REQUEST)?;
    validate_scope_fields(&value).map_err(|_| StatusCode::BAD_REQUEST)?;

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

    #[test]
    fn path_safety_rejects_traversal_in_path_like_fields() {
        let value = json!({"project_root":"/tmp/focusa/../../etc", "mission":"test"});
        assert_eq!(
            validate_json_path_safety(&value),
            Err("json_path_traversal")
        );
    }

    #[test]
    fn path_safety_rejects_nested_ref_arrays() {
        let value = json!({"ontology_context":{"artifact_refs":["commit:abc", "../etc/passwd"]}});
        assert_eq!(
            validate_json_path_safety(&value),
            Err("json_path_traversal")
        );
    }

    #[test]
    fn path_safety_rejects_encoded_slash_traversal() {
        let value = json!({"evidence_refs":["..%2fetc/passwd"]});
        assert_eq!(
            validate_json_path_safety(&value),
            Err("json_path_traversal")
        );
    }

    #[test]
    fn path_safety_allows_text_fields_with_dots() {
        let value = json!({"content":"explain ../ only as prose", "mission":"test..ok"});
        assert!(validate_json_path_safety(&value).is_ok());
    }

    #[test]
    fn scope_guard_accepts_session_start_checkpoint_reason() {
        let value = json!({"checkpoint_reason":"session_start"});
        assert!(validate_scope_fields(&value).is_ok());
    }

    #[test]
    fn scope_guard_accepts_typed_project_and_host_scope_kinds() {
        for scope_kind in ["project", "host"] {
            let value = json!({
                "scope": {
                    "root_scope": {
                        "scope_kind": scope_kind,
                        "scope_id": "project:focusa",
                        "root_path": "/workspace/focusa",
                        "canonical_name": "focusa",
                        "fingerprint": "sha256:focusa"
                    },
                    "continuity_id": "release-v135"
                }
            });
            assert!(validate_scope_fields(&value).is_ok());
        }
    }

    #[test]
    fn scope_guard_rejects_unknown_scope_kinds() {
        let value = json!({"scope_kind":"untrusted"});
        assert_eq!(validate_scope_fields(&value), Err("invalid_scope_kind"));
    }
}
