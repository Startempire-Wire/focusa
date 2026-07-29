//! Spec 140 zero-hidden-change instruction migration planning.

use crate::agent_runtime_constitution::{
    InstructionConflict, InstructionSource, InstructionTrustClass,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MigrationDisposition {
    CanonicalRoot,
    NestedDelta,
    DuplicateSource,
    QuarantineUntrusted,
    ExcludeVolatile,
    ManualReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionMigrationEntry {
    pub source_id: String,
    pub source_ref: String,
    pub content_sha256: String,
    pub disposition: MigrationDisposition,
    pub reason_code: String,
    pub target_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRuntimeMigrationPlan {
    pub plan_id: String,
    pub entries: Vec<InstructionMigrationEntry>,
    pub unresolved_conflict_refs: Vec<String>,
    pub ordered_phases: Vec<String>,
    pub hidden_behavior_changes_allowed: bool,
    pub delivery_blocked: bool,
}

pub fn plan_instruction_migration(
    plan_id: impl Into<String>,
    sources: &[InstructionSource],
    conflicts: &[InstructionConflict],
) -> AgentRuntimeMigrationPlan {
    let mut first_by_hash = BTreeMap::<String, String>::new();
    let mut entries = Vec::new();
    for source in sources {
        let duplicate_of = first_by_hash.get(&source.content_sha256).cloned();
        let (disposition, reason_code, target_ref) = if let Some(original) = duplicate_of {
            (
                MigrationDisposition::DuplicateSource,
                "content_hash_duplicate".into(),
                Some(original),
            )
        } else {
            first_by_hash.insert(source.content_sha256.clone(), source.source_ref.clone());
            if matches!(
                source.trust,
                InstructionTrustClass::Untrusted | InstructionTrustClass::Quarantined
            ) {
                (
                    MigrationDisposition::QuarantineUntrusted,
                    "source_not_approved".into(),
                    None,
                )
            } else if is_volatile(&source.source_ref) {
                (
                    MigrationDisposition::ExcludeVolatile,
                    "volatile_runtime_state_not_stable_instruction".into(),
                    None,
                )
            } else if source.source_ref == "AGENTS.md" || !source.source_ref.contains('/') {
                (
                    MigrationDisposition::CanonicalRoot,
                    "trusted_root_instruction".into(),
                    Some("AGENTS.md".into()),
                )
            } else if source.source_ref.ends_with("AGENTS.md") {
                (
                    MigrationDisposition::NestedDelta,
                    "trusted_nested_delta".into(),
                    Some(source.source_ref.clone()),
                )
            } else {
                (
                    MigrationDisposition::ManualReview,
                    "trusted_noncanonical_source_requires_mapping".into(),
                    None,
                )
            }
        };
        entries.push(InstructionMigrationEntry {
            source_id: source.source_id.clone(),
            source_ref: source.source_ref.clone(),
            content_sha256: source.content_sha256.clone(),
            disposition,
            reason_code,
            target_ref,
        });
    }
    entries.sort_by(|left, right| left.source_ref.cmp(&right.source_ref));
    let unresolved_conflict_refs: Vec<_> = conflicts
        .iter()
        .map(|conflict| conflict.conflict_id.clone())
        .collect();
    let delivery_blocked = !unresolved_conflict_refs.is_empty()
        || entries
            .iter()
            .any(|entry| entry.disposition == MigrationDisposition::ManualReview);
    AgentRuntimeMigrationPlan {
        plan_id: plan_id.into(),
        entries,
        unresolved_conflict_refs,
        ordered_phases: vec![
            "inventory_and_quarantine".into(),
            "canonical_contracts".into(),
            "crist_integration".into(),
            "agents_rules_skills".into(),
            "prompt_compiler".into(),
            "enforcement".into(),
            "runtime_studio_and_evaluation".into(),
        ],
        hidden_behavior_changes_allowed: false,
        delivery_blocked,
    }
}

pub fn verify_migration_plan(
    plan: &AgentRuntimeMigrationPlan,
    expected_source_ids: &BTreeSet<String>,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let actual: BTreeSet<_> = plan
        .entries
        .iter()
        .map(|entry| entry.source_id.clone())
        .collect();
    if &actual != expected_source_ids {
        errors.push("migration_source_coverage_incomplete".into());
    }
    if plan.hidden_behavior_changes_allowed {
        errors.push("hidden_behavior_change_forbidden".into());
    }
    if plan.ordered_phases.len() != 7 {
        errors.push("migration_phase_sequence_incomplete".into());
    }
    if plan.delivery_blocked
        && plan.unresolved_conflict_refs.is_empty()
        && !plan
            .entries
            .iter()
            .any(|entry| entry.disposition == MigrationDisposition::ManualReview)
    {
        errors.push("delivery_block_without_reason".into());
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn is_volatile(source_ref: &str) -> bool {
    let normalized = source_ref.to_ascii_lowercase();
    [
        "terminal-output",
        "presence",
        "current-time",
        "lease",
        "resource-pressure",
        "live-topology",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}
