use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionRuntimeFacts {
    pub provider_raw: Option<String>,
    pub api: Option<String>,
    pub model_id_raw: Option<String>,
    pub response_model: Option<String>,
    pub endpoint_class: Option<String>,
    pub api_version: Option<String>,
    #[serde(default)]
    pub beta_features: Vec<String>,
    pub adapter_revision: String,
    pub capability_evidence_revision: String,
    pub context_window: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub reasoning_enabled: Option<bool>,
    pub transport: Option<String>,
    pub state_mode: Option<String>,
    pub cache_mode: Option<String>,
    pub harness_mode: Option<String>,
    pub objective_profile: Option<String>,
    pub session_id: String,
    pub attachment_id: String,
    pub project_root: Option<String>,
    pub continuity_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionRuntimeFingerprint {
    pub schema: String,
    pub provider_raw: Option<String>,
    pub provider_canonical: Option<String>,
    pub api: Option<String>,
    pub model_id_raw: Option<String>,
    pub model_key: Option<String>,
    pub response_model: Option<String>,
    pub endpoint_fingerprint: Option<String>,
    pub api_version: Option<String>,
    pub beta_features: Vec<String>,
    pub adapter_revision: String,
    pub capability_evidence_revision: String,
    pub context_window: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub reasoning_enabled: Option<bool>,
    pub transport: Option<String>,
    pub state_mode: Option<String>,
    pub cache_mode: Option<String>,
    pub harness_mode: Option<String>,
    pub objective_profile: Option<String>,
    pub session_id: String,
    pub attachment_id: String,
    pub project_root_hash: Option<String>,
    pub continuity_id_hash: Option<String>,
    pub segment_key: String,
}

fn bounded(value: Option<String>, max: usize) -> Option<String> {
    value
        .map(|value| value.trim().chars().take(max).collect::<String>())
        .filter(|value| !value.is_empty())
}

fn hash(value: &str) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(value.as_bytes())))
}

fn canonical_provider(raw: Option<&str>, endpoint_class: Option<&str>) -> Option<String> {
    let raw = raw?.trim().to_ascii_lowercase();
    if raw.is_empty() {
        return None;
    }
    let endpoint = endpoint_class.unwrap_or("unknown").to_ascii_lowercase();
    // Compatible gateways are distinct authority segments. Names never grant
    // first-party capability.
    Some(
        if endpoint.contains("gateway") || endpoint.contains("compatible") {
            format!("gateway:{raw}")
        } else {
            raw
        },
    )
}

pub fn resolve_runtime_fingerprint(facts: CompactionRuntimeFacts) -> CompactionRuntimeFingerprint {
    let provider_raw = bounded(facts.provider_raw, 160);
    let endpoint_class = bounded(facts.endpoint_class, 256);
    let provider_canonical = canonical_provider(provider_raw.as_deref(), endpoint_class.as_deref());
    let model_id_raw = bounded(facts.model_id_raw, 256);
    let response_model = bounded(facts.response_model, 256);
    let model_key = response_model.clone().or_else(|| model_id_raw.clone());
    let endpoint_fingerprint = endpoint_class.as_deref().map(hash);
    let mut beta_features: Vec<String> = facts
        .beta_features
        .into_iter()
        .filter_map(|value| bounded(Some(value), 120))
        .collect();
    beta_features.sort();
    beta_features.dedup();
    beta_features.truncate(32);
    let project_root_hash = facts.project_root.as_deref().map(hash);
    let continuity_id_hash = facts.continuity_id.as_deref().map(hash);
    let segment_material = serde_json::json!({
        "provider": provider_canonical,
        "api": facts.api,
        "model": model_key,
        "response_model": response_model,
        "endpoint": endpoint_fingerprint,
        "api_version": facts.api_version,
        "beta": beta_features,
        "adapter_revision": facts.adapter_revision,
        "capability_revision": facts.capability_evidence_revision,
        "transport": facts.transport,
        "state_mode": facts.state_mode,
        "cache_mode": facts.cache_mode,
        "harness_mode": facts.harness_mode,
        "objective_profile": facts.objective_profile,
        "context_window": facts.context_window,
    });
    let segment_key = hash(&serde_json::to_string(&segment_material).unwrap_or_default());
    CompactionRuntimeFingerprint {
        schema: "focusa.compaction_runtime_fingerprint.v1".into(),
        provider_raw,
        provider_canonical,
        api: bounded(facts.api, 120),
        model_id_raw,
        model_key,
        response_model,
        endpoint_fingerprint,
        api_version: bounded(facts.api_version, 120),
        beta_features,
        adapter_revision: facts.adapter_revision,
        capability_evidence_revision: facts.capability_evidence_revision,
        context_window: facts.context_window,
        max_output_tokens: facts.max_output_tokens,
        reasoning_enabled: facts.reasoning_enabled,
        transport: bounded(facts.transport, 120),
        state_mode: bounded(facts.state_mode, 120),
        cache_mode: bounded(facts.cache_mode, 120),
        harness_mode: bounded(facts.harness_mode, 120),
        objective_profile: bounded(facts.objective_profile, 48),
        session_id: facts.session_id,
        attachment_id: facts.attachment_id,
        project_root_hash,
        continuity_id_hash,
        segment_key,
    }
}
