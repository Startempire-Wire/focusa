//! Spec 137 temporal closure, missed-target, lost-time, receipt, and learning settlement.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use focusa_core::{
    temporal::{TemporalEvent, TemporalEventKind},
    temporal_progress::{LostTimeIncident, validate_lost_time_incident},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::server::AppState;

use super::temporal::{
    TemporalScopeDimensions, append_signed_events, fail, ledger, project_active_focus_frame,
    read_events, scope,
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalClosureOutcome {
    Completed,
    MissedTarget,
    Blocked,
    Cancelled,
}

#[derive(Debug, Deserialize)]
pub struct TemporalClosureSettlementRequest {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    idempotency_key: String,
    confirm: bool,
    subject_ref: String,
    outcome: TemporalClosureOutcome,
    target_ref: Option<String>,
    actual_duration_ms: Option<u64>,
    #[serde(default)]
    evidence_refs: Vec<String>,
    receipt_ref: String,
    #[serde(default)]
    forecast_evaluation_refs: Vec<String>,
    #[serde(default)]
    reflection_refs: Vec<String>,
    #[serde(default)]
    learning_candidate_refs: Vec<String>,
    lost_time_incident: Option<LostTimeIncident>,
}

fn validate_request(
    req: &TemporalClosureSettlementRequest,
    exact_scope: &focusa_core::temporal::TemporalScope,
) -> Result<(), (StatusCode, Json<Value>)> {
    if !req.confirm {
        return Err(fail(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "temporal closure settlement requires confirm=true",
        ));
    }
    if req.idempotency_key.trim().is_empty()
        || req.subject_ref.trim().is_empty()
        || req.receipt_ref.trim().is_empty()
        || req.evidence_refs.is_empty()
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            "closure_evidence_required",
            "subject_ref, receipt_ref, idempotency_key, and evidence_refs are required",
        ));
    }
    if matches!(req.outcome, TemporalClosureOutcome::MissedTarget)
        && req.target_ref.as_deref().is_none_or(str::is_empty)
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            "missed_target_ref_required",
            "missed_target settlement requires target_ref",
        ));
    }
    if let Some(incident) = req.lost_time_incident.as_ref() {
        if !incident.scope.same_workstream(exact_scope)
            || !exact_scope.matches_filter(&incident.scope)
        {
            return Err(fail(
                StatusCode::CONFLICT,
                "scope_mismatch",
                "lost-time incident scope must match closure project and continuity",
            ));
        }
        validate_lost_time_incident(incident).map_err(|error| {
            fail(
                StatusCode::PRECONDITION_FAILED,
                "lost_time_incident_invalid",
                format!("{error:?}"),
            )
        })?;
    }
    Ok(())
}

fn event(
    scope: focusa_core::temporal::TemporalScope,
    kind: TemporalEventKind,
    metadata: BTreeMap<String, Value>,
) -> TemporalEvent {
    TemporalEvent {
        event_id: Uuid::now_v7().to_string(),
        sequence: 0,
        event_kind: kind,
        scope,
        claim: None,
        clock_sample: None,
        metadata,
        signature: None,
        predecessor_digest: None,
        recorded_at: Utc::now(),
        idempotency_key: String::new(),
        digest: String::new(),
    }
}

