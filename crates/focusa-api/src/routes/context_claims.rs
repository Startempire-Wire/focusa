use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::{
    tool_result::{FailureClass, ToolResultV1, ToolStatus},
    types::{
        Action, ContextClaimRecord, ContextContradictionRecord, ContextDecisionRecord, FocusaEvent,
        FocusaState, ReactiveContextProjection,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Duration};

type ApiError = (StatusCode, Json<Box<ToolResultV1>>);
const OPERATION_ID: &str = "focusa.context.graph.mutate";

#[derive(Debug, Clone, Deserialize)]
pub struct ContextGraphScope {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ContextGraphMutationRequest {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    pub idempotency_key: String,
    pub expected_state_version: u64,
    pub action: String,
    #[serde(default)]
    pub claim_id: Option<String>,
    #[serde(default)]
    pub claim: Option<String>,
    #[serde(default)]
    pub source_citation_refs: Vec<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub supersedes_claim_id: Option<String>,
    #[serde(default)]
    pub review_outcome: Option<String>,
    #[serde(default)]
    pub contradiction_id: Option<String>,
    #[serde(default)]
    pub left_claim_id: Option<String>,
    #[serde(default)]
    pub right_claim_id: Option<String>,
    #[serde(default)]
    pub resolution: Option<String>,
    #[serde(default)]
    pub selected_claim_id: Option<String>,
    #[serde(default)]
    pub actor: Option<String>,
    #[serde(default)]
    pub rationale: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ContextGraphResponse {
    pub schema: &'static str,
    pub canonical: bool,
    pub replayed: bool,
    pub state_version: u64,
    pub claims: Vec<ContextClaimRecord>,
    pub contradictions: Vec<ContextContradictionRecord>,
    pub decisions: Vec<ContextDecisionRecord>,
    pub projection: ReactiveContextProjection,
    pub evidence_ref: String,
    pub receipt_ref: String,
    pub tool_result: ToolResultV1,
}

#[derive(Debug, Serialize)]
pub struct ContextGraphReadResponse {
    pub schema: &'static str,
    pub canonical: bool,
    pub state_version: u64,
    pub claims: Vec<ContextClaimRecord>,
    pub contradictions: Vec<ContextContradictionRecord>,
    pub decisions: Vec<ContextDecisionRecord>,
    pub projection: ReactiveContextProjection,
}

fn failure(
    status: StatusCode,
    tool_status: ToolStatus,
    class: FailureClass,
    summary: impl Into<String>,
) -> ApiError {
    let mut result = ToolResultV1::failure(tool_status, class, summary);
    result.tool = Some("focusa_context_graph_mutate".to_string());
    result.family = Some("context".to_string());
    result.endpoint = Some("/v1/context/graph/mutate".to_string());
    result.next_tools = vec![
        "focusa_context_graph_read".to_string(),
        "focusa_context_retrieve".to_string(),
    ];
    (status, Json(Box::new(result)))
}

fn required(value: Option<&str>, field: &str, max: usize) -> Result<String, ApiError> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() || value.len() > max {
        return Err(failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            format!("{field} must contain 1-{max} characters"),
        ));
    }
    Ok(value.to_string())
}

fn validate_scope(scope: ContextGraphScope) -> Result<ContextGraphScope, ApiError> {
    Ok(ContextGraphScope {
        project_root: required(Some(&scope.project_root), "project_root", 4096)?,
        continuity_id: required(Some(&scope.continuity_id), "continuity_id", 256)?,
        attachment_id: required(Some(&scope.attachment_id), "attachment_id", 256)?,
    })
}

fn stable_ref(prefix: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part.as_bytes());
    }
    format!("{prefix}:{}", hex::encode(&hasher.finalize()[..12]))
}

fn same_scope_claim(claim: &ContextClaimRecord, scope: &ContextGraphScope) -> bool {
    claim.project_root == scope.project_root
        && claim.continuity_id == scope.continuity_id
        && claim.attachment_id == scope.attachment_id
}

fn same_scope_edge(edge: &ContextContradictionRecord, scope: &ContextGraphScope) -> bool {
    edge.project_root == scope.project_root
        && edge.continuity_id == scope.continuity_id
        && edge.attachment_id == scope.attachment_id
}

