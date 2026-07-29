//! Spec 140 A2UI Agent Runtime Studio projection.

use crate::{routes::permissions::permission_context, server::AppState};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

type ApiError = (StatusCode, Json<Value>);
const CATALOG: &str = "https://a2ui.org/specification/v0_9/basic_catalog.json";

#[derive(Debug, Deserialize)]
struct StudioQuery {
    constitution_id: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/agent-runtime/studio", get(studio))
}

async fn studio(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<StudioQuery>,
) -> Result<Json<Value>, ApiError> {
    if !permission_context(&headers, state.config.auth_token.is_some()).allows("work-loop:read") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error":"permission_denied"})),
        ));
    }
    let constitution = state
        .persistence
        .load_runtime_constitution(&query.constitution_id)
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error":"studio_persistence_failed","detail":error.to_string()})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error":"constitution_not_found"})),
            )
        })?;
    let events = state
        .persistence
        .runtime_constitution_events(&query.constitution_id)
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error":"studio_event_read_failed","detail":error.to_string()})),
            )
        })?;
    let active = events
        .iter()
        .rev()
        .find(|event| event.kind == "runtime_constitution.activated");
    let revoked = events
        .iter()
        .rev()
        .find(|event| event.kind == "runtime_constitution.revoked");
    let panels = vec![
        panel(
            "role-grounding",
            "Role & Grounding",
            json!({"role":constitution.role_profile_ref,"identity":constitution.agent_identity_ref,"kernel":constitution.base_agent_constitution_ref}),
        ),
        panel(
            "source-inventory",
            "Instruction Sources",
            json!({"count":constitution.instruction_sources.len(),"sources":constitution.instruction_sources}),
        ),
        panel(
            "conflict-workbench",
            "Conflict Workbench",
            json!({"resolution_refs":constitution.resolution_refs,"unresolved_blocked":true}),
        ),
        panel(
            "prompt-composition",
            "Prompt Composition",
            json!({"claim_refs":constitution.claim_refs,"stable_dynamic_split":true}),
        ),
        panel(
            "prompt-modes",
            "Prompt Modes",
            json!({"default":"append","replacement_requires_evaluation":true}),
        ),
        panel(
            "environment-variants",
            "Environment Variants",
            json!({"dynamic_state_injected_at_runtime":true}),
        ),
        panel(
            "skills-tools",
            "Skills & Tools",
            json!({"progressive_disclosure":true,"typed_routes_preferred":true}),
        ),
        panel(
            "execution-boundaries",
            "Execution Boundaries",
            json!({"boundaries":constitution.operating_contract.execution_boundaries}),
        ),
        panel(
            "targets",
            "Harness Targets",
            json!({"targets":["pi","claude","gemini","copilot","generic"]}),
        ),
        panel(
            "delivery",
            "Delivery",
            json!({"verified_events":events.iter().filter(|event|event.kind=="artifact.delivery_verified").count(),"receipt_required":true}),
        ),
        panel(
            "activation",
            "Activation",
            json!({"active":active.is_some(),"self_activation_forbidden":true}),
        ),
        panel(
            "rollback",
            "Rollback & Revocation",
            json!({"revoked":revoked.is_some(),"rollback_events":events.iter().filter(|event|event.kind=="contract.rollback_activated").count()}),
        ),
    ];
    let components: Vec<Value> = panels
        .iter()
        .enumerate()
        .map(|(index, panel)| {
            json!({
                "id":format!("panel-{index}"),
                "component":"Text",
                "text":format!("{}: {}",panel["title"].as_str().unwrap_or("Panel"),panel["data"])
            })
        })
        .collect();
    Ok(Json(json!({
        "schema":"focusa.agent_runtime_studio.v1",
        "constitution_id":query.constitution_id,
        "panels":panels,
        "a2ui_messages":[
            {"version":"v0.9","createSurface":{"surfaceId":"agent-runtime-studio","catalogId":CATALOG}},
            {"version":"v0.9","updateComponents":{"surfaceId":"agent-runtime-studio","components":components}}
        ]
    })))
}

fn panel(id: &str, title: &str, data: Value) -> Value {
    json!({"id":id,"title":title,"data":data})
}
