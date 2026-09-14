//! Legacy credential model-verification HTTP surface (#299/#526).
//!
//! Caller-supplied metadata produces advisory verdicts and lifecycle states,
//! never credential-use authorization. Provider enumeration remains explicitly
//! unsupported until its canonical backing registry is wired.

use axum::{Json, http::StatusCode};
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
        "advisory": true,
        "canonical": false,
        "satisfied": verdict.satisfied,
        "reasons": verdict.reasons,
    }))
}

pub async fn grant_status(Json(body): Json<GrantStateBody>) -> Json<Value> {
    let state = grant_state(&body.grant, &body.now);
    let redacted = body.grant.credential_role_ref;
    Json(json!({
        "status": "ok",
        "advisory": true,
        "canonical": false,
        "state": state,
        "credential_role_ref": redacted,
    }))
}

/// Provider enrollment/custody is not implemented by the pure-verdict routes.
/// Return an explicit unsupported operation, not a successful empty registry.
pub async fn providers() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "status": "unsupported",
            "code": "credential_provider_registry_not_implemented",
            "providers": [],
            "note": "provider registry is not wired; pure requirement verification does not establish provider fulfillment",
            "owner_issue": "https://github.com/Startempire-Wire/focusa/issues/526",
        })),
    )
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
    async fn unavailable_provider_registry_preserves_pure_verification() {
        let app: axum::Router = router();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/v1/credentials/providers")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
        let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["status"], "unsupported");
        assert_eq!(body["code"], "credential_provider_registry_not_implemented");
        assert_eq!(body["providers"], json!([]));
        assert!(!body["note"].as_str().unwrap().contains("ledger-backed"));
        let mut fixture = json!({
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
        });
        let request = Request::builder()
            .method("POST")
            .uri("/v1/credentials/verify-requirement")
            .header("content-type", "application/json")
            .body(Body::from(fixture.to_string()))
            .expect("credential verify request");

        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("credential verify response");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("credential verify body");
        let body: Value = serde_json::from_slice(&bytes).expect("credential verify JSON");
        assert_eq!(body["status"], "ok");
        assert_eq!(body["advisory"], true);
        assert_eq!(body["canonical"], false);
        assert_eq!(body["satisfied"], false);
        assert_eq!(body["reasons"], json!(["no grant matches the requirement"]));

        // A matching caller-owned model still cannot establish credential authority.
        let grant = json!({
            "schema": focusa_core::credential_authority::CREDENTIAL_USE_GRANT_SCHEMA,
            "grant_id": "fixture-grant",
            "credential_role_ref": fixture["requirement"]["credential_role_ref"],
            "operation": "use",
            "exposure_mode": "token_file",
            "exact_target_refs": fixture["requirement"]["exact_target_refs"],
            "consumer_ref": fixture["requirement"]["exact_consumer_ref"],
            "granted_at": fixture["now"],
            "expires_at": "2026-09-02T00:00:00Z",
            "use_count_allowed": 1,
            "use_count_used": 0
        });
        fixture["grants"] = json!([grant.clone()]);
        for (path, input, field, expected) in [
            (
                "/v1/credentials/verify-requirement",
                fixture.clone(),
                "satisfied",
                json!(true),
            ),
            (
                "/v1/credentials/grant-status",
                json!({"grant": grant, "now": fixture["now"]}),
                "state",
                json!("active"),
            ),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(path)
                        .header("content-type", "application/json")
                        .body(Body::from(input.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
            let body: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(body[field], expected);
            assert_eq!(body["advisory"], true);
            assert_eq!(body["canonical"], false);
        }
    }
}