fn same_scope_decision(decision: &ContextDecisionRecord, scope: &ContextGraphScope) -> bool {
    decision.project_root == scope.project_root
        && decision.continuity_id == scope.continuity_id
        && decision.attachment_id == scope.attachment_id
}

fn graph_parts(
    state: &FocusaState,
    scope: &ContextGraphScope,
) -> (
    Vec<ContextClaimRecord>,
    Vec<ContextContradictionRecord>,
    Vec<ContextDecisionRecord>,
    ReactiveContextProjection,
) {
    let mut claims: Vec<_> = state
        .context_claims
        .iter()
        .filter(|claim| same_scope_claim(claim, scope))
        .cloned()
        .collect();
    claims.sort_by(|left, right| left.claim_id.cmp(&right.claim_id));
    let mut contradictions: Vec<_> = state
        .context_contradictions
        .iter()
        .filter(|edge| same_scope_edge(edge, scope))
        .cloned()
        .collect();
    contradictions.sort_by(|left, right| left.contradiction_id.cmp(&right.contradiction_id));
    let mut decisions: Vec<_> = state
        .context_decisions
        .iter()
        .filter(|decision| same_scope_decision(decision, scope))
        .cloned()
        .collect();
    decisions.sort_by(|left, right| left.decision_id.cmp(&right.decision_id));
    let projection = state
        .reactive_context
        .iter()
        .find(|projection| {
            projection.project_root == scope.project_root
                && projection.continuity_id == scope.continuity_id
                && projection.attachment_id == scope.attachment_id
        })
        .cloned()
        .unwrap_or_else(|| ReactiveContextProjection {
            project_root: scope.project_root.clone(),
            continuity_id: scope.continuity_id.clone(),
            attachment_id: scope.attachment_id.clone(),
            revision: state.version,
            ..Default::default()
        });
    (claims, contradictions, decisions, projection)
}

fn response_from_state(
    state: &FocusaState,
    scope: &ContextGraphScope,
    idempotency_key: &str,
    action: &str,
    replayed: bool,
) -> ContextGraphResponse {
    let (claims, contradictions, decisions, projection) = graph_parts(state, scope);
    let evidence_ref = stable_ref(
        "evidence:context-graph",
        &[
            &scope.project_root,
            &scope.continuity_id,
            &scope.attachment_id,
            action,
            idempotency_key,
        ],
    );
    let receipt_ref = stable_ref(
        "receipt:context-graph",
        &[
            &scope.project_root,
            &scope.continuity_id,
            &scope.attachment_id,
            action,
            idempotency_key,
        ],
    );
    let status = if replayed {
        ToolStatus::NoOp
    } else {
        ToolStatus::Completed
    };
    let mut tool_result = ToolResultV1::success(
        status,
        if replayed {
            "Context graph mutation replayed idempotently"
        } else {
            "Context graph mutation committed to canonical reducer state"
        },
    );
    tool_result.tool = Some("focusa_context_graph_mutate".to_string());
    tool_result.family = Some("context".to_string());
    tool_result.endpoint = Some("/v1/context/graph/mutate".to_string());
    tool_result.side_effects = if replayed {
        Vec::new()
    } else {
        vec!["canonical_context_graph_updated".to_string()]
    };
    tool_result.evidence_refs = vec![evidence_ref.clone()];
    tool_result.next_tools = vec![
        "focusa_context_graph_read".to_string(),
        "focusa_context_retrieve".to_string(),
    ];
    ContextGraphResponse {
        schema: "focusa.context_graph_mutation_result.v1",
        canonical: true,
        replayed,
        state_version: state.version,
        claims,
        contradictions,
        decisions,
        projection,
        evidence_ref,
        receipt_ref,
        tool_result,
    }
}

async fn read_graph(
    State(state): State<Arc<AppState>>,
    Query(scope): Query<ContextGraphScope>,
) -> Result<Json<ContextGraphReadResponse>, ApiError> {
    let scope = validate_scope(scope)?;
    let state = state.focusa.read().await;
    let (claims, contradictions, decisions, projection) = graph_parts(&state, &scope);
    Ok(Json(ContextGraphReadResponse {
        schema: "focusa.context_graph.v1",
        canonical: true,
        state_version: state.version,
        claims,
        contradictions,
        decisions,
        projection,
    }))
}

