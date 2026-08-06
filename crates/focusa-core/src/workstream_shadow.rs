//! Advisory shadow materialization for the Spec 158 Workstream migration.
//!
//! This module is deliberately a read-and-compare seam. [`LegacyState::read`]
//! clones typed legacy state; [`ShadowWorkstreamStore::write`] only writes to a
//! private shadow store and quarantine; [`ParityComparator::compare`] produces
//! a bounded report. None of these APIs mutate the legacy `FocusaState`, call a
//! reducer, or expose a mutable Workstream projection. The shadow store is not
//! a second canonical truth and cannot be used as a reducer input.

use crate::types::FocusaState;
use crate::workstream_identity::{ScopeRef, WorkstreamKey};
use crate::workstream_migration::{
    MigrationConfidence, WORKSTREAM_MIGRATION_MAPPING_SCHEMA_V1, WorkstreamMigrationMapping,
};
use crate::workstream_quarantine::{
    LegacyQuarantine, LegacyQuarantineError, LegacyQuarantineRow, QuarantineReason,
};
use crate::workstream_state::WorkstreamState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use thiserror::Error;

pub const SHADOW_WORKSTREAM_ROW_SCHEMA_V1: &str = "focusa.shadow_workstream_row.v1";
pub const SHADOW_PARITY_REPORT_SCHEMA_V1: &str = "focusa.shadow_workstream_parity_report.v1";
pub const MAX_SHADOW_RECORDS: usize = 4096;
pub const MAX_PARITY_RECORDS: usize = 4096;
pub const MAX_PARITY_DETAILS: usize = 64;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ShadowPersistenceError {
    #[error("shadow migration field is required: {0}")]
    MissingField(&'static str),
    #[error("legacy state serialization failed: {0}")]
    Serialization(String),
    #[error("legacy state payload is invalid: {0}")]
    InvalidPayload(String),
    #[error("legacy state contains duplicate source identity: {0}")]
    DuplicateSource(String),
    #[error("shadow migration input exceeds bounded record limit {limit}")]
    InputLimitExceeded { limit: usize },
    #[error("shadow quarantine failed: {0}")]
    Quarantine(#[from] LegacyQuarantineError),
}

/// Whether a legacy record is eligible for a Workstream row.
///
/// Unsafe fallback records and deprecated records remain immutable legacy
/// evidence, but are intentionally omitted from shadow canonical rows.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LegacyRecordKind {
    Canonical,
    UnsafeFallback,
    Deprecated,
}

/// One typed record in the immutable legacy migration read model.
///
/// A record carries explicit source identity and optional evidence supplied by
/// the inventory. No field is interpreted as an owner when a proven mapping is
/// absent. The typed `FocusaState` is cloned at construction and is never
/// borrowed mutably by the shadow store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyStateRecord {
    source_ref: String,
    source_hash: String,
    payload_ref: String,
    kind: LegacyRecordKind,
    state: FocusaState,
    declared_scope: Option<ScopeRef>,
    candidate_workstreams: Vec<WorkstreamKey>,
    evidence_refs: Vec<String>,
    serialized_payload: Vec<u8>,
}

impl LegacyStateRecord {
    pub fn canonical(
        source_ref: impl Into<String>,
        state: &FocusaState,
    ) -> Result<Self, ShadowPersistenceError> {
        Self::from_state(source_ref, state, LegacyRecordKind::Canonical)
    }

    pub fn unsafe_fallback(
        source_ref: impl Into<String>,
        state: &FocusaState,
    ) -> Result<Self, ShadowPersistenceError> {
        Self::from_state(source_ref, state, LegacyRecordKind::UnsafeFallback)
    }

    pub fn deprecated(
        source_ref: impl Into<String>,
        state: &FocusaState,
    ) -> Result<Self, ShadowPersistenceError> {
        Self::from_state(source_ref, state, LegacyRecordKind::Deprecated)
    }

    /// Read a typed legacy payload while retaining its exact bytes for the
    /// serialization-only parity distinction.
    pub fn from_payload(
        source_ref: impl Into<String>,
        payload: &[u8],
        kind: LegacyRecordKind,
    ) -> Result<Self, ShadowPersistenceError> {
        if payload.is_empty() {
            return Err(ShadowPersistenceError::MissingField("payload"));
        }
        let state = serde_json::from_slice::<FocusaState>(payload)
            .map_err(|error| ShadowPersistenceError::InvalidPayload(error.to_string()))?;
        Self::from_serialized_state(source_ref, state, kind, payload.to_vec())
    }

    fn from_state(
        source_ref: impl Into<String>,
        state: &FocusaState,
        kind: LegacyRecordKind,
    ) -> Result<Self, ShadowPersistenceError> {
        let serialized_payload = serde_json::to_vec(state)
            .map_err(|error| ShadowPersistenceError::Serialization(error.to_string()))?;
        Self::from_serialized_state(source_ref, state.clone(), kind, serialized_payload)
    }

