//! Spec138 learning-memory consolidation, forgetting, retention, and reactivation.

use crate::prediction_authority::EpistemicScope;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryState {
    Active,
    Consolidated,
    Decayed,
    Archived,
    Reactivated,
    Expired,
    Revoked,
    DeletedTombstone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub policy_id: String,
    pub version: u64,
    pub review_interval_seconds: i64,
    pub expiry_seconds: i64,
    pub archive_before_delete: bool,
    pub encryption_required: bool,
    pub deletion_receipt_required: bool,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearningMemoryRecord {
    pub memory_id: String,
    pub scope: EpistemicScope,
    pub state: MemoryState,
    pub applicability_refs: Vec<String>,
    pub source_memory_refs: Vec<String>,
    pub exception_refs: Vec<String>,
    pub conflict_refs: Vec<String>,
    pub provenance_refs: Vec<String>,
    pub effectiveness_score: f64,
    pub version: u64,
    pub supersedes_version: Option<u64>,
    pub legal_hold_ref: Option<String>,
    pub encrypted: bool,
    pub retention_policy_ref: String,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsolidationAuthority {
    pub authority_id: String,
    pub actor_ref: String,
    pub policy_ref: String,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum MemoryLifecycleAction {
    Decay,
    Archive,
    Expire,
    Revoke {
        reason: String,
    },
    DeleteTombstone,
    Reactivate {
        evaluation_ref: String,
        new_evidence_refs: Vec<String>,
    },
    ApplyLegalHold {
        legal_hold_ref: String,
    },
    ReleaseLegalHold {
        legal_hold_ref: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryLifecycleEvent {
    pub event_id: String,
    pub memory_id: String,
    pub from_state: MemoryState,
    pub to_state: MemoryState,
    pub action: MemoryLifecycleAction,
    pub authority_ref: String,
    pub occurred_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryLifecycleError {
    EmptyInput,
    ScopeMismatch,
    ApplicabilityMismatch,
    MissingIdentity,
    MissingEvidence,
    MissingReceipt,
    MissingExceptionsOrConflicts,
    InvalidVersion,
    InvalidPolicy,
    EncryptionRequired,
    InvalidTransition,
    LegalHoldBlocksDeletion,
    ReactivationProofRequired,
    StateMismatch,
}

pub fn consolidate_memories(
    memory_id: impl Into<String>,
    memories: &[LearningMemoryRecord],
    policy: &RetentionPolicy,
    authority: &ConsolidationAuthority,
    now: DateTime<Utc>,
) -> Result<LearningMemoryRecord, MemoryLifecycleError> {
    if memories.len() < 2 {
        return Err(MemoryLifecycleError::EmptyInput);
    }
    validate_policy(policy)?;
    if authority.policy_ref != policy.policy_id || authority.evidence_refs.is_empty() {
        return Err(MemoryLifecycleError::MissingEvidence);
    }
    if authority.receipt_ref.trim().is_empty() {
        return Err(MemoryLifecycleError::MissingReceipt);
    }
    let scope = memories[0].scope.clone();
    let applicability = memories[0].applicability_refs.clone();
    if memories.iter().any(|memory| memory.scope != scope) {
        return Err(MemoryLifecycleError::ScopeMismatch);
    }
    if memories
        .iter()
        .any(|memory| memory.applicability_refs != applicability)
    {
        return Err(MemoryLifecycleError::ApplicabilityMismatch);
    }
    if policy.encryption_required && memories.iter().any(|memory| !memory.encrypted) {
        return Err(MemoryLifecycleError::EncryptionRequired);
    }
    let union = |values: Vec<String>| {
        values
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    };
    let source_memory_refs = memories
        .iter()
        .map(|memory| memory.memory_id.clone())
        .collect();
    let exception_refs = union(
        memories
            .iter()
            .flat_map(|memory| memory.exception_refs.clone())
            .collect(),
    );
    let conflict_refs = union(
        memories
            .iter()
            .flat_map(|memory| memory.conflict_refs.clone())
            .collect(),
    );
    let provenance_refs = union(
        memories
            .iter()
            .flat_map(|memory| memory.provenance_refs.clone())
            .collect(),
    );
    let mut evidence_refs = union(
        memories
            .iter()
            .flat_map(|memory| memory.evidence_refs.clone())
            .collect(),
    );
    evidence_refs.extend(authority.evidence_refs.clone());
    evidence_refs = union(evidence_refs);
    let effectiveness_score = memories
        .iter()
        .map(|memory| memory.effectiveness_score)
        .sum::<f64>()
        / memories.len() as f64;
    Ok(LearningMemoryRecord {
        memory_id: memory_id.into(),
        scope,
        state: MemoryState::Consolidated,
        applicability_refs: applicability,
        source_memory_refs,
        exception_refs,
        conflict_refs,
        provenance_refs,
        effectiveness_score,
        version: 1,
        supersedes_version: None,
        legal_hold_ref: None,
        encrypted: memories.iter().all(|memory| memory.encrypted),
        retention_policy_ref: policy.policy_id.clone(),
        created_at: now,
        reviewed_at: now,
        expires_at: now + chrono::Duration::seconds(policy.expiry_seconds),
        evidence_refs,
        receipt_ref: authority.receipt_ref.clone(),
    })
}

pub fn apply_memory_lifecycle(
    memory: &mut LearningMemoryRecord,
    action: MemoryLifecycleAction,
    authority_ref: impl Into<String>,
    evidence_refs: Vec<String>,
    receipt_ref: impl Into<String>,
    now: DateTime<Utc>,
) -> Result<MemoryLifecycleEvent, MemoryLifecycleError> {
    if evidence_refs.is_empty() {
        return Err(MemoryLifecycleError::MissingEvidence);
    }
    let authority_ref = authority_ref.into();
    let receipt_ref = receipt_ref.into();
    if authority_ref.trim().is_empty() || memory.memory_id.trim().is_empty() {
        return Err(MemoryLifecycleError::MissingIdentity);
    }
    if receipt_ref.trim().is_empty() {
        return Err(MemoryLifecycleError::MissingReceipt);
    }
    let from_state = memory.state;
    let to_state = match &action {
        MemoryLifecycleAction::Decay
            if matches!(
                from_state,
                MemoryState::Active | MemoryState::Consolidated | MemoryState::Reactivated
            ) =>
        {
            MemoryState::Decayed
        }
        MemoryLifecycleAction::Archive
            if matches!(from_state, MemoryState::Decayed | MemoryState::Expired) =>
        {
            MemoryState::Archived
        }
        MemoryLifecycleAction::Expire if now >= memory.expires_at => MemoryState::Expired,
        MemoryLifecycleAction::Revoke { .. } => MemoryState::Revoked,
        MemoryLifecycleAction::DeleteTombstone => {
            if memory.legal_hold_ref.is_some() {
                return Err(MemoryLifecycleError::LegalHoldBlocksDeletion);
            }
            if !matches!(from_state, MemoryState::Archived | MemoryState::Revoked) {
                return Err(MemoryLifecycleError::InvalidTransition);
            }
            MemoryState::DeletedTombstone
        }
        MemoryLifecycleAction::Reactivate {
            evaluation_ref,
            new_evidence_refs,
        } if matches!(
            from_state,
            MemoryState::Archived | MemoryState::Expired | MemoryState::Decayed
        ) =>
        {
            if evaluation_ref.trim().is_empty() || new_evidence_refs.is_empty() {
                return Err(MemoryLifecycleError::ReactivationProofRequired);
            }
            memory.evidence_refs.extend(new_evidence_refs.clone());
            MemoryState::Reactivated
        }
        MemoryLifecycleAction::ApplyLegalHold { legal_hold_ref } => {
            if legal_hold_ref.trim().is_empty() {
                return Err(MemoryLifecycleError::MissingIdentity);
            }
            memory.legal_hold_ref = Some(legal_hold_ref.clone());
            from_state
        }
        MemoryLifecycleAction::ReleaseLegalHold { legal_hold_ref } => {
            if memory.legal_hold_ref.as_deref() != Some(legal_hold_ref) {
                return Err(MemoryLifecycleError::StateMismatch);
            }
            memory.legal_hold_ref = None;
            from_state
        }
        _ => return Err(MemoryLifecycleError::InvalidTransition),
    };
    memory.state = to_state;
    memory.reviewed_at = now;
    memory.version += 1;
    memory.supersedes_version = Some(memory.version - 1);
    memory.evidence_refs.extend(evidence_refs.clone());
    Ok(MemoryLifecycleEvent {
        event_id: format!("memory-event:{}:{}", memory.memory_id, memory.version),
        memory_id: memory.memory_id.clone(),
        from_state,
        to_state,
        action,
        authority_ref,
        occurred_at: now,
        evidence_refs,
        receipt_ref,
    })
}

fn validate_policy(policy: &RetentionPolicy) -> Result<(), MemoryLifecycleError> {
    if policy.policy_id.trim().is_empty()
        || policy.version == 0
        || policy.review_interval_seconds <= 0
        || policy.expiry_seconds <= policy.review_interval_seconds
        || policy.evidence_refs.is_empty()
    {
        Err(MemoryLifecycleError::InvalidPolicy)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn memory(id: &str) -> LearningMemoryRecord {
        let now = Utc::now();
        LearningMemoryRecord {
            memory_id: id.into(),
            scope: EpistemicScope {
                project_root: "/project".into(),
                continuity_id: "main".into(),
            },
            state: MemoryState::Active,
            applicability_refs: vec!["task:release".into()],
            source_memory_refs: vec![],
            exception_refs: vec![format!("exception:{id}")],
            conflict_refs: vec![format!("conflict:{id}")],
            provenance_refs: vec![format!("provenance:{id}")],
            effectiveness_score: 0.8,
            version: 1,
            supersedes_version: None,
            legal_hold_ref: None,
            encrypted: true,
            retention_policy_ref: "retention:v1".into(),
            created_at: now,
            reviewed_at: now,
            expires_at: now + chrono::Duration::days(30),
            evidence_refs: vec![format!("evidence:{id}")],
            receipt_ref: format!("receipt:{id}"),
        }
    }
    fn policy() -> RetentionPolicy {
        RetentionPolicy {
            policy_id: "retention:v1".into(),
            version: 1,
            review_interval_seconds: 60,
            expiry_seconds: 3600,
            archive_before_delete: true,
            encryption_required: true,
            deletion_receipt_required: true,
            evidence_refs: vec!["evidence:policy".into()],
        }
    }
    #[test]
    fn consolidation_preserves_exceptions_conflicts_and_provenance() {
        let result = consolidate_memories(
            "combined",
            &[memory("a"), memory("b")],
            &policy(),
            &ConsolidationAuthority {
                authority_id: "authority".into(),
                actor_ref: "operator".into(),
                policy_ref: "retention:v1".into(),
                evidence_refs: vec!["evidence:authority".into()],
                receipt_ref: "receipt:consolidation".into(),
            },
            Utc::now(),
        )
        .unwrap();
        assert_eq!(result.source_memory_refs.len(), 2);
        assert_eq!(result.exception_refs.len(), 2);
        assert_eq!(result.conflict_refs.len(), 2);
        assert_eq!(result.provenance_refs.len(), 2);
    }
    #[test]
    fn legal_hold_blocks_deletion_and_reactivation_requires_new_proof() {
        let now = Utc::now();
        let mut value = memory("a");
        value.state = MemoryState::Archived;
        value.legal_hold_ref = Some("hold".into());
        assert_eq!(
            apply_memory_lifecycle(
                &mut value,
                MemoryLifecycleAction::DeleteTombstone,
                "authority",
                vec!["evidence".into()],
                "receipt",
                now
            ),
            Err(MemoryLifecycleError::LegalHoldBlocksDeletion)
        );
        value.legal_hold_ref = None;
        assert_eq!(
            apply_memory_lifecycle(
                &mut value,
                MemoryLifecycleAction::Reactivate {
                    evaluation_ref: "".into(),
                    new_evidence_refs: vec![]
                },
                "authority",
                vec!["evidence".into()],
                "receipt",
                now
            ),
            Err(MemoryLifecycleError::ReactivationProofRequired)
        );
    }
}
