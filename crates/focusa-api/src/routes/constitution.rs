//! Constitution routes.

use crate::routes::permissions::{forbid, permission_context};
use crate::server::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::{Action, FocusaEvent};
use serde_json::{Value, json};
use std::sync::Arc;

fn constitution_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
    next_tools: Vec<&'static str>,
) -> (StatusCode, Json<Value>) {
    let error = error.into();
    let why = why.into();
    let next_tools_value = json!(next_tools);
    let retry_safe = !matches!(
        failure_class,
        "validation_rejected" | "not_found" | "permission_denied"
    );
    let retry_posture = if retry_safe {
        "safe_retry"
    } else {
        "do_not_retry_unchanged"
    };
    (
        http_status,
        Json(json!({
            "status": "blocked",
            "canonical": false,
            "degraded": true,
            "error": error,
            "failure_class": failure_class,
            "why": why,
            "recovery_hint": recovery_hint,
            "misuse_hint": misuse_hint,
            "next_tools": next_tools_value.clone(),
            "details": {
                "tool_result_v1": {
                    "ok": false,
                    "status": "blocked",
                    "canonical": false,
                    "degraded": true,
                    "failure_class": failure_class,
                    "summary": why,
                    "retry": {"safe": retry_safe, "posture": retry_posture, "reason": failure_class},
                    "recovery_hint": recovery_hint,
                    "misuse_hint": misuse_hint,
                    "side_effects": [],
                    "evidence_refs": [],
                    "next_tools": next_tools_value,
                    "error": {"code": failure_class, "message": error}
                }
            }
        })),
    )
}

fn constitution_not_active() -> Value {
    json!({
        "status": "blocked",
        "canonical": false,
        "degraded": true,
        "error": "No active constitution",
        "failure_class": "not_found",
        "why": "No constitution version is currently active in Focusa state.",
        "recovery_hint": "Load a constitution version or resolve a constitution_revision proposal before reading active constitution.",
        "misuse_hint": "Likely fresh daemon state, no seed constitution loaded, or wrong daemon instance.",
        "next_tools": ["focusa_tool_doctor", "focusa_traverse"],
        "details": {
            "tool_result_v1": {
                "ok": false,
                "status": "blocked",
                "canonical": false,
                "degraded": true,
                "failure_class": "not_found",
                "summary": "No constitution version is currently active in Focusa state.",
                "retry": {"safe": false, "posture": "do_not_retry_unchanged", "reason": "not_found"},
                "side_effects": [],
                "evidence_refs": [],
                "next_tools": ["focusa_tool_doctor", "focusa_traverse"],
                "error": {"code": "not_found", "message": "No active constitution"}
            }
        }
    })
}

fn constitution_content_required() -> (StatusCode, Json<Value>) {
    constitution_failure(
        StatusCode::BAD_REQUEST,
        "content field required",
        "validation_rejected",
        "Constitution load requires a non-empty content field.",
        "Send JSON with a non-empty content string before retrying unchanged.",
        "Likely missing content, wrong payload key, or stale caller contract.",
        vec!["focusa_tool_doctor"],
    )
}

fn constitution_dispatch_failed(error: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    constitution_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("failed to dispatch constitution load: {error}"),
        "daemon_unavailable",
        "Constitution load event could not be dispatched to daemon command channel.",
        "Check daemon health and retry after command channel recovery is clear.",
        "Likely daemon command channel closed, runtime shutdown, or writer/transport ownership issue.",
        vec!["focusa_tool_doctor", "focusa_work_loop_status"],
    )
}

/// GET /v1/constitution/active — active constitution.
async fn active(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("constitution:read") {
        return Err(forbid("constitution:read"));
    }
    let s = state.focusa.read().await;
    match focusa_core::constitution::active(&s.constitution) {
        Some(c) => Ok(Json(serde_json::to_value(c).unwrap_or(json!({})))),
        None => Ok(Json(constitution_not_active())),
    }
}