    fn from_serialized_state(
        source_ref: impl Into<String>,
        state: FocusaState,
        kind: LegacyRecordKind,
        serialized_payload: Vec<u8>,
    ) -> Result<Self, ShadowPersistenceError> {
        let source_ref = required(source_ref, "source_ref")?;
        if serialized_payload.is_empty() {
            return Err(ShadowPersistenceError::MissingField("payload"));
        }
        let payload_digest = payload_hash(&serialized_payload);
        let source_hash = source_identity_hash(&source_ref, &serialized_payload);
        Ok(Self {
            source_ref,
            source_hash,
            payload_ref: format!("legacy-payload:{payload_digest}"),
            kind,
            state,
            declared_scope: None,
            candidate_workstreams: Vec::new(),
            evidence_refs: Vec::new(),
            serialized_payload,
        })
    }

    /// Add a declared legacy scope. It is checked against the proven mapping;
    /// it is never used to manufacture a Workstream mapping.
    pub fn with_declared_scope(mut self, scope: ScopeRef) -> Self {
        self.declared_scope = Some(scope);
        self
    }

    /// Retain candidate ownership evidence for quarantine when no unique
    /// approved mapping exists.
    pub fn with_candidate_workstreams(mut self, candidates: Vec<WorkstreamKey>) -> Self {
        self.candidate_workstreams = candidates;
        self
    }

    pub fn with_evidence_refs(mut self, evidence_refs: Vec<String>) -> Self {
        self.evidence_refs = evidence_refs;
        self
    }

    pub fn with_payload_ref(mut self, payload_ref: impl Into<String>) -> Self {
        self.payload_ref = payload_ref.into();
        self
    }

    pub fn source_ref(&self) -> &str {
        &self.source_ref
    }

    pub fn source_hash(&self) -> &str {
        &self.source_hash
    }

    pub fn payload_ref(&self) -> &str {
        &self.payload_ref
    }

    pub fn kind(&self) -> LegacyRecordKind {
        self.kind
    }

    pub fn state(&self) -> &FocusaState {
        &self.state
    }

    pub fn declared_scope(&self) -> Option<&ScopeRef> {
        self.declared_scope.as_ref()
    }

    pub fn candidate_workstreams(&self) -> &[WorkstreamKey] {
        &self.candidate_workstreams
    }

    pub fn evidence_refs(&self) -> &[String] {
        &self.evidence_refs
    }

    pub fn serialized_payload(&self) -> &[u8] {
        &self.serialized_payload
    }

    fn integrity_reason(&self) -> Option<QuarantineReason> {
        if self.source_ref.trim().is_empty()
            || self.source_hash.trim().is_empty()
            || self.payload_ref.trim().is_empty()
            || self.serialized_payload.is_empty()
        {
            return Some(QuarantineReason::CorruptSnapshot);
        }
        if self.source_hash != source_identity_hash(&self.source_ref, &self.serialized_payload) {
            return Some(QuarantineReason::CorruptSnapshot);
        }
        let payload_state = match serde_json::from_slice::<FocusaState>(&self.serialized_payload) {
            Ok(state) => state,
            Err(_) => return Some(QuarantineReason::CorruptSnapshot),
        };
        let payload_value = match serde_json::to_value(&payload_state) {
            Ok(value) => value,
            Err(_) => return Some(QuarantineReason::CorruptSnapshot),
        };
        let state_value = match serde_json::to_value(&self.state) {
            Ok(value) => value,
            Err(_) => return Some(QuarantineReason::CorruptSnapshot),
        };
        if canonical_json(&payload_value) != canonical_json(&state_value) {
            return Some(QuarantineReason::CorruptSnapshot);
        }
        if let Some(scope) = self.declared_scope.as_ref() {
            if scope.legacy_scope().validate().is_err() {
                return Some(QuarantineReason::MissingScope);
            }
        }
        if self.candidate_workstreams.iter().any(|candidate| {
            candidate.workstream_id.as_str().trim().is_empty()
                || candidate.legacy_scope().validate().is_err()
        }) {
            return Some(QuarantineReason::MissingWorkstreamIdentity);
        }
        None
    }
}

/// Immutable, typed read of the legacy canonical snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyState {
    records: Vec<LegacyStateRecord>,
}

impl LegacyState {
    /// Clone legacy canonical state into a migration read model. The input is a
    /// shared reference by design, so this operation cannot mutate the legacy
    /// canonical owner.
    pub fn read(state: &FocusaState) -> Result<Self, ShadowPersistenceError> {
        Self::read_with_source_ref("legacy-state", state)
    }

    pub fn read_with_source_ref(
        source_ref: impl Into<String>,
        state: &FocusaState,
    ) -> Result<Self, ShadowPersistenceError> {
        let record = LegacyStateRecord::canonical(source_ref, state)?;
        Self::from_records(vec![record])
    }

    pub fn from_records(records: Vec<LegacyStateRecord>) -> Result<Self, ShadowPersistenceError> {
        if records.len() > MAX_SHADOW_RECORDS {
            return Err(ShadowPersistenceError::InputLimitExceeded {
                limit: MAX_SHADOW_RECORDS,
            });
        }
        let mut source_refs = BTreeSet::new();
        for record in &records {
            if !source_refs.insert(record.source_ref.clone()) {
                return Err(ShadowPersistenceError::DuplicateSource(
                    record.source_ref.clone(),
                ));
            }
        }
        Ok(Self { records })
    }

