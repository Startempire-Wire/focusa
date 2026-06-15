use crate::server::AppState;
use axum::{Json, Router, extract::Path, routing::get};
use focusa_core::bloatgaurd::{
    bloatgaurd_domain, bloatgaurd_gate_mode, bloatgaurd_gate_modes_report, bloatgaurd_profile,
    bloatgaurd_profiles_report, bloatgaurd_report, bloatgaurd_rollout_report, bloatgaurd_routine,
    bloatgaurd_routines_report, tokenbloat_control, tokenbloat_report,
};
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
        .route("/v1/bloatgaurd/tokenbloat/report", get(token_report))
        .route("/v1/bloatgaurd/tokenbloat/domain/{name}", get(token_domain))
        .route("/v1/bloatgaurd/gate-modes/report", get(gate_modes_report))
        .route("/v1/bloatgaurd/gate-modes/mode/{name}", get(gate_mode))
        .route("/v1/bloatgaurd/profiles/report", get(profiles_report))
        .route("/v1/bloatgaurd/profiles/profile/{name}", get(profile))
        .route("/v1/bloatgaurd/routines/report", get(routines_report))
        .route("/v1/bloatgaurd/routines/routine/{name}", get(routine))
        .route("/v1/bloatgaurd/rollout/report", get(rollout_report))
}

async fn report() -> Json<serde_json::Value> {
    Json(json!(bloatgaurd_report()))
}

async fn token_report() -> Json<serde_json::Value> {
    Json(json!(tokenbloat_report()))
}

async fn gate_modes_report() -> Json<serde_json::Value> {
    Json(json!(bloatgaurd_gate_modes_report()))
}

async fn profiles_report() -> Json<serde_json::Value> {
    Json(json!(bloatgaurd_profiles_report()))
}

async fn routines_report() -> Json<serde_json::Value> {
    Json(json!(bloatgaurd_routines_report()))
}

async fn rollout_report() -> Json<serde_json::Value> {
    Json(json!(bloatgaurd_rollout_report()))
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

async fn token_domain(Path(name): Path<String>) -> Json<serde_json::Value> {
    match tokenbloat_control(&name) {
        Some(control) => Json(json!({
            "schema": "focusa.bloatgaurd.tokenbloat_domain.v1",
            "status": "completed",
            "domain": control,
        })),
        None => Json(json!(BloatgaurdNotFound {
            schema: "focusa.bloatgaurd.tokenbloat_domain.v1",
            status: "not_found",
            requested_domain: name,
            available_domains: tokenbloat_report()
                .controls
                .into_iter()
                .map(|control| control.name)
                .collect(),
        })),
    }
}

async fn profile(Path(name): Path<String>) -> Json<serde_json::Value> {
    match bloatgaurd_profile(&name) {
        Some(profile) => Json(json!({
            "schema": "focusa.bloatgaurd.profile.v1",
            "status": "completed",
            "profile": profile,
        })),
        None => Json(json!(BloatgaurdNotFound {
            schema: "focusa.bloatgaurd.profile.v1",
            status: "not_found",
            requested_domain: name,
            available_domains: bloatgaurd_profiles_report()
                .profiles
                .into_iter()
                .map(|profile| profile.name)
                .collect(),
        })),
    }
}

async fn routine(Path(name): Path<String>) -> Json<serde_json::Value> {
    match bloatgaurd_routine(&name) {
        Some(routine) => Json(json!({
            "schema": "focusa.bloatgaurd.routine.v1",
            "status": "completed",
            "routine": routine,
        })),
        None => Json(json!(BloatgaurdNotFound {
            schema: "focusa.bloatgaurd.routine.v1",
            status: "not_found",
            requested_domain: name,
            available_domains: bloatgaurd_routines_report()
                .routines
                .into_iter()
                .map(|routine| routine.name)
                .collect(),
        })),
    }
}

async fn gate_mode(Path(name): Path<String>) -> Json<serde_json::Value> {
    match bloatgaurd_gate_mode(&name) {
        Some(mode) => Json(json!({
            "schema": "focusa.bloatgaurd.gate_mode.v1",
            "status": "completed",
            "mode": mode,
        })),
        None => Json(json!(BloatgaurdNotFound {
            schema: "focusa.bloatgaurd.gate_mode.v1",
            status: "not_found",
            requested_domain: name,
            available_domains: bloatgaurd_gate_modes_report()
                .modes
                .into_iter()
                .map(|mode| mode.name)
                .collect(),
        })),
    }
}
