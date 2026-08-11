//! Canonical Spec 138 HTTP adapters over the scoped durable authority ledger.

use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use focusa_core::{
    prediction_authority::{PredictionAuthorityEvent, ScopedAuthorityEvent},
    prediction_authority_storage::{PersistentPredictionAuthorityLedger, PredictionStorageError},
    scoped_state::{ScopeKind, ScopeRef, WorkstreamKey},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct WriteBody {
    scope: WorkstreamKey,
    event: ScopedAuthorityEvent,
}

#[derive(Debug, Deserialize)]
struct ScopeQuery {
    scope_kind: ScopeKind,
    scope_id: String,
    root_path: std::path::PathBuf,
    canonical_name: String,
    fingerprint: String,
    continuity_id: String,
}

impl ScopeQuery {
    fn scope(self) -> Result<WorkstreamKey, ApiError> {
        WorkstreamKey::new(
            ScopeRef {
                scope_kind: self.scope_kind,
                scope_id: self.scope_id,
                root_path: self.root_path,
                canonical_name: self.canonical_name,
                fingerprint: self.fingerprint,
            },
            self.continuity_id,
        )
        .and_then(|scope| {
            scope.validate()?;
            Ok(scope)
        })
        .map_err(|error| invalid("typed_scope_required", error.to_string()))
    }
}

type ApiError = (StatusCode, Json<Value>);
type ApiResult = Result<Json<Value>, ApiError>;

#[derive(Clone, Copy)]
enum Operation {
    Question,
    InformationSet,
    Commitment,
    Supersede,
    OutcomeClaim,
    OutcomeDispute,
    OutcomeResolve,
    Evaluation,
    MetacogSignal,
    Reflection,
    Adjustment,
    MetacogEvaluation,
    CandidateDecision,
    LearningApply,
    TransferResolve,
    LearningExpire,
    LearningRevoke,
    LearningRollback,
    LearningConsolidate,
}

impl Operation {
    fn accepts(self, event: &PredictionAuthorityEvent) -> bool {
        match self {
            Self::Question => matches!(event, PredictionAuthorityEvent::Question(_)),
            Self::InformationSet => matches!(event, PredictionAuthorityEvent::EpistemicPrimitive(value) if value.descriptor.family_section == 7),
            Self::Commitment => matches!(event, PredictionAuthorityEvent::Commitment(_)),
            Self::Supersede => matches!(event, PredictionAuthorityEvent::EpistemicPrimitive(value) if value.descriptor.primitive == "PredictionSupersession"),
            Self::OutcomeClaim => matches!(event, PredictionAuthorityEvent::OutcomeClaim(_)),
            Self::OutcomeDispute => matches!(event, PredictionAuthorityEvent::OutcomeAuthority(value) if matches!(value.action, focusa_core::outcome_resolution::OutcomeAuthorityAction::Dispute { .. })),
            Self::OutcomeResolve => matches!(event, PredictionAuthorityEvent::OutcomeResolution(_) | PredictionAuthorityEvent::OutcomeAuthority(_)),
            Self::Evaluation => matches!(event, PredictionAuthorityEvent::Evaluation(_)),
            Self::MetacogSignal => matches!(event, PredictionAuthorityEvent::EpistemicPrimitive(_)),
            Self::Reflection => matches!(event, PredictionAuthorityEvent::ReflectionClaim(_)),
            Self::Adjustment => matches!(event, PredictionAuthorityEvent::PromotionAssessment(_)),
            Self::MetacogEvaluation | Self::CandidateDecision => {
                matches!(event, PredictionAuthorityEvent::PromotionDecision(_))
            }
            Self::LearningApply => matches!(event, PredictionAuthorityEvent::LearningRecord(_)),
            Self::TransferResolve => matches!(event, PredictionAuthorityEvent::TransferOutcome(_)),
            Self::LearningExpire | Self::LearningRevoke | Self::LearningRollback => {
                matches!(event, PredictionAuthorityEvent::LearningRecord(_) | PredictionAuthorityEvent::MemoryLifecycle(_))
            }
            Self::LearningConsolidate => {
                matches!(event, PredictionAuthorityEvent::LearningSettlement(_) | PredictionAuthorityEvent::MemoryLifecycle(_))
            }
        }
    }
}

async fn append(
    state: Arc<AppState>,
    body: WriteBody,
    operation: Operation,
    path_id: Option<&str>,
) -> ApiResult {
    if body.scope != body.event.scope {
        return Err(invalid("scope_mismatch", "body scope and event scope differ"));
    }
    if !operation.accepts(&body.event.event) {
        return Err(invalid(
            "event_kind_mismatch",
            "route does not accept this ScopedAuthorityEvent variant",
        ));
    }
    if let Some(path_id) = path_id {
        if path_id.trim().is_empty() || !event_references_id(&body.event.event, path_id) {
            return Err(invalid(
                "path_identity_mismatch",
                "path id is not referenced by the authority event",
            ));
        }
    }
    let ledger = PersistentPredictionAuthorityLedger::for_scope(
        body.scope.clone(),
        Some(&state.config.data_dir),
    )
    .map_err(storage_error)?;
    let rows = ledger
        .append_batch(vec![body.event.clone()])
        .map_err(storage_error)?;
    let row = rows.into_iter().next().expect("non-empty append result");
    let crdt_warning = state
        .prediction_authority_store
        .upsert(body.scope, row.event.event_id.clone(), row.event.clone())
        .await
        .err()
        .map(|error| error.to_string());
    Ok(Json(json!({
        "status": if crdt_warning.is_some() { "completed_degraded" } else { "completed" },
        "canonical": true,
        "durability": "atomic_fsync_causal_jsonl",
        "event_id": row.event.event_id,
        "sequence": row.event.sequence,
        "evidence_refs": row.event.evidence_refs,
        "receipt_ref": row.event.receipt_ref,
        "digest": row.digest,
        "warnings": crdt_warning.into_iter().collect::<Vec<_>>()
    })))
}

fn event_references_id(event: &PredictionAuthorityEvent, id: &str) -> bool {
    match event {
        PredictionAuthorityEvent::Commitment(value) => {
            value.commitment_id == id || value.question_id == id
        }
        PredictionAuthorityEvent::EpistemicPrimitive(value) => {
            value.value.get("prediction_id").and_then(Value::as_str) == Some(id)
                || value.value.get("supersedes_prediction_id").and_then(Value::as_str) == Some(id)
                || value.value.get("commitment_id").and_then(Value::as_str) == Some(id)
        }
        PredictionAuthorityEvent::OutcomeAuthority(value) => {
            value.event_id == id || value.commitment_id == id
        }
        PredictionAuthorityEvent::PromotionDecision(value) => value.candidate_id == id,
        PredictionAuthorityEvent::LearningRecord(value) => value.learning_id == id,
        PredictionAuthorityEvent::MemoryLifecycle(value) => value.memory_id == id,
        _ => false,
    }
}

macro_rules! write_handler {
    ($name:ident, $operation:expr) => {
        async fn $name(State(state): State<Arc<AppState>>, Json(body): Json<WriteBody>) -> ApiResult {
            append(state, body, $operation, None).await
        }
    };
}

macro_rules! path_write_handler {
    ($name:ident, $operation:expr) => {
        async fn $name(
            State(state): State<Arc<AppState>>,
            Path(id): Path<String>,
            Json(body): Json<WriteBody>,
        ) -> ApiResult {
            append(state, body, $operation, Some(&id)).await
        }
    };
}

write_handler!(question_create, Operation::Question);
write_handler!(information_set_commit, Operation::InformationSet);
write_handler!(prediction_commit, Operation::Commitment);
path_write_handler!(prediction_supersede, Operation::Supersede);
write_handler!(outcome_claim, Operation::OutcomeClaim);
path_write_handler!(outcome_dispute, Operation::OutcomeDispute);
write_handler!(outcome_resolve, Operation::OutcomeResolve);
write_handler!(prediction_evaluate, Operation::Evaluation);
write_handler!(metacog_signal, Operation::MetacogSignal);
write_handler!(metacog_reflection, Operation::Reflection);
write_handler!(metacog_adjustment, Operation::Adjustment);
write_handler!(metacog_evaluation, Operation::MetacogEvaluation);
path_write_handler!(candidate_decide, Operation::CandidateDecision);
path_write_handler!(learning_apply, Operation::LearningApply);
write_handler!(transfer_resolve, Operation::TransferResolve);
path_write_handler!(learning_expire, Operation::LearningExpire);
path_write_handler!(learning_revoke, Operation::LearningRevoke);
path_write_handler!(learning_rollback, Operation::LearningRollback);
write_handler!(learning_consolidate, Operation::LearningConsolidate);

fn projection(state: &AppState, scope: WorkstreamKey) -> Result<focusa_core::prediction_authority_ledger::PredictionAuthorityProjection, ApiError> {
    PersistentPredictionAuthorityLedger::for_scope(scope, Some(&state.config.data_dir))
        .map_err(storage_error)?
        .projection()
        .map_err(storage_error)
}

async fn prediction_get(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    let projection = projection(&state, query.scope()?)?;
    projection
        .commitments
        .get(&id)
        .map(|record| Json(json!({"status":"completed","canonical":true,"prediction":record,"sequence":projection.sequence})))
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"status":"blocked","error":"prediction_not_found"}))))
}