    pub fn records(&self) -> &[LegacyStateRecord] {
        &self.records
    }
}

/// The explicit reason a legacy record is omitted from the shadow rows.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShadowOmissionReason {
    UnsafeFallback,
    Deprecated,
}

/// Advisory record of data intentionally not materialized as Workstream state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShadowOmission {
    pub source_ref: String,
    pub source_hash: String,
    pub reason: ShadowOmissionReason,
}

/// Bounded parity classification mandated by Spec 158 §6.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParityDisposition {
    EqualMappedState,
    ExpectedRemovalOfUnsafeFallback,
    MigrationMismatch,
    QuarantinedAmbiguity,
    DeprecatedData,
    SerializationOnlyDifference,
}

impl ParityDisposition {
    pub fn is_acceptable(self) -> bool {
        matches!(
            self,
            Self::EqualMappedState
                | Self::ExpectedRemovalOfUnsafeFallback
                | Self::DeprecatedData
                | Self::SerializationOnlyDifference
        )
    }
}

/// One bounded source-level parity observation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParityRecord {
    pub source_ref: String,
    pub workstream: Option<WorkstreamKey>,
    pub disposition: ParityDisposition,
    pub detail: String,
}

/// Bounded parity output. It is evidence for migration review, not a cutover
/// command and not a canonical state write.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParityReport {
    pub schema: String,
    pub bounded: bool,
    pub truncated: bool,
    pub records: Vec<ParityRecord>,
}

impl ParityReport {
    pub fn passes(&self) -> bool {
        self.records
            .iter()
            .all(|record| record.disposition.is_acceptable())
            && !self.truncated
    }

    pub fn has_blocking_difference(&self) -> bool {
        self.records
            .iter()
            .any(|record| !record.disposition.is_acceptable())
            || self.truncated
    }
}

/// One typed shadow row. Its fields are private and only shared accessors are
/// provided, preventing a caller from turning the advisory store into a
/// mutable reducer source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowWorkstreamRow {
    schema: String,
    key: WorkstreamKey,
    source_ref: String,
    source_hash: String,
    mapping: WorkstreamMigrationMapping,
    state: WorkstreamState,
    serialized_payload: Vec<u8>,
    materialized_at: DateTime<Utc>,
}

impl ShadowWorkstreamRow {
    fn new(
        key: WorkstreamKey,
        record: &LegacyStateRecord,
        mapping: &WorkstreamMigrationMapping,
        materialized_at: DateTime<Utc>,
    ) -> Result<Self, ShadowPersistenceError> {
        let state = WorkstreamState::from_focusa_state(key.clone(), record.state.clone());
        let serialized_payload = serde_json::to_vec(state.cognitive_state())
            .map_err(|error| ShadowPersistenceError::Serialization(error.to_string()))?;
        Ok(Self {
            schema: SHADOW_WORKSTREAM_ROW_SCHEMA_V1.to_string(),
            key,
            source_ref: record.source_ref.clone(),
            source_hash: record.source_hash.clone(),
            mapping: mapping.clone(),
            state,
            serialized_payload,
            materialized_at,
        })
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn key(&self) -> &WorkstreamKey {
        &self.key
    }

    pub fn source_ref(&self) -> &str {
        &self.source_ref
    }

    pub fn source_hash(&self) -> &str {
        &self.source_hash
    }

    pub fn mapping(&self) -> &WorkstreamMigrationMapping {
        &self.mapping
    }

    pub fn state(&self) -> &WorkstreamState {
        &self.state
    }

    pub fn serialized_payload(&self) -> &[u8] {
        &self.serialized_payload
    }

    pub fn materialized_at(&self) -> DateTime<Utc> {
        self.materialized_at
    }
}

/// Result of one advisory shadow write.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ShadowWriteReport {
    pub materialized_rows: usize,
    pub quarantined_rows: usize,
    pub omitted_rows: usize,
}

/// Advisory-only Workstream store. It has no reducer, no canonical write path,
/// and no mutable state accessor. A later cutover must explicitly promote data
/// through the canonical reducer/persistence owner.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShadowWorkstreamStore {
    rows: Vec<ShadowWorkstreamRow>,
    quarantine: LegacyQuarantine,
    omissions: Vec<ShadowOmission>,
}

