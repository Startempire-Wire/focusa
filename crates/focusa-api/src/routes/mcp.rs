//! Minimal MCP JSON-RPC bridge (focusa-112-mcp-jsonrpc).
//!
//! Additive route only: POST /mcp and POST /v1/mcp. This intentionally
//! exposes a small, safe MCP-compatible surface first (initialize,
//! tools/list, tools/call focusa.health) rather than bypassing existing
//! HTTP route scope enforcement.

use axum::{Json, Router, routing::post};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/mcp", post(handle_jsonrpc))
        .route("/v1/mcp", post(handle_jsonrpc))
}

async fn handle_jsonrpc(Json(request): Json<Value>) -> Json<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    if request.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Json(error_response(
            id,
            -32600,
            "Invalid Request",
            Some(json!({"hint":"MCP requests must use jsonrpc=2.0"})),
        ));
    }
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return Json(error_response(
            id,
            -32600,
            "Invalid Request",
            Some(json!({"hint":"missing method"})),
        ));
    };

    match method {
        "initialize" => Json(success_response(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {
                    "name": "focusa",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        )),
        "ping" => Json(success_response(id, json!({}))),
        "tools/list" => Json(success_response(
            id,
            json!({
                "tools": [
                    {
                        "name": "focusa.health",
                        "description": "Read Focusa daemon health. Safe/unscoped; project-bound tools must use existing scoped HTTP routes.",
                        "inputSchema": {"type":"object", "properties": {}, "additionalProperties": false}
                    }
                ]
            }),
        )),
        "tools/call" => handle_tool_call(id, request.get("params").cloned().unwrap_or(Value::Null)),
        _ => Json(error_response(
            id,
            -32601,
            "Method not found",
            Some(
                json!({"method": method, "supported": ["initialize", "ping", "tools/list", "tools/call"]}),
            ),
        )),
    }
}

fn handle_tool_call(id: Value, params: Value) -> Json<Value> {
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return Json(error_response(
            id,
            -32602,
            "Invalid params",
            Some(json!({"hint":"tools/call requires params.name"})),
        ));
    };
    match name {
        "focusa.health" => Json(success_response(
            id,
            json!({
                "content": [
                    {"type":"text", "text":"Focusa daemon JSON-RPC MCP bridge is reachable."}
                ],
                "structuredContent": {
                    "ok": true,
                    "status": "ok",
                    "bridge": "mcp-jsonrpc",
                    "scope": "unscoped-health-only"
                }
            }),
        )),
        _ => Json(error_response(
            id,
            -32601,
            "Tool not found",
            Some(json!({
                "tool": name,
                "hint": "Project-bound tools are intentionally not exposed through this minimal MCP bridge; use scoped HTTP routes with project_root+continuity_id."
            })),
        )),
    }
}

fn success_response(id: Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0", "id": id, "result": result})
}

fn error_response(id: Value, code: i64, message: &str, data: Option<Value>) -> Value {
    let mut error = json!({"code": code, "message": message});
    if let Some(data) = data {
        error["data"] = data;
    }
    json!({"jsonrpc":"2.0", "id": id, "error": error})
}