async fn predictions_recent(State(state): State<Arc<AppState>>, Query(query): Query<ScopeQuery>) -> ApiResult {
    let projection = projection(&state, query.scope()?)?;
    Ok(Json(json!({"status":"completed","canonical":true,"predictions":projection.commitments.values().collect::<Vec<_>>(),"sequence":projection.sequence})))
}

async fn calibration_reports(State(state): State<Arc<AppState>>, Query(query): Query<ScopeQuery>) -> ApiResult {
    let projection = projection(&state, query.scope()?)?;
    let value = serde_json::to_value(&projection)
        .map_err(|error| invalid("projection_encoding_failed", error.to_string()))?;
    Ok(Json(json!({
        "status":"completed","canonical":true,
        "scoring_policies":value["scoring_policies"],
        "evaluations":value["evaluations"],"sequence":projection.sequence
    })))
}

async fn learning_retrieve(State(state): State<Arc<AppState>>, Query(query): Query<ScopeQuery>) -> ApiResult {
    let projection = projection(&state, query.scope()?)?;
    let value = serde_json::to_value(&projection)
        .map_err(|error| invalid("projection_encoding_failed", error.to_string()))?;
    Ok(Json(json!({
        "status":"completed","canonical":true,"learning":value["learning"],
        "candidates":value["learning_candidates"],
        "promotion_decisions":value["promotion_decisions"],
        "transfers":value["transfer_predictions"],"sequence":projection.sequence
    })))
}

