use super::{ContextPolicyBundle, ValidationState, legacy_current_v1};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyMode {
    Fixed,
    Shadow,
    Canary,
    Adaptive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicySelectionContext {
    pub mode: PolicyMode,
    pub context_window: u64,
    pub sample_size: u64,
    pub measured_confidence: Option<f64>,
    pub minimum_samples: u64,
    pub required_confidence: f64,
    pub dev_fleet_enrolled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyResolution {
    pub schema: String,
    pub mode: PolicyMode,
    pub selected: ContextPolicyBundle,
    pub shadow_candidate: Option<ContextPolicyBundle>,
    pub reason: String,
    pub sample_size: u64,
    pub confidence: Option<f64>,
    pub fallback_policy_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompactionPolicyLease {
    pub schema: String,
    pub lease_id: String,
    pub runtime_fingerprint_hash: String,
    pub capability_revision: String,
    pub policy_id: String,
    pub policy_revision: String,
    pub resolution_reason: String,
    pub feature_vector_hash: String,
    pub predicted_utility: Option<f64>,
    pub confidence: Option<f64>,
    pub fallback_policy_id: String,
    pub rollback_conditions: Vec<String>,
}

fn confidence(value: Option<f64>) -> Option<f64> {
    value
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(0.0, 1.0))
}

/// Stable production can select only the exact baseline or a validated policy.
/// Shadow never changes execution; canary additionally requires explicit dev
/// fleet enrollment.
pub fn resolve_policy(
    context: &PolicySelectionContext,
    candidates: &[ContextPolicyBundle],
) -> PolicyResolution {
    let baseline = candidates
        .iter()
        .find(|policy| policy.policy_id == "legacy_current_v1")
        .cloned()
        .unwrap_or_else(|| legacy_current_v1(context.context_window));
    let confidence = confidence(context.measured_confidence);
    let eligible = |policy: &&ContextPolicyBundle| {
        policy.validation == ValidationState::Validated
            && context.sample_size >= context.minimum_samples
            && confidence.is_some_and(|value| value >= context.required_confidence)
    };
    let validated = candidates.iter().filter(eligible).min_by(|left, right| {
        left.compact_at_tokens
            .unwrap_or(u64::MAX)
            .cmp(&right.compact_at_tokens.unwrap_or(u64::MAX))
            .then_with(|| left.policy_id.cmp(&right.policy_id))
    });
    let shadow = candidates
        .iter()
        .find(|policy| policy.validation == ValidationState::Shadow)
        .cloned();
    let (selected, reason) = match context.mode {
        PolicyMode::Fixed => (baseline.clone(), "fixed_legacy_baseline"),
        PolicyMode::Shadow => (baseline.clone(), "shadow_does_not_change_execution"),
        PolicyMode::Adaptive => validated
            .cloned()
            .map(|policy| (policy, "validated_adaptive_policy"))
            .unwrap_or_else(|| (baseline.clone(), "no_validated_noninferior_candidate")),
        PolicyMode::Canary if context.dev_fleet_enrolled => candidates
            .iter()
            .find(|policy| policy.validation == ValidationState::Canary)
            .cloned()
            .map(|policy| (policy, "operator_dev_fleet_canary"))
            .unwrap_or_else(|| (baseline.clone(), "no_enrolled_canary_candidate")),
        PolicyMode::Canary => (baseline.clone(), "canary_enrollment_required"),
    };
    PolicyResolution {
        schema: "focusa.compaction_policy_resolution.v1".into(),
        mode: context.mode,
        selected,
        shadow_candidate: shadow,
        reason: reason.into(),
        sample_size: context.sample_size,
        confidence,
        fallback_policy_id: baseline.policy_id,
    }
}

impl CompactionPolicyLease {
    pub fn freeze(
        resolution: &PolicyResolution,
        runtime_fingerprint_hash: &str,
        capability_revision: &str,
        feature_vector_hash: &str,
    ) -> Self {
        let material = serde_json::json!({
            "runtime": runtime_fingerprint_hash,
            "capabilities": capability_revision,
            "policy": resolution.selected.policy_id,
            "revision": resolution.selected.revision,
            "features": feature_vector_hash,
        });
        let lease_id = format!(
            "sha256:{}",
            hex::encode(Sha256::digest(
                serde_json::to_vec(&material).unwrap_or_default()
            ))
        );
        Self {
            schema: "focusa.compaction_policy_lease.v1".into(),
            lease_id,
            runtime_fingerprint_hash: runtime_fingerprint_hash.into(),
            capability_revision: capability_revision.into(),
            policy_id: resolution.selected.policy_id.clone(),
            policy_revision: resolution.selected.revision.clone(),
            resolution_reason: resolution.reason.clone(),
            feature_vector_hash: feature_vector_hash.into(),
            predicted_utility: None,
            confidence: resolution.confidence,
            fallback_policy_id: resolution.fallback_policy_id.clone(),
            rollback_conditions: vec![
                "scope_or_authority_mismatch".into(),
                "provider_protocol_failure".into(),
                "operator_input_loss_or_duplication".into(),
                "opaque_state_round_trip_failure".into(),
                "ineffective_compaction".into(),
                "operator_rollback".into(),
            ],
        }
    }
}
