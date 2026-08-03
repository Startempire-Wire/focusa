use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::post};
use focusa_core::{
    convergence_platform::build_platform_convergence_plan,
    installation_convergence::{
        CurrentInstallationState, DesiredInstallationState, InstallationEnrollment,
        plan_convergence,
    },
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub fn router() -> Router<Arc<crate::server::AppState>> {
    Router::new().route("/v1/installations/convergence/plan", post(plan))
}

#[derive(Debug, Deserialize)]
struct ConvergencePlanRequest {
    enrollment: InstallationEnrollment,
    desired: DesiredInstallationState,
    current: CurrentInstallationState,
}

async fn plan(Json(request): Json<ConvergencePlanRequest>) -> impl IntoResponse {
    let plan = match plan_convergence(&request.enrollment, &request.desired, &request.current) {
        Ok(plan) => plan,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "schema": "focusa.convergence_plan_response.v1",
                    "status": "blocked",
                    "failure_class": "invalid_convergence_authority",
                    "reason": error.to_string(),
                })),
            );
        }
    };
    let platform_plan = match build_platform_convergence_plan(&plan) {
        Ok(platform_plan) => platform_plan,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "schema": "focusa.convergence_plan_response.v1",
                    "status": "blocked",
                    "failure_class": "unsupported_platform_surface",
                    "reason": error.to_string(),
                })),
            );
        }
    };
    (
        StatusCode::OK,
        Json(json!({
            "schema": "focusa.convergence_plan_response.v1",
            "status": "planned",
            "mutation_executed": false,
            "plan": plan,
            "platform_plan": platform_plan,
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::installation_convergence::{
        InstallationPlatform, InstalledSurfaceState, ManagedSurface, SurfaceHealth,
    };
    use std::collections::{BTreeMap, BTreeSet};

    #[tokio::test]
    async fn planning_endpoint_never_executes_mutation() {
        let response = plan(Json(ConvergencePlanRequest {
            enrollment: InstallationEnrollment {
                installation_id: "install-1".into(),
                operator_id: "operator-1".into(),
                host_id: "host-1".into(),
                platform: InstallationPlatform::WindowsX64,
                channel: "stable".into(),
                enrolled_surfaces: BTreeSet::from([ManagedSurface::Cli]),
                authority_signature_ref: "signature:1".into(),
                generation: 1,
                revoked: false,
            },
            desired: DesiredInstallationState {
                installation_id: "install-1".into(),
                generation: 3,
                version: "0.9.144".into(),
                channel: "stable".into(),
                surfaces: BTreeSet::from([ManagedSurface::Cli]),
                artifact_manifest_digest: "sha256:manifest".into(),
                operator_approval_ref: "approval:1".into(),
            },
            current: CurrentInstallationState {
                installation_id: "install-1".into(),
                generation: 2,
                channel: "stable".into(),
                surfaces: BTreeMap::from([(
                    ManagedSurface::Cli,
                    InstalledSurfaceState {
                        version: "0.9.143".into(),
                        artifact_digest: "sha256:old".into(),
                        health: SurfaceHealth::Healthy,
                    },
                )]),
            },
        }))
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
