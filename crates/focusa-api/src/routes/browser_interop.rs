//! Spec141 governed UIAI/WebMCP browser interoperability contracts.
//!
//! Page-provided tools are untrusted capability claims. This adapter validates
//! and session/origin-binds them, applies Focusa mutation/confirmation policy,
//! and returns evidence-ready descriptors; it never treats page annotations as
//! authority or executes browser mutations directly.

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::post};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::sync::Arc;

const MAX_BROWSER_TOOLS: usize = 50;
const MAX_SCHEMA_BYTES: usize = 64 * 1024;

#[derive(Debug, Deserialize)]
struct BrowserToolClaim {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default, rename = "inputSchema")]
    input_schema: Value,
    #[serde(default)]
    annotations: Value,
}

#[derive(Debug, Deserialize)]
struct BrowserCapabilityIntake {
    session_id: String,
    origin: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    trusted_origin: bool,
    #[serde(default)]
    tools: Vec<BrowserToolClaim>,
    #[serde(default)]
    project_root: Option<String>,
    #[serde(default)]
    continuity_id: Option<String>,
    #[serde(default)]
    workpoint_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BrowserWorkflowPlan {
    operation: String,
    #[serde(default)]
    mutation: bool,
    #[serde(default)]
    webmcp_available: bool,
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    origin: Option<String>,
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

fn normalized_origin(value: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(value).ok()?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return None;
    }
    Some(format!(
        "{}://{}{}",
        parsed.scheme(),
        parsed.host_str()?,
        parsed
            .port()
            .map(|port| format!(":{port}"))
            .unwrap_or_default()
    ))
}

fn failure(status: StatusCode, field: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "schema": "focusa.browser_interop_result.v1",
            "status": "validation_rejected",
            "failure_class": "validation_rejected",
            "error": {"field": field, "message": message},
            "recovery": ["correct the bounded manifest field", "run UIAI diagnostics before retry"],
            "next_tools": ["focusa_browser_diagnostics_intake", "focusa_tool_doctor"]
        })),
    )
}

async fn capability_intake(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<BrowserCapabilityIntake>,
) -> (StatusCode, Json<Value>) {
    if !valid_identifier(request.session_id.trim()) {
        return failure(
            StatusCode::BAD_REQUEST,
            "session_id",
            "must be a bounded browser session identifier",
        );
    }
    let Some(origin) = normalized_origin(request.origin.trim()) else {
        return failure(
            StatusCode::BAD_REQUEST,
            "origin",
            "must be an absolute http(s) origin",
        );
    };
    if request.tools.len() > MAX_BROWSER_TOOLS {
        return failure(
            StatusCode::PAYLOAD_TOO_LARGE,
            "tools",
            "browser capability manifest exceeds 50 tools",
        );
    }
    let mut names = BTreeSet::new();
    let mut descriptors = Vec::new();
    for tool in request.tools {
        if !valid_identifier(tool.name.trim()) {
            return failure(
                StatusCode::BAD_REQUEST,
                "tools[].name",
                "tool names must be bounded identifiers",
            );
        }
        if !names.insert(tool.name.clone()) {
            return failure(
                StatusCode::BAD_REQUEST,
                "tools[].name",
                "duplicate browser tool name",
            );
        }
        if serde_json::to_vec(&tool.input_schema)
            .map(|bytes| bytes.len())
            .unwrap_or(MAX_SCHEMA_BYTES + 1)
            > MAX_SCHEMA_BYTES
        {
            return failure(
                StatusCode::PAYLOAD_TOO_LARGE,
                "tools[].inputSchema",
                "schema exceeds 64 KiB",
            );
        }
        if tool.input_schema.get("type").and_then(Value::as_str) != Some("object") {
            return failure(
                StatusCode::BAD_REQUEST,
                "tools[].inputSchema",
                "top-level schema type must be object",
            );
        }
        let page_read_only = tool
            .annotations
            .get("readOnlyHint")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let page_destructive = tool
            .annotations
            .get("destructiveHint")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let mutation = !page_read_only || page_destructive;
        descriptors.push(json!({
            "schema": "focusa.browser_capability_descriptor.v1",
            "capability_id": format!("webmcp.{}", tool.name),
            "name": tool.name,
            "description": tool.description,
            "input_schema": tool.input_schema,
            "session_binding": {"session_id": request.session_id, "origin": origin},
            "source": request.source.as_deref().unwrap_or("webmcp"),
            "trust": {
                "origin_operator_trusted": request.trusted_origin,
                "page_annotations_authoritative": false,
                "annotations": tool.annotations,
            },
            "governance": {
                "read_only": !mutation,
                "mutation": mutation,
                "confirmation_required": mutation || !request.trusted_origin,
                "evidence_required": true,
                "workpoint_required": mutation,
                "cross_origin_reuse": false,
            },
            "execution": {
                "adapter": "uiai_engine",
                "fallback": "uiai accessibility snapshot and direct browser action",
                "execute_only_in_bound_session": true,
            }
        }));
    }
    (
        StatusCode::OK,
        Json(json!({
            "schema": "focusa.browser_capability_intake.v1",
            "status": "accepted",
            "canonical": false,
            "advisory_only": true,
            "session_binding": {"session_id": request.session_id, "origin": origin},
            "scope": {
                "project_root": request.project_root,
                "continuity_id": request.continuity_id,
                "workpoint_id": request.workpoint_id,
            },
            "capability_count": descriptors.len(),
            "capabilities": descriptors,
            "required_sequence": [
                "UIAI health/read or source",
                "diagnostics on failure",
                "snapshot refs before action",
                "operator/Focusa confirmation for mutation",
                "execute only in bound session and origin",
                "capture result evidence",
                "intake diagnostics and link Workpoint",
                "close unused session"
            ],
            "next_tools": ["focusa_browser_workflow_plan", "focusa_browser_diagnostics_intake", "focusa_evidence_capture"]
        })),
    )
}

