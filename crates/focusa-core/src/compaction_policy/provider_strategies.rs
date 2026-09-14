use super::{CapabilityEvidence, CapabilityState, CompactionRuntimeFingerprint};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStrategy {
    OpenAiOpaqueCompaction,
    AnthropicServerCompaction,
    AnthropicToolResultEditing,
    GeminiStatefulInteraction,
    GeminiSignatureReplay,
    PiStructuredFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenAiCompactionState {
    pub opaque_compaction_item: Vec<u8>,
    pub encrypted_reasoning_items: Vec<Vec<u8>>,
    pub previous_response_id: Option<String>,
    pub full_output_replay: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnthropicCompactionState {
    pub beta_revision: String,
    pub compaction_block: Vec<u8>,
    pub stop_reason: String,
    pub usage_iterations: Vec<ProviderUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiContinuationState {
    pub previous_interaction_id: Option<String>,
    pub thought_signatures: Vec<Vec<u8>>,
    pub parallel_call_signatures: BTreeMap<String, Vec<u8>>,
    pub request_scoped_tools_digest: String,
    pub system_instruction_digest: String,
    pub generation_config_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "strategy", content = "state", rename_all = "snake_case")]
pub enum ProviderContinuationState {
    OpenAi(OpenAiCompactionState),
    Anthropic(AnthropicCompactionState),
    Gemini(GeminiContinuationState),
    Fallback { structured_summary_ref: String },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResultState {
    pub tool_call_id: String,
    pub tokens: u64,
    pub action_critical: bool,
    pub evidence_critical: bool,
    pub active_blocker: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheCostObservation {
    pub clearable_tokens: u64,
    pub cache_rewrite_tokens: u64,
    pub edit_overhead_tokens: u64,
}

fn proven(
    evidence: &BTreeMap<&str, &CapabilityEvidence>,
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

fn all(
    evidence: &BTreeMap<&str, &CapabilityEvidence>,
    names: &[&str],
    fingerprint: &CompactionRuntimeFingerprint,
    now: DateTime<Utc>,
) -> bool {
    names
        .iter()
        .all(|name| proven(evidence, name, fingerprint, now))
}

/// Select only from exact adapter proof. Provider/model names are not inputs to
/// strategy authorization; first-party identity itself is a proven capability.
pub fn provider_strategy(
    fingerprint: &CompactionRuntimeFingerprint,
    evidence: &[CapabilityEvidence],
    now: DateTime<Utc>,
) -> ProviderStrategy {
    let evidence: BTreeMap<&str, &CapabilityEvidence> = evidence
        .iter()
        .map(|item| (item.capability.as_str(), item))
        .collect();
    if all(
        &evidence,
        &[
            "first_party_openai_identity",
            "openai_opaque_compaction_request",
            "openai_opaque_compaction_item_round_trip",
            "reasoning_state_round_trip",
            "continuation_survives_process_resume",
            "continuation_survives_transport_fallback",
        ],
        fingerprint,
        now,
    ) && (proven(
        &evidence,
        "previous_response_continuation",
        fingerprint,
        now,
    ) || proven(&evidence, "full_output_replay", fingerprint, now))
    {
        return ProviderStrategy::OpenAiOpaqueCompaction;
    }
    if all(
        &evidence,
        &[
            "anthropic_compaction_request",
            "anthropic_compaction_block_round_trip",
            "anthropic_stop_reason_compaction",
            "anthropic_usage_iterations",
            "reasoning_state_round_trip",
            "continuation_survives_process_resume",
            "continuation_survives_transport_fallback",
        ],
        fingerprint,
        now,
    ) {
        return ProviderStrategy::AnthropicServerCompaction;
    }
    if proven(&evidence, "tool_result_context_editing", fingerprint, now)
        && proven(&evidence, "prompt_cache_cost_accounting", fingerprint, now)
    {
        return ProviderStrategy::AnthropicToolResultEditing;
    }
    if all(
        &evidence,
        &[
            "previous_interaction_continuation",
            "gemini_request_scoped_config_replay",
            "thought_signature_round_trip",
        ],
        fingerprint,
        now,
    ) {
        return ProviderStrategy::GeminiStatefulInteraction;
    }
    if all(
        &evidence,
        &[
            "thought_signature_round_trip",
            "gemini_request_scoped_config_replay",
        ],
        fingerprint,
        now,
    ) {
        return ProviderStrategy::GeminiSignatureReplay;
    }
    ProviderStrategy::PiStructuredFallback
}

pub fn aggregate_anthropic_usage(iterations: &[ProviderUsage]) -> ProviderUsage {
    iterations
        .iter()
        .fold(ProviderUsage::default(), |total, item| ProviderUsage {
            input_tokens: total.input_tokens.saturating_add(item.input_tokens),
            output_tokens: total.output_tokens.saturating_add(item.output_tokens),
            cache_read_tokens: total
                .cache_read_tokens
                .saturating_add(item.cache_read_tokens),
            cache_write_tokens: total
                .cache_write_tokens
                .saturating_add(item.cache_write_tokens),
        })
}

pub fn tool_edit_break_even(
    tools: &[ToolResultState],
    cache: CacheCostObservation,
) -> (bool, Vec<String>) {
    let protected: Vec<String> = tools
        .iter()
        .filter(|tool| tool.action_critical || tool.evidence_critical || tool.active_blocker)
        .map(|tool| tool.tool_call_id.clone())
        .collect();
    let safe_clearable: u64 = tools
        .iter()
        .filter(|tool| !protected.contains(&tool.tool_call_id))
        .map(|tool| tool.tokens)
        .sum();
    let break_even = safe_clearable.min(cache.clearable_tokens)
        > cache
            .cache_rewrite_tokens
            .saturating_add(cache.edit_overhead_tokens);
    (break_even, protected)
}
