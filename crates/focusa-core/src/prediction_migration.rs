//! Spec138 legacy prediction/metacognition migration without manufactured authority.

use crate::prediction_authority::{EpistemicScope, PredictionAuthorityEvent, ScopedAuthorityEvent};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyEpistemicSource {
    PredictionValueV1,
    MetacognitionCaptureV1,
    ReflectionV1,
    AdjustmentV1,
    EvaluationV1,
    LegacyScores,
    LegacyPromotions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyAuthorityStatus {
    ReadableAdvisory,
    QuarantinedAmbiguous,
    ScopedCanonicalMigration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegacyMigrationRecord {
    pub migration_id: String,
    pub source: LegacyEpistemicSource,
    pub source_record_ref: String,
    pub source_sha256: String,
    pub authority_status: LegacyAuthorityStatus,
    pub ambiguity_labels: Vec<String>,
    pub mapped_primitive_refs: Vec<String>,
    pub lineage_refs: Vec<String>,
    pub rollback_ref: String,
    pub migrated_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
    #[serde(default)]
    pub scope_migration_plan_ref: Option<String>,
    #[serde(default)]
    pub destination_scope: Option<EpistemicScope>,
    #[serde(default)]
    pub source_timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub source_vector_clock_ref: Option<String>,
    #[serde(default)]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredictionMigrationError {
    MissingIdentity,
    MissingEvidence,
    MissingReceipt,
    EmptyPayload,
    InvalidScope,
    InvalidSequence,
    InvalidSourceDigest,
    InvalidScopeEvidence,
    ConflictingAuthoritativeScopes,
    MigrationPlanMismatch,
    NotLegacyMigrationEvent,
    LegacySourceUnreadable,
    InvalidLegacyJson(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegacyUnscopedPredictionRow {
    pub line_number: usize,
    pub source_sha256: String,
    pub source_record_ref: Option<String>,
    pub source_timestamp: Option<DateTime<Utc>>,
    pub legacy_project_root: Option<String>,
    pub legacy_continuity_id: Option<String>,
    pub raw_event: serde_json::Value,
}

pub fn scan_legacy_unscoped_prediction_rows(
    path: &Path,
) -> Result<Vec<LegacyUnscopedPredictionRow>, PredictionMigrationError> {
    let file = File::open(path).map_err(|_| PredictionMigrationError::LegacySourceUnreadable)?;
    let mut rows = Vec::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|_| PredictionMigrationError::LegacySourceUnreadable)?;
        if line.trim().is_empty() {
            continue;
        }
        let value = serde_json::from_str::<serde_json::Value>(&line)
            .map_err(|_| PredictionMigrationError::InvalidLegacyJson(index + 1))?;
        let scope = value.pointer("/event/scope");
        if scope.and_then(|scope| scope.get("root_scope")).is_some() {
            continue;
        }
        rows.push(LegacyUnscopedPredictionRow {
            line_number: index + 1,
            source_sha256: hex::encode(Sha256::digest(line.as_bytes())),
            source_record_ref: value
                .pointer("/event/event_id")
                .and_then(|value| value.as_str())
                .map(str::to_string),
            source_timestamp: value
                .pointer("/event/recorded_at")
                .and_then(|value| value.as_str())
                .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                .map(|value| value.with_timezone(&Utc)),
            legacy_project_root: scope
                .and_then(|scope| scope.get("project_root"))
                .and_then(|value| value.as_str())
                .map(str::to_string),
            legacy_continuity_id: scope
                .and_then(|scope| scope.get("continuity_id"))
                .and_then(|value| value.as_str())
                .map(str::to_string),
            raw_event: value,
        });
    }
    Ok(rows)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeAttributionEvidenceKind {
    TypedScopeIdentity,
    VerifiedProjectMarker,
    VerifiedHostIdentity,
    SessionOwnership,
    WorkpointOwnership,
    AttachmentOwnership,
    ParentEventOwnership,
    OperatorConfirmation,
    PathSimilarity,
    TitleSimilarity,
    TagSimilarity,
}

impl ScopeAttributionEvidenceKind {
    fn authoritative(self) -> bool {
        !matches!(
            self,
            Self::PathSimilarity | Self::TitleSimilarity | Self::TagSimilarity
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeAttributionEvidence {
    pub evidence_id: String,
    pub kind: ScopeAttributionEvidenceKind,
    pub candidate_scope: Option<EpistemicScope>,
    pub source_ref: String,
    pub source_digest: String,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyScopeMigrationDisposition {
    ScopedCanonical,
    QuarantinedNoAuthoritativeEvidence,
    QuarantinedConflictingEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyScopeMigrationPlan {
    pub plan_id: String,
    pub source_record_ref: String,
    pub source_sha256: String,
    pub source_timestamp: DateTime<Utc>,
    pub source_vector_clock_ref: Option<String>,
    pub destination_scope: Option<EpistemicScope>,
    pub disposition: LegacyScopeMigrationDisposition,
    pub evidence: Vec<ScopeAttributionEvidence>,
    pub reason_codes: Vec<String>,
    pub idempotency_key: String,
    pub receipt_ref: String,
}

#[allow(clippy::too_many_arguments)]
pub fn plan_legacy_scope_migration(
    plan_id: impl Into<String>,
    source_record_ref: impl Into<String>,
    source_sha256: impl Into<String>,
    source_timestamp: DateTime<Utc>,
    source_vector_clock_ref: Option<String>,
    evidence: Vec<ScopeAttributionEvidence>,
    idempotency_key: impl Into<String>,
    receipt_ref: impl Into<String>,
) -> Result<LegacyScopeMigrationPlan, PredictionMigrationError> {
    let plan_id = plan_id.into();
    let source_record_ref = source_record_ref.into();
    let source_sha256 = source_sha256.into();
    let idempotency_key = idempotency_key.into();
    let receipt_ref = receipt_ref.into();
    if plan_id.trim().is_empty()
        || source_record_ref.trim().is_empty()
        || idempotency_key.trim().is_empty()
    {
        return Err(PredictionMigrationError::MissingIdentity);
    }
    if source_sha256.len() != 64 || !source_sha256.chars().all(|value| value.is_ascii_hexdigit()) {
        return Err(PredictionMigrationError::InvalidSourceDigest);
    }
    if receipt_ref.trim().is_empty() {
        return Err(PredictionMigrationError::MissingReceipt);
    }
    if evidence.iter().any(|item| {
        item.evidence_id.trim().is_empty()
            || item.source_ref.trim().is_empty()
            || item.source_digest.len() != 64
            || item
                .candidate_scope
                .as_ref()
                .is_some_and(|scope| scope.validate().is_err())
    }) {
        return Err(PredictionMigrationError::InvalidScopeEvidence);
    }
    let authoritative_scopes = evidence
        .iter()
        .filter(|item| item.kind.authoritative())
        .filter_map(|item| item.candidate_scope.clone())
        .collect::<std::collections::HashSet<_>>();
    let (destination_scope, disposition, reason_codes) = match authoritative_scopes.len() {
        0 => (
            None,
            LegacyScopeMigrationDisposition::QuarantinedNoAuthoritativeEvidence,
            vec!["no_authoritative_scope_evidence".to_string()],
        ),
        1 => (
            authoritative_scopes.iter().next().cloned(),
            LegacyScopeMigrationDisposition::ScopedCanonical,
            vec!["authoritative_scope_evidence_converged".to_string()],
        ),
        _ => (
            None,
            LegacyScopeMigrationDisposition::QuarantinedConflictingEvidence,
            vec!["conflicting_authoritative_scope_evidence".to_string()],
        ),
    };
    Ok(LegacyScopeMigrationPlan {
        plan_id,
        source_record_ref,
        source_sha256,
        source_timestamp,
        source_vector_clock_ref,
        destination_scope,
        disposition,
        evidence,
        reason_codes,
        idempotency_key,
        receipt_ref,
    })
}

pub fn apply_legacy_scope_migration_plan(
    mut event: ScopedAuthorityEvent,
    plan: &LegacyScopeMigrationPlan,
) -> Result<Option<ScopedAuthorityEvent>, PredictionMigrationError> {
    if plan.disposition != LegacyScopeMigrationDisposition::ScopedCanonical {
        return Ok(None);
    }
    let destination = plan
        .destination_scope
        .as_ref()
        .ok_or(PredictionMigrationError::MigrationPlanMismatch)?;
    if &event.scope != destination {
        return Err(PredictionMigrationError::MigrationPlanMismatch);
    }
    let PredictionAuthorityEvent::LegacyMigration(record) = &mut event.event else {
        return Err(PredictionMigrationError::NotLegacyMigrationEvent);
    };
    if record.source_record_ref != plan.source_record_ref
        || record.source_sha256 != plan.source_sha256
    {
        return Err(PredictionMigrationError::MigrationPlanMismatch);
    }
    record.authority_status = LegacyAuthorityStatus::ScopedCanonicalMigration;
    record.scope_migration_plan_ref = Some(plan.plan_id.clone());
    record.destination_scope = Some(destination.clone());
    record.source_timestamp = Some(plan.source_timestamp);
    record.source_vector_clock_ref = plan.source_vector_clock_ref.clone();
    record.idempotency_key = Some(plan.idempotency_key.clone());
    record.evidence_refs.extend(
        plan.evidence
            .iter()
            .map(|evidence| evidence.evidence_id.clone()),
    );
    record.evidence_refs.sort();
    record.evidence_refs.dedup();
    event.receipt_ref = plan.receipt_ref.clone();
    Ok(Some(event))
}

#[allow(clippy::too_many_arguments)]
pub fn migrate_legacy_record(
    migration_id: impl Into<String>,
    source: LegacyEpistemicSource,
    source_record_ref: impl Into<String>,
    payload: &serde_json::Value,
    scope: EpistemicScope,
    sequence: u64,
    lineage_refs: Vec<String>,
    evidence_refs: Vec<String>,
    receipt_ref: impl Into<String>,
    now: DateTime<Utc>,
) -> Result<ScopedAuthorityEvent, PredictionMigrationError> {
    let migration_id = migration_id.into();
    let source_record_ref = source_record_ref.into();
    let receipt_ref = receipt_ref.into();
    if migration_id.trim().is_empty() || source_record_ref.trim().is_empty() {
        return Err(PredictionMigrationError::MissingIdentity);
    }
    if scope.validate().is_err() {
        return Err(PredictionMigrationError::InvalidScope);
    }
    if sequence == 0 {
        return Err(PredictionMigrationError::InvalidSequence);
    }
    if evidence_refs.is_empty() || lineage_refs.is_empty() {
        return Err(PredictionMigrationError::MissingEvidence);
    }
    if receipt_ref.trim().is_empty() {
        return Err(PredictionMigrationError::MissingReceipt);
    }
    if payload.is_null() {
        return Err(PredictionMigrationError::EmptyPayload);
    }
    let payload_bytes =
        serde_json::to_vec(payload).map_err(|_| PredictionMigrationError::EmptyPayload)?;
    if payload_bytes.is_empty() {
        return Err(PredictionMigrationError::EmptyPayload);
    }
    let mut ambiguity_labels = Vec::new();
    let mapped_primitive_refs = source_primitive_refs(source);
    let required = required_legacy_fields(source);
    for field in required {
        if payload.get(field).is_none() {
            ambiguity_labels.push(format!("missing_legacy_field:{field}"));
        }
    }
    if payload.get("authority_ref").is_none() {
        ambiguity_labels.push("legacy_authority_unverified".into());
    }
    if payload.get("scoring_policy_ref").is_none()
        && matches!(
            source,
            LegacyEpistemicSource::LegacyScores | LegacyEpistemicSource::EvaluationV1
        )
    {
        ambiguity_labels.push("legacy_scoring_policy_unfrozen".into());
    }
    let authority_status = if ambiguity_labels.is_empty() {
        LegacyAuthorityStatus::ReadableAdvisory
    } else {
        LegacyAuthorityStatus::QuarantinedAmbiguous
    };
    let record = LegacyMigrationRecord {
        migration_id: migration_id.clone(),
        source,
        source_record_ref,
        source_sha256: hex::encode(Sha256::digest(payload_bytes)),
        authority_status,
        ambiguity_labels,
        mapped_primitive_refs,
        lineage_refs,
        rollback_ref: format!("rollback:migration:{migration_id}"),
        migrated_at: now,
        evidence_refs: evidence_refs.clone(),
        receipt_ref: receipt_ref.clone(),
        scope_migration_plan_ref: None,
        destination_scope: None,
        source_timestamp: None,
        source_vector_clock_ref: None,
        idempotency_key: None,
    };
    Ok(ScopedAuthorityEvent {
        event_id: format!("migration-event:{migration_id}"),
        sequence,
        scope,
        recorded_at: now,
        event: PredictionAuthorityEvent::LegacyMigration(record),
        evidence_refs,
        receipt_ref,
    })
}

fn required_legacy_fields(source: LegacyEpistemicSource) -> &'static [&'static str] {
    match source {
        LegacyEpistemicSource::PredictionValueV1 => {
            &["prediction_type", "predicted_outcome", "confidence"]
        }
        LegacyEpistemicSource::MetacognitionCaptureV1 => &["kind", "content"],
        LegacyEpistemicSource::ReflectionV1 => &["reflection_id", "hypotheses"],
        LegacyEpistemicSource::AdjustmentV1 => &["adjustment_id", "selected_updates"],
        LegacyEpistemicSource::EvaluationV1 => &["adjustment_id", "observed_metrics"],
        LegacyEpistemicSource::LegacyScores => &["score"],
        LegacyEpistemicSource::LegacyPromotions => &["learning_id", "promotion_status"],
    }
}

fn source_primitive_refs(source: LegacyEpistemicSource) -> Vec<String> {
    match source {
        LegacyEpistemicSource::PredictionValueV1 => {
            vec!["PredictionCommitment".into(), "ConfidenceDimensions".into()]
        }
        LegacyEpistemicSource::MetacognitionCaptureV1 => vec!["MetacognitiveSignal".into()],
        LegacyEpistemicSource::ReflectionV1 => vec!["ReflectionClaim".into()],
        LegacyEpistemicSource::AdjustmentV1 => vec!["AdjustmentProposal".into()],
        LegacyEpistemicSource::EvaluationV1 => vec!["LearningEvaluation".into()],
        LegacyEpistemicSource::LegacyScores => vec!["PredictionEvaluation".into()],
        LegacyEpistemicSource::LegacyPromotions => vec!["PromotionDecision".into()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> EpistemicScope {
        crate::scoped_state::WorkstreamKey::new(
            crate::scoped_state::ScopeRef::project(
                "project:migration-test",
                "/project",
                "migration-test",
                "fingerprint:migration-test",
            )
            .unwrap(),
            "main",
        )
        .unwrap()
    }

    #[test]
    fn all_legacy_sources_migrate_as_advisory_or_quarantined_never_canonical() {
        let sources = [
            LegacyEpistemicSource::PredictionValueV1,
            LegacyEpistemicSource::MetacognitionCaptureV1,
            LegacyEpistemicSource::ReflectionV1,
            LegacyEpistemicSource::AdjustmentV1,
            LegacyEpistemicSource::EvaluationV1,
            LegacyEpistemicSource::LegacyScores,
            LegacyEpistemicSource::LegacyPromotions,
        ];
        for (index, source) in sources.into_iter().enumerate() {
            let event = migrate_legacy_record(
                format!("migration-{index}"),
                source,
                format!("legacy:{index}"),
                &serde_json::json!({"legacy":true}),
                scope(),
                index as u64 + 1,
                vec![format!("lineage:{index}")],
                vec![format!("evidence:{index}")],
                format!("receipt:{index}"),
                Utc::now(),
            )
            .unwrap();
            let PredictionAuthorityEvent::LegacyMigration(record) = event.event else {
                panic!("wrong event")
            };
            assert!(matches!(
                record.authority_status,
                LegacyAuthorityStatus::ReadableAdvisory
                    | LegacyAuthorityStatus::QuarantinedAmbiguous
            ));
            assert!(!record.rollback_ref.is_empty());
        }
    }

    #[test]
    fn complete_legacy_record_preserves_hash_lineage_and_advisory_status() {
        let event=migrate_legacy_record("migration",LegacyEpistemicSource::PredictionValueV1,"legacy:prediction",&serde_json::json!({"prediction_type":"release","predicted_outcome":"success","confidence":0.8,"authority_ref":"legacy-operator"}),scope(),1,vec!["lineage:legacy".into()],vec!["evidence:legacy".into()],"receipt:migration",Utc::now()).unwrap();
        let PredictionAuthorityEvent::LegacyMigration(record) = event.event else {
            panic!("wrong event")
        };
        assert_eq!(
            record.authority_status,
            LegacyAuthorityStatus::ReadableAdvisory
        );
        assert_eq!(record.source_sha256.len(), 64);
    }
}
