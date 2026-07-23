//! Spec 119 receipt projections and governed closure for Silent Sessions.
//! This module never owns a ledger; it emits requests for the existing event chain.

use crate::silent_session::{SilentSessionId, SilentSessionRunId, WorkpointBinding};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

pub const SILENT_SESSION_RECEIPT_PROJECTION_SCHEMA: &str =
    "focusa.silent_session_receipt_projection.v1";
pub const RECEIPT_EVENT_APPEND_SCHEMA: &str = "focusa.receipt_event_append_request.v1";
pub const CLOSURE_PROPOSAL_SCHEMA: &str = "focusa.silent_session_closure_proposal.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptType {
    WorkSession,
    RiskyMutation,
    BlockedClaim,
    Handoff,
    BootstrapDelivery,
    WorkItemClosure,
    FinalReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    SilentSession,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSessionReceiptProjection {
    pub schema: String,
    pub receipt_id: Uuid,
    pub receipt_type: ReceiptType,
    pub execution_mode: ExecutionMode,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub workpoint_ref: WorkpointBinding,
    pub work_item_ref: Option<String>,
    pub claim_ref: String,
    pub evidence_refs: Vec<String>,
    pub event_cursor: String,
    pub created_at: DateTime<Utc>,
    pub payload: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExistingEventChainKind {
    ExistingSilentSessionEventChain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExistingEventChainBinding {
    pub chain_kind: ExistingEventChainKind,
    pub append_target_ref: String,
    pub expected_previous_event_hash: String,
    pub event_cursor: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceiptEventAppendRequest {
    pub schema: String,
    pub idempotency_key: String,
    pub event_kind: String,
    pub event_source: String,
    pub payload_sha256: String,
    pub chain: ExistingEventChainBinding,
    pub payload: Value,
    pub creates_new_ledger: bool,
}

impl SilentSessionReceiptProjection {
    pub fn validate(&self) -> Result<(), SilentSessionReceiptError> {
        if self.schema != SILENT_SESSION_RECEIPT_PROJECTION_SCHEMA
            || self.receipt_id.get_version() != Some(uuid::Version::SortRand)
            || !self.session_id.is_uuid_v7()
            || !self.run_id.is_uuid_v7()
            || self.project_identity_ref.trim().is_empty()
            || self.continuity_id.trim().is_empty()
            || self.workpoint_ref.workpoint_id.trim().is_empty()
            || self.claim_ref.trim().is_empty()
            || self.event_cursor.trim().is_empty()
            || self.evidence_refs.is_empty()
            || self
                .evidence_refs
                .iter()
                .any(|value| value.trim().is_empty())
            || self
                .work_item_ref
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            || self.payload.is_null()
        {
            return Err(SilentSessionReceiptError::InvalidReceipt);
        }
        if self.execution_mode != ExecutionMode::SilentSession {
            return Err(SilentSessionReceiptError::InvalidExecutionMode);
        }
        Ok(())
    }

    pub fn into_existing_event_append(
        self,
        chain: ExistingEventChainBinding,
    ) -> Result<ReceiptEventAppendRequest, SilentSessionReceiptError> {
        self.validate()?;
        if chain.chain_kind != ExistingEventChainKind::ExistingSilentSessionEventChain
            || chain.append_target_ref.trim().is_empty()
            || !valid_sha256(&chain.expected_previous_event_hash)
            || chain.event_cursor != self.event_cursor
        {
            return Err(SilentSessionReceiptError::InvalidEventChainBinding);
        }
        let payload =
            serde_json::to_value(&self).map_err(|_| SilentSessionReceiptError::Serialization)?;
        let bytes =
            serde_json::to_vec(&payload).map_err(|_| SilentSessionReceiptError::Serialization)?;
        Ok(ReceiptEventAppendRequest {
            schema: RECEIPT_EVENT_APPEND_SCHEMA.into(),
            idempotency_key: format!("silent-session-receipt:{}", self.receipt_id),
            event_kind: "receipt_commit_requested".into(),
            event_source: "silent_session".into(),
            payload_sha256: hex::encode(Sha256::digest(bytes)),
            chain,
            payload,
            creates_new_ledger: false,
        })
    }
}

#[allow(clippy::too_many_arguments)]
pub fn project_receipt(
    receipt_type: ReceiptType,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    project_identity_ref: impl Into<String>,
    continuity_id: impl Into<String>,
    workpoint_ref: WorkpointBinding,
    work_item_ref: Option<String>,
    claim_ref: impl Into<String>,
    evidence_refs: Vec<String>,
    event_cursor: impl Into<String>,
    payload: Value,
    created_at: DateTime<Utc>,
) -> Result<SilentSessionReceiptProjection, SilentSessionReceiptError> {
    let receipt = SilentSessionReceiptProjection {
        schema: SILENT_SESSION_RECEIPT_PROJECTION_SCHEMA.into(),
        receipt_id: Uuid::now_v7(),
        receipt_type,
        execution_mode: ExecutionMode::SilentSession,
        session_id,
        run_id,
        project_identity_ref: project_identity_ref.into(),
        continuity_id: continuity_id.into(),
        workpoint_ref,
        work_item_ref,
        claim_ref: claim_ref.into(),
        evidence_refs,
        event_cursor: event_cursor.into(),
        created_at,
        payload,
    };
    receipt.validate()?;
    Ok(receipt)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClosureStage {
    Proposed,
    Validated,
    Authorized,
    ProviderSubmitted,
    Reconciled,
    Audited,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosureProposal {
    pub schema: String,
    pub proposal_id: Uuid,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub session_actor_ref: String,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub workpoint_ref: WorkpointBinding,
    pub work_item_ref: String,
    pub claim_ref: String,
    pub stage: ClosureStage,
    pub validation_ref: Option<String>,
    pub authority_ref: Option<String>,
    pub authority_actor_ref: Option<String>,
    pub provider_submission_ref: Option<String>,
    pub provider_reconciliation_ref: Option<String>,
    pub provider_reports_closed: bool,
    pub audit_ref: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_closure_proposal(
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    session_actor_ref: impl Into<String>,
    project_identity_ref: impl Into<String>,
    continuity_id: impl Into<String>,
    workpoint_ref: WorkpointBinding,
    work_item_ref: impl Into<String>,
    claim_ref: impl Into<String>,
    now: DateTime<Utc>,
) -> Result<ClosureProposal, SilentSessionReceiptError> {
    let proposal = ClosureProposal {
        schema: CLOSURE_PROPOSAL_SCHEMA.into(),
        proposal_id: Uuid::now_v7(),
        session_id,
        run_id,
        session_actor_ref: session_actor_ref.into(),
        project_identity_ref: project_identity_ref.into(),
        continuity_id: continuity_id.into(),
        workpoint_ref,
        work_item_ref: work_item_ref.into(),
        claim_ref: claim_ref.into(),
        stage: ClosureStage::Proposed,
        validation_ref: None,
        authority_ref: None,
        authority_actor_ref: None,
        provider_submission_ref: None,
        provider_reconciliation_ref: None,
        provider_reports_closed: false,
        audit_ref: None,
        updated_at: now,
    };
    validate_closure_identity(&proposal)?;
    Ok(proposal)
}

pub enum ClosureTransition<'a> {
    Validate {
        validation_ref: &'a str,
    },
    Authorize {
        authority_ref: &'a str,
        authority_actor_ref: &'a str,
    },
    ProviderSubmit {
        provider_submission_ref: &'a str,
    },
    Reconcile {
        provider_reconciliation_ref: &'a str,
        provider_reports_closed: bool,
    },
    Audit {
        audit_ref: &'a str,
    },
}

pub fn advance_closure(
    mut proposal: ClosureProposal,
    transition: ClosureTransition<'_>,
    now: DateTime<Utc>,
) -> Result<ClosureProposal, SilentSessionReceiptError> {
    validate_closure_identity(&proposal)?;
    match (proposal.stage, transition) {
        (ClosureStage::Proposed, ClosureTransition::Validate { validation_ref }) => {
            require_ref(validation_ref)?;
            proposal.validation_ref = Some(validation_ref.into());
            proposal.stage = ClosureStage::Validated;
        }
        (
            ClosureStage::Validated,
            ClosureTransition::Authorize {
                authority_ref,
                authority_actor_ref,
            },
        ) => {
            require_ref(authority_ref)?;
            require_ref(authority_actor_ref)?;
            if authority_actor_ref == proposal.session_actor_ref {
                return Err(SilentSessionReceiptError::SelfClosureForbidden);
            }
            proposal.authority_ref = Some(authority_ref.into());
            proposal.authority_actor_ref = Some(authority_actor_ref.into());
            proposal.stage = ClosureStage::Authorized;
        }
        (
            ClosureStage::Authorized,
            ClosureTransition::ProviderSubmit {
                provider_submission_ref,
            },
        ) => {
            require_ref(provider_submission_ref)?;
            proposal.provider_submission_ref = Some(provider_submission_ref.into());
            proposal.stage = ClosureStage::ProviderSubmitted;
        }
        (
            ClosureStage::ProviderSubmitted,
            ClosureTransition::Reconcile {
                provider_reconciliation_ref,
                provider_reports_closed,
            },
        ) => {
            require_ref(provider_reconciliation_ref)?;
            proposal.provider_reconciliation_ref = Some(provider_reconciliation_ref.into());
            proposal.provider_reports_closed = provider_reports_closed;
            proposal.stage = ClosureStage::Reconciled;
        }
        (ClosureStage::Reconciled, ClosureTransition::Audit { audit_ref }) => {
            require_ref(audit_ref)?;
            if !proposal.provider_reports_closed {
                return Err(SilentSessionReceiptError::ProviderClosureNotObserved);
            }
            proposal.audit_ref = Some(audit_ref.into());
            proposal.stage = ClosureStage::Audited;
        }
        _ => return Err(SilentSessionReceiptError::InvalidClosureTransition),
    }
    proposal.updated_at = now;
    Ok(proposal)
}

pub fn project_closure_receipt(
    proposal: &ClosureProposal,
    event_cursor: impl Into<String>,
    now: DateTime<Utc>,
) -> Result<SilentSessionReceiptProjection, SilentSessionReceiptError> {
    validate_closure_identity(proposal)?;
    if proposal.stage != ClosureStage::Audited
        || !proposal.provider_reports_closed
        || proposal.validation_ref.is_none()
        || proposal.authority_ref.is_none()
        || proposal.provider_submission_ref.is_none()
        || proposal.provider_reconciliation_ref.is_none()
        || proposal.audit_ref.is_none()
    {
        return Err(SilentSessionReceiptError::ClosureNotAudited);
    }
    project_receipt(
        ReceiptType::WorkItemClosure,
        proposal.session_id,
        proposal.run_id,
        proposal.project_identity_ref.clone(),
        proposal.continuity_id.clone(),
        proposal.workpoint_ref.clone(),
        Some(proposal.work_item_ref.clone()),
        proposal.claim_ref.clone(),
        vec![
            proposal.validation_ref.clone().unwrap_or_default(),
            proposal.authority_ref.clone().unwrap_or_default(),
            proposal.provider_submission_ref.clone().unwrap_or_default(),
            proposal
                .provider_reconciliation_ref
                .clone()
                .unwrap_or_default(),
            proposal.audit_ref.clone().unwrap_or_default(),
        ],
        event_cursor,
        json!({
            "proposal_id": proposal.proposal_id,
            "provider_reports_closed": true,
            "closure_authority": "provider_reconciled_and_audited",
        }),
        now,
    )
}

fn validate_closure_identity(proposal: &ClosureProposal) -> Result<(), SilentSessionReceiptError> {
    if proposal.schema != CLOSURE_PROPOSAL_SCHEMA
        || proposal.proposal_id.get_version() != Some(uuid::Version::SortRand)
        || !proposal.session_id.is_uuid_v7()
        || !proposal.run_id.is_uuid_v7()
        || proposal.session_actor_ref.trim().is_empty()
        || proposal.project_identity_ref.trim().is_empty()
        || proposal.continuity_id.trim().is_empty()
        || proposal.workpoint_ref.workpoint_id.trim().is_empty()
        || proposal.work_item_ref.trim().is_empty()
        || proposal.claim_ref.trim().is_empty()
    {
        return Err(SilentSessionReceiptError::InvalidClosureProposal);
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn require_ref(value: &str) -> Result<(), SilentSessionReceiptError> {
    if value.trim().is_empty() {
        Err(SilentSessionReceiptError::MissingTransitionEvidence)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SilentSessionReceiptError {
    #[error("Silent Session receipt projection is invalid")]
    InvalidReceipt,
    #[error("Silent Session receipt execution mode must be silent_session")]
    InvalidExecutionMode,
    #[error("receipt must append through the existing event/hash chain")]
    InvalidEventChainBinding,
    #[error("receipt serialization failed")]
    Serialization,
    #[error("closure proposal identity is invalid")]
    InvalidClosureProposal,
    #[error("closure transition skipped or reordered a governed stage")]
    InvalidClosureTransition,
    #[error("Silent Session cannot authorize its own closure")]
    SelfClosureForbidden,
    #[error("closure transition evidence is missing")]
    MissingTransitionEvidence,
    #[error("provider closure was not observed during reconciliation")]
    ProviderClosureNotObserved,
    #[error("closure receipt requires audited provider-observed closure")]
    ClosureNotAudited,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workpoint() -> WorkpointBinding {
        WorkpointBinding {
            workpoint_id: "workpoint:test".into(),
            revision: Some(4),
        }
    }

    fn receipt(receipt_type: ReceiptType) -> SilentSessionReceiptProjection {
        project_receipt(
            receipt_type,
            SilentSessionId::new(),
            SilentSessionRunId::new(),
            "project:focusa",
            "continuity:test",
            workpoint(),
            Some("focusa-a6yq6.7.5".into()),
            "claim:test",
            vec!["evidence:test".into()],
            "cursor:10",
            json!({"bounded": true}),
            Utc::now(),
        )
        .unwrap()
    }

    #[test]
    fn every_receipt_type_projects_to_existing_event_chain_only() {
        for receipt_type in [
            ReceiptType::WorkSession,
            ReceiptType::RiskyMutation,
            ReceiptType::BlockedClaim,
            ReceiptType::Handoff,
            ReceiptType::BootstrapDelivery,
            ReceiptType::WorkItemClosure,
            ReceiptType::FinalReport,
        ] {
            let append = receipt(receipt_type)
                .into_existing_event_append(ExistingEventChainBinding {
                    chain_kind: ExistingEventChainKind::ExistingSilentSessionEventChain,
                    append_target_ref: "silent-session-events:existing".into(),
                    expected_previous_event_hash: "a".repeat(64),
                    event_cursor: "cursor:10".into(),
                })
                .unwrap();
            assert_eq!(append.event_kind, "receipt_commit_requested");
            assert_eq!(append.event_source, "silent_session");
            assert!(!append.creates_new_ledger);
            assert_eq!(append.payload["execution_mode"], "silent_session");
        }
    }

    fn proposal() -> ClosureProposal {
        prepare_closure_proposal(
            SilentSessionId::new(),
            SilentSessionRunId::new(),
            "actor:session",
            "project:focusa",
            "continuity:test",
            workpoint(),
            "focusa-a6yq6.7.5",
            "claim:closure",
            Utc::now(),
        )
        .unwrap()
    }

    #[test]
    fn session_can_propose_but_cannot_self_authorize_or_skip_stages() {
        let proposed = proposal();
        assert_eq!(
            advance_closure(
                proposed.clone(),
                ClosureTransition::Authorize {
                    authority_ref: "authority:self",
                    authority_actor_ref: "actor:session",
                },
                Utc::now(),
            ),
            Err(SilentSessionReceiptError::InvalidClosureTransition)
        );
        let validated = advance_closure(
            proposed,
            ClosureTransition::Validate {
                validation_ref: "validation:passed",
            },
            Utc::now(),
        )
        .unwrap();
        assert_eq!(
            advance_closure(
                validated,
                ClosureTransition::Authorize {
                    authority_ref: "authority:self",
                    authority_actor_ref: "actor:session",
                },
                Utc::now(),
            ),
            Err(SilentSessionReceiptError::SelfClosureForbidden)
        );
    }

    #[test]
    fn closure_receipt_requires_prepare_validate_authorize_provider_reconcile_audit() {
        let validated = advance_closure(
            proposal(),
            ClosureTransition::Validate {
                validation_ref: "validation:passed",
            },
            Utc::now(),
        )
        .unwrap();
        let authorized = advance_closure(
            validated,
            ClosureTransition::Authorize {
                authority_ref: "authority:operator",
                authority_actor_ref: "actor:operator",
            },
            Utc::now(),
        )
        .unwrap();
        let submitted = advance_closure(
            authorized,
            ClosureTransition::ProviderSubmit {
                provider_submission_ref: "provider-submit:1",
            },
            Utc::now(),
        )
        .unwrap();
        let reconciled = advance_closure(
            submitted,
            ClosureTransition::Reconcile {
                provider_reconciliation_ref: "provider-reconcile:1",
                provider_reports_closed: true,
            },
            Utc::now(),
        )
        .unwrap();
        let audited = advance_closure(
            reconciled,
            ClosureTransition::Audit {
                audit_ref: "audit:closure",
            },
            Utc::now(),
        )
        .unwrap();
        let receipt = project_closure_receipt(&audited, "cursor:closure", Utc::now()).unwrap();
        assert_eq!(receipt.receipt_type, ReceiptType::WorkItemClosure);
        assert_eq!(receipt.execution_mode, ExecutionMode::SilentSession);
        assert_eq!(receipt.evidence_refs.len(), 5);
    }
}
