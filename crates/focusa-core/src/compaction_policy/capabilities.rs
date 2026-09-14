use super::{CompactionRuntimeFingerprint, ContextManagementAction};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    Proven,
    Unsupported,
    Unknown,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityEvidence {
    pub capability: String,
    pub runtime_segment: String,
    pub state: CapabilityState,
    pub source: String,
    pub adapter_revision: String,
    pub proof_ref: Option<String>,
    pub proof_digest: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub expires_after: Option<DateTime<Utc>>,
}

fn proven(
    evidence: &BTreeMap<String, CapabilityEvidence>,
    capability: &str,
    fingerprint: &CompactionRuntimeFingerprint,
    now: DateTime<Utc>,
) -> bool {
    evidence.get(capability).is_some_and(|item| {
        item.state == CapabilityState::Proven
            && item.runtime_segment == fingerprint.segment_key
            && item.adapter_revision == fingerprint.adapter_revision
            && item.expires_after.is_none_or(|expiry| expiry > now)
            && item
                .proof_ref
                .as_deref()
                .is_some_and(|value| !value.is_empty())
    })
}

/// Deterministic legal-action kernel. Provider/model names are deliberately
/// absent; only exact, current capability proof can expand the mask.
pub fn legal_action_mask(
    fingerprint: &CompactionRuntimeFingerprint,
    evidence: &[CapabilityEvidence],
    now: DateTime<Utc>,
) -> BTreeSet<ContextManagementAction> {
    let by_name: BTreeMap<String, CapabilityEvidence> = evidence
        .iter()
        .cloned()
        .map(|item| (item.capability.clone(), item))
        .collect();
    let mut actions = BTreeSet::from([
        ContextManagementAction::NoAction,
        ContextManagementAction::CheckpointOnly,
        ContextManagementAction::RebuildMinimalProjection,
        ContextManagementAction::ExternalizeToolArtifacts,
        ContextManagementAction::SelectiveRehydrate,
        ContextManagementAction::PiStructuredCompaction,
        ContextManagementAction::CheckpointAndRollover,
    ]);
    let native_contracts = [
        [
            "openai_opaque_compaction_request",
            "openai_opaque_compaction_item_round_trip",
        ]
        .as_slice(),
        [
            "anthropic_compaction_request",
            "anthropic_compaction_block_round_trip",
        ]
        .as_slice(),
    ];
    if native_contracts.iter().any(|contract| {
        contract
            .iter()
            .all(|name| proven(&by_name, name, fingerprint, now))
            && proven(&by_name, "reasoning_state_round_trip", fingerprint, now)
            && proven(
                &by_name,
                "continuation_survives_process_resume",
                fingerprint,
                now,
            )
            && proven(
                &by_name,
                "continuation_survives_transport_fallback",
                fingerprint,
                now,
            )
    }) {
        actions.insert(ContextManagementAction::ProviderNativeCompaction);
    }
    if proven(&by_name, "tool_result_context_editing", fingerprint, now) {
        actions.insert(ContextManagementAction::ProviderToolResultEdit);
    }
    if proven(&by_name, "thinking_context_editing", fingerprint, now)
        && proven(&by_name, "reasoning_state_round_trip", fingerprint, now)
    {
        actions.insert(ContextManagementAction::ProviderThinkingEdit);
    }
    actions
}
