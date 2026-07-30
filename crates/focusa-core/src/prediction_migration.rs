//! Spec138 legacy prediction/metacognition migration without manufactured authority.

use crate::prediction_authority::{EpistemicScope, PredictionAuthorityEvent, ScopedAuthorityEvent};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredictionMigrationError {
    MissingIdentity,
    MissingEvidence,
    MissingReceipt,
    EmptyPayload,
    InvalidScope,
    InvalidSequence,
}

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
    if scope.project_root.trim().is_empty() || scope.continuity_id.trim().is_empty() {
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
                EpistemicScope {
                    project_root: "/project".into(),
                    continuity_id: "main".into(),
                },
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
        let event=migrate_legacy_record("migration",LegacyEpistemicSource::PredictionValueV1,"legacy:prediction",&serde_json::json!({"prediction_type":"release","predicted_outcome":"success","confidence":0.8,"authority_ref":"legacy-operator"}),EpistemicScope{project_root:"/project".into(),continuity_id:"main".into()},1,vec!["lineage:legacy".into()],vec!["evidence:legacy".into()],"receipt:migration",Utc::now()).unwrap();
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