/// GET /v1/constitution/versions — version history.
async fn versions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("constitution:read") {
        return Err(forbid("constitution:read"));
    }
    let s = state.focusa.read().await;
    let versions = focusa_core::constitution::version_history(&s.constitution);
    Ok(Json(json!({
        "versions": versions,
        "active": s.constitution.active_version,
    })))
}

/// POST /v1/constitution/load — load constitution from content.
///
/// Accepts raw markdown content, parses principles/safety/expression rules,
/// creates a new version and activates it. Used by `wb soul reload`.
async fn load_constitution(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("constitution:write") {
        return Err(forbid("constitution:write"));
    }

    let content = body.get("content").and_then(|v| v.as_str()).unwrap_or("");
    if content.is_empty() {
        return Err(constitution_content_required());
    }

    let source = body.get("source").and_then(|v| v.as_str()).unwrap_or("api");

    // Parse principles from content (lines starting with numbered patterns)
    let mut principles = Vec::new();
    let mut safety_rules = Vec::new();
    let mut expression_rules = Vec::new();
    let mut current_section = "";

    for line in content.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();

        // Detect section headers
        if lower.contains("principle") || lower.contains("pillar") || lower.contains("behavioral") {
            current_section = "principles";
            continue;
        }
        if lower.contains("safety")
            || lower.contains("banned")
            || (lower.contains("never") && lower.contains("rule"))
        {
            current_section = "safety";
            continue;
        }
        if lower.contains("expression") || (lower.contains("constraint") && lower.contains("rule"))
        {
            current_section = "expression";
            continue;
        }

        // Extract items from bullet points or numbered lists
        if (trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false))
            && trimmed.len() > 5
        {
            let text = trimmed.trim_start_matches(|c: char| {
                c == '-' || c == '*' || c.is_ascii_digit() || c == '.' || c == ' '
            });
            match current_section {
                "principles" => {
                    principles.push(focusa_core::types::ConstitutionPrinciple {
                        id: format!("p{}", principles.len() + 1),
                        text: text.to_string(),
                        priority: (principles.len() + 1) as u32,
                        rationale: String::new(),
                    });
                }
                "safety" => safety_rules.push(text.to_string()),
                "expression" => expression_rules.push(text.to_string()),
                _ => {}
            }
        }
    }

    // If no structured content found, use seed defaults + note the source
    if principles.is_empty() && safety_rules.is_empty() {
        return Ok(Json(json!({
            "status": "no_structured_content",
            "message": "Could not parse principles/safety/expression rules from content",
            "content_length": content.len(),
            "source": source,
        })));
    }

    // Create new version and activate
    let version = format!("soul-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    let event = FocusaEvent::ConstitutionLoaded {
        version: version.clone(),
        agent_id: "wirebot".to_string(),
        principles: principles.clone(),
        safety_rules: safety_rules.clone(),
        expression_rules: expression_rules.clone(),
    };

    state
        .command_tx
        .send(Action::EmitEvent { event })
        .await
        .map_err(constitution_dispatch_failed)?;

    let mut visible = false;
    for _ in 0..80 {
        {
            let s = state.focusa.read().await;
            if focusa_core::constitution::active(&s.constitution)
                .is_some_and(|c| c.version == version)
            {
                visible = true;
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }

    if !visible {
        return Err((
            StatusCode::ACCEPTED,
            Json(
                json!({"status": "accepted", "warning": "constitution load dispatched but not yet visible", "version": version}),
            ),
        ));
    }

    Ok(Json(json!({
        "status": "loaded",
        "version": version,
        "principles": principles.len(),
        "safety_rules": safety_rules.len(),
        "expression_rules": expression_rules.len(),
        "source": source,
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/constitution/active", get(active))
        .route("/v1/constitution/versions", get(versions))
        .route("/v1/constitution", post(load_constitution))
        .route("/v1/constitution/load", post(load_constitution))
}