async fn mutate_graph(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ContextGraphMutationRequest>,
) -> Result<Json<ContextGraphResponse>, ApiError> {
    let scope = validate_scope(ContextGraphScope {
        project_root: request.project_root,
        continuity_id: request.continuity_id,
        attachment_id: request.attachment_id,
    })?;
    let idempotency_key = required(Some(&request.idempotency_key), "idempotency_key", 256)?;
    let action = required(Some(&request.action), "action", 64)?;
    let actor = request.actor.as_deref().unwrap_or("operator").trim();
    let rationale = request.rationale.as_deref().unwrap_or("").trim();

    let _writer = state.write_serial_lock.lock().await;
    let snapshot = state.focusa.read().await.clone();
    if let Some(existing) = replay_match(&snapshot, &scope, &idempotency_key, &action) {
        return Ok(Json(response_from_state(
            &snapshot,
            &scope,
            &idempotency_key,
            &action,
            existing,
        )));
    }
    if snapshot.version != request.expected_state_version {
        return Err(failure(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            format!(
                "expected_state_version={} does not match canonical version={}",
                request.expected_state_version, snapshot.version
            ),
        ));
    }

    let now = Utc::now();
    let receipt_ref = stable_ref(
        "receipt:context-graph",
        &[
            &scope.project_root,
            &scope.continuity_id,
            &scope.attachment_id,
            &action,
            &idempotency_key,
        ],
    );
    let event = match action.as_str() {
        "propose_claim" => {
            let claim_text = required(request.claim.as_deref(), "claim", 4096)?;
            if request.source_citation_refs.is_empty() || request.source_citation_refs.len() > 32 {
                return Err(failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "source_citation_refs must contain 1-32 references",
                ));
            }
            let confidence = request.confidence.unwrap_or(0.5);
            if !(0.0..=1.0).contains(&confidence) || !confidence.is_finite() {
                return Err(failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "confidence must be finite and between 0 and 1",
                ));
            }
            FocusaEvent::ContextClaimProposed {
                claim: ContextClaimRecord {
                    claim_id: stable_ref(
                        "context-claim",
                        &[
                            &scope.project_root,
                            &scope.continuity_id,
                            &scope.attachment_id,
                            &idempotency_key,
                            &claim_text,
                        ],
                    ),
                    project_root: scope.project_root.clone(),
                    continuity_id: scope.continuity_id.clone(),
                    attachment_id: scope.attachment_id.clone(),
                    claim: claim_text,
                    source_citation_refs: request.source_citation_refs,
                    confidence,
                    status: "candidate".to_string(),
                    contradiction_refs: Vec::new(),
                    reviewed_by: None,
                    reviewed_at: None,
                    supersedes_claim_id: request.supersedes_claim_id,
                    idempotency_key: idempotency_key.clone(),
                    revision: 1,
                    committed_at: now,
                },
            }
        }
        "review_claim" => {
            let claim_id = required(request.claim_id.as_deref(), "claim_id", 256)?;
            let outcome = required(request.review_outcome.as_deref(), "review_outcome", 32)?;
            if !matches!(outcome.as_str(), "accept" | "reject") {
                return Err(failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "review_outcome must be accept or reject",
                ));
            }
            let mut claim = snapshot
                .context_claims
                .iter()
                .find(|claim| claim.claim_id == claim_id && same_scope_claim(claim, &scope))
                .cloned()
                .ok_or_else(|| {
                    failure(
                        StatusCode::NOT_FOUND,
                        ToolStatus::Blocked,
                        FailureClass::NotFound,
                        "Context claim not found in exact scope",
                    )
                })?;
            claim.revision += 1;
            claim.status = if outcome == "accept" {
                "accepted"
            } else {
                "rejected"
            }
            .to_string();
            claim.reviewed_by = Some(required(Some(actor), "actor", 256)?);
            claim.reviewed_at = Some(now);
            claim.committed_at = now;
            let decision_id = stable_ref(
                "context-decision",
                &[
                    &scope.project_root,
                    &scope.continuity_id,
                    &scope.attachment_id,
                    &idempotency_key,
                    &action,
                ],
            );
            FocusaEvent::ContextClaimReviewed {
                claim,
                decision: ContextDecisionRecord {
                    decision_id,
                    project_root: scope.project_root.clone(),
                    continuity_id: scope.continuity_id.clone(),
                    attachment_id: scope.attachment_id.clone(),
                    decision_kind: "claim_review".to_string(),
                    target_ref: claim_id,
                    outcome,
                    rationale: required(Some(rationale), "rationale", 2048)?,
                    decided_by: actor.to_string(),
                    decided_at: now,
                    evidence_refs: request.source_citation_refs,
                    receipt_ref: receipt_ref.clone(),
                },
            }
        }
        "open_contradiction" => {
            let left_id = required(request.left_claim_id.as_deref(), "left_claim_id", 256)?;
            let right_id = required(request.right_claim_id.as_deref(), "right_claim_id", 256)?;
            if left_id == right_id {
                return Err(failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "contradiction claims must be distinct",
                ));
            }
            let mut claims = Vec::new();
            for claim_id in [&left_id, &right_id] {
                let mut claim = snapshot
                    .context_claims
                    .iter()
                    .find(|claim| claim.claim_id == *claim_id && same_scope_claim(claim, &scope))
                    .cloned()
                    .ok_or_else(|| {
                        failure(
                            StatusCode::NOT_FOUND,
                            ToolStatus::Blocked,
                            FailureClass::NotFound,
                            format!("Context claim not found: {claim_id}"),
                        )
                    })?;
                claim.revision += 1;
                claim.status = "contradicted".to_string();
                claim.committed_at = now;
                claims.push(claim);
            }
            let contradiction_id = stable_ref(
                "context-contradiction",
                &[
                    &scope.project_root,
                    &scope.continuity_id,
                    &scope.attachment_id,
                    &left_id,
                    &right_id,
                ],
            );
            for claim in &mut claims {
                if !claim.contradiction_refs.contains(&contradiction_id) {
                    claim.contradiction_refs.push(contradiction_id.clone());
                    claim.contradiction_refs.sort();
                }
            }
            FocusaEvent::ContextContradictionOpened {
                contradiction: ContextContradictionRecord {
                    contradiction_id,
                    project_root: scope.project_root.clone(),
                    continuity_id: scope.continuity_id.clone(),
                    attachment_id: scope.attachment_id.clone(),
                    left_claim_id: left_id,
                    right_claim_id: right_id,
                    status: "open".to_string(),
                    selected_claim_id: None,
                    resolution: Some(required(Some(rationale), "rationale", 2048)?),
                    resolved_by: None,
                    resolved_at: None,
                    idempotency_key: idempotency_key.clone(),
                    revision: 1,
                    committed_at: now,
                },
                claims,
            }
        }
        "resolve_contradiction" => {
            let contradiction_id =
                required(request.contradiction_id.as_deref(), "contradiction_id", 256)?;
            let resolution = required(request.resolution.as_deref(), "resolution", 32)?;
            if !matches!(
                resolution.as_str(),
                "accept_left" | "accept_right" | "reject_both"
            ) {
                return Err(failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "resolution must be accept_left, accept_right, or reject_both",
                ));
            }
            let mut contradiction = snapshot
                .context_contradictions
                .iter()
                .find(|edge| {
                    edge.contradiction_id == contradiction_id && same_scope_edge(edge, &scope)
                })
                .cloned()
                .ok_or_else(|| {
                    failure(
                        StatusCode::NOT_FOUND,
                        ToolStatus::Blocked,
                        FailureClass::NotFound,
                        "Context contradiction not found in exact scope",
                    )
                })?;
            if contradiction.status != "open" {
                return Err(failure(
                    StatusCode::CONFLICT,
                    ToolStatus::Blocked,
                    FailureClass::WriterConflict,
                    "Context contradiction is already resolved",
                ));
            }
            let selected_claim_id = match resolution.as_str() {
                "accept_left" => Some(contradiction.left_claim_id.clone()),
                "accept_right" => Some(contradiction.right_claim_id.clone()),
                _ => None,
            };
            if request.selected_claim_id.is_some() && request.selected_claim_id != selected_claim_id
            {
                return Err(failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "selected_claim_id does not match resolution",
                ));
            }
            let mut claims = Vec::new();
            for claim_id in [&contradiction.left_claim_id, &contradiction.right_claim_id] {
                let mut claim = snapshot
                    .context_claims
                    .iter()
                    .find(|claim| claim.claim_id == *claim_id)
                    .cloned()
                    .ok_or_else(|| {
                        failure(
                            StatusCode::NOT_FOUND,
                            ToolStatus::Blocked,
                            FailureClass::NotFound,
                            "Contradiction claim missing",
                        )
                    })?;
                claim.revision += 1;
                claim.status = if selected_claim_id.as_ref() == Some(claim_id) {
                    "accepted"
                } else {
                    "rejected"
                }
                .to_string();
                claim.reviewed_by = Some(required(Some(actor), "actor", 256)?);
                claim.reviewed_at = Some(now);
                claim.committed_at = now;
                claims.push(claim);
            }
            contradiction.revision += 1;
            contradiction.status = "resolved".to_string();
            contradiction.selected_claim_id = selected_claim_id;
            contradiction.resolution = Some(resolution.clone());
            contradiction.resolved_by = Some(actor.to_string());
            contradiction.resolved_at = Some(now);
            contradiction.committed_at = now;
            FocusaEvent::ContextContradictionResolved {
                contradiction,
                claims,
                decision: ContextDecisionRecord {
                    decision_id: stable_ref(
                        "context-decision",
                        &[
                            &scope.project_root,
                            &scope.continuity_id,
                            &scope.attachment_id,
                            &idempotency_key,
                            &action,
                        ],
                    ),
                    project_root: scope.project_root.clone(),
                    continuity_id: scope.continuity_id.clone(),
                    attachment_id: scope.attachment_id.clone(),
                    decision_kind: "contradiction_resolution".to_string(),
                    target_ref: contradiction_id,
                    outcome: resolution,
                    rationale: required(Some(rationale), "rationale", 2048)?,
                    decided_by: actor.to_string(),
                    decided_at: now,
                    evidence_refs: request.source_citation_refs,
                    receipt_ref: receipt_ref.clone(),
                },
            }
        }
        _ => {
            return Err(failure(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                "action must be propose_claim, review_claim, open_contradiction, or resolve_contradiction",
            ));
        }
    };

    drop(_writer);
    state
        .command_tx
        .send(Action::EmitEvent { event })
        .await
        .map_err(|_| {
            failure(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "canonical Context graph command channel unavailable",
            )
        })?;
    for _ in 0..100 {
        let current = state.focusa.read().await;
        if current.version > snapshot.version {
            return Ok(Json(response_from_state(
                &current,
                &scope,
                &idempotency_key,
                &action,
                false,
            )));
        }
        drop(current);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(failure(
        StatusCode::SERVICE_UNAVAILABLE,
        ToolStatus::Degraded,
        FailureClass::ReadModelLag,
        "Context graph mutation dispatched but canonical read model did not advance",
    ))
}

fn replay_match(
    state: &FocusaState,
    scope: &ContextGraphScope,
    idempotency_key: &str,
    action: &str,
) -> Option<bool> {
    let found = match action {
        "propose_claim" => state.context_claims.iter().any(|claim| {
            same_scope_claim(claim, scope) && claim.idempotency_key == idempotency_key
        }),
        "open_contradiction" => state
            .context_contradictions
            .iter()
            .any(|edge| same_scope_edge(edge, scope) && edge.idempotency_key == idempotency_key),
        "review_claim" | "resolve_contradiction" => {
            let decision_id = stable_ref(
                "context-decision",
                &[
                    &scope.project_root,
                    &scope.continuity_id,
                    &scope.attachment_id,
                    idempotency_key,
                    action,
                ],
            );
            state.context_decisions.iter().any(|decision| {
                same_scope_decision(decision, scope) && decision.decision_id == decision_id
            })
        }
        _ => false,
    };
    found.then_some(true)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/context/graph", get(read_graph))
        .route("/v1/context/graph/mutate", post(mutate_graph))
}
