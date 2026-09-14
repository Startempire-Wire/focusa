use crate::server::AppState;
use axum::{Json, Router, http::StatusCode, routing::post};
use focusa_core::daemon_multiplex::{
    DaemonRegistryProjection, DaemonRoutingAuthority, ProjectRouteKey, project_routing_authority,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct ResolveRoutingRequest {
    schema: String,
    registry: DaemonRegistryProjection,
    route: ProjectRouteKey,
    native_session_id: String,
}

async fn resolve_routing(
    Json(request): Json<ResolveRoutingRequest>,
) -> Result<Json<DaemonRoutingAuthority>, (StatusCode, Json<Value>)> {
    if request.schema != "focusa.daemon_routing_resolve.v1" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "unsupported_schema",
                "expected": "focusa.daemon_routing_resolve.v1"
            })),
        ));
    }
    Ok(Json(project_routing_authority(
        &request.registry,
        &request.route,
        &request.native_session_id,
    )))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/daemon-routing/resolve", post(resolve_routing))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_unknown_contract_schema() {
        let result = resolve_routing(Json(ResolveRoutingRequest {
            schema: "foreign".into(),
            registry: DaemonRegistryProjection::default(),
            route: ProjectRouteKey {
                project_root: "/srv/project".into(),
                continuity_id: "continuity".into(),
                working_subpath_id: "working-subpath:main".into(),
            },
            native_session_id: "session".into(),
        }))
        .await;
        assert_eq!(result.unwrap_err().0, StatusCode::BAD_REQUEST);
    }
}
