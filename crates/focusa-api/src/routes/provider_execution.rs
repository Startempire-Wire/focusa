//! Provider-neutral governance contract and conformance routes (Spec 135 P1).

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use focusa_core::provider_execution::{
    ProviderExecutionRequest, evaluate_provider_request, supported_provider_contracts,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::agent_capabilities::registered_operation_ids;
use crate::server::AppState;
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
pub struct ProviderScopeQuery {
    project_root: String,
    continuity_id: String,
    attachment_id: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/providers/contracts", get(list_contracts))
        .route("/v1/providers/conformance", post(evaluate_conformance))
}

fn invalid(message: impl Into<String>) -> axum::response::Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({
            "schema": "focusa.error.v1",
            "code": "provider_contract_violation",
            "message": message.into(),
            "recovery": "Supply exact scope, explicit permission, idempotency, Receipt, and a generated Operation Registry operation.",
        })),
    )
        .into_response()
}

pub async fn list_contracts(
    State(_state): State<Arc<AppState>>,
    Query(scope): Query<ProviderScopeQuery>,
) -> axum::response::Response {
    if scope.project_root.trim().is_empty()
        || scope.continuity_id.trim().is_empty()
        || scope.attachment_id.trim().is_empty()
    {
        return invalid("exact project_root, continuity_id, and attachment_id are required");
    }
    Json(json!({
        "schema": "focusa.provider_contract_list.v1",
        "scope": scope,
        "contracts": supported_provider_contracts(),
        "parity": {
            "exact_scope_required": true,
            "permission_required": true,
            "idempotency_required": true,
            "receipt_required": true,
            "operation_registry_required": true,
            "direct_canonical_mutation_allowed": false,
        }
    }))
    .into_response()
}

pub async fn evaluate_conformance(
    State(_state): State<Arc<AppState>>,
    Query(scope): Query<ProviderScopeQuery>,
    Json(request): Json<ProviderExecutionRequest>,
) -> axum::response::Response {
    if request.scope.project_root != scope.project_root
        || request.scope.continuity_id != scope.continuity_id
        || request.scope.attachment_id != scope.attachment_id
    {
        return invalid("query scope must exactly match the provider execution envelope");
    }
    let result = evaluate_provider_request(&request, &registered_operation_ids());
    let status = if result.conformant {
        StatusCode::OK
    } else {
        StatusCode::UNPROCESSABLE_ENTITY
    };
    (
        status,
        Json(json!({
            "schema": "focusa.provider_conformance_response.v1",
            "result": result,
            "execution_performed": false,
            "canonical_state_mutated": false,
            "evidence_ref": format!("evidence:provider-conformance:{}", request.idempotency_key),
            "tool_result": {
                "status": if status == StatusCode::OK { "completed" } else { "blocked" },
                "summary": if status == StatusCode::OK { "Provider request conforms to mandatory governance gates." } else { "Provider request cannot execute because governance conformance failed." },
                "next_action": if status == StatusCode::OK { "Dispatch through the provider's registered adapter; capture its governed mutation Receipt." } else { "Correct every violation and retry with the same bounded intent." },
            }
        })),
    )
        .into_response()
}
