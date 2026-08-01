//! Spec 144 semantic-integrity operation registry and bounded API projection.
//!
//! This surface deliberately distinguishes registered schema from executable core
//! integration. An operation is never reported as successful unless this daemon
//! actually owns the implementation.

use super::semantic_integrity_executor;
use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

pub(super) const CONTRACT: &str = "focusa.semantic_integrity.operation.v1";
const MAX_PAGE: u16 = 100;
const ARTIFACT_REGISTRY_JSON: &str =
    include_str!("../../../../docs/contracts/spec144/semantic-artifact-registry-v1.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExactScope {
    pub project_root: String,
    pub continuity_id: String,
}

impl ExactScope {
    fn valid(&self) -> bool {
        !self.project_root.trim().is_empty() && !self.continuity_id.trim().is_empty()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Supported,
    SchemaOnly,
    Unsupported,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Read,
    Mutation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationDescriptor {
    pub operation_id: &'static str,
    pub family: &'static str,
    pub kind: OperationKind,
    pub availability: Availability,
    pub exact_scope_required: bool,
    pub idempotency_required: bool,
    pub confirmation_required: bool,
    pub evidence_refs: bool,
    pub receipt_refs: bool,
}

macro_rules! read_op {
    ($id:literal, $family:literal, $availability:ident) => {
        OperationDescriptor {
            operation_id: $id,
            family: $family,
            kind: OperationKind::Read,
            availability: Availability::$availability,
            exact_scope_required: true,
            idempotency_required: false,
            confirmation_required: false,
            evidence_refs: true,
            receipt_refs: true,
        }
    };
}
macro_rules! mutation_op {
    ($id:literal, $family:literal) => {
        OperationDescriptor {
            operation_id: $id,
            family: $family,
            kind: OperationKind::Mutation,
            availability: Availability::SchemaOnly,
            exact_scope_required: true,
            idempotency_required: true,
            confirmation_required: true,
            evidence_refs: true,
            receipt_refs: true,
        }
    };
}

/// The API, CLI and static grounding tests consume these stable identifiers.
pub const OPERATIONS: &[OperationDescriptor] = &[
    read_op!("semantic.integrity.status", "status", Supported),
    read_op!("semantic.integrity.registry", "registry", Supported),
    read_op!("semantic.integrity.artifact.list", "artifact", SchemaOnly),
    read_op!("semantic.integrity.artifact.get", "artifact", SchemaOnly),
    read_op!("semantic.integrity.validate", "validation", SchemaOnly),
    read_op!(
        "semantic.integrity.reason.preview",
        "validation",
        SchemaOnly
    ),
    read_op!(
        "semantic.integrity.reason.explain",
        "validation",
        SchemaOnly
    ),
    read_op!("semantic.integrity.receipt.get", "receipt", SchemaOnly),
    mutation_op!("semantic_pair.create", "build"),
    read_op!("semantic_pair.get", "status", SchemaOnly),
    mutation_op!("semantic_pair.pause", "build"),
    mutation_op!("semantic_pair.resume", "build"),
    mutation_op!("semantic_pair.cancel", "build"),
    read_op!("semantic_pair.contract.preview", "build", SchemaOnly),
    mutation_op!("semantic_pair.contract.commit", "build"),
    mutation_op!("semantic_pair.builder.start", "build"),
    mutation_op!("semantic_pair.builder.claim", "build"),
    mutation_op!("semantic_pair.builder.respond", "build"),
    mutation_op!("semantic_pair.builder.repair", "build"),
    mutation_op!("semantic_pair.snapshot.freeze", "build"),
    read_op!("semantic_pair.snapshot.get", "artifact", SchemaOnly),
    mutation_op!("semantic_pair.obligations.compile", "validation"),
    read_op!(
        "semantic_pair.verification.plan.preview",
        "verify",
        SchemaOnly
    ),
    mutation_op!("semantic_pair.verification.plan.commit", "verify"),
    mutation_op!("semantic_pair.verify.start", "verify"),
    read_op!("semantic_pair.verify.findings", "verify", SchemaOnly),
    read_op!("semantic_pair.verify.verdict", "verify", SchemaOnly),
    mutation_op!("semantic_pair.finding.respond", "verify"),
    mutation_op!("semantic_pair.finding.resolve", "verify"),
    read_op!("semantic_pair.settlement.preview", "settlement", SchemaOnly),
    mutation_op!("semantic_pair.settlement.commit", "settlement"),
    read_op!("semantic_pair.receipt.get", "receipt", SchemaOnly),
    mutation_op!("semantic_pair.replay", "replay"),
    read_op!("semantic_pair.eval", "validation", SchemaOnly),
    read_op!("semantic_pair.migration.status", "migration", SchemaOnly),
    mutation_op!("semantic_pair.migration.run", "migration"),
    read_op!("semantic_pair.rollback.preview", "rollback", SchemaOnly),
    mutation_op!("semantic_pair.rollback.commit", "rollback"),
    read_op!("vertical.bundle.validate", "vertical", SchemaOnly),
    read_op!("vertical.bundle.preview", "vertical", SchemaOnly),
    mutation_op!("vertical.bundle.activate", "vertical"),
    read_op!("vertical.bundle.conformance", "vertical", SchemaOnly),
    read_op!("semantic.reflex.visibility", "reflex", SchemaOnly),
];

#[derive(Debug, Deserialize)]
struct PageQuery {
    project_root: String,
    continuity_id: String,
    cursor: Option<u32>,
    limit: Option<u16>,
    family: Option<String>,
}

#[derive(Debug, Serialize)]
struct Page<T> {
    contract: &'static str,
    scope: ExactScope,
    items: Vec<T>,
    next_cursor: Option<String>,
    limit: u16,
    degraded: bool,
    degraded_reason: Option<&'static str>,
    evidence_refs: Vec<String>,
    receipt_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OperationRequest {
    pub contract: Option<String>,
    pub operation_id: String,
    pub scope: ExactScope,
    #[serde(default)]
    pub payload: Value,
    pub idempotency_key: Option<String>,
    pub confirmation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationResult {
    pub contract: &'static str,
    pub operation_id: String,
    pub scope: ExactScope,
    pub state: Availability,
    pub degraded: bool,
    pub message: String,
    pub data: Value,
    pub evidence_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
    pub observed_at: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/semantic-integrity/status", get(status))
        .route("/v1/semantic-integrity/operations", get(registry))
        .route("/v1/semantic-integrity/artifacts", get(artifacts))
        .route(
            "/v1/semantic-integrity/artifacts/{artifact_id}",
            get(artifact),
        )
        .route(
            "/v1/semantic-integrity/operations/{operation_id}",
            post(invoke),
        )
}

#[allow(clippy::result_large_err)]
fn checked_scope(q: &PageQuery) -> Result<ExactScope, Response> {
    let scope = ExactScope {
        project_root: q.project_root.clone(),
        continuity_id: q.continuity_id.clone(),
    };
    if scope.valid() {
        Ok(scope)
    } else {
        Err(problem(
            StatusCode::BAD_REQUEST,
            "exact project_root and continuity_id are required",
        ))
    }
}

async fn status(Query(q): Query<PageQuery>) -> Response {
    let scope = match checked_scope(&q) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Json(OperationResult {
        contract: CONTRACT,
        operation_id: "semantic.integrity.status".into(), scope,
        state: Availability::Supported, degraded: false,
        message: "durable semantic event persistence, replay, migration, and settlement preview are integrated".into(),
        data: json!({"registered_operations": OPERATIONS.len(), "executable_operations": OPERATIONS.iter().filter(|op| semantic_integrity_executor::operation_is_executable(op.operation_id)).count()}),
        evidence_refs: vec!["operation-registry:semantic-integrity:v1".into()], receipt_refs: vec![],
        observed_at: Utc::now().to_rfc3339(),
    }).into_response()
}

async fn registry(Query(q): Query<PageQuery>) -> Response {
    let scope = match checked_scope(&q) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let limit = q.limit.unwrap_or(50).clamp(1, MAX_PAGE);
    let offset = q.cursor.unwrap_or(0) as usize;
    let filtered: Vec<_> = OPERATIONS
        .iter()
        .filter(|op| q.family.as_deref().is_none_or(|f| op.family == f))
        .cloned()
        .map(|mut op| {
            if semantic_integrity_executor::operation_is_executable(op.operation_id) {
                op.availability = Availability::Supported;
            }
            op
        })
        .collect();
    let items = filtered
        .iter()
        .skip(offset)
        .take(limit as usize)
        .cloned()
        .collect::<Vec<_>>();
    let next = (offset + items.len() < filtered.len()).then(|| (offset + items.len()).to_string());
    Json(Page {
        contract: CONTRACT,
        scope,
        items,
        next_cursor: next,
        limit,
        degraded: false,
        degraded_reason: None,
        evidence_refs: vec!["operation-registry:semantic-integrity:v1".into()],
        receipt_refs: vec![],
    })
    .into_response()
}

async fn artifacts(Query(q): Query<PageQuery>) -> Response {
    let scope = match checked_scope(&q) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let limit = q.limit.unwrap_or(50).clamp(1, MAX_PAGE);
    let offset = q.cursor.unwrap_or(0) as usize;
    let registry: Value = serde_json::from_str(ARTIFACT_REGISTRY_JSON)
        .expect("embedded Spec144 artifact registry is valid JSON");
    let artifacts = registry["artifacts"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let items = artifacts
        .iter()
        .skip(offset)
        .take(limit as usize)
        .cloned()
        .collect::<Vec<_>>();
    let next_cursor =
        (offset + items.len() < artifacts.len()).then(|| (offset + items.len()).to_string());
    Json(Page::<Value> {
        contract: CONTRACT,
        scope,
        items,
        next_cursor,
        limit,
        degraded: false,
        degraded_reason: None,
        evidence_refs: vec!["embedded-registry:spec144:v1".into()],
        receipt_refs: vec![],
    })
    .into_response()
}

async fn artifact(Path(artifact_id): Path<String>, Query(q): Query<PageQuery>) -> Response {
    let scope = match checked_scope(&q) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let registry: Value = serde_json::from_str(ARTIFACT_REGISTRY_JSON)
        .expect("embedded Spec144 artifact registry is valid JSON");
    let artifact = registry["artifacts"].as_array().and_then(|items| {
        items.iter().find(|item| {
            item["path"].as_str().is_some_and(|path| {
                path == artifact_id || path.rsplit('/').next() == Some(artifact_id.as_str())
            }) || item["sha256"].as_str() == Some(artifact_id.as_str())
        })
    });
    let Some(artifact) = artifact else {
        return problem(StatusCode::NOT_FOUND, "semantic artifact is not registered");
    };
    Json(OperationResult {
        contract: CONTRACT,
        operation_id: "semantic.integrity.artifact.get".into(),
        scope,
        state: Availability::Supported,
        degraded: false,
        message: "registered semantic artifact resolved from the embedded signed release registry"
            .into(),
        data: artifact.clone(),
        evidence_refs: vec!["embedded-registry:spec144:v1".into()],
        receipt_refs: vec![],
        observed_at: Utc::now().to_rfc3339(),
    })
    .into_response()
}

async fn invoke(
    State(state): State<Arc<AppState>>,
    Path(path_id): Path<String>,
    Json(req): Json<OperationRequest>,
) -> Response {
    if path_id != req.operation_id {
        return problem(
            StatusCode::BAD_REQUEST,
            "path and envelope operation_id must match",
        );
    }
    if !req.scope.valid() {
        return problem(
            StatusCode::BAD_REQUEST,
            "exact project_root and continuity_id are required",
        );
    }
    if req.contract.as_deref().is_some_and(|v| v != CONTRACT) {
        return problem(StatusCode::BAD_REQUEST, "unsupported request contract");
    }
    let Some(op) = OPERATIONS.iter().find(|op| op.operation_id == path_id) else {
        return problem(StatusCode::NOT_FOUND, "operation_id is not registered");
    };
    if op.kind == OperationKind::Mutation
        && req.idempotency_key.as_deref().is_none_or(str::is_empty)
    {
        return problem(
            StatusCode::PRECONDITION_REQUIRED,
            "mutation requires a non-empty idempotency_key",
        );
    }
    if op.confirmation_required && req.confirmation.as_deref() != Some("confirm") {
        return problem(
            StatusCode::PRECONDITION_REQUIRED,
            "mutation requires confirmation=confirm",
        );
    }
    if let Some(response) = semantic_integrity_executor::execute(state, &req).await {
        return response;
    }
    let message =
        "operation has a stable schema but no integrated executor; no mutation was performed";
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(OperationResult {
            contract: CONTRACT,
            operation_id: path_id,
            scope: req.scope,
            state: op.availability,
            degraded: true,
            message: message.into(),
            data: json!({"accepted_payload": false}),
            evidence_refs: vec![],
            receipt_refs: vec![],
            observed_at: Utc::now().to_rfc3339(),
        }),
    )
        .into_response()
}

fn problem(status: StatusCode, message: &'static str) -> Response {
    (status, Json(json!({"contract": CONTRACT, "error": message, "degraded": true, "evidence_refs": [], "receipt_refs": []}))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn operation_ids_are_unique_stable_and_ground_all_families() {
        let ids: HashSet<_> = OPERATIONS.iter().map(|op| op.operation_id).collect();
        assert_eq!(ids.len(), OPERATIONS.len());
        for family in [
            "status",
            "artifact",
            "registry",
            "validation",
            "build",
            "verify",
            "settlement",
            "replay",
            "migration",
            "rollback",
            "vertical",
            "reflex",
        ] {
            assert!(
                OPERATIONS.iter().any(|op| op.family == family),
                "missing {family}"
            );
        }
        assert!(OPERATIONS.iter().all(|op| !op.operation_id.contains(' ')));
    }

    #[test]
    fn every_mutation_is_guarded_and_not_claimed_supported() {
        for op in OPERATIONS
            .iter()
            .filter(|op| op.kind == OperationKind::Mutation)
        {
            assert!(op.idempotency_required && op.confirmation_required);
            assert_ne!(op.availability, Availability::Supported);
        }
    }

    #[test]
    fn operation_envelope_serialization_is_canonical() {
        let value = serde_json::to_value(OperationResult {
            contract: CONTRACT,
            operation_id: "semantic.integrity.validate".into(),
            scope: ExactScope {
                project_root: "/p".into(),
                continuity_id: "c".into(),
            },
            state: Availability::SchemaOnly,
            degraded: true,
            message: "unavailable".into(),
            data: json!({}),
            evidence_refs: vec!["e:1".into()],
            receipt_refs: vec![],
            observed_at: "2026-01-01T00:00:00Z".into(),
        })
        .unwrap();
        assert_eq!(value["state"], "schema_only");
        assert_eq!(value["scope"]["continuity_id"], "c");
        assert_eq!(value["evidence_refs"][0], "e:1");
    }
}