async fn learning_conflicts(State(state): State<Arc<AppState>>, Query(query): Query<ScopeQuery>) -> ApiResult {
    let projection = projection(&state, query.scope()?)?;
    let conflicts = projection.learning.values().filter(|record| !record.applicability.excludes.is_empty()).collect::<Vec<_>>();
    Ok(Json(json!({"status":"completed","canonical":true,"conflicts":conflicts,"sequence":projection.sequence})))
}

async fn self_model(State(state): State<Arc<AppState>>, Query(query): Query<ScopeQuery>) -> ApiResult {
    let projection = projection(&state, query.scope()?)?;
    Ok(Json(json!({"status":"completed","canonical":true,"self_model":projection.self_model,"sequence":projection.sequence})))
}

fn invalid(code: &str, reason: impl Into<String>) -> ApiError {
    (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"status":"blocked","error":code,"reason":reason.into()})))
}

fn storage_error(error: PredictionStorageError) -> ApiError {
    (StatusCode::CONFLICT, Json(json!({"status":"blocked","failure_class":"prediction_authority_storage","error":format!("{error:?}")})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/prediction-questions", post(question_create))
        .route("/v1/information-sets", post(information_set_commit))
        .route("/v1/predictions/commit", post(prediction_commit))
        .route("/v1/predictions/{id}/supersede", post(prediction_supersede))
        .route("/v1/predictions/{id}", get(prediction_get))
        .route("/v1/predictions/recent", get(predictions_recent))
        .route("/v1/outcomes/claim", post(outcome_claim))
        .route("/v1/outcomes/{id}/dispute", post(outcome_dispute))
        .route("/v1/outcomes/resolve", post(outcome_resolve))
        .route("/v1/evaluations/predictions", post(prediction_evaluate))
        .route("/v1/calibration/reports", get(calibration_reports))
        .route("/v1/metacognition/signals", post(metacog_signal))
        .route("/v1/metacognition/reflections", post(metacog_reflection))
        .route("/v1/metacognition/adjustments", post(metacog_adjustment))
        .route("/v1/metacognition/evaluations", post(metacog_evaluation))
        .route("/v1/learning/candidates/{id}/decide", post(candidate_decide))
        .route("/v1/learning/{id}/apply", post(learning_apply))
        .route("/v1/learning/transfers/resolve", post(transfer_resolve))
        .route("/v1/learning/retrieve", get(learning_retrieve))
        .route("/v1/learning/conflicts", get(learning_conflicts))
        .route("/v1/learning/{id}/expire", post(learning_expire))
        .route("/v1/learning/{id}/revoke", post(learning_revoke))
        .route("/v1/learning/{id}/rollback", post(learning_rollback))
        .route("/v1/learning/consolidate", post(learning_consolidate))
        .route("/v1/self-model", get(self_model))
}
