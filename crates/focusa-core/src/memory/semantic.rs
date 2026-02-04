//! Semantic memory — facts and preferences.
//!
//! MVP keys: user.response_style, project.name, env.preferences
//! Only whitelisted keys injected into prompt.
//! Serialized as: `PREFS: user.response_style=concise_steps`

use crate::types::{ExplicitMemory, MemorySource, SemanticRecord};
use chrono::Utc;

/// Upsert a semantic record.
pub fn upsert(memory: &mut ExplicitMemory, key: String, value: String, source: MemorySource) {
    let now = Utc::now();
    if let Some(existing) = memory.semantic.iter_mut().find(|r| r.key == key) {
        existing.value = value;
        existing.updated_at = now;
        existing.source = source;
    } else {
        memory.semantic.push(SemanticRecord {
            key,
            value,
            created_at: now,
            updated_at: now,
            source,
            confidence: 1.0,
            ttl: None,
            tags: vec![],
            pinned: false,
        });
    }
}

/// Get a semantic record by key.
pub fn get<'a>(memory: &'a ExplicitMemory, key: &str) -> Option<&'a SemanticRecord> {
    memory.semantic.iter().find(|r| r.key == key)
}

/// Serialize whitelisted keys for prompt injection.
pub fn to_prompt_string(memory: &ExplicitMemory) -> String {
    let whitelisted = ["user.response_style", "project.name"];
    memory
        .semantic
        .iter()
        .filter(|r| whitelisted.contains(&r.key.as_str()))
        .map(|r| format!("{}={}", r.key, r.value))
        .collect::<Vec<_>>()
        .join("; ")
}