impl ShadowWorkstreamStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Materialize proven mappings into the advisory shadow store. Legacy state
    /// and mappings are shared inputs; ambiguous, unmapped, foreign, and
    /// conflicting inputs become quarantine rows and never receive an owner.
    pub fn write(
        &mut self,
        legacy: &LegacyState,
        mappings: &[WorkstreamMigrationMapping],
    ) -> Result<ShadowWriteReport, ShadowPersistenceError> {
        if legacy.records.len() > MAX_SHADOW_RECORDS || mappings.len() > MAX_SHADOW_RECORDS {
            return Err(ShadowPersistenceError::InputLimitExceeded {
                limit: MAX_SHADOW_RECORDS,
            });
        }

        let mut matched_mappings = vec![false; mappings.len()];
        let mut result = ShadowWriteReport::default();
        let materialized_at = Utc::now();

        for record in &legacy.records {
            if let Some(reason) = record.integrity_reason() {
                self.quarantine_record(record, reason, record.candidate_workstreams().to_vec())?;
                result.quarantined_rows += 1;
                continue;
            }

            let matching_indices: Vec<usize> = mappings
                .iter()
                .enumerate()
                .filter(|(_, mapping)| {
                    mapping
                        .source_refs
                        .iter()
                        .any(|source_ref| source_ref == record.source_ref())
                })
                .map(|(index, _)| index)
                .collect();

            if matching_indices.is_empty() {
                if matches!(record.kind(), LegacyRecordKind::UnsafeFallback) {
                    self.record_omission(record, ShadowOmissionReason::UnsafeFallback);
                    result.omitted_rows += 1;
                    continue;
                }
                if matches!(record.kind(), LegacyRecordKind::Deprecated) {
                    self.record_omission(record, ShadowOmissionReason::Deprecated);
                    result.omitted_rows += 1;
                    continue;
                }
                let reason = if record.candidate_workstreams().len() > 1 {
                    QuarantineReason::MultipleCandidateWorkstreams
                } else {
                    QuarantineReason::UnmappedLegacyRecord
                };
                self.quarantine_record(record, reason, record.candidate_workstreams().to_vec())?;
                result.quarantined_rows += 1;
                continue;
            }

            let candidate_keys: Vec<WorkstreamKey> = matching_indices
                .iter()
                .map(|index| mapping_key(&mappings[*index]))
                .collect();
            let distinct_candidate_keys = distinct_keys(&candidate_keys);
            if matching_indices.len() != 1 {
                let reason = if distinct_candidate_keys.len() > 1 {
                    QuarantineReason::MultipleCandidateWorkstreams
                } else {
                    QuarantineReason::ConflictingWorkstreamMappings
                };
                self.quarantine_record(record, reason, distinct_candidate_keys)?;
                result.quarantined_rows += 1;
                for index in matching_indices {
                    matched_mappings[index] = true;
                }
                continue;
            }

            let mapping_index = matching_indices[0];
            matched_mappings[mapping_index] = true;
            let mapping = &mappings[mapping_index];
            let key = mapping_key(mapping);

            if let Err(reason) = validate_mapping(mapping) {
                self.quarantine_record(record, reason, vec![key])?;
                result.quarantined_rows += 1;
                continue;
            }

            if let Some(declared_scope) = record.declared_scope() {
                if declared_scope != &mapping.scope_ref {
                    self.quarantine_record(
                        record,
                        scope_conflict_reason(declared_scope, &mapping.scope_ref),
                        vec![key],
                    )?;
                    result.quarantined_rows += 1;
                    continue;
                }
            }

            if !record.candidate_workstreams().is_empty()
                && !record
                    .candidate_workstreams()
                    .iter()
                    .any(|candidate| candidate == &key)
            {
                self.quarantine_record(
                    record,
                    QuarantineReason::ConflictingThreadLineage,
                    vec![key],
                )?;
                result.quarantined_rows += 1;
                continue;
            }

            if matches!(record.kind(), LegacyRecordKind::UnsafeFallback) {
                self.record_omission(record, ShadowOmissionReason::UnsafeFallback);
                result.omitted_rows += 1;
                continue;
            }
            if matches!(record.kind(), LegacyRecordKind::Deprecated) {
                self.record_omission(record, ShadowOmissionReason::Deprecated);
                result.omitted_rows += 1;
                continue;
            }

            if let Some(existing) = self.rows.iter().find(|row| row.key() == &key) {
                if existing.source_hash() == record.source_hash() {
                    continue;
                }
                self.quarantine_record(
                    record,
                    QuarantineReason::ConflictingWorkstreamMappings,
                    vec![key],
                )?;
                result.quarantined_rows += 1;
                continue;
            }

            self.rows.push(ShadowWorkstreamRow::new(
                key,
                record,
                mapping,
                materialized_at,
            )?);
            result.materialized_rows += 1;
        }

        for (index, mapping) in mappings.iter().enumerate() {
            if matched_mappings[index] {
                continue;
            }
            let reason = validate_mapping(mapping)
                .err()
                .unwrap_or(QuarantineReason::UnmappedLegacyRecord);
            self.quarantine_mapping(mapping, reason)?;
            result.quarantined_rows += 1;
        }

        Ok(result)
    }

    pub fn rows(&self) -> &[ShadowWorkstreamRow] {
        &self.rows
    }

    pub fn quarantine(&self) -> &LegacyQuarantine {
        &self.quarantine
    }

    pub fn omissions(&self) -> &[ShadowOmission] {
        &self.omissions
    }

    pub fn is_advisory(&self) -> bool {
        true
    }

    fn record_omission(&mut self, record: &LegacyStateRecord, reason: ShadowOmissionReason) {
        if self
            .omissions
            .iter()
            .any(|existing| existing.source_hash == record.source_hash())
        {
            return;
        }
        self.omissions.push(ShadowOmission {
            source_ref: record.source_ref.clone(),
            source_hash: record.source_hash.clone(),
            reason,
        });
    }

    fn quarantine_record(
        &mut self,
        record: &LegacyStateRecord,
        reason: QuarantineReason,
        candidates: Vec<WorkstreamKey>,
    ) -> Result<(), ShadowPersistenceError> {
        if !record.source_hash.trim().is_empty()
            && self
                .quarantine
                .rows()
                .iter()
                .any(|row| row.source_hash == record.source_hash)
        {
            return Ok(());
        }
        let payload_digest = payload_hash(record.serialized_payload());
        let source_ref = if record.source_ref.trim().is_empty() {
            format!("legacy-corrupt:{payload_digest}")
        } else {
            record.source_ref.clone()
        };
        let source_hash = if record.source_hash.trim().is_empty() {
            payload_digest.clone()
        } else {
            record.source_hash.clone()
        };
        let payload_ref = if record.payload_ref.trim().is_empty() {
            format!("legacy-payload:{payload_digest}")
        } else {
            record.payload_ref.clone()
        };
        let evidence_refs = evidence_or_source(record.evidence_refs(), &source_ref);
        let row = LegacyQuarantineRow::classify(
            source_ref,
            source_hash,
            payload_ref,
            reason,
            candidates,
            evidence_refs,
            Utc::now(),
        )?;
        self.quarantine.append(row)?;
        Ok(())
    }

    fn quarantine_mapping(
        &mut self,
        mapping: &WorkstreamMigrationMapping,
        reason: QuarantineReason,
    ) -> Result<(), ShadowPersistenceError> {
        let serialized = serde_json::to_vec(mapping)
            .map_err(|error| ShadowPersistenceError::Serialization(error.to_string()))?;
        let source_hash = payload_hash(&serialized);
        if self
            .quarantine
            .rows()
            .iter()
            .any(|row| row.source_hash == source_hash)
        {
            return Ok(());
        }
        let source_ref = if mapping.source_refs.is_empty() {
            format!("migration-mapping:{source_hash}")
        } else {
            mapping.source_refs.join("|")
        };
        let key = mapping_key(mapping);
        let evidence_refs = evidence_or_source(&mapping.evidence_refs, &source_ref);
        let row = LegacyQuarantineRow::classify(
            source_ref,
            source_hash,
            format!("migration-mapping:{}", key.storage_key()),
            reason,
            vec![key],
            evidence_refs,
            Utc::now(),
        )?;
        self.quarantine.append(row)?;
        Ok(())
    }
}

