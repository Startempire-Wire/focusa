//! Append-only quarantine for legacy records without proven Workstream ownership.

use crate::workstream_identity::WorkstreamKey;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuarantineReason {
    MissingScope,
    MissingWorkstreamIdentity,
    MultipleCandidateWorkstreams,
    ConflictingProjectRoots,
    ConflictingThreadLineage,
    ContinuityCollision,
    SessionOnlyIdentity,
    ForeignHostOrWorktree,
    InvalidCausalHistory,
    CorruptSnapshot,
    UnsupportedProjectionVersion,
}

/// Immutable evidence for one quarantined legacy record. Candidate Workstreams are
/// advisory only and never grant canonical ownership.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyQuarantineRow {
    pub source_ref: String,
    pub source_hash: String,
    pub payload_ref: String,
    pub reason: QuarantineReason,
    pub candidate_workstreams: Vec<WorkstreamKey>,
    pub evidence_refs: Vec<String>,
    pub quarantined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LegacyQuarantine {
    rows: Vec<LegacyQuarantineRow>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LegacyQuarantineError {
    #[error("quarantine field is required: {0}")]
    MissingField(&'static str),
    #[error("source hash is already quarantined")]
    DuplicateSource,
}

impl LegacyQuarantineRow {
    pub fn classify(
        source_ref: impl Into<String>,
        source_hash: impl Into<String>,
        payload_ref: impl Into<String>,
        reason: QuarantineReason,
        candidate_workstreams: Vec<WorkstreamKey>,
        evidence_refs: Vec<String>,
        quarantined_at: DateTime<Utc>,
    ) -> Result<Self, LegacyQuarantineError> {
        let source_ref = required(source_ref, "source_ref")?;
        let source_hash = required(source_hash, "source_hash")?;
        let payload_ref = required(payload_ref, "payload_ref")?;
        if evidence_refs
            .iter()
            .any(|reference| reference.trim().is_empty())
        {
            return Err(LegacyQuarantineError::MissingField("evidence_refs"));
        }
        Ok(Self {
            source_ref,
            source_hash,
            payload_ref,
            reason,
            candidate_workstreams,
            evidence_refs,
            quarantined_at,
        })
    }
}

impl LegacyQuarantine {
    pub fn append(&mut self, row: LegacyQuarantineRow) -> Result<(), LegacyQuarantineError> {
        if self
            .rows
            .iter()
            .any(|existing| existing.source_hash == row.source_hash)
        {
            return Err(LegacyQuarantineError::DuplicateSource);
        }
        self.rows.push(row);
        Ok(())
    }

    pub fn rows(&self) -> &[LegacyQuarantineRow] {
        &self.rows
    }
}

fn required(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, LegacyQuarantineError> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        Err(LegacyQuarantineError::MissingField(field))
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguous_record_is_retained_without_canonical_assignment() {
        let row = LegacyQuarantineRow::classify(
            "legacy-record:42",
            "sha256:42",
            "artifact:legacy-record-42",
            QuarantineReason::MultipleCandidateWorkstreams,
            Vec::new(),
            vec!["evidence:inventory-42".into()],
            Utc::now(),
        )
        .unwrap();
        let mut quarantine = LegacyQuarantine::default();
        quarantine.append(row).unwrap();
        assert_eq!(quarantine.rows().len(), 1);
        assert!(quarantine.rows()[0].candidate_workstreams.is_empty());
    }

    #[test]
    fn session_only_identity_is_quarantined_not_promoted() {
        let row = LegacyQuarantineRow::classify(
            "legacy-session:session-a",
            "sha256:session-a",
            "artifact:legacy-session-a",
            QuarantineReason::SessionOnlyIdentity,
            Vec::new(),
            vec!["evidence:session-record".into()],
            Utc::now(),
        )
        .unwrap();
        assert_eq!(row.reason, QuarantineReason::SessionOnlyIdentity);
        assert!(row.candidate_workstreams.is_empty());
    }

    #[test]
    fn quarantine_is_append_only_per_source_hash() {
        let row = LegacyQuarantineRow::classify(
            "legacy-record:42",
            "sha256:42",
            "artifact:legacy-record-42",
            QuarantineReason::MissingWorkstreamIdentity,
            Vec::new(),
            Vec::new(),
            Utc::now(),
        )
        .unwrap();
        let mut quarantine = LegacyQuarantine::default();
        quarantine.append(row.clone()).unwrap();
        assert_eq!(
            quarantine.append(row),
            Err(LegacyQuarantineError::DuplicateSource)
        );
    }
}
