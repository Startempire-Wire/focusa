//! Credential authority HTTP surface (#299 slice 3).
//!
//! Secret-free by construction: the route never sees or returns secret
//! values — only requirement verdicts, grant lifecycle states, and
//! redacted provider descriptors. The provider adapter seam consumes
//! these verdicts before any use.

use axum::Json;
use axum::extract::State;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::server::AppState;
use focusa_core::credential_authority::{
    CredentialRequirement, CredentialUseGrant, grant_state, verify_requirement,
};

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

pub async fn verify(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<VerifyBody>,
) -> Json<Value> {
    let verdict = verify_requirement(&body.requirement, &body.grants, &body.now);
    Json(json!({
        "status": "ok",
        "satisfied": verdict.satisfied,
        "reasons": verdict.reasons,
    }))
}

pub async fn grant_status(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<GrantStateBody>,
) -> Json<Value> {
    let state = grant_state(&body.grant, &body.now);
    let redacted = body.grant.credential_role_ref;
    Json(json!({
        "status": "ok",
        "state": state,
        "credential_role_ref": redacted,
    }))
}

pub async fn providers(State(_state): State<Arc<AppState>>) -> Json<Value> {
    // The provider adapter seam registers descriptors through the ledger;
    // this projection lists the redaction-guarded model shapes only.
    Json(json!({
        "status": "ok",
        "providers": [],
        "note": "provider registry is ledger-backed; descriptors are registered by the adapter seam",
    }))
}

pub fn router() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/v1/credentials/verify-requirement", axum::routing::post(verify))
        .route("/v1/credentials/grant-status", axum::routing::post(grant_status))
        .route("/v1/credentials/providers", axum::routing::get(providers))
}