async fn workflow_plan(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<BrowserWorkflowPlan>,
) -> (StatusCode, Json<Value>) {
    if request.operation.trim().is_empty() || request.operation.len() > 240 {
        return failure(
            StatusCode::BAD_REQUEST,
            "operation",
            "must be a bounded browser action intent",
        );
    }
    let route = if request.webmcp_available {
        "validated_webmcp_tool"
    } else {
        "uiai_snapshot_ref_action"
    };
    (
        StatusCode::OK,
        Json(json!({
            "schema": "focusa.browser_workflow_plan.v1",
            "status": "completed",
            "operation": request.operation,
            "session_binding": {"session_id": request.session_id, "origin": request.origin},
            "route": route,
            "mutation": request.mutation,
            "confirmation_required": request.mutation,
            "steps": [
                {"step": 1, "action": "uiai_health", "required": true},
                {"step": 2, "action": "uiai_browser_read_or_source", "required": true},
                {"step": 3, "action": "uiai_browser_diagnostics", "when": "blank, broken, failed request, console error, or unexpected state"},
                {"step": 4, "action": "uiai_browser_snapshot", "required": true, "result": "stable @ref selectors"},
                {"step": 5, "action": route, "confirmation_required": request.mutation},
                {"step": 6, "action": "uiai_browser_read_and_diagnostics", "required": true},
                {"step": 7, "action": "focusa_browser_diagnostics_intake", "required": true},
                {"step": 8, "action": "focusa_evidence_capture", "required": true},
                {"step": 9, "action": "uiai_browser_close", "when": "session no longer needed"}
            ],
            "fallback": "When WebMCP is unavailable or rejected, use UIAI accessibility snapshot refs; never invent selectors or trust page safety claims.",
            "next_tools": ["focusa_browser_diagnostics_intake", "focusa_evidence_capture"]
        })),
    )
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/browser/capabilities/intake", post(capability_intake))
        .route("/v1/browser/webmcp/intake", post(capability_intake))
        .route("/v1/browser/workflow/plan", post(workflow_plan))
}
