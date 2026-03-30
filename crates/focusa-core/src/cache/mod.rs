//! Cache Permission Matrix — docs/18-19
//!
//! 5-tier cache classes: C0 (immutable) → C4 (forbidden).
//! 6 bust categories (A–F): Fresh evidence, Authority change, Compaction,
//! Staleness, Salience collapse, Provider mismatch.

use crate::types::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Cache entry.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub class: CacheClass,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub ttl_secs: Option<u64>,
    pub session_id: Option<SessionId>,
}

/// Cache store.
#[derive(Debug, Default)]
pub struct CacheStore {
    entries: HashMap<String, CacheEntry>,
}

impl CacheStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Put a value. C4 class is rejected.
    pub fn put(
        &mut self,
        key: &str,
        class: CacheClass,
        data: serde_json::Value,
        ttl_secs: Option<u64>,
        session_id: Option<SessionId>,
    ) -> Result<(), String> {
        if class == CacheClass::C4 {
            return Err("C4 class: caching forbidden".into());
        }
        self.entries.insert(
            key.into(),
            CacheEntry {
                key: key.into(),
                class,
                data,
                created_at: Utc::now(),
                ttl_secs,
                session_id,
            },
        );
        Ok(())
    }

    /// Get a cached value. Enforces TTL and session scope.
    pub fn get(&self, key: &str, current_session: Option<SessionId>) -> Option<&CacheEntry> {
        let entry = self.entries.get(key)?;

        // C0 always valid.
        if entry.class == CacheClass::C0 {
            return Some(entry);
        }

        // TTL check for C1.
        if entry.class == CacheClass::C1 {
            if let Some(ttl) = entry.ttl_secs {
                let elapsed = (Utc::now() - entry.created_at).num_seconds().max(0) as u64;
                if elapsed > ttl {
                    return None;
                }
            }
            return Some(entry);
        }

        // C2: session-scoped.
        if entry.class == CacheClass::C2 {
            if entry.session_id == current_session {
                return Some(entry);
            }
            return None;
        }

        // C3: turn-scoped — always stale for reads outside the inserting turn.
        // In practice, C3 is managed by the caller clearing per-turn.
        Some(entry)
    }

    /// Bust cache by category.
    pub fn bust(&mut self, category: CacheBustCategory) {
        let classes_to_bust: Vec<CacheClass> = match category {
            CacheBustCategory::FreshEvidence => {
                vec![CacheClass::C1, CacheClass::C2, CacheClass::C3]
            }
            CacheBustCategory::AuthorityChange => {
                vec![CacheClass::C1, CacheClass::C2, CacheClass::C3]
            }
            CacheBustCategory::Compaction => vec![CacheClass::C2, CacheClass::C3],
            CacheBustCategory::Staleness => vec![CacheClass::C1],
            CacheBustCategory::SalienceCollapse => vec![CacheClass::C2, CacheClass::C3],
            CacheBustCategory::ProviderMismatch => vec![CacheClass::C1, CacheClass::C2],
        };

        self.entries
            .retain(|_, entry| !classes_to_bust.contains(&entry.class));
    }

    /// Clear session-scoped entries.
    pub fn clear_session(&mut self, session_id: SessionId) {
        self.entries.retain(|_, e| e.session_id != Some(session_id));
    }

    /// Clear all turn-scoped entries.
    pub fn clear_turn(&mut self) {
        self.entries.retain(|_, e| e.class != CacheClass::C3);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c4_rejected() {
        let mut store = CacheStore::new();
        let result = store.put("key", CacheClass::C4, serde_json::json!("val"), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_c0_always_valid() {
        let mut store = CacheStore::new();
        store
            .put(
                "immutable",
                CacheClass::C0,
                serde_json::json!("data"),
                None,
                None,
            )
            .unwrap();
        assert!(store.get("immutable", None).is_some());
    }

    #[test]
    fn test_bust_fresh_evidence() {
        let mut store = CacheStore::new();
        store
            .put("c0", CacheClass::C0, serde_json::json!("safe"), None, None)
            .unwrap();
        store
            .put(
                "c1",
                CacheClass::C1,
                serde_json::json!("ttl"),
                Some(3600),
                None,
            )
            .unwrap();
        store.bust(CacheBustCategory::FreshEvidence);
        assert!(store.get("c0", None).is_some()); // C0 survives.
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_session_scope() {
        use uuid::Uuid;
        let mut store = CacheStore::new();
        let sid = Uuid::now_v7();
        let other = Uuid::now_v7();
        store
            .put(
                "scoped",
                CacheClass::C2,
                serde_json::json!("data"),
                None,
                Some(sid),
            )
            .unwrap();
        assert!(store.get("scoped", Some(sid)).is_some());
        assert!(store.get("scoped", Some(other)).is_none());
    }
}