/// Bounded comparator for the immutable legacy read model and advisory rows.
#[derive(Debug, Clone, Copy)]
pub struct ParityComparator {
    max_records: usize,
    max_details: usize,
}

impl Default for ParityComparator {
    fn default() -> Self {
        Self {
            max_records: MAX_PARITY_RECORDS,
            max_details: MAX_PARITY_DETAILS,
        }
    }
}

impl ParityComparator {
    pub fn new(max_records: usize, max_details: usize) -> Self {
        Self {
            max_records: max_records.max(1),
            max_details: max_details.max(1),
        }
    }

    /// Compare only bounded typed projections. Unsafe fallback removal and
    /// deprecated data are expected outcomes; ambiguity and migration mismatch
    /// remain blocking observations. Serialization formatting is reported
    /// separately from semantic state differences.
    pub fn compare(&self, legacy: &LegacyState, shadow: &ShadowWorkstreamStore) -> ParityReport {
        let mut records = Vec::new();
        let mut truncated = false;
        let mut seen_sources = BTreeSet::new();

        for legacy_record in legacy.records.iter() {
            if records.len() >= self.max_records {
                truncated = true;
                break;
            }
            seen_sources.insert(legacy_record.source_ref.clone());
            let row = shadow
                .rows
                .iter()
                .find(|candidate| candidate.source_ref() == legacy_record.source_ref());
            let quarantine = shadow
                .quarantine
                .rows()
                .iter()
                .find(|candidate| candidate.source_ref == legacy_record.source_ref());
            let detail = |text: &str| bounded_detail(text, self.max_details);

            let (disposition, workstream, text) = if quarantine.is_some() {
                (
                    ParityDisposition::QuarantinedAmbiguity,
                    quarantine.and_then(|entry| entry.candidate_workstreams.first().cloned()),
                    "legacy record remains quarantined; no owner was assigned".to_string(),
                )
            } else if matches!(legacy_record.kind(), LegacyRecordKind::UnsafeFallback)
                && row.is_none()
            {
                (
                    ParityDisposition::ExpectedRemovalOfUnsafeFallback,
                    None,
                    "unsafe global fallback was intentionally omitted from shadow rows".to_string(),
                )
            } else if matches!(legacy_record.kind(), LegacyRecordKind::Deprecated) && row.is_none()
            {
                (
                    ParityDisposition::DeprecatedData,
                    None,
                    "deprecated legacy data was retained as evidence and omitted".to_string(),
                )
            } else if let Some(row) = row {
                if !shadow_row_matches_record(row, legacy_record) {
                    (
                        ParityDisposition::MigrationMismatch,
                        Some(row.key().clone()),
                        "shadow row identity, mapping, or payload is inconsistent".to_string(),
                    )
                } else {
                    let legacy_value = serde_json::to_value(legacy_record.state()).ok();
                    let shadow_value = serde_json::to_value(row.state().cognitive_state()).ok();
                    match (legacy_value, shadow_value) {
                        (Some(legacy_value), Some(shadow_value))
                            if canonical_json(&legacy_value) == canonical_json(&shadow_value) =>
                        {
                            if legacy_record.serialized_payload() != row.serialized_payload() {
                                (
                                    ParityDisposition::SerializationOnlyDifference,
                                    Some(row.key().clone()),
                                    "typed state is equal; only serialized bytes differ"
                                        .to_string(),
                                )
                            } else {
                                (
                                    ParityDisposition::EqualMappedState,
                                    Some(row.key().clone()),
                                    "mapped typed state is equal".to_string(),
                                )
                            }
                        }
                        _ => (
                            ParityDisposition::MigrationMismatch,
                            Some(row.key().clone()),
                            "mapped Workstream projection differs from legacy typed state"
                                .to_string(),
                        ),
                    }
                }
            } else {
                (
                    ParityDisposition::MigrationMismatch,
                    legacy_record.candidate_workstreams.first().cloned(),
                    "canonical legacy record has no materialized row or quarantine evidence"
                        .to_string(),
                )
            };

            records.push(ParityRecord {
                source_ref: bounded_detail(legacy_record.source_ref(), self.max_details),
                workstream,
                disposition,
                detail: detail(&text),
            });
        }

        if !truncated {
            for quarantine in shadow.quarantine.rows() {
                if seen_sources.contains(&quarantine.source_ref) {
                    continue;
                }
                if records.len() >= self.max_records {
                    truncated = true;
                    break;
                }
                records.push(ParityRecord {
                    source_ref: bounded_detail(&quarantine.source_ref, self.max_details),
                    workstream: quarantine.candidate_workstreams.first().cloned(),
                    disposition: ParityDisposition::QuarantinedAmbiguity,
                    detail: bounded_detail(
                        "quarantine contains a source absent from the bounded legacy read",
                        self.max_details,
                    ),
                });
            }
        }

        if !truncated {
            for row in &shadow.rows {
                if seen_sources.contains(row.source_ref()) {
                    continue;
                }
                if records.len() >= self.max_records {
                    truncated = true;
                    break;
                }
                records.push(ParityRecord {
                    source_ref: bounded_detail(row.source_ref(), self.max_details),
                    workstream: Some(row.key().clone()),
                    disposition: ParityDisposition::MigrationMismatch,
                    detail: bounded_detail(
                        "shadow row has no corresponding legacy source",
                        self.max_details,
                    ),
                });
            }
        }

        ParityReport {
            schema: SHADOW_PARITY_REPORT_SCHEMA_V1.to_string(),
            bounded: true,
            truncated,
            records,
        }
    }
}

