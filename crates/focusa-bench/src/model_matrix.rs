//! Model Matrix — pinned LLM provider/version/class/pricing metadata (Spec 113).
//!
//! Defines the canonical set of models used for benchmark runs.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelClass {
    /// Frontier closed-source model (e.g., Claude Sonnet, GPT-4o).
    Frontier,
    /// Open-weights large model (e.g., Llama 3 70B).
    OpenLarge,
    /// Open-weights small model (e.g., Llama 3 8B, Qwen 1.5B).
    OpenSmall,
    /// Mid-tier closed-source (e.g., Claude Haiku, GPT-4o-mini).
    MidTier,
    /// Local / quantized / fine-tuned (e.g., domain-specific adapters).
    Specialized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub provider: String,
    pub model_id: String,
    pub version: String,
    pub class: ModelClass,
    /// USD per 1M input tokens.
    pub input_cost_per_mtok: f64,
    /// USD per 1M output tokens.
    pub output_cost_per_mtok: f64,
    /// Context window size in tokens.
    pub context_window: u32,
    /// Maximum output tokens.
    pub max_output: u32,
    /// Pinned SHA for the served model artifact, if applicable.
    pub artifact_sha256: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelMatrix {
    pub models: BTreeMap<String, ModelEntry>,
    pub pinned_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ModelMatrix {
    /// Built-in model matrix with the canonical 6-model pin.
    pub fn canonical() -> Self {
        let mut m = BTreeMap::new();
        // Frontier
        m.insert(
            "claude-sonnet-4".to_string(),
            ModelEntry {
                provider: "anthropic".to_string(),
                model_id: "claude-sonnet-4-20250508".to_string(),
                version: "1.0".to_string(),
                class: ModelClass::Frontier,
                input_cost_per_mtok: 3.0,
                output_cost_per_mtok: 15.0,
                context_window: 200_000,
                max_output: 16_000,
                artifact_sha256: None,
            },
        );
        m.insert(
            "gpt-4o".to_string(),
            ModelEntry {
                provider: "openai".to_string(),
                model_id: "gpt-4o-2024-08-06".to_string(),
                version: "1.0".to_string(),
                class: ModelClass::Frontier,
                input_cost_per_mtok: 2.5,
                output_cost_per_mtok: 10.0,
                context_window: 128_000,
                max_output: 16_000,
                artifact_sha256: None,
            },
        );
        // Mid-tier
        m.insert(
            "claude-haiku-3.5".to_string(),
            ModelEntry {
                provider: "anthropic".to_string(),
                model_id: "claude-3-5-haiku-20241022".to_string(),
                version: "1.0".to_string(),
                class: ModelClass::MidTier,
                input_cost_per_mtok: 0.8,
                output_cost_per_mtok: 4.0,
                context_window: 200_000,
                max_output: 8_000,
                artifact_sha256: None,
            },
        );
        m.insert(
            "gpt-4o-mini".to_string(),
            ModelEntry {
                provider: "openai".to_string(),
                model_id: "gpt-4o-mini-2024-07-18".to_string(),
                version: "1.0".to_string(),
                class: ModelClass::MidTier,
                input_cost_per_mtok: 0.15,
                output_cost_per_mtok: 0.6,
                context_window: 128_000,
                max_output: 16_000,
                artifact_sha256: None,
            },
        );
        // Open large
        m.insert(
            "llama-3-70b".to_string(),
            ModelEntry {
                provider: "meta".to_string(),
                model_id: "llama-3-70b-instruct".to_string(),
                version: "1.0".to_string(),
                class: ModelClass::OpenLarge,
                input_cost_per_mtok: 0.0,
                output_cost_per_mtok: 0.0,
                context_window: 8_000,
                max_output: 4_000,
                artifact_sha256: None,
            },
        );
        // Open small
        m.insert(
            "qwen-1.5b".to_string(),
            ModelEntry {
                provider: "alibaba".to_string(),
                model_id: "qwen-1.5b-chat".to_string(),
                version: "1.0".to_string(),
                class: ModelClass::OpenSmall,
                input_cost_per_mtok: 0.0,
                output_cost_per_mtok: 0.0,
                context_window: 32_000,
                max_output: 4_000,
                artifact_sha256: None,
            },
        );
        ModelMatrix {
            models: m,
            pinned_at: Some(chrono::Utc::now()),
        }
    }

    pub fn get(&self, key: &str) -> Option<&ModelEntry> {
        self.models.get(key)
    }

    pub fn list(&self) -> Vec<&ModelEntry> {
        self.models.values().collect()
    }

    pub fn count(&self) -> usize {
        self.models.len()
    }
}
