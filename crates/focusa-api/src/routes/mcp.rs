//! Spec141 MCP interoperability projection.
//!
//! The catalog is generated from the same runtime Pi schemas and canonical
//! tool contracts as every other harness projection. Calls are bridged to the
//! matching scoped REST operation so existing auth, permission, authority,
//! idempotency, and receipt checks remain authoritative.

use crate::server::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, header::AUTHORIZATION};
use axum::{Json, Router, routing::post};
use reqwest::Method;
use serde_json::{Map, Value, json};
use std::sync::{Arc, LazyLock};

const PROTOCOL_VERSION: &str = "2025-11-25";
const PAGE_SIZE: usize = 25;

static MCP_PROJECTION: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../../../docs/contracts/spec141/generated-capability-v2/mcp-tools.json"
    ))
    .expect("generated Spec141 MCP projection must be valid JSON")
});

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/mcp", post(handle_jsonrpc))
        .route("/v1/mcp", post(handle_jsonrpc))
}

fn success_response(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn error_response(id: Value, code: i64, message: &str, data: Option<Value>) -> Value {
    let mut error = json!({"code": code, "message": message});
    if let Some(data) = data {
        error["data"] = data;
    }
    json!({"jsonrpc": "2.0", "id": id, "error": error})
}

fn tools() -> &'static [Value] {
    MCP_PROJECTION
        .get("tools")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn tool_error(message: impl Into<String>, code: &str, recovery: &[&str]) -> Value {
    let message = message.into();
    json!({
        "content": [{"type": "text", "text": message}],
        "structuredContent": {
            "schema": "focusa.mcp_tool_error.v1",
            "status": "error",
            "failure_class": code,
            "message": message,
            "recovery": recovery,
        },
        "isError": true,
    })
}

fn scalar_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn safe_path_value(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':'))
}

fn bind_path(path: &str, arguments: &mut Map<String, Value>) -> Result<String, Value> {
    let mut bound = path.to_string();
    loop {
        let Some(start) = bound.find('{') else { break };
        let Some(relative_end) = bound[start + 1..].find('}') else {
            return Err(tool_error(
                format!("invalid generated REST path template: {path}"),
                "schema_invalid",
                &["run the Spec141 capability generator drift check"],
            ));
        };
        let end = start + 1 + relative_end;
        let key = &bound[start + 1..end];
        let Some(value) = arguments
            .remove(key)
            .and_then(|value| scalar_string(&value))
        else {
            return Err(tool_error(
                format!("missing path parameter: {key}"),
                "validation_rejected",
                &[
                    "provide the required path parameter",
                    "call focusa_tool_describe for the full schema",
                ],
            ));
        };
        if !safe_path_value(&value) {
            return Err(tool_error(
                format!("unsafe path parameter: {key}"),
                "validation_rejected",
                &[
                    "use an identifier containing only letters, numbers, dash, underscore, dot, or colon",
                ],
            ));
        }
        bound.replace_range(start..=end, &value);
    }
    Ok(bound)
}

