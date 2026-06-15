use crate::server::AppState;
use axum::{Json, Router, extract::Path, routing::get};
use focusa_core::bloatgaurd::{bloatgaurd_domain, bloatgaurd_report};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Serialize)]
struct BloatgaurdNotFound {
    schema: &'static str,
    status: &'static str,
    requested_domain: String,
    available_domains: Vec<String>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/bloatgaurd/report", get(report))
        .route("/v1/bloatgaurd/domain/{name}", get(domain))
}

async fn report() -> Json<serde_json::Value> {
    Json(json!(bloatgaurd_report()))
}

async fn domain(Path(name): Path<String>) -> Json<serde_json::Value> {
    match bloatgaurd_domain(&name) {
        Some(domain) => Json(json!({
            "schema": "focusa.bloatgaurd.domain.v1",
            "status": "completed",
            "domain": domain,
        })),
        None => Json(json!(BloatgaurdNotFound {
            schema: "focusa.bloatgaurd.domain.v1",
            status: "not_found",
            requested_domain: name,
            available_domains: bloatgaurd_report()
                .domains
                .into_iter()
                .map(|domain| domain.name)
                .collect(),
        })),
    }
}
