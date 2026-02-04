//! Agent Skills routes.

use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::server::AppState;

/// GET /v1/skills — list all skills.
async fn list_skills(State(_state): State<Arc<AppState>>) -> Json<Value> {
    let registry = focusa_core::skills::SkillRegistry::new();
    let skills: Vec<Value> = registry.skills.iter().map(|s| {
        json!({
            "id": s.id,
            "name": s.name,
            "category": s.category,
            "endpoint": s.api_endpoint,
            "permission_class": s.permission_class,
            "enabled": s.enabled,
        })
    }).collect();

    Json(json!({
        "skills": skills,
        "total": skills.len(),
        "prohibited": focusa_core::types::PROHIBITED_SKILLS,
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/skills", get(list_skills))
}