fn mapping_key(mapping: &WorkstreamMigrationMapping) -> WorkstreamKey {
    WorkstreamKey::new(mapping.scope_ref.clone(), mapping.workstream_id.clone())
}

fn validate_mapping(mapping: &WorkstreamMigrationMapping) -> Result<(), QuarantineReason> {
    if mapping.schema != WORKSTREAM_MIGRATION_MAPPING_SCHEMA_V1
        || mapping.confidence != MigrationConfidence::Proven
    {
        return Err(QuarantineReason::InvalidMigrationMapping);
    }
    if mapping.source_refs.is_empty()
        || mapping
            .source_refs
            .iter()
            .any(|value| value.trim().is_empty())
    {
        return Err(QuarantineReason::InvalidMigrationMapping);
    }
    if mapping.evidence_refs.is_empty()
        || mapping
            .evidence_refs
            .iter()
            .any(|value| value.trim().is_empty())
    {
        return Err(QuarantineReason::InvalidMigrationMapping);
    }
    if mapping.rationale.trim().is_empty() || mapping.approval_ref.trim().is_empty() {
        return Err(QuarantineReason::InvalidMigrationMapping);
    }
    if mapping.workstream_id.as_str().trim().is_empty()
        || mapping.scope_ref.legacy_scope().validate().is_err()
    {
        return Err(QuarantineReason::InvalidMigrationMapping);
    }
    Ok(())
}

