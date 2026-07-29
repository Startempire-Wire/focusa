//! Spec 140 instruction migration inventory and quarantine preview.

use crate::{routes::permissions::permission_context, server::AppState};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use focusa_core::{
    agent_runtime_constitution_authority::{
        default_authority_graph, detect_conflicts, discover_project_instructions,
    },
    agent_runtime_constitution_migration::{plan_instruction_migration, verify_migration_plan},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::PathBuf, sync::Arc};
use uuid::Uuid;

type ApiError = (StatusCode, Json<Value>);

#[derive(Debug, Deserialize)]
struct MigrationPreviewRequest {
    project_root: String,
    max_source_bytes: Option<u64>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/agent-runtime/migration/preview", post(preview))
}

async fn preview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<MigrationPreviewRequest>,
) -> Result<Json<Value>, ApiError> {
    if !permission_context(&headers, state.config.auth_token.is_some()).allows("work-loop:read") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error":"permission_denied"})),
        ));
    }
    let root = PathBuf::from(&request.project_root);
    if !root.is_absolute() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"absolute_project_root_required"})),
        ));
    }
    let discovered =
        discover_project_instructions(&root, request.max_source_bytes.unwrap_or(262_144)).map_err(
            |reason| {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({"error":reason})),
                )
            },
        )?;
    let conflicts = detect_conflicts(&discovered.claims, &default_authority_graph());
    let plan = plan_instruction_migration(
        format!("migration:{}", Uuid::now_v7()),
        &discovered.sources,
        &conflicts,
    );
    let expected: BTreeSet<_> = discovered
        .sources
        .iter()
        .map(|source| source.source_id.clone())
        .collect();
    verify_migration_plan(&plan, &expected).map_err(|errors| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error":"migration_plan_invalid","details":errors})),
        )
    })?;
    Ok(Json(json!({
        "schema":"focusa.agent_runtime_migration_preview.v1",
        "plan":plan,
        "findings":discovered.findings,
        "committed":false,
        "hidden_behavior_changes":false
    })))
}
