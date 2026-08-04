//! Deterministic, provider-neutral context-management policy authority.

mod candidate;
mod capabilities;
mod identity;
mod pressure;
mod provider_strategies;
mod registry;
mod selector;
mod semantic_pressure;
#[cfg(test)]
mod tests;

pub use candidate::{ContextPolicyBundle, compile_policy_lattice, legacy_current_v1};
pub use capabilities::{CapabilityEvidence, CapabilityState, legal_action_mask};
pub use identity::{
    CompactionRuntimeFacts, CompactionRuntimeFingerprint, resolve_runtime_fingerprint,
};
pub use pressure::{PressurePrediction, PressurePredictionInput, PressureStatistics};
pub use provider_strategies::{
    AnthropicCompactionState, CacheCostObservation, GeminiContinuationState, OpenAiCompactionState,
    ProviderContinuationState, ProviderStrategy, ProviderUsage, ToolResultState,
    aggregate_anthropic_usage, provider_strategy, tool_edit_break_even,
};
pub use registry::{CompactionPolicyObservation, CompactionPolicyRegistry, SegmentProjection};
pub use selector::{
    CompactionPolicyLease, PolicyMode, PolicyResolution, PolicySelectionContext, resolve_policy,
};
pub use semantic_pressure::{SemanticPressureSignals, recommend_semantic_repair};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextManagementAction {
    NoAction,
    CheckpointOnly,
    RebuildMinimalProjection,
    ExternalizeToolArtifacts,
    SelectiveRehydrate,
    ProviderToolResultEdit,
    ProviderThinkingEdit,
    ProviderNativeCompaction,
    PiStructuredCompaction,
    CheckpointAndRollover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationState {
    LegacyBaseline,
    Shadow,
    Canary,
    Validated,
    Quarantined,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEvidenceRef {
    pub evidence_ref: String,
    pub digest: Option<String>,
}
