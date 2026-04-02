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

/// Detect and resolve contradictions in semantic memory.
///
/// Precedence (highest first): operator > constitution > focus_state > context_core > mem0 > worker
/// When a newer entry contradicts an older one at the same or higher precedence,
/// the older entry is marked with a `superseded_by` tag and its confidence is halved.
/// Per UNIFIED_ORGANISM_SPEC §7 (8-level precedence) and §10.7 (contradiction-driven forgetting).
pub fn resolve_contradictions(memory: &mut ExplicitMemory) {
    // Group by key prefix (e.g., all "project.*" entries)
    let len = memory.semantic.len();
    if len < 2 {
        return;
    }

    let mut to_demote: Vec<usize> = Vec::new();

    for i in 0..len {
        for j in (i + 1)..len {
            let a = &memory.semantic[i];
            let b = &memory.semantic[j];

            // Same key = direct conflict (newer wins)
            if a.key == b.key {
                continue; // upsert already handles same-key
            }

            // Same key prefix + different value = potential contradiction
            let a_prefix = a.key.split('.').next().unwrap_or(&a.key);
            let b_prefix = b.key.split('.').next().unwrap_or(&b.key);
            if a_prefix != b_prefix {
                continue;
            }

            // Check for semantic contradiction via negation patterns
            let a_lower = a.value.to_lowercase();
            let b_lower = b.value.to_lowercase();
            let contradicts = (
                a_lower.contains("not ") && !b_lower.contains("not ") ||
                !a_lower.contains("not ") && b_lower.contains("not ")
            ) || (
                a_lower.contains("never") && b_lower.contains("always") ||
                a_lower.contains("always") && b_lower.contains("never")
            ) || (
                a_lower.contains("disable") && b_lower.contains("enable") ||
                a_lower.contains("enable") && b_lower.contains("disable")
            );

            if !contradicts {
                continue;
            }

            // Resolve: newer entry wins (by updated_at)
            let precedence = |s: &SemanticRecord| -> u8 {
                match s.source {
                    MemorySource::Operator => 7,
                    MemorySource::Constitution => 6,
                    MemorySource::FocusState => 5,
                    MemorySource::ContextCore => 4,
                    MemorySource::Mem0 => 3,
                    MemorySource::Worker => 2,
                    _ => 1,
                }
            };

            let a_prec = precedence(a);
            let b_prec = precedence(b);

            if a_prec > b_prec {
                to_demote.push(j);
            } else if b_prec > a_prec {
                to_demote.push(i);
            } else {
                // Same precedence: newer wins
                if a.updated_at > b.updated_at {
                    to_demote.push(j);
                } else {
                    to_demote.push(i);
                }
            }
        }
    }

    to_demote.sort_unstable();
    to_demote.dedup();

    for &idx in &to_demote {
        if idx < memory.semantic.len() {
            let record = &mut memory.semantic[idx];
            record.confidence *= 0.5;
            if !record.tags.contains(&"superseded".to_string()) {
                record.tags.push("superseded".to_string());
            }
            tracing::info!(
                key = %record.key,
                confidence = record.confidence,
                "Semantic memory superseded by contradiction"
            );
        }
    }

    // Remove entries with confidence below threshold (§10.7 forgetting)
    let before = memory.semantic.len();
    memory.semantic.retain(|r| r.confidence > 0.05 || r.pinned);
    let removed = before - memory.semantic.len();
    if removed > 0 {
        tracing::info!(removed, "Semantic memories removed by contradiction-driven forgetting");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Utc;

    fn make_record(key: &str, value: &str, source: MemorySource) -> SemanticRecord {
        SemanticRecord {
            key: key.to_string(),
            value: value.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source,
            confidence: 1.0,
            ttl: None,
            tags: vec![],
            pinned: false,
        }
    }

    #[test]
    fn test_contradiction_precedence_higher_wins() {
        let mut memory = ExplicitMemory::default();
        // Mem0 says "enable caching"
        memory.semantic.push(make_record("config.cache", "enable caching always", MemorySource::Mem0));
        // Operator says "not enable caching" (higher precedence)
        memory.semantic.push(make_record("config.nocache", "not enable caching", MemorySource::Operator));

        resolve_contradictions(&mut memory);

        // Mem0 entry should be demoted (lower precedence)
        let mem0 = memory.semantic.iter().find(|r| r.key == "config.cache").unwrap();
        assert!(mem0.confidence < 1.0, "Mem0 entry should have reduced confidence");
        assert!(mem0.tags.contains(&"superseded".to_string()));

        // Operator entry should be untouched
        let op = memory.semantic.iter().find(|r| r.key == "config.nocache").unwrap();
        assert_eq!(op.confidence, 1.0);
    }

    #[test]
    fn test_contradiction_same_precedence_newer_wins() {
        let mut memory = ExplicitMemory::default();
        let mut old = make_record("pref.style", "always use verbose style", MemorySource::Worker);
        old.updated_at = Utc::now() - chrono::Duration::hours(1);
        memory.semantic.push(old);

        let new = make_record("pref.concise", "never use verbose style", MemorySource::Worker);
        memory.semantic.push(new);

        resolve_contradictions(&mut memory);

        // Older entry should be demoted
        let old_rec = memory.semantic.iter().find(|r| r.key == "pref.style").unwrap();
        assert!(old_rec.confidence < 1.0);
    }

    #[test]
    fn test_no_contradiction_different_prefix() {
        let mut memory = ExplicitMemory::default();
        memory.semantic.push(make_record("project.name", "enable feature X", MemorySource::Worker));
        memory.semantic.push(make_record("user.pref", "not enable feature Y", MemorySource::Worker));

        resolve_contradictions(&mut memory);

        // Different prefixes — no contradiction
        assert!(memory.semantic.iter().all(|r| r.confidence == 1.0));
    }

    #[test]
    fn test_forgetting_removes_low_confidence_after_contradiction() {
        let mut memory = ExplicitMemory::default();
        // Two contradicting entries from same source — newer wins
        let mut old = make_record("test.a", "always enable feature", MemorySource::Worker);
        old.updated_at = Utc::now() - chrono::Duration::hours(2);
        old.confidence = 0.08; // Already low
        memory.semantic.push(old);
        memory.semantic.push(make_record("test.b", "never enable feature", MemorySource::Worker));

        resolve_contradictions(&mut memory);

        // Old entry got halved (0.08 * 0.5 = 0.04 < 0.05) so removed
        assert_eq!(memory.semantic.len(), 1);
        assert_eq!(memory.semantic[0].key, "test.b");
    }

    #[test]
    fn test_pinned_immune_to_forgetting() {
        let mut memory = ExplicitMemory::default();
        let mut rec = make_record("test.pinned", "value", MemorySource::Worker);
        rec.confidence = 0.01;
        rec.pinned = true;
        memory.semantic.push(rec);

        resolve_contradictions(&mut memory);

        // Pinned should survive
        assert_eq!(memory.semantic.len(), 1);
    }
}
