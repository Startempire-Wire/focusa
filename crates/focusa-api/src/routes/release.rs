use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::json;
use std::sync::Arc;

async fn proof_status(State(_state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "manual_proof_required",
        "canonical": true,
        "degraded": false,
        "version": env!("CARGO_PKG_VERSION"),
        "summary": "Run release proof before publishing or relying on attached artifacts.",
        "required_command": "focusa release prove --tag <tag> --github",
        "evidence_refs": [
            "docs/current/VALIDATION_AND_RELEASE_PROOF.md",
            "docs/current/PRODUCTION_RELEASE_COMMANDS.md"
        ],
        "next_tools": ["focusa doctor", "focusa release prove --tag <tag> --github"],
        "details": {
            "tool_result_v1": {
                "ok": true,
                "status": "manual_proof_required",
                "canonical": true,
                "degraded": false,
                "failure_class": null,
                "retry": {"safe": true, "posture": "manual_gate"},
                "side_effects": [],
                "evidence_refs": ["docs/current/VALIDATION_AND_RELEASE_PROOF.md"]
            }
        }
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/release/proof/status", get(proof_status))
}
