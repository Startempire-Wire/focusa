//! Constitution routes.

use crate::routes::permissions::{forbid, permission_context};
use crate::server::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::{Json, Router, routing::{get, post}};
use focusa_core::types::{Action, FocusaEvent};
use serde_json::{Value, json};
use std::sync::Arc;

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
        None => Ok(Json(json!({ "error": "No active constitution" }))),
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
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "content field required"})),
        ));
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
        if lower.contains("safety") || lower.contains("banned") || (lower.contains("never") && lower.contains("rule")) {
            current_section = "safety";
            continue;
        }
        if lower.contains("expression") || (lower.contains("constraint") && lower.contains("rule")) {
            current_section = "expression";
            continue;
        }

        // Extract items from bullet points or numbered lists
        if (trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
            && trimmed.len() > 5
        {
            let text = trimmed.trim_start_matches(|c: char| c == '-' || c == '*' || c.is_ascii_digit() || c == '.' || c == ' ');
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

    state.command_tx.send(Action::EmitEvent { event }).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "failed to dispatch constitution load"})),
        )
    })?;

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
            Json(json!({"status": "accepted", "warning": "constitution load dispatched but not yet visible", "version": version})),
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
