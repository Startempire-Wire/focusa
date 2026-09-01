//! Credential authority HTTP surface (#299 slice 3).
//!
//! Secret-free by construction: the route never sees or returns secret
//! values — only requirement verdicts, grant lifecycle states, and
//! redacted provider descriptors. The provider adapter seam consumes
//! these verdicts before any use.

use axum::Json;
use focusa_core::credential_authority::{
    CredentialRequirement, CredentialUseGrant, grant_state, verify_requirement,
};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
pub struct VerifyBody {
    requirement: CredentialRequirement,
    #[serde(default)]
    grants: Vec<CredentialUseGrant>,
    #[serde(default = "default_now")]
    now: String,
}

fn default_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(serde::Deserialize)]
pub struct GrantStateBody {
    grant: CredentialUseGrant,
    #[serde(default = "default_now")]
    now: String,
}

pub async fn verify(Json(body): Json<VerifyBody>) -> Json<Value> {
    let verdict = verify_requirement(&body.requirement, &body.grants, &body.now);
    Json(json!({
        "status": "ok",
        "satisfied": verdict.satisfied,
        "reasons": verdict.reasons,
    }))
}

pub async fn grant_status(Json(body): Json<GrantStateBody>) -> Json<Value> {
    let state = grant_state(&body.grant, &body.now);
    let redacted = body.grant.credential_role_ref;
    Json(json!({
        "status": "ok",
        "state": state,
        "credential_role_ref": redacted,
    }))
}

pub async fn providers() -> Json<Value> {
    // The provider adapter seam registers descriptors through the ledger;
    // this projection lists the redaction-guarded model shapes only.
    Json(json!({
        "status": "ok",
        "providers": [],
        "note": "provider registry is ledger-backed; descriptors are registered by the adapter seam",
    }))
}

pub fn router<S>() -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    axum::Router::new()
        .route(
            "/v1/credentials/verify-requirement",
            axum::routing::post(verify),
        )
        .route(
            "/v1/credentials/grant-status",
            axum::routing::post(grant_status),
        )
        .route("/v1/credentials/providers", axum::routing::get(providers))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn verify_route_returns_typed_denial_instead_of_not_found() {
        let app: axum::Router = router();
        let request = Request::builder()
            .method("POST")
            .uri("/v1/credentials/verify-requirement")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "requirement": {
                        "schema": "focusa.credential_requirement.v1",
                        "requirement_id": "req-epwa-provider-sync",
                        "project_scope_ref": "focusa-dev-homepage",
                        "workstream_ref": "workstream:epwa",
                        "callgraph_frame_ref": "frame:public-delivery",
                        "attempt_generation": 1,
                        "credential_role_ref": "role:focusa-provider-read",
                        "required_operation": "use",
                        "required_exposure_mode": "token_file",
                        "exact_target_refs": ["focusa-daemon:provider-read"],
                        "exact_consumer_ref": "uiai-engine:epwa-provider-sync",
                        "required_auth_challenge_support": [],
                        "precondition_refs": [],
                        "validity_minimum_seconds": 60,
                        "use_count_required": 1,
                        "evidence_requirement_refs": []
                    },
                    "grants": [],
                    "now": "2026-09-01T00:00:00Z"
                })
                .to_string(),
            ))
            .expect("credential verify request");

        let response = app
            .oneshot(request)
            .await
            .expect("credential verify response");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("credential verify body");
        let body: Value = serde_json::from_slice(&bytes).expect("credential verify JSON");
        assert_eq!(body["status"], "ok");
        assert_eq!(body["satisfied"], false);
        assert_eq!(body["reasons"], json!(["no grant matches the requirement"]));
    }
}
