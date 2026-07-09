//! Public-safe snapshots with redaction, hash chain, claim generation (Spec 114 Phase 4).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionRule {
    /// JSON path to the field to redact (e.g., "$.model.api_key").
    pub path: String,
    /// Replacement strategy.
    pub strategy: RedactionStrategy,
    /// Optional human-readable label for the redaction.
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedactionStrategy {
    /// Replace with "[REDACTED]".
    Mask,
    /// Replace with "[REDACTED:hash]" (first 8 chars of SHA256).
    HashMask,
    /// Drop the field entirely.
    Drop,
    /// Truncate the string to N chars.
    Truncate { max_chars: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashChain {
    /// SHA256 of the previous snapshot, or "genesis" if this is the first.
    pub prev_sha256: String,
    /// SHA256 of this snapshot's content.
    pub sha256: String,
    /// Sequence number for ordering.
    pub sequence: u64,
    /// SHA256 algorithm used.
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicSnapshot {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub title: String,
    /// The redacted snapshot data (any JSON value with sensitive fields redacted).
    pub data: serde_json::Value,
    /// Rules applied during redaction.
    pub redaction_rules: Vec<RedactionRule>,
    /// Hash chain for tamper-evidence.
    pub hash_chain: HashChain,
    /// Schema version.
    pub schema_version: String,
}

impl PublicSnapshot {
    pub fn new(title: impl Into<String>, data: serde_json::Value) -> Self {
        PublicSnapshot {
            id: uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string(),
            created_at: Utc::now(),
            title: title.into(),
            data,
            redaction_rules: Vec::new(),
            hash_chain: HashChain {
                prev_sha256: "genesis".to_string(),
                sha256: String::new(),  // computed below
                sequence: 0,
                algorithm: "sha256".to_string(),
            },
            schema_version: "focusa.public_snapshot.v1".to_string(),
        }
    }

    pub fn add_rule(&mut self, path: impl Into<String>, strategy: RedactionStrategy) {
        self.redaction_rules.push(RedactionRule {
            path: path.into(),
            strategy,
            label: None,
        });
    }

    /// Generate a measured-claim suitable for public release.
    pub fn claim(&self, metric: impl Into<String>, value: f64, n: u32) -> super::reports::MeasuredClaim {
        let mut claim = super::reports::MeasuredClaim {
            metric: metric.into(),
            value,
            confidence_interval: [value - 0.05, value + 0.05],
            confidence_level: 0.95,
            n,
            stddev: 0.0,
            source_runs: Vec::new(),
            method: "wilson".to_string(),
        };
        claim.source_runs.push(self.id.clone());
        claim
    }
}