use super::{ContextPolicyBundle, ValidationState, legacy_current_v1};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromotionInput {
    pub policy_id: String,
    pub runtime_segment: String,
    pub sample_size: u64,
    pub minimum_samples: u64,
    pub confidence: f64,
    pub required_confidence: f64,
    pub task_success_delta_lcb: f64,
    pub noninferiority_epsilon: f64,
    pub authority_fidelity_regressions: u64,
    pub operator_input_regressions: u64,
    pub recovery_regressions: u64,
    pub provider_round_trip_failures: u64,
    pub productive_efficiency_delta: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromotionVerdict {
    pub schema: String,
    pub policy_id: String,
    pub runtime_segment: String,
    pub eligible: bool,
    pub target_state: ValidationState,
    pub reasons: Vec<String>,
    pub evidence_digest: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryEnrollmentReceipt {
    pub schema: String,
    pub receipt_id: String,
    pub runtime_segment: String,
    pub policy_id: String,
    pub operator_ref: String,
    pub session_budget: u32,
    pub enrolled_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub reversible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftInput {
    pub prior_runtime_segment: String,
    pub current_runtime_segment: String,
    pub response_model_changed: bool,
    pub adapter_revision_changed: bool,
    pub capability_revision_changed: bool,
    pub transport_fallback: bool,
    pub context_window_changed: bool,
    pub protocol_error_count: u32,
    pub latency_ratio: Option<f64>,
    pub context_release_delta: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftVerdict {
    pub schema: String,
    pub affected_runtime_segment: String,
    pub drifted: bool,
    pub quarantine_segment: bool,
    pub invalidate_capability_proof: bool,
    pub reasons: Vec<String>,
    pub fallback_policy_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollbackReceipt {
    pub schema: String,
    pub receipt_id: String,
    pub runtime_segment: String,
    pub failed_policy_id: String,
    pub primary_finding: String,
    pub fallback: ContextPolicyBundle,
    pub avoid_additional_model_turn: bool,
    pub prepared_packet_preserved: bool,
    pub recorded_at: DateTime<Utc>,
}

fn digest<T: Serialize>(value: &T) -> String {
    format!(
        "sha256:{}",
        hex::encode(Sha256::digest(
            serde_json::to_vec(value).unwrap_or_default()
        ))
    )
}

pub fn evaluate_promotion(input: &PromotionInput) -> PromotionVerdict {
    let mut reasons = Vec::new();
    if input.sample_size < input.minimum_samples {
        reasons.push("minimum_samples_not_met".into());
    }
    if !input.confidence.is_finite()
        || !input.required_confidence.is_finite()
        || input.confidence < input.required_confidence
    {
        reasons.push("confidence_gate_failed".into());
    }
    if !input.task_success_delta_lcb.is_finite()
        || input.task_success_delta_lcb < -input.noninferiority_epsilon.abs()
    {
        reasons.push("task_success_noninferiority_failed".into());
    }
    for (count, reason) in [
        (
            input.authority_fidelity_regressions,
            "authority_fidelity_regression",
        ),
        (
            input.operator_input_regressions,
            "operator_input_regression",
        ),
        (input.recovery_regressions, "recovery_regression"),
        (
            input.provider_round_trip_failures,
            "provider_round_trip_failure",
        ),
    ] {
        if count > 0 {
            reasons.push(reason.into());
        }
    }
    if !input.productive_efficiency_delta.is_finite() {
        reasons.push("efficiency_measurement_invalid".into());
    }
    PromotionVerdict {
        schema: "focusa.compaction_policy_promotion_verdict.v1".into(),
        policy_id: input.policy_id.clone(),
        runtime_segment: input.runtime_segment.clone(),
        eligible: reasons.is_empty(),
        target_state: if reasons.is_empty() {
            ValidationState::Validated
        } else {
            ValidationState::Shadow
        },
        evidence_digest: digest(input),
        reasons,
    }
}

pub fn enroll_dev_canary(
    runtime_segment: &str,
    policy_id: &str,
    operator_ref: &str,
    session_budget: u32,
    now: DateTime<Utc>,
) -> Result<CanaryEnrollmentReceipt, String> {
    if runtime_segment.trim().is_empty()
        || policy_id.trim().is_empty()
        || operator_ref.trim().is_empty()
    {
        return Err("explicit_segment_policy_and_operator_required".into());
    }
    if !(1..=100).contains(&session_budget) {
        return Err("canary_session_budget_out_of_range".into());
    }
    let material = (
        runtime_segment,
        policy_id,
        operator_ref,
        session_budget,
        now,
    );
    Ok(CanaryEnrollmentReceipt {
        schema: "focusa.compaction_canary_enrollment_receipt.v1".into(),
        receipt_id: digest(&material),
        runtime_segment: runtime_segment.into(),
        policy_id: policy_id.into(),
        operator_ref: operator_ref.into(),
        session_budget,
        enrolled_at: now,
        expires_at: now + Duration::hours(24),
        reversible: true,
    })
}

pub fn evaluate_drift(input: &DriftInput) -> DriftVerdict {
    let mut reasons = Vec::new();
    if input.prior_runtime_segment != input.current_runtime_segment {
        reasons.push("runtime_segment_changed".into());
    }
    for (changed, reason) in [
        (input.response_model_changed, "response_model_changed"),
        (input.adapter_revision_changed, "adapter_revision_changed"),
        (
            input.capability_revision_changed,
            "capability_revision_changed",
        ),
        (input.transport_fallback, "transport_fallback"),
        (input.context_window_changed, "context_window_changed"),
    ] {
        if changed {
            reasons.push(reason.into());
        }
    }
    if input.protocol_error_count > 0 {
        reasons.push("provider_protocol_error".into());
    }
    if input
        .latency_ratio
        .is_some_and(|ratio| !ratio.is_finite() || ratio > 2.0)
    {
        reasons.push("latency_change_point".into());
    }
    if input
        .context_release_delta
        .is_some_and(|delta| !delta.is_finite() || delta < -0.2)
    {
        reasons.push("context_release_change_point".into());
    }
    DriftVerdict {
        schema: "focusa.compaction_policy_drift_verdict.v1".into(),
        affected_runtime_segment: input.prior_runtime_segment.clone(),
        drifted: !reasons.is_empty(),
        quarantine_segment: !reasons.is_empty(),
        invalidate_capability_proof: input.protocol_error_count > 0
            || input.adapter_revision_changed
            || input.capability_revision_changed,
        reasons,
        fallback_policy_id: "legacy_current_v1".into(),
    }
}

pub fn rollback_to_legacy(
    context_window: u64,
    runtime_segment: &str,
    failed_policy_id: &str,
    primary_finding: &str,
    now: DateTime<Utc>,
) -> Result<RollbackReceipt, String> {
    if runtime_segment.trim().is_empty()
        || failed_policy_id.trim().is_empty()
        || primary_finding.trim().is_empty()
    {
        return Err("rollback_scope_policy_and_finding_required".into());
    }
    let material = (runtime_segment, failed_policy_id, primary_finding, now);
    Ok(RollbackReceipt {
        schema: "focusa.compaction_policy_rollback_receipt.v1".into(),
        receipt_id: digest(&material),
        runtime_segment: runtime_segment.into(),
        failed_policy_id: failed_policy_id.into(),
        primary_finding: primary_finding.chars().take(240).collect(),
        fallback: legacy_current_v1(context_window),
        avoid_additional_model_turn: true,
        prepared_packet_preserved: true,
        recorded_at: now,
    })
}