fn shadow_row_matches_record(row: &ShadowWorkstreamRow, record: &LegacyStateRecord) -> bool {
    let mapped_key = mapping_key(row.mapping());
    if row.schema() != SHADOW_WORKSTREAM_ROW_SCHEMA_V1
        || row.source_hash() != record.source_hash()
        || !row
            .mapping()
            .source_refs
            .iter()
            .any(|source_ref| source_ref == record.source_ref())
        || &mapped_key != row.key()
        || &row.state().key != row.key()
        || validate_mapping(row.mapping()).is_err()
    {
        return false;
    }
    let serialized_state = match serde_json::from_slice::<FocusaState>(row.serialized_payload()) {
        Ok(state) => state,
        Err(_) => return false,
    };
    let serialized_value = match serde_json::to_value(serialized_state) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let row_value = match serde_json::to_value(row.state().cognitive_state()) {
        Ok(value) => value,
        Err(_) => return false,
    };
    canonical_json(&serialized_value) == canonical_json(&row_value)
}

fn distinct_keys(values: &[WorkstreamKey]) -> Vec<WorkstreamKey> {
    let mut result = Vec::new();
    for value in values {
        if !result.iter().any(|existing| existing == value) {
            result.push(value.clone());
        }
    }
    result
}

fn scope_conflict_reason(expected: &ScopeRef, actual: &ScopeRef) -> QuarantineReason {
    match (expected, actual) {
        (ScopeRef::Project(_), ScopeRef::Project(_)) => QuarantineReason::ConflictingProjectRoots,
        (ScopeRef::Host(_), ScopeRef::Host(_)) => QuarantineReason::ConflictingProjectRoots,
        _ => QuarantineReason::ForeignHostOrWorktree,
    }
}

fn evidence_or_source(evidence_refs: &[String], source_ref: &str) -> Vec<String> {
    let evidence_refs: Vec<String> = evidence_refs
        .iter()
        .filter(|reference| !reference.trim().is_empty())
        .cloned()
        .collect();
    if evidence_refs.is_empty() {
        vec![format!("legacy-source:{source_ref}")]
    } else {
        evidence_refs
    }
}

fn required(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, ShadowPersistenceError> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        Err(ShadowPersistenceError::MissingField(field))
    } else {
        Ok(value)
    }
}

fn payload_hash(payload: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(payload)))
}

fn source_identity_hash(source_ref: &str, payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_ref.as_bytes());
    hasher.update([0]);
    hasher.update(payload);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

