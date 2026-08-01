//! Spec 144 §23 executable, authority-bounded semantic reflex routing.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const SHARED_SEMANTIC_REFLEXES: &[&str] = &[
    "detect_verification_domain_impact",
    "compile_verification_obligations",
    "detect_uncovered_mandatory_obligation",
    "detect_cross_domain_verification_conflict",
    "freeze_verification_snapshot",
    "invalidate_verification_after_snapshot_change",
    "resume_open_verification_obligations",
    "supersede_stale_verifier_context",
    "route_verification_portfolio",
    "enforce_verifier_capability_eligibility",
    "enforce_verifier_independence",
    "escalate_assurance_tier",
    "reroute_on_new_finding",
    "reroute_on_verifier_failure",
    "require_evidence_for_verifier_finding",
    "reject_unsupported_critical_finding",
    "block_settlement_on_open_critical_finding",
    "block_settlement_on_uncovered_obligation",
    "verify_final_snapshot_matches_verified_snapshot",
    "retry_verifier_with_bounded_fallback",
    "replace_unavailable_verifier",
    "route_disagreement_to_arbiter",
    "escalate_inconclusive_verification",
    "detect_build_verify_oscillation",
    "record_verifier_prediction",
    "evaluate_verifier_after_settlement",
    "detect_verifier_false_positive_pattern",
    "detect_verifier_false_negative_pattern",
    "detect_negative_verifier_transfer",
    "propose_verifier_policy_adjustment",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflexRuntimeStatus {
    Executable,
    SchemaOnly,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticReflexDefinition {
    pub reflex_id: String,
    pub trigger_types: BTreeSet<String>,
    pub required_context_keys: BTreeSet<String>,
    pub action_type: String,
    pub evidence_types: BTreeSet<String>,
    pub escalation_boundary: String,
    pub authority_scope: String,
    pub requirement_ids: BTreeSet<String>,
    pub max_actions: u32,
    pub timeout_ms: u64,
    pub failure_envelope: String,
    pub runtime_status: ReflexRuntimeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticReflexInvocation {
    pub reflex_id: String,
    pub trigger_type: String,
    pub project_root: String,
    pub continuity_id: String,
    pub authority_scope: String,
    pub context: BTreeMap<String, String>,
    pub requested_actions: u32,
    pub mutation_requested: bool,
    pub operator_confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticReflexOutcome {
    pub reflex_id: String,
    pub action_type: String,
    pub evidence_refs: Vec<String>,
    pub escalation_required: bool,
    pub escalation_boundary: String,
    pub degraded: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SemanticReflexError {
    #[error("semantic reflex is registry-only schema")]
    SchemaOnly,
    #[error("semantic reflex is disabled")]
    Disabled,
    #[error("semantic reflex trigger does not match")]
    TriggerMismatch,
    #[error("semantic reflex context is incomplete: {0}")]
    MissingContext(String),
    #[error("semantic reflex authority does not match")]
    AuthorityMismatch,
    #[error("semantic reflex action budget exceeded")]
    BudgetExceeded,
    #[error("semantic reflex mutation requires operator confirmation")]
    ConfirmationRequired,
    #[error("semantic reflex produced no evidence")]
    EvidenceRequired,
}

pub fn execute_semantic_reflex(
    definition: &SemanticReflexDefinition,
    invocation: &SemanticReflexInvocation,
    evidence_refs: Vec<String>,
) -> Result<SemanticReflexOutcome, SemanticReflexError> {
    match definition.runtime_status {
        ReflexRuntimeStatus::SchemaOnly => return Err(SemanticReflexError::SchemaOnly),
        ReflexRuntimeStatus::Disabled => return Err(SemanticReflexError::Disabled),
        ReflexRuntimeStatus::Executable => {}
    }
    if invocation.reflex_id != definition.reflex_id
        || !definition.trigger_types.contains(&invocation.trigger_type)
    {
        return Err(SemanticReflexError::TriggerMismatch);
    }
    if invocation.authority_scope != definition.authority_scope {
        return Err(SemanticReflexError::AuthorityMismatch);
    }
    if let Some(key) = definition
        .required_context_keys
        .iter()
        .find(|key| !invocation.context.contains_key(*key))
    {
        return Err(SemanticReflexError::MissingContext(key.clone()));
    }
    if invocation.requested_actions > definition.max_actions {
        return Err(SemanticReflexError::BudgetExceeded);
    }
    if invocation.mutation_requested && !invocation.operator_confirmed {
        return Err(SemanticReflexError::ConfirmationRequired);
    }
    if evidence_refs.is_empty() || definition.evidence_types.is_empty() {
        return Err(SemanticReflexError::EvidenceRequired);
    }
    Ok(SemanticReflexOutcome {
        reflex_id: definition.reflex_id.clone(),
        action_type: definition.action_type.clone(),
        evidence_refs,
        escalation_required: invocation
            .context
            .get("critical")
            .is_some_and(|value| value == "true"),
        escalation_boundary: definition.escalation_boundary.clone(),
        degraded: false,
    })
}

pub fn shared_reflex_catalog_is_complete(definitions: &[SemanticReflexDefinition]) -> bool {
    let ids: BTreeSet<_> = definitions
        .iter()
        .map(|item| item.reflex_id.as_str())
        .collect();
    SHARED_SEMANTIC_REFLEXES.iter().all(|id| ids.contains(id))
}

#[cfg(test)]
#[path = "semantic_reflex_tests.rs"]
mod tests;
