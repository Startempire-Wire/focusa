//! Rust-authoritative compaction policy resolution and lifecycle surfaces.

use super::compaction_policy_store::{self as store, ControllerRecord};
use crate::{scope::ScopeContext, server::AppState};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::compaction_policy::{
    CompactionPolicyLease, CompactionPolicyObservation, CompactionRuntimeFacts, DriftInput,
    PolicyMode, PolicySelectionContext, PromotionInput, ValidationState, compile_policy_lattice,
    enroll_dev_canary, evaluate_drift, evaluate_promotion, legal_action_mask, resolve_policy,
    resolve_runtime_fingerprint, rollback_to_legacy,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct ResolveRequest {
    schema: String,
    runtime_facts: CompactionRuntimeFacts,
    #[serde(default = "default_mode")]
    mode: PolicyMode,
    objective_profile: Option<String>,
    predicted_safe_tokens: Option<u64>,
    sample_size: Option<u64>,
    confidence: Option<f64>,
    minimum_samples: Option<u64>,
    required_confidence: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ObserveRequest {
    schema: String,
    observation: CompactionPolicyObservation,
    promotion: Option<PromotionInput>,
    drift: Option<DriftInput>,
}

#[derive(Debug, Deserialize)]
struct CanaryRequest {
    policy_id: String,
    operator_ref: String,
    session_budget: u32,
}

#[derive(Debug, Deserialize)]
struct CanaryPauseRequest {
    operator_ref: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct RollbackRequest {
    failed_policy_id: String,
    primary_finding: String,
}

fn default_mode() -> PolicyMode {
    PolicyMode::Shadow
}

fn scope_key(scope: &ScopeContext) -> Result<(String, String, String), String> {
    let workstream = scope.require_workstream_key()?;
    let root = workstream
        .root_scope
        .root_path
        .to_string_lossy()
        .to_string();
    let continuity = workstream.continuity_id;
    let key = format!(
        "sha256:{}",
        hex::encode(Sha256::digest(format!("{root}\0{continuity}").as_bytes()))
    );
    Ok((key, root, continuity))
}

async fn resolve(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(mut req): Json<ResolveRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.schema != "focusa.compaction_policy_resolve_request.v1" {
        return Err(blocked(StatusCode::BAD_REQUEST, "invalid_resolve_schema"));
    }
    let (scope_key, root, continuity) =
        scope_key(&scope).map_err(|error| blocked(StatusCode::UNPROCESSABLE_ENTITY, &error))?;
    req.runtime_facts.project_root = Some(root);
    req.runtime_facts.continuity_id = Some(continuity);
    let fingerprint = resolve_runtime_fingerprint(req.runtime_facts);
    let prior = store::get(&state.config.data_dir, &scope_key);
    let evidence = prior
        .as_ref()
        .filter(|record| record.runtime_fingerprint.segment_key == fingerprint.segment_key)
        .map(|record| record.evidence.clone())
        .unwrap_or_default();
    let legal_actions = legal_action_mask(&fingerprint, &evidence, Utc::now());
    let context_window = fingerprint.context_window.unwrap_or(256_000);
    let mut candidates = compile_policy_lattice(
        context_window,
        &legal_actions,
        req.objective_profile.as_deref().unwrap_or("daily_driver"),
        req.predicted_safe_tokens,
    );
    if let Some(prior) = &prior {
        for candidate in &mut candidates {
            if let Some(previous) = prior
                .candidates
                .iter()
                .find(|previous| previous.policy_id == candidate.policy_id)
            {
                candidate.validation = previous.validation;
            }
        }
    }
    let canary_enrolled = prior
        .as_ref()
        .and_then(|record| record.canary_enrollment.as_ref())
        .is_some_and(|receipt| receipt.expires_at > Utc::now());
    let selection_context = PolicySelectionContext {
        mode: req.mode,
        context_window,
        sample_size: req.sample_size.unwrap_or(0),
        measured_confidence: req.confidence,
        minimum_samples: req.minimum_samples.unwrap_or(20).max(1),
        required_confidence: req.required_confidence.unwrap_or(0.95).clamp(0.5, 0.999),
        dev_fleet_enrolled: canary_enrolled,
    };
    let resolution = resolve_policy(&selection_context, &candidates);
    let lease = CompactionPolicyLease::freeze(
        &resolution,
        &fingerprint.segment_key,
        &fingerprint.capability_evidence_revision,
        "sha256:runtime-facts-v1",
    );
    let record = ControllerRecord {
        scope_key: scope_key.clone(),
        runtime_fingerprint: fingerprint.clone(),
        legal_actions: legal_actions.clone(),
        candidates: candidates.clone(),
        resolution: resolution.clone(),
        lease: lease.clone(),
        evidence,
        observations: prior
            .as_ref()
            .map(|record| record.observations.clone())
            .unwrap_or_default(),
        canary_enrollment: prior
            .as_ref()
            .and_then(|record| record.canary_enrollment.clone()),
        last_promotion: prior
            .as_ref()
            .and_then(|record| record.last_promotion.clone()),
        last_drift: prior.as_ref().and_then(|record| record.last_drift.clone()),
        last_rollback: prior
            .as_ref()
            .and_then(|record| record.last_rollback.clone()),
    };
    store::replace(&state.config.data_dir, record).map_err(|error| internal(&error))?;
    Ok(Json(json!({
        "schema": "focusa.compaction_policy_resolve_result.v1",
        "status": "resolved",
        "runtime_fingerprint": fingerprint,
        "workstream_hash": scope_key,
        "legal_actions": legal_actions,
        "resolution": resolution,
        "lease": lease,
        "candidate_count": candidates.len(),
        "capability_posture": if candidates.iter().any(|candidate| candidate.validation == ValidationState::Validated) { "validated" } else { "fallback_until_daemon_registry_proof" }
    })))
}

async fn status(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (key, _, _) =
        scope_key(&scope).map_err(|error| blocked(StatusCode::UNPROCESSABLE_ENTITY, &error))?;
    let record = store::get(&state.config.data_dir, &key)
        .ok_or_else(|| blocked(StatusCode::NOT_FOUND, "policy_not_resolved"))?;
    Ok(Json(json!({
        "schema":"focusa.compaction_policy_controller_status.v1",
        "status":"resolved",
        "runtime_identity":record.runtime_fingerprint,
        "legal_actions":record.legal_actions,
        "selected_policy":record.resolution.selected,
        "selection_reason":record.resolution.reason,
        "sample_size":record.resolution.sample_size,
        "confidence":record.resolution.confidence,
        "lease":record.lease,
        "canary_enrollment":record.canary_enrollment,
        "last_promotion":record.last_promotion,
        "drift_state":record.last_drift,
        "last_rollback":record.last_rollback,
        "observation_count":record.observations.len()
    })))
}

async fn candidates(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (key, _, _) =
        scope_key(&scope).map_err(|error| blocked(StatusCode::UNPROCESSABLE_ENTITY, &error))?;
    let record = store::get(&state.config.data_dir, &key)
        .ok_or_else(|| blocked(StatusCode::NOT_FOUND, "policy_not_resolved"))?;
    Ok(Json(
        json!({"schema":"focusa.compaction_policy_candidates.v1","status":"completed","candidates":record.candidates}),
    ))
}

async fn evidence(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (key, _, _) =
        scope_key(&scope).map_err(|error| blocked(StatusCode::UNPROCESSABLE_ENTITY, &error))?;
    let record = store::get(&state.config.data_dir, &key)
        .ok_or_else(|| blocked(StatusCode::NOT_FOUND, "policy_not_resolved"))?;
    Ok(Json(
        json!({"schema":"focusa.compaction_policy_evidence.v1","status":"completed","runtime_segment":record.runtime_fingerprint.segment_key,"evidence":record.evidence}),
    ))
}

async fn observe(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(req): Json<ObserveRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.schema != "focusa.compaction_policy_observe_request.v1" {
        return Err(blocked(StatusCode::BAD_REQUEST, "invalid_observe_schema"));
    }
    let (key, _, _) =
        scope_key(&scope).map_err(|error| blocked(StatusCode::UNPROCESSABLE_ENTITY, &error))?;
    let result = store::mutate(&state.config.data_dir, &key, |slot| {
        let record = slot.as_mut().ok_or_else(|| "policy_not_resolved".to_string())?;
        if req.observation.runtime_segment != record.runtime_fingerprint.segment_key
            || req.observation.workstream_hash != key
        {
            return Err("observation_scope_or_segment_mismatch".into());
        }
        if !record
            .observations
            .iter()
            .any(|item| item.epoch_id == req.observation.epoch_id)
        {
            record.observations.push_back(req.observation);
        }
        if let Some(input) = req.promotion {
            let verdict = evaluate_promotion(&input);
            if verdict.eligible
                && verdict.runtime_segment == record.runtime_fingerprint.segment_key
            {
                if let Some(policy) = record
                    .candidates
                    .iter_mut()
                    .find(|policy| policy.policy_id == verdict.policy_id)
                {
                    policy.validation = ValidationState::Validated;
                }
            }
            record.last_promotion = Some(verdict);
        }
        if let Some(input) = req.drift {
            let verdict = evaluate_drift(&input);
            if verdict.drifted
                && verdict.affected_runtime_segment == record.runtime_fingerprint.segment_key
            {
                for policy in &mut record.candidates {
                    if policy.validation != ValidationState::LegacyBaseline {
                        policy.validation = ValidationState::Quarantined;
                    }
                }
            }
            record.last_drift = Some(verdict);
        }
        Ok(json!({"schema":"focusa.compaction_policy_observe_result.v1","status":"recorded","observation_count":record.observations.len(),"promotion":record.last_promotion,"drift":record.last_drift}))
    })
    .map_err(|error| blocked(StatusCode::CONFLICT, &error))?;
    Ok(Json(result))
}

async fn canary_enroll(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(req): Json<CanaryRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (key, _, _) =
        scope_key(&scope).map_err(|error| blocked(StatusCode::UNPROCESSABLE_ENTITY, &error))?;
    let result = store::mutate(&state.config.data_dir, &key, |slot| {
        let record = slot.as_mut().ok_or_else(|| "policy_not_resolved".to_string())?;
        let receipt = enroll_dev_canary(
            &record.runtime_fingerprint.segment_key,
            &req.policy_id,
            &req.operator_ref,
            req.session_budget,
            Utc::now(),
        )?;
        let policy = record
            .candidates
            .iter_mut()
            .find(|policy| policy.policy_id == req.policy_id)
            .ok_or_else(|| "candidate_not_found".to_string())?;
        if policy.validation != ValidationState::Shadow {
            return Err("only_shadow_candidate_can_enter_canary".into());
        }
        policy.validation = ValidationState::Canary;
        record.canary_enrollment = Some(receipt.clone());
        Ok(json!({"schema":"focusa.compaction_canary_control_result.v1","status":"enrolled","receipt":receipt}))
    })
    .map_err(|error| blocked(StatusCode::CONFLICT, &error))?;
    Ok(Json(result))
}

async fn canary_pause(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(req): Json<CanaryPauseRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.operator_ref.trim().is_empty() || req.reason.trim().is_empty() {
        return Err(blocked(
            StatusCode::BAD_REQUEST,
            "operator_and_reason_required",
        ));
    }
    let (key, _, _) =
        scope_key(&scope).map_err(|error| blocked(StatusCode::UNPROCESSABLE_ENTITY, &error))?;
    let result = store::mutate(&state.config.data_dir, &key, |slot| {
        let record = slot.as_mut().ok_or_else(|| "policy_not_resolved".to_string())?;
        for policy in &mut record.candidates {
            if policy.validation == ValidationState::Canary {
                policy.validation = ValidationState::Shadow;
            }
        }
        record.canary_enrollment = None;
        Ok(json!({"schema":"focusa.compaction_canary_control_result.v1","status":"paused","operator_ref":req.operator_ref,"reason":req.reason,"reversible":true}))
    })
    .map_err(|error| blocked(StatusCode::CONFLICT, &error))?;
    Ok(Json(result))
}

async fn rollback(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(req): Json<RollbackRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (key, _, _) =
        scope_key(&scope).map_err(|error| blocked(StatusCode::UNPROCESSABLE_ENTITY, &error))?;
    let result = store::mutate(&state.config.data_dir, &key, |slot| {
        let record = slot.as_mut().ok_or_else(|| "policy_not_resolved".to_string())?;
        let context_window = record.runtime_fingerprint.context_window.unwrap_or(256_000);
        let receipt = rollback_to_legacy(
            context_window,
            &record.runtime_fingerprint.segment_key,
            &req.failed_policy_id,
            &req.primary_finding,
            Utc::now(),
        )?;
        if let Some(policy) = record
            .candidates
            .iter_mut()
            .find(|policy| policy.policy_id == req.failed_policy_id)
        {
            policy.validation = ValidationState::Quarantined;
        }
        let fixed = PolicySelectionContext {
            mode: PolicyMode::Fixed,
            context_window,
            sample_size: 0,
            measured_confidence: None,
            minimum_samples: 20,
            required_confidence: 0.95,
            dev_fleet_enrolled: false,
        };
        record.resolution = resolve_policy(&fixed, &record.candidates);
        record.lease = CompactionPolicyLease::freeze(
            &record.resolution,
            &record.runtime_fingerprint.segment_key,
            &record.runtime_fingerprint.capability_evidence_revision,
            "sha256:rollback",
        );
        record.last_rollback = Some(receipt.clone());
        Ok(json!({"schema":"focusa.compaction_policy_rollback_result.v1","status":"rolled_back","receipt":receipt,"lease":record.lease}))
    })
    .map_err(|error| blocked(StatusCode::CONFLICT, &error))?;
    Ok(Json(result))
}

fn blocked(status: StatusCode, error: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(
            json!({"schema":"focusa.compaction_policy_controller_error.v1","status":"blocked","error":error.chars().take(240).collect::<String>()}),
        ),
    )
}

fn internal(error: &str) -> (StatusCode, Json<Value>) {
    blocked(StatusCode::INTERNAL_SERVER_ERROR, error)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/compaction/policy/resolve", post(resolve))
        .route("/v1/compaction/policy/observe", post(observe))
        .route("/v1/compaction/policy/status", get(status))
        .route("/v1/compaction/policy/candidates", get(candidates))
        .route("/v1/compaction/policy/evidence", get(evidence))
        .route("/v1/compaction/policy/canary/enroll", post(canary_enroll))
        .route("/v1/compaction/policy/canary/pause", post(canary_pause))
        .route("/v1/compaction/policy/rollback", post(rollback))
}