fn bounded_detail(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

/// Recursively sort object keys before semantic comparison. Array order remains
/// meaningful because reducer state lists are ordered projections.
fn canonical_json(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries: Vec<(&String, &Value)> = object.iter().collect();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            let mut normalized = Map::new();
            for (key, child) in entries {
                normalized.insert(key.clone(), canonical_json(child));
            }
            Value::Object(normalized)
        }
        Value::Array(values) => Value::Array(values.iter().map(canonical_json).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod workstream_shadow_materialization {
    use super::*;
    use crate::scoped_state::ScopeRef as LegacyScopeRef;
    use crate::workstream_identity::{ScopeRef, WorkstreamId};

    fn scope(fingerprint: &str) -> ScopeRef {
        let legacy =
            LegacyScopeRef::project("project:focusa", "/workspace/focusa", "Focusa", fingerprint)
                .expect("valid project scope");
        ScopeRef::project(legacy).expect("canonical project scope")
    }

    fn mapping(source_ref: &str, id: &str) -> WorkstreamMigrationMapping {
        WorkstreamMigrationMapping {
            schema: WORKSTREAM_MIGRATION_MAPPING_SCHEMA_V1.to_string(),
            source_refs: vec![source_ref.to_string()],
            scope_ref: scope("host-a:worktree-main"),
            workstream_id: WorkstreamId::parse(id).expect("workstream id"),
            confidence: MigrationConfidence::Proven,
            evidence_refs: vec!["evidence:unique-lineage".to_string()],
            rationale: "one unique durable workspace with compatible lineage".to_string(),
            approved_by: crate::workstream_migration::MigrationApprovalSource::MigrationRule,
            approval_ref: "migration-rule:unique-durable-workspace:v1".to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn legacy_read_is_shared_and_does_not_mutate_canonical_state() {
        let state = FocusaState::default();
        let before = serde_json::to_vec(&state).expect("state serializes");
        let legacy = LegacyState::read(&state).expect("legacy read");
        let after = serde_json::to_vec(&state).expect("state serializes");
        assert_eq!(before, after);
        assert_eq!(legacy.records().len(), 1);
    }

    #[test]
    fn equal_mapped_state_passes_bounded_parity() {
        let state = FocusaState::default();
        let legacy = LegacyState::read(&state).expect("legacy read");
        let source_ref = legacy.records()[0].source_ref().to_string();
        let mut shadow = ShadowWorkstreamStore::new();
        let write = shadow
            .write(&legacy, &[mapping(&source_ref, "delivery")])
            .expect("shadow write");
        assert_eq!(write.materialized_rows, 1);
        let report = ParityComparator::default().compare(&legacy, &shadow);
        assert!(report.passes());
        assert_eq!(
            report.records[0].disposition,
            ParityDisposition::EqualMappedState
        );
    }

    #[test]
    fn ambiguous_mapping_is_quarantined_without_default_assignment() {
        let state = FocusaState::default();
        let legacy = LegacyState::read(&state).expect("legacy read");
        let source_ref = legacy.records()[0].source_ref().to_string();
        let mut shadow = ShadowWorkstreamStore::new();
        let write = shadow
            .write(
                &legacy,
                &[
                    mapping(&source_ref, "planning"),
                    mapping(&source_ref, "delivery"),
                ],
            )
            .expect("ambiguous input is quarantined, not rejected");
        assert_eq!(write.materialized_rows, 0);
        assert_eq!(shadow.rows().len(), 0);
        assert_eq!(shadow.quarantine().rows().len(), 1);
        assert_eq!(
            shadow.quarantine().rows()[0].reason,
            QuarantineReason::MultipleCandidateWorkstreams
        );
        assert!(
            shadow.quarantine().rows()[0]
                .candidate_workstreams
                .iter()
                .any(|key| key.workstream_id.as_str() == "planning")
        );
        assert!(
            shadow.quarantine().rows()[0]
                .candidate_workstreams
                .iter()
                .any(|key| key.workstream_id.as_str() == "delivery")
        );
        let report = ParityComparator::default().compare(&legacy, &shadow);
        assert!(!report.passes());
        assert_eq!(
            report.records[0].disposition,
            ParityDisposition::QuarantinedAmbiguity
        );
    }

    #[test]
    fn two_unique_mappings_materialize_distinct_exact_workstream_keys() {
        let state = FocusaState::default();
        let legacy = LegacyState::from_records(vec![
            LegacyStateRecord::canonical("legacy-planning", &state).expect("planning record"),
            LegacyStateRecord::canonical("legacy-delivery", &state).expect("delivery record"),
        ])
        .expect("legacy records");
        let mut shadow = ShadowWorkstreamStore::new();
        shadow
            .write(
                &legacy,
                &[
                    mapping("legacy-planning", "planning"),
                    mapping("legacy-delivery", "delivery"),
                ],
            )
            .expect("distinct mappings");
        assert_eq!(shadow.rows().len(), 2);
        assert!(
            shadow
                .rows()
                .iter()
                .any(|row| row.key().workstream_id.as_str() == "planning")
        );
        assert!(
            shadow
                .rows()
                .iter()
                .any(|row| row.key().workstream_id.as_str() == "delivery")
        );
        assert!(
            shadow
                .rows()
                .iter()
                .all(|row| { row.key().scope == scope("host-a:worktree-main") })
        );
    }

    #[test]
    fn unmapped_record_is_quarantined_without_owner() {
        let state = FocusaState::default();
        let legacy = LegacyState::from_records(vec![
            LegacyStateRecord::canonical("legacy-unmapped", &state).expect("legacy record"),
        ])
        .expect("legacy records");
        let mut shadow = ShadowWorkstreamStore::new();
        shadow.write(&legacy, &[]).expect("quarantine unmapped");
        assert!(shadow.rows().is_empty());
        assert_eq!(
            shadow.quarantine().rows()[0].reason,
            QuarantineReason::UnmappedLegacyRecord
        );
    }

    #[test]
    fn foreign_declared_scope_is_quarantined_without_repair() {
        let state = FocusaState::default();
        let legacy = LegacyState::from_records(vec![
            LegacyStateRecord::canonical("legacy-foreign", &state)
                .expect("legacy record")
                .with_declared_scope(scope("host-b:worktree-main")),
        ])
        .expect("legacy records");
        let mut shadow = ShadowWorkstreamStore::new();
        shadow
            .write(&legacy, &[mapping("legacy-foreign", "delivery")])
            .expect("quarantine foreign scope");
        assert!(shadow.rows().is_empty());
        assert_eq!(
            shadow.quarantine().rows()[0].reason,
            QuarantineReason::ConflictingProjectRoots
        );
    }

    #[test]
    fn serialization_only_difference_is_not_a_migration_mismatch() {
        let state = FocusaState::default();
        let payload = serde_json::to_vec_pretty(&state).expect("pretty state");
        let legacy = LegacyState::from_records(vec![
            LegacyStateRecord::from_payload("legacy-pretty", &payload, LegacyRecordKind::Canonical)
                .expect("typed payload"),
        ])
        .expect("legacy records");
        let mut shadow = ShadowWorkstreamStore::new();
        shadow
            .write(&legacy, &[mapping("legacy-pretty", "delivery")])
            .expect("shadow write");
        let report = ParityComparator::default().compare(&legacy, &shadow);
        assert_eq!(
            report.records[0].disposition,
            ParityDisposition::SerializationOnlyDifference
        );
        assert!(report.passes());
    }

    #[test]
    fn unsafe_fallback_is_reported_as_expected_removal() {
        let state = FocusaState::default();
        let record =
            LegacyStateRecord::unsafe_fallback("legacy-fallback", &state).expect("fallback record");
        let legacy = LegacyState::from_records(vec![record]).expect("legacy records");
        let mut shadow = ShadowWorkstreamStore::new();
        shadow.write(&legacy, &[]).expect("fallback omission");
        let report = ParityComparator::default().compare(&legacy, &shadow);
        assert_eq!(
            report.records[0].disposition,
            ParityDisposition::ExpectedRemovalOfUnsafeFallback
        );
        assert!(report.passes());
    }
}
