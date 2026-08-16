//! CallGraph HTTP surface (#254 slice 2) — Spec 155 §19.1 (first routes).
//!
//! Validation and eligibility are pure core functions; this module wraps
//! them in typed HTTP responses. Definition storage and the run ledger
//! arrive in slice 3+.

use axum::extract::State;
use axum::routing::post;
use axum::Json;
use axum::Router;
use focusa_core::callgraph::{eligibility_for_frame, validate_graph, FocusaCallGraphDefinition};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/callgraphs/validate", post(validate))
        .route("/v1/callgraphs/eligibility", post(eligibility))
}

#[derive(Deserialize)]
pub struct EligibilityBody {
    pub graph: FocusaCallGraphDefinition,
    pub frame_id: String,
    #[serde(default)]
    pub parent_frame_id: Option<String>,
    #[serde(default)]
    pub settled_edges: Vec<String>,
}

async fn validate(
    State(_state): State<Arc<AppState>>,
    Json(graph): Json<FocusaCallGraphDefinition>,
) -> Json<Value> {
    let report = validate_graph(&graph);
    Json(json!({
        "status": if report.valid { "valid" } else { "invalid" },
        "valid": report.valid,
        "issues": report.issues,
        "graph_id": graph.graph_id,
        "revision": graph.revision,
    }))
}

async fn eligibility(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<EligibilityBody>,
) -> Json<Value> {
    let settled: HashSet<String> = body.settled_edges.into_iter().collect();
    let disposition = eligibility_for_frame(
        &body.graph,
        &body.frame_id,
        body.parent_frame_id.as_deref(),
        &settled,
    );
    Json(json!({
        "status": "computed",
        "frame_id": body.frame_id,
        "disposition": disposition,
    }))
}
