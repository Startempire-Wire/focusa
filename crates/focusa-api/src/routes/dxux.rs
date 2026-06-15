use axum::{Json, Router, extract::Path, routing::get};
use focusa_core::dxux::{dxux_explain, dxux_report, dxux_requirement};
use serde_json::json;
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/dxux/report", get(report))
        .route("/v1/dxux/requirement/{id}", get(requirement))
        .route("/v1/dxux/explain/{failure}", get(explain))
        .route("/v1/dxux/digest", get(digest))
}

async fn report() -> Json<serde_json::Value> {
    Json(json!(dxux_report()))
}

async fn requirement(Path(id): Path<String>) -> Json<serde_json::Value> {
    match dxux_requirement(&id) {
        Some(requirement) => Json(json!({
            "schema": "focusa.dxux.requirement.v1",
            "status": "completed",
            "requirement": requirement,
        })),
        None => Json(json!({
            "schema": "focusa.dxux.requirement.v1",
            "status": "not_found",
            "requested_id": id,
            "available_ids": dxux_report().requirements.into_iter().map(|req| req.id).collect::<Vec<_>>(),
        })),
    }
}

async fn explain(Path(failure): Path<String>) -> Json<serde_json::Value> {
    Json(json!(dxux_explain(&failure)))
}

async fn digest() -> Json<serde_json::Value> {
    let report = dxux_report();
    Json(json!({
        "schema": "focusa.dxux.digest.v1",
        "status": "completed",
        "authority": "project_root_plus_continuity_plus_workpoint",
        "why": "Spec105 requires one compact continuation/doability digest with exact next action and evidence refs.",
        "exact_next_action": "Run focusa workpoint resume, verify project scope, then execute focusa preflight before durable closure.",
        "can_continue": true,
        "blocked_reason_code": null,
        "evidence_refs": report.requirements.into_iter().flat_map(|req| req.evidence_refs).take(12).collect::<Vec<_>>(),
        "rehydrate_refs": ["/v1/workpoint/resume", "/v1/trajectory/view", "/v1/dxux/report"],
    }))
}
