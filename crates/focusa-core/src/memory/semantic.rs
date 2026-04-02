//! Semantic memory — facts and preferences.
//!
//! MVP keys: user.response_style, project.name, env.preferences
//! Only whitelisted keys injected into prompt.
//! Serialized as: `PREFS: user.response_style=concise_steps`

use crate::types::{ExplicitMemory, MemorySource, SemanticRecord, FocusaEvent};
use chrono::Utc;

/// Upsert a semantic record.
///
/// Returns an event for the upsert operation.
pub fn upsert(
    memory: &mut ExplicitMemory,
    key: String,
    value: String,
    source: MemorySource,
) -> FocusaEvent {
    let now = Utc::now();
    let source_str = format!("{:?}", source);
    if let Some(existing) = memory.semantic.iter_mut().find(|r| r.key == key) {
        existing.value = value.clone();
        existing.updated_at = now;
        existing.source = source;
    } else {
        memory.semantic.push(SemanticRecord {
            key: key.clone(),
            value: value.clone(),
            created_at: now,
            updated_at: now,
            source,
            confidence: 1.0,
            ttl: None,
            tags: vec![],
            pinned: false,
        });
    }
    FocusaEvent::SemanticMemoryUpserted {
        key,
        value,
        source: source_str,
    }
}

/// Get a semantic record by key.
pub fn get<'a>(memory: &'a ExplicitMemory, key: &str) -> Option<&'a SemanticRecord> {
    memory.semantic.iter().find(|r| r.key == key)
}

/// Enforce TTLs on semantic memories.
///
/// Removes entries whose TTL has expired. Called from decay_tick.
/// Per UNIFIED_ORGANISM_SPEC §10.4.
pub fn enforce_ttls(memory: &mut ExplicitMemory) {
    let now = Utc::now();
    let before = memory.semantic.len();
    memory.semantic.retain(|record| {
        if record.pinned {
            return true;
        }
        if let Some(ttl) = record.ttl {
            if now > record.created_at + ttl {
                tracing::info!(
                    key = %record.key,
                    "Semantic memory expired (TTL)"
                );
                return false;
            }
        }
        true
    });
    let removed = before - memory.semantic.len();
    if removed > 0 {
        tracing::info!(removed, "Semantic memories removed by TTL enforcement");
    }
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