async fn call_rest_tool(
    headers: &HeaderMap,
    tool: &Value,
    mut arguments: Map<String, Value>,
) -> Value {
    let Some(route) = tool.pointer("/_meta/rest/0").and_then(Value::as_object) else {
        return tool_error(
            "This capability has no MCP-callable REST binding.",
            "not_available",
            &["use its Pi-local or CLI binding shown by focusa_tool_describe"],
        );
    };
    let method = route.get("method").and_then(Value::as_str).unwrap_or("GET");
    let path = route
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let path = match bind_path(path, &mut arguments) {
        Ok(path) => path,
        Err(error) => return error,
    };
    let base = std::env::var("FOCUSA_DAEMON_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8787".to_string())
        .trim_end_matches('/')
        .to_string();
    let url = format!("{base}{path}");
    let method = match Method::from_bytes(method.as_bytes()) {
        Ok(method) => method,
        Err(_) => {
            return tool_error(
                format!("unsupported generated REST method: {method}"),
                "schema_invalid",
                &["regenerate the Spec141 operation projection"],
            );
        }
    };
    let client = reqwest::Client::new();
    let mut request = client.request(method.clone(), &url);
    if let Some(value) = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        request = request.header(AUTHORIZATION.as_str(), value);
    }
    if method == Method::GET || method == Method::DELETE {
        let query: Vec<(String, String)> = arguments
            .iter()
            .filter_map(|(key, value)| scalar_string(value).map(|value| (key.clone(), value)))
            .collect();
        request = request.query(&query);
    } else {
        request = request.json(&Value::Object(arguments));
    }
    let response = match request.send().await {
        Ok(response) => response,
        Err(error) => {
            return tool_error(
                format!("Focusa REST bridge unavailable: {error}"),
                "daemon_unavailable",
                &[
                    "run focusa_tool_doctor",
                    "verify FOCUSA_DAEMON_URL and retry safely",
                ],
            );
        }
    };
    let status = response.status();
    let body = match response.text().await {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| json!({"text": text})),
        Err(error) => json!({"message": error.to_string()}),
    };
    json!({
        "content": [{"type": "text", "text": serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string())}],
        "structuredContent": body,
        "isError": !status.is_success(),
        "_meta": {
            "http_status": status.as_u16(),
            "capability_id": tool.pointer("/_meta/capability_id"),
            "docs_ref": tool.pointer("/_meta/docs_ref"),
        }
    })
}

async fn handle_jsonrpc(
    State(_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> Json<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return Json(error_response(
            id,
            -32600,
            "Invalid Request",
            Some(json!({"hint": "missing method"})),
        ));
    };

    let result = match method {
        "initialize" => success_response(
            id,
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {"tools": {"listChanged": true}},
                "serverInfo": {"name": "focusa", "version": env!("CARGO_PKG_VERSION")},
                "instructions": "Use tools/list with pagination. Tool schemas, structured outputs, safety annotations, docs, skills, and REST authority bindings are generated from Focusa Agent Capability Descriptor V2."
            }),
        ),
        "notifications/initialized" | "notifications/cancelled" => Value::Null,
        "ping" => success_response(id, json!({})),
        "tools/list" => {
            let cursor = request
                .pointer("/params/cursor")
                .and_then(Value::as_str)
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            let catalog = tools();
            if cursor > catalog.len() {
                error_response(
                    id,
                    -32602,
                    "Invalid cursor",
                    Some(json!({"cursor": cursor})),
                )
            } else {
                let end = (cursor + PAGE_SIZE).min(catalog.len());
                let mut payload = json!({
                    "tools": catalog[cursor..end],
                    "_meta": {
                        "schema": "focusa.mcp_tool_projection.v2",
                        "registry_digest": MCP_PROJECTION.get("registry_digest"),
                        "total": catalog.len(),
                        "page_size": PAGE_SIZE,
                    }
                });
                if end < catalog.len() {
                    payload["nextCursor"] = Value::String(end.to_string());
                }
                success_response(id, payload)
            }
        }
        "tools/call" => {
            let Some(name) = request.pointer("/params/name").and_then(Value::as_str) else {
                return Json(error_response(
                    id,
                    -32602,
                    "Invalid params",
                    Some(json!({"hint": "missing tool name"})),
                ));
            };
            let Some(tool) = tools()
                .iter()
                .find(|tool| tool.get("name").and_then(Value::as_str) == Some(name))
            else {
                return Json(success_response(
                    id,
                    tool_error(
                        format!("Unknown Focusa MCP tool: {name}"),
                        "not_found",
                        &[
                            "call tools/list",
                            "use focusa_tool_search through a supported binding",
                        ],
                    ),
                ));
            };
            let arguments = request
                .pointer("/params/arguments")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            success_response(id, call_rest_tool(&headers, tool, arguments).await)
        }
        _ => error_response(
            id,
            -32601,
            "Method not found",
            Some(json!({"method": method})),
        ),
    };
    Json(result)
}
