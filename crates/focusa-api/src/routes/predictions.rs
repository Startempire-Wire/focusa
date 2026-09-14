use crate::routes::metacognition::capture_learning_signal_scoped;
use crate::scope::ScopeContext;
use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::prediction::{
    PredictionOntologyContext, PredictionOutcomeCapture, PredictionValue,
};
use focusa_core::scoped_state::{
    AuthorityEnvelope, AuthorityStatus, HumanReadableSummary, ScopeKind, ScopeRef,
    ScopedCrdtRecord, ScopedResultEnvelope, WorkstreamKey,
};
use focusa_core::types::{FocusaState, TrajectoryLadderContext};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct PredictionBody {
    scope: WorkstreamKey,
    prediction_type: String,
    #[serde(default)]
    context_refs: Vec<String>,
    #[serde(default)]
    ontology_context: PredictionOntologyContext,
    predicted_outcome: String,
    confidence: f64,
    recommended_action: String,
    why: String,
}

#[derive(Debug, Deserialize)]
struct EvaluateBody {
    scope: WorkstreamKey,
    actual_outcome: String,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    learning_signal_ref: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CaptureOutcomeBody {
    scope: WorkstreamKey,
    actual_outcome: String,
    #[serde(default)]
    prediction_type: Option<String>,
    #[serde(default)]
    context_refs: Vec<String>,
    #[serde(default)]
    ontology_context: PredictionOntologyContext,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    learning_signal_ref: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ScopedPredictionQuery {
    scope_kind: ScopeKind,
    scope_id: String,
    root_path: String,
    canonical_name: String,
    fingerprint: String,
    continuity_id: String,
}

impl ScopedPredictionQuery {
    fn workstream(&self) -> Result<WorkstreamKey, String> {
        let scope = ScopeRef {
            scope_kind: self.scope_kind,
            scope_id: self.scope_id.clone(),
            root_path: self.root_path.clone().into(),
            canonical_name: self.canonical_name.clone(),
            fingerprint: self.fingerprint.clone(),
        };
        let workstream = WorkstreamKey {
            root_scope: scope,
            continuity_id: self.continuity_id.clone(),
        };
        workstream.validate().map_err(|error| error.to_string())?;
        Ok(workstream)
    }
}

fn request_scope_matches(request: &ScopeContext, scope: &WorkstreamKey) -> bool {
    let root_matches = request.project_root.as_ref().is_none_or(|root| {
        root.trim_end_matches('/')
            == scope
                .root_scope
                .root_path
                .to_string_lossy()
                .trim_end_matches('/')
    });
    let continuity_matches = request
        .continuity_id
        .as_ref()
        .is_none_or(|continuity| continuity == &scope.continuity_id);
    root_matches && continuity_matches
}

// This boundary mirrors the typed scoped-result envelope fields one-for-one;
// grouping them would hide authority and operator-facing semantics at call sites.
#[allow(clippy::too_many_arguments)]
fn response(
    scope: WorkstreamKey,
    authority: AuthorityStatus,
    status: &str,
    summary: impl Into<String>,
    next_action: impl Into<String>,
    why: impl Into<String>,
    data: Value,
    warnings: Vec<String>,
) -> Json<Value> {
    let why = why.into();
    Json(
        serde_json::to_value(ScopedResultEnvelope::new(
            scope,
            AuthorityEnvelope {
                status: authority,
                why: why.clone(),
            },
            HumanReadableSummary {
                status: status.to_string(),
                summary: summary.into(),
                next_action: next_action.into(),
                why,
                evidence_refs: Vec::new(),
                warnings,
            },
            data,
        ))
        .unwrap_or_else(
            |error| json!({"schema":"focusa.scoped_result.v1","error":error.to_string()}),
        ),
    )
}

fn blocked(
    scope: WorkstreamKey,
    summary: impl Into<String>,
    why: impl Into<String>,
) -> Json<Value> {
    response(
        scope,
        AuthorityStatus::Blocked,
        "blocked",
        summary,
        "Provide one valid typed project/host scope and continuity id, then retry.",
        why,
        json!({"accepted": false}),
        vec![],
    )
}

fn validate_scope(scope: &WorkstreamKey) -> Result<(), String> {
    scope.validate().map_err(|error| error.to_string())
}

fn prediction_score(predicted: &str, actual: &str, explicit: Option<f64>) -> f64 {
    explicit
        .unwrap_or_else(|| {
            if !predicted.is_empty() && actual.to_lowercase().contains(&predicted.to_lowercase()) {
                1.0
            } else {
                0.0
            }
        })
        .clamp(0.0, 1.0)
}

fn scoped_trajectory(
    focusa: &FocusaState,
    scope: &WorkstreamKey,
) -> Option<TrajectoryLadderContext> {
    focusa.trajectory.records.iter().rev().find_map(|record| {
        let project_matches = record.project_root.as_deref().is_some_and(|root| {
            root.trim_end_matches('/')
                == scope
                    .root_scope
                    .root_path
                    .to_string_lossy()
                    .trim_end_matches('/')
        });
        let continuity_matches =
            record.continuity_id.as_deref() == Some(scope.continuity_id.as_str());
        (project_matches && continuity_matches).then(|| TrajectoryLadderContext {
            trajectory_id: Some(record.trajectory_id.clone()),
            project_root: record.project_root.clone(),
            continuity_id: record.continuity_id.clone(),
            hlt: Some(record.long_term_goal.clone()),
            mlg: record.mid_level_goal.clone(),
            stg: record.short_term_goal.clone(),
            waypoints: record
                .waypoints
                .iter()
                .map(|waypoint| waypoint.title.clone())
                .collect(),
            ..TrajectoryLadderContext::default()
        })
    })
}

fn compact_record(record: &ScopedCrdtRecord<PredictionValue>) -> Value {
    json!({
        "record_id": record.record_id,
        "scope": record.scope,
        "vector_clock": record.vector_clock,
        "lamport_ts": record.lamport_ts,
        "updated_at": record.updated_at,
        "prediction": record.value,
    })
}

fn contexts_match(record_refs: &[String], outcome_refs: &[String]) -> bool {
    outcome_refs.is_empty()
        || outcome_refs
            .iter()
            .any(|needle| record_refs.iter().any(|candidate| candidate == needle))
}

fn evaluate_hint(records: &[ScopedCrdtRecord<PredictionValue>]) -> Value {
    let candidate = records
        .iter()
        .filter(|record| record.value.evaluated_at.is_none())
        .max_by(|left, right| {
            left.value
                .confidence
                .partial_cmp(&right.value.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    candidate.map_or_else(
        || json!({"action":"record_new_prediction","next_tool":"focusa_predict_record"}),
        |record| {
            json!({
                "action":"evaluate_prediction",
                "prediction_id":record.record_id,
                "confidence":record.value.confidence,
                "next_tool":"focusa_predict_evaluate"
            })
        },
    )
}

pub(crate) async fn append_prediction_record_scoped(
    state: &AppState,
    scope: WorkstreamKey,
    value: PredictionValue,
) -> anyhow::Result<ScopedCrdtRecord<PredictionValue>> {
    validate_scope(&scope).map_err(anyhow::Error::msg)?;
    state
        .prediction_store
        .upsert(scope, Uuid::now_v7().to_string(), value)
        .await
}

async fn record(
    request_scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PredictionBody>,
) -> Json<Value> {
    if let Err(error) = validate_scope(&body.scope) {
        return blocked(body.scope, "Prediction scope is invalid", error);
    }
    if !request_scope_matches(&request_scope, &body.scope) {
        return blocked(
            body.scope,
            "Request scope and prediction scope differ",
            "typed request scope mismatch",
        );
    }
    let trajectory = {
        let focusa = state.focusa.read().await;
        scoped_trajectory(&focusa, &body.scope)
    };
    let value = PredictionValue {
        prediction_type: body.prediction_type,
        context_refs: body.context_refs,
        ontology_context: body.ontology_context,
        predicted_outcome: body.predicted_outcome,
        confidence: body.confidence.clamp(0.0, 1.0),
        recommended_action: body.recommended_action,
        why: body.why,
        trajectory,
        actual_outcome: None,
        evaluated_at: None,
        score: None,
        learning_signal_ref: None,
        outcome_capture: None,
    };
    match append_prediction_record_scoped(&state, body.scope.clone(), value).await {
        Ok(record) => {
            state.mark_external_mutation();
            response(
                body.scope,
                AuthorityStatus::Canonical,
                "recorded",
                format!(
                    "Prediction {} recorded in its typed workstream",
                    record.record_id
                ),
                format!(
                    "Evaluate prediction {} when the outcome is known",
                    record.record_id
                ),
                "The record was appended to a scope-partitioned CRDT ledger.",
                json!({"record": compact_record(&record)}),
                vec![],
            )
        }
        Err(error) => blocked(body.scope, "Prediction write failed", error.to_string()),
    }
}

async fn evaluate(
    request_scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Path(prediction_id): Path<String>,
    Json(body): Json<EvaluateBody>,
) -> Json<Value> {
    if let Err(error) = validate_scope(&body.scope) {
        return blocked(body.scope, "Prediction scope is invalid", error);
    }
    if !request_scope_matches(&request_scope, &body.scope) {
        return blocked(
            body.scope,
            "Request scope and prediction scope differ",
            "typed request scope mismatch",
        );
    }
    let Some(current) = state
        .prediction_store
        .get(&body.scope, &prediction_id)
        .await
        .ok()
        .flatten()
    else {
        return response(
            body.scope,
            AuthorityStatus::Blocked,
            "not_found",
            format!("Prediction {prediction_id} was not found in this workstream"),
            "List predictions for this exact scope before evaluating.",
            "Prediction ids never cross typed workstream boundaries.",
            json!({"prediction_id":prediction_id}),
            vec![],
        );
    };
    let mut value = current.value.clone();
    let score = prediction_score(&value.predicted_outcome, &body.actual_outcome, body.score);
    value.actual_outcome = Some(body.actual_outcome);
    value.evaluated_at = Some(Utc::now());
    value.score = Some(score);
    value.learning_signal_ref = body.learning_signal_ref;
    value.outcome_capture = Some(PredictionOutcomeCapture {
        mode: "manual".into(),
        matched_by: "prediction_id_and_workstream".into(),
        context_refs: vec![],
        ontology_context: PredictionOntologyContext::default(),
    });
    match state
        .prediction_store
        .upsert(body.scope.clone(), prediction_id.clone(), value.clone())
        .await
    {
        Ok(record) => {
            let metacog_capture_id = if score >= 0.5 {
                capture_learning_signal_scoped(
                    &state,
                    body.scope.clone(),
                    "prediction_outcome",
                    &format!(
                        "Prediction {} scored {}. Expected: {}. Actual: {}.",
                        prediction_id,
                        score,
                        value.predicted_outcome,
                        value.actual_outcome.as_deref().unwrap_or("")
                    ),
                    Some("Scoped prediction evaluation fed scoped metacognition memory.".into()),
                    Some(score),
                    Some("prediction_metacog_flywheel".into()),
                )
                .await
            } else {
                None
            };
            response(
                body.scope,
                AuthorityStatus::Canonical,
                "evaluated",
                format!("Prediction {prediction_id} evaluated with score {score:.2}"),
                "Review scoped prediction stats and retrieve scoped metacognition.",
                "Evaluation updated only the matching record in the exact typed workstream.",
                json!({"record":compact_record(&record),"metacog_capture_id":metacog_capture_id}),
                vec![],
            )
        }
        Err(error) => blocked(
            body.scope,
            "Prediction evaluation write failed",
            error.to_string(),
        ),
    }
}

async fn capture_outcome(
    request_scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CaptureOutcomeBody>,
) -> Json<Value> {
    if let Err(error) = validate_scope(&body.scope) {
        return blocked(body.scope, "Prediction scope is invalid", error);
    }
    if !request_scope_matches(&request_scope, &body.scope) {
        return blocked(
            body.scope,
            "Request scope and prediction scope differ",
            "typed request scope mismatch",
        );
    }
    let limit = body.limit.unwrap_or(10).clamp(1, 50);
    let records = match state.prediction_store.recent(&body.scope, 1000).await {
        Ok(records) => records,
        Err(error) => {
            return blocked(
                body.scope,
                "Prediction outcome read failed",
                error.to_string(),
            );
        }
    };
    let mut updated = Vec::new();
    let mut capture_ids = Vec::new();
    for record in records.into_iter().rev() {
        if updated.len() >= limit || record.value.score.is_some() {
            continue;
        }
        if body
            .prediction_type
            .as_deref()
            .is_some_and(|kind| kind != record.value.prediction_type)
        {
            continue;
        }
        if !contexts_match(&record.value.context_refs, &body.context_refs) {
            continue;
        }
        let mut value = record.value.clone();
        let score = prediction_score(&value.predicted_outcome, &body.actual_outcome, body.score);
        value.actual_outcome = Some(body.actual_outcome.clone());
        value.evaluated_at = Some(Utc::now());
        value.score = Some(score);
        value.learning_signal_ref = body.learning_signal_ref.clone();
        value.outcome_capture = Some(PredictionOutcomeCapture {
            mode: "auto_capture".into(),
            matched_by: if body.context_refs.is_empty() {
                "prediction_type_and_workstream".into()
            } else {
                "context_refs_and_workstream".into()
            },
            context_refs: body.context_refs.clone(),
            ontology_context: body.ontology_context.clone(),
        });
        if let Ok(next) = state
            .prediction_store
            .upsert(body.scope.clone(), record.record_id.clone(), value.clone())
            .await
        {
            if score >= 0.5
                && let Some(capture_id) = capture_learning_signal_scoped(
                    &state,
                    body.scope.clone(),
                    "prediction_outcome",
                    &format!(
                        "Auto-captured prediction {} scored {}.",
                        record.record_id, score
                    ),
                    Some("Scoped outcome capture fed scoped metacognition memory.".into()),
                    Some(score),
                    Some("prediction_metacog_flywheel".into()),
                )
                .await
            {
                capture_ids.push(capture_id);
            }
            updated.push(compact_record(&next));
        }
    }
    response(
        body.scope,
        AuthorityStatus::Canonical,
        "completed",
        format!(
            "Captured outcomes for {} scoped prediction(s)",
            updated.len()
        ),
        if updated.is_empty() {
            "Record a matching scoped prediction first."
        } else {
            "Retrieve scoped metacognition before the next similar decision."
        },
        "Matching was constrained to one typed workstream before type/context matching.",
        json!({"updated":updated,"metacog_capture_ids":capture_ids}),
        vec![],
    )
}

async fn stats(
    request_scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ScopedPredictionQuery>,
) -> Json<Value> {
    let scope = match query.workstream() {
        Ok(scope) => scope,
        Err(error) => {
            let fallback = WorkstreamKey {
                root_scope: ScopeRef {
                    scope_kind: query.scope_kind,
                    scope_id: query.scope_id,
                    root_path: query.root_path.into(),
                    canonical_name: query.canonical_name,
                    fingerprint: query.fingerprint,
                },
                continuity_id: query.continuity_id,
            };
            return blocked(fallback, "Prediction stats scope is invalid", error);
        }
    };
    if !request_scope_matches(&request_scope, &scope) {
        return blocked(
            scope,
            "Request scope and prediction stats scope differ",
            "typed request scope mismatch",
        );
    }
    let records = match state.prediction_store.recent(&scope, 1000).await {
        Ok(records) => records,
        Err(error) => return blocked(scope, "Prediction stats read failed", error.to_string()),
    };
    let mut by_type: HashMap<String, (usize, usize, f64)> = HashMap::new();
    let mut evaluated = 0usize;
    let mut score_sum = 0.0;
    for record in &records {
        let entry = by_type
            .entry(record.value.prediction_type.clone())
            .or_insert((0, 0, 0.0));
        entry.0 += 1;
        if let Some(score) = record.value.score {
            evaluated += 1;
            score_sum += score;
            entry.1 += 1;
            entry.2 += score;
        }
    }
    let by_type = by_type.into_iter().map(|(kind, (total, evaluated, sum))| {
        (kind, json!({"total":total,"evaluated":evaluated,"accuracy":if evaluated > 0 {sum / evaluated as f64} else {0.0}}))
    }).collect::<HashMap<_,_>>();
    let accuracy = if evaluated > 0 {
        score_sum / evaluated as f64
    } else {
        0.0
    };
    response(
        scope,
        AuthorityStatus::Canonical,
        "completed",
        format!(
            "{} scoped predictions, {} evaluated, {:.1}% accuracy",
            records.len(),
            evaluated,
            accuracy * 100.0
        ),
        "Record predictions at decision points and evaluate them after evidence arrives.",
        "Statistics include only the exact typed workstream; no global aggregate fallback was used.",
        json!({"total":records.len(),"evaluated":evaluated,"accuracy":accuracy,"by_type":by_type}),
        vec![],
    )
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/predictions", post(record))
        .route("/v1/predictions/capture-outcome", post(capture_outcome))
        .route("/v1/predictions/stats", get(stats))
        .route("/v1/predictions/{prediction_id}/evaluate", post(evaluate))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(project: &str, continuity: &str) -> WorkstreamKey {
        WorkstreamKey::new(
            ScopeRef::project(
                format!("project:{project}"),
                format!("/workspace/{project}"),
                project,
                format!("sha256:{project}"),
            )
            .unwrap(),
            continuity,
        )
        .unwrap()
    }

    #[test]
    fn record_body_accepts_partial_ontology_context() {
        let body: PredictionBody = serde_json::from_value(json!({
            "scope": scope("focusa", "auto"),
            "prediction_type": "rare_feature_gap_hunt",
            "context_refs": ["proof:one"],
            "ontology_context": {
                "object_refs": ["object:one"],
                "evidence_refs": ["proof:one"]
            },
            "predicted_outcome": "success",
            "confidence": 0.8,
            "recommended_action": "continue",
            "why": "bounded test"
        }))
        .expect("partial ontology context should not reject the prediction request");

        assert_eq!(body.ontology_context.object_refs, vec!["object:one"]);
        assert!(body.ontology_context.action_refs.is_empty());
        assert!(body.ontology_context.tool_refs.is_empty());
        assert!(body.ontology_context.relation_refs.is_empty());
    }

    #[test]
    fn same_continuity_never_matches_different_projects() {
        assert_ne!(scope("a", "same"), scope("b", "same"));
    }

    #[test]
    fn prediction_responses_include_human_readable_field() {
        let response = response(
            scope("focusa", "auto"),
            AuthorityStatus::Canonical,
            "recorded",
            "Prediction recorded",
            "Evaluate when outcome is known",
            "Stored in the scoped ledger",
            json!({"record_id":"pred-one"}),
            vec![],
        );
        assert!(
            response
                .0
                .get("human_readable")
                .and_then(Value::as_str)
                .is_some_and(|text| text.contains("recorded: Prediction recorded"))
        );
    }

    #[test]
    fn evaluate_hint_is_bounded_to_provided_scope_records() {
        let record = ScopedCrdtRecord::new(
            scope("a", "cont"),
            "pred-a",
            "test",
            PredictionValue {
                prediction_type: "next_action_success".into(),
                context_refs: vec![],
                ontology_context: PredictionOntologyContext::default(),
                predicted_outcome: "success".into(),
                confidence: 0.9,
                recommended_action: "continue".into(),
                why: "test".into(),
                trajectory: None,
                actual_outcome: None,
                evaluated_at: None,
                score: None,
                learning_signal_ref: None,
                outcome_capture: None,
            },
        )
        .unwrap();
        let hint = evaluate_hint(&[record]);
        assert_eq!(hint["prediction_id"], "pred-a");
    }
}
