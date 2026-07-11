use axum::{Json, Router, routing::get};
use focusa_core::awareness::{
    self, AwarenessInput, SURFACE_AGENT_PRELOAD, SURFACE_POST_COMPACTION, SURFACE_PRELOAD_FAIL,
    SURFACE_PRELOAD_RECEIPT, SURFACE_PRELOAD_REMEDIATION, SURFACE_RELOAD, SURFACE_TOOL_GUIDANCE,
    SURFACE_UIAI_BRIDGE, SURFACE_WARNING,
};
use serde_json::json;
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/utility/card", get(card))
        .route("/v1/utility/bootstrap", get(bootstrap))
        .route("/v1/utility/post-compaction", get(post_compaction))
        .route("/v1/awareness/packet", get(awareness_packet))
        .route(
            "/v1/awareness/packet/{surface}",
            get(awareness_packet_by_surface),
        )
}

async fn card() -> Json<serde_json::Value> {
    // Fallback to legacy static card for backward compat
    let legacy = focusa_core::utility_card::utility_card();
    Json(json!(legacy))
}

async fn bootstrap() -> Json<serde_json::Value> {
    let card = focusa_core::utility_card::utility_card();
    Json(json!({
        "schema": "focusa.utility_bootstrap.v1",
        "status": card.status,
        "authority_boundary": card.authority_boundary,
        "usefulness_bar": card.usefulness_bar,
        "scope_gate": card.scope_gate,
        "bootstrap_card": card.bootstrap_card,
        "exact_next_actions": card.exact_next_actions,
        "do_not_drift": card.do_not_drift,
        "next_tools": card.next_tools,
    }))
}

async fn post_compaction() -> Json<serde_json::Value> {
    let card = focusa_core::utility_card::utility_card();
    Json(json!({
        "schema": "focusa.utility_post_compaction.v1",
        "status": card.status,
        "authority_boundary": card.authority_boundary,
        "usefulness_bar": card.usefulness_bar,
        "post_compaction_card": card.post_compaction_card,
        "exact_next_actions": card.exact_next_actions,
        "do_not_drift": card.do_not_drift,
        "evidence_policy": card.evidence_policy,
        "recovery_order": card.recovery_order,
        "next_tools": card.next_tools,
    }))
}

/// `GET /v1/awareness/packet` — surface-aware awareness packet with default surface.
async fn awareness_packet() -> Json<serde_json::Value> {
    let input = AwarenessInput {
        surface: SURFACE_RELOAD.to_string(),
        ..Default::default()
    };
    let packet = awareness::render_packet(&input);
    Json(json!(packet))
}

/// `GET /v1/awareness/packet/:surface` — surface-aware awareness packet.
async fn awareness_packet_by_surface(
    axum::extract::Path(surface): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let surface: String = match surface.as_str() {
        "reload" => SURFACE_RELOAD.to_string(),
        "post_compaction" => SURFACE_POST_COMPACTION.to_string(),
        "warning" => SURFACE_WARNING.to_string(),
        "tool_guidance" => SURFACE_TOOL_GUIDANCE.to_string(),
        "uiai_bridge" => SURFACE_UIAI_BRIDGE.to_string(),
        "agent_preload" => SURFACE_AGENT_PRELOAD.to_string(),
        "preload_fail" => SURFACE_PRELOAD_FAIL.to_string(),
        "preload_remediation" => SURFACE_PRELOAD_REMEDIATION.to_string(),
        "preload_receipt" => SURFACE_PRELOAD_RECEIPT.to_string(),
        _ => {
            return Json(json!({
                "error": "unknown_surface",
                "allowed": ["reload", "post_compaction", "warning", "tool_guidance", "uiai_bridge", "agent_preload", "preload_fail", "preload_remediation", "preload_receipt"]
            }));
        }
    };

    let input = AwarenessInput {
        surface,
        ..Default::default()
    };
    let packet = awareness::render_packet(&input);
    Json(json!(packet))
}
