//! Explicit, provenance-preserving Workstream migration mappings for Spec 158.

use crate::workstream_identity::{ScopeRef, WorkstreamId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const WORKSTREAM_MIGRATION_MAPPING_SCHEMA_V1: &str = "focusa.workstream_migration_mapping.v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MigrationConfidence {
    Proven,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MigrationApprovalSource {
    MigrationRule,
    Operator,
}

/// One evidence-backed candidate read from the bounded legacy inventory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkstreamMigrationCandidate {
    pub source_refs: Vec<String>,
    pub scope_ref: ScopeRef,
    pub workstream_id: WorkstreamId,
    pub evidence_refs: Vec<String>,
    pub rationale: String,
    pub approved_by: MigrationApprovalSource,
    pub approval_ref: String,
}

/// Bounded candidates for one legacy ownership decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MigrationInventory {
    pub candidates: Vec<WorkstreamMigrationCandidate>,
}

/// Durable canonical mapping. It records provenance; it does not mutate legacy data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkstreamMigrationMapping {
    pub schema: String,
    pub source_refs: Vec<String>,
    pub scope_ref: ScopeRef,
    pub workstream_id: WorkstreamId,
    pub confidence: MigrationConfidence,
    pub evidence_refs: Vec<String>,
    pub rationale: String,
    pub approved_by: MigrationApprovalSource,
    pub approval_ref: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WorkstreamMigrationError {
    #[error("migration inventory contains no candidate")]
    MissingCandidate,
    #[error("migration inventory contains multiple candidate Workstreams")]
    AmbiguousCandidates,
    #[error("migration mapping field is required: {0}")]
    MissingField(&'static str),
}

impl MigrationInventory {
    pub fn resolve(
        self,
        created_at: DateTime<Utc>,
    ) -> Result<WorkstreamMigrationMapping, WorkstreamMigrationError> {
        if self.candidates.is_empty() {
            return Err(WorkstreamMigrationError::MissingCandidate);
        }
        if self.candidates.len() != 1 {
            return Err(WorkstreamMigrationError::AmbiguousCandidates);
        }
        WorkstreamMigrationMapping::resolve(
            self.candidates
                .into_iter()
                .next()
                .expect("one candidate checked above"),
            created_at,
        )
    }
}

impl WorkstreamMigrationMapping {
    pub fn resolve(
        candidate: WorkstreamMigrationCandidate,
        created_at: DateTime<Utc>,
    ) -> Result<Self, WorkstreamMigrationError> {
        require_refs(&candidate.source_refs, "source_refs")?;
        require_refs(&candidate.evidence_refs, "evidence_refs")?;
        require_text(&candidate.rationale, "rationale")?;
        require_text(&candidate.approval_ref, "approval_ref")?;

        Ok(Self {
            schema: WORKSTREAM_MIGRATION_MAPPING_SCHEMA_V1.to_string(),
            source_refs: candidate.source_refs,
            scope_ref: candidate.scope_ref,
            workstream_id: candidate.workstream_id,
            confidence: MigrationConfidence::Proven,
            evidence_refs: candidate.evidence_refs,
            rationale: candidate.rationale,
            approved_by: candidate.approved_by,
            approval_ref: candidate.approval_ref,
            created_at,
        })
    }
}

fn require_refs(values: &[String], field: &'static str) -> Result<(), WorkstreamMigrationError> {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        return Err(WorkstreamMigrationError::MissingField(field));
    }
    Ok(())
}

fn require_text(value: &str, field: &'static str) -> Result<(), WorkstreamMigrationError> {
    if value.trim().is_empty() {
        return Err(WorkstreamMigrationError::MissingField(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoped_state::ScopeRef as LegacyScopeRef;

    fn candidate(id: &str) -> WorkstreamMigrationCandidate {
        let scope = LegacyScopeRef::project(
            "project:focusa",
            "/workspace/focusa",
            "Focusa",
            "host-a:worktree-main",
        )
        .unwrap();
        WorkstreamMigrationCandidate {
            source_refs: vec![
                "legacy-thread:thread-a".into(),
                "continuity:continuity-a".into(),
            ],
            scope_ref: ScopeRef::project(scope).unwrap(),
            workstream_id: WorkstreamId::parse(id).unwrap(),
            evidence_refs: vec!["evidence:lineage-a".into()],
            rationale: "One durable workspace and compatible lineage".into(),
            approved_by: MigrationApprovalSource::MigrationRule,
            approval_ref: "migration-rule:unique-durable-workspace:v1".into(),
        }
    }

    #[test]
    fn unique_proven_candidate_retains_full_provenance() {
        let inventory = MigrationInventory {
            candidates: vec![candidate("delivery")],
        };
        let created_at = DateTime::parse_from_rfc3339("2026-08-06T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let mapping = inventory.resolve(created_at).unwrap();
        assert_eq!(mapping.schema, WORKSTREAM_MIGRATION_MAPPING_SCHEMA_V1);
        assert_eq!(mapping.workstream_id.as_str(), "delivery");
        assert_eq!(mapping.confidence, MigrationConfidence::Proven);
        assert_eq!(mapping.source_refs.len(), 2);
        assert_eq!(
            mapping.approval_ref,
            "migration-rule:unique-durable-workspace:v1"
        );
    }

    #[test]
    fn multiple_candidate_workstreams_fail_closed() {
        let inventory = MigrationInventory {
            candidates: vec![candidate("planning"), candidate("delivery")],
        };
        assert_eq!(
            inventory.resolve(Utc::now()),
            Err(WorkstreamMigrationError::AmbiguousCandidates)
        );
    }

    #[test]
    fn continuity_source_without_evidence_or_approval_is_rejected() {
        let mut unresolved = candidate("delivery");
        unresolved.evidence_refs.clear();
        unresolved.approval_ref.clear();
        assert_eq!(
            WorkstreamMigrationMapping::resolve(unresolved, Utc::now()),
            Err(WorkstreamMigrationError::MissingField("evidence_refs"))
        );
    }
}
