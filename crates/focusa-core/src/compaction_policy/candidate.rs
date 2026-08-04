use super::{ContextManagementAction, ValidationState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextPolicyBundle {
    pub schema: String,
    pub policy_id: String,
    pub revision: String,
    pub validation: ValidationState,
    pub actions: Vec<ContextManagementAction>,
    pub checkpoint_at_tokens: u64,
    pub compact_at_tokens: Option<u64>,
    pub hard_at_tokens: u64,
    pub reserve_tokens: u64,
    pub projection_budget_tokens: u64,
    pub attempt_cooldown_ms: u64,
    pub retry_cooldown_ms: u64,
    pub successful_compaction_cooldown_ms: u64,
    pub max_transient_retries: u8,
    pub max_compactions_per_hour: u8,
    pub min_turns_between_compactions: u8,
    pub objective_profile: String,
    pub rollback_policy_id: String,
}

pub fn legacy_current_v1(context_window: u64) -> ContextPolicyBundle {
    let window = context_window.max(32_768);
    let reserve_tokens = 16_384_u64.max(window / 10);
    let compact_at = (window.saturating_mul(70) / 100).min(256_000);
    let hard_at = window.saturating_mul(85) / 100;
    ContextPolicyBundle {
        schema: "focusa.context_policy_bundle.v1".into(),
        policy_id: "legacy_current_v1".into(),
        revision: "1".into(),
        validation: ValidationState::LegacyBaseline,
        actions: vec![
            ContextManagementAction::CheckpointOnly,
            ContextManagementAction::PiStructuredCompaction,
        ],
        checkpoint_at_tokens: compact_at.saturating_sub(8_192),
        compact_at_tokens: Some(compact_at),
        hard_at_tokens: hard_at,
        reserve_tokens,
        projection_budget_tokens: 900,
        attempt_cooldown_ms: 60_000,
        retry_cooldown_ms: 60_000,
        successful_compaction_cooldown_ms: 180_000,
        max_transient_retries: 1,
        max_compactions_per_hour: 8,
        min_turns_between_compactions: 3,
        objective_profile: "daily_driver".into(),
        rollback_policy_id: "legacy_current_v1".into(),
    }
}

/// Compile a finite neighboring lattice. No arbitrary production parameter
/// search is permitted and hard pressure is never raised.
pub fn compile_policy_lattice(
    context_window: u64,
    legal_actions: &BTreeSet<ContextManagementAction>,
    objective_profile: &str,
    predicted_safe_tokens: Option<u64>,
) -> Vec<ContextPolicyBundle> {
    let legacy = legacy_current_v1(context_window);
    let hard = legacy.hard_at_tokens;
    let reserve = legacy.reserve_tokens;
    let maximum = context_window.saturating_sub(reserve).min(hard);
    let mut triggers = vec![
        context_window.saturating_mul(55) / 100,
        context_window.saturating_mul(60) / 100,
        context_window.saturating_mul(65) / 100,
        legacy.compact_at_tokens.unwrap_or(maximum),
        context_window.saturating_mul(75) / 100,
        context_window.saturating_mul(80) / 100,
    ];
    if let Some(predicted) = predicted_safe_tokens {
        triggers.push(predicted);
    }
    triggers
        .iter_mut()
        .for_each(|value| *value = (*value).min(maximum).min(256_000));
    triggers.sort_unstable();
    triggers.dedup();
    let mut policies = vec![legacy.clone()];
    for trigger in triggers {
        if Some(trigger) == legacy.compact_at_tokens || trigger < 16_384 {
            continue;
        }
        let mut policy = legacy.clone();
        policy.policy_id = format!("candidate_{}_v1", trigger);
        policy.validation = ValidationState::Shadow;
        policy.compact_at_tokens = Some(trigger);
        policy.checkpoint_at_tokens = trigger.saturating_sub(8_192);
        policy.objective_profile = objective_profile.chars().take(48).collect();
        policy.actions =
            if legal_actions.contains(&ContextManagementAction::ProviderNativeCompaction) {
                vec![
                    ContextManagementAction::CheckpointOnly,
                    ContextManagementAction::ProviderNativeCompaction,
                ]
            } else {
                legacy.actions.clone()
            };
        policies.push(policy);
    }
    policies.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
    policies
}