pub(super) async fn settle_closure(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TemporalClosureSettlementRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let exact_scope = scope(
        req.project_root.clone(),
        req.continuity_id.clone(),
        req.dimensions.clone(),
    );
    validate_request(&req, &exact_scope)?;
    let ledger = ledger(exact_scope.clone())?;
    let existing = read_events(&ledger)?
        .into_iter()
        .filter(|event| event.idempotency_key == req.idempotency_key)
        .collect::<Vec<_>>();
    if !existing.is_empty() {
        return Ok(Json(json!({
            "schema":"focusa.temporal_closure_settlement.v1",
            "status":"completed",
            "canonical":true,
            "idempotent_replay":true,
            "events":existing,
            "receipt_ref":req.receipt_ref,
            "temporal_context":super::temporal_context::bounded_temporal_context(
                &exact_scope.project_root,
                &exact_scope.continuity_id,
                exact_scope.workpoint_id.clone(),
                exact_scope.item_id.clone()
            )
        })));
    }

    let mut base = BTreeMap::new();
    base.insert("subject_ref".into(), json!(req.subject_ref));
    base.insert("outcome".into(), json!(req.outcome));
    base.insert("target_ref".into(), json!(req.target_ref));
    base.insert("actual_duration_ms".into(), json!(req.actual_duration_ms));
    base.insert("evidence_refs".into(), json!(req.evidence_refs));
    base.insert("receipt_ref".into(), json!(req.receipt_ref));
    base.insert(
        "forecast_evaluation_refs".into(),
        json!(req.forecast_evaluation_refs),
    );
    base.insert("reflection_refs".into(), json!(req.reflection_refs));
    base.insert(
        "learning_candidate_refs".into(),
        json!(req.learning_candidate_refs),
    );

    let outcome_kind = match req.outcome {
        TemporalClosureOutcome::Completed => TemporalEventKind::TargetSatisfied,
        TemporalClosureOutcome::MissedTarget => TemporalEventKind::MissedTargetRecorded,
        TemporalClosureOutcome::Blocked | TemporalClosureOutcome::Cancelled => {
            TemporalEventKind::ClosurePostureRecorded
        }
    };
    let mut pending = vec![event(exact_scope.clone(), outcome_kind, base.clone())];
    if let Some(incident) = req.lost_time_incident.as_ref() {
        let mut metadata = base.clone();
        metadata.insert("lost_time_incident".into(), json!(incident));
        pending.push(event(
            exact_scope.clone(),
            TemporalEventKind::LostTimeIncidentRecorded,
            metadata,
        ));
    }
    if outcome_kind != TemporalEventKind::ClosurePostureRecorded {
        pending.push(event(
            exact_scope.clone(),
            TemporalEventKind::ClosurePostureRecorded,
            base.clone(),
        ));
    }
    pending.push(event(
        exact_scope.clone(),
        TemporalEventKind::ReceiptLinked,
        base,
    ));

    let appended = append_signed_events(&ledger, &req.idempotency_key, pending)?;
    project_active_focus_frame(state.as_ref(), &exact_scope, &ledger).await?;
    Ok(Json(json!({
        "schema":"focusa.temporal_closure_settlement.v1",
        "status":"completed",
        "canonical":true,
        "idempotent_replay":false,
        "events":appended,
        "receipt_ref":req.receipt_ref,
        "evidence_refs":req.evidence_refs,
        "reflection_refs":req.reflection_refs,
        "learning_candidate_refs":req.learning_candidate_refs,
        "temporal_context":super::temporal_context::bounded_temporal_context(
            &exact_scope.project_root,
            &exact_scope.continuity_id,
            exact_scope.workpoint_id.clone(),
            exact_scope.item_id.clone()
        )
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> TemporalClosureSettlementRequest {
        TemporalClosureSettlementRequest {
            project_root: "/project".into(),
            continuity_id: "continuity".into(),
            dimensions: TemporalScopeDimensions::default(),
            idempotency_key: "closure:test".into(),
            confirm: true,
            subject_ref: "workpoint:test".into(),
            outcome: TemporalClosureOutcome::Completed,
            target_ref: None,
            actual_duration_ms: Some(1_000),
            evidence_refs: vec!["evidence:test".into()],
            receipt_ref: "receipt:test".into(),
            forecast_evaluation_refs: vec![],
            reflection_refs: vec!["reflection:test".into()],
            learning_candidate_refs: vec!["learning:test".into()],
            lost_time_incident: None,
        }
    }

    #[test]
    fn closure_requires_confirmation_evidence_and_missed_target_ref() {
        let mut req = request();
        let scope = focusa_core::temporal::TemporalScope::project("/project", "continuity");
        assert!(validate_request(&req, &scope).is_ok());

        req.confirm = false;
        assert_eq!(
            validate_request(&req, &scope).unwrap_err().0,
            StatusCode::PRECONDITION_REQUIRED
        );
        req.confirm = true;
        req.outcome = TemporalClosureOutcome::MissedTarget;
        assert_eq!(
            validate_request(&req, &scope).unwrap_err().0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
}
