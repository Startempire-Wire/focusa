//! Spec137 claim authority, freshness, urgency, and action preflight.

use crate::temporal::{
    TemporalClaim, TemporalClaimKind, TemporalClaimStatus, TemporalConfidence, TemporalScope,
    validate_claim,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalAuthorityClass {
    Operator,
    ExternalContract,
    VerifiedProvider,
    ProjectPolicy,
    Inference,
    Presentation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalClaimAuthority {
    pub authority_class: TemporalAuthorityClass,
    pub authority_ref: String,
    pub may_commit_external_deadline: bool,
    pub may_set_internal_target: bool,
    pub evidence_required: bool,
    pub operator_confirmation_required: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalClaimEnvelope {
    pub schema: String,
    pub claim: TemporalClaim,
    pub authority: TemporalClaimAuthority,
    pub freshness_ms: u64,
    pub stale_after_ms: u64,
    pub presentation_timezone: String,
    pub privacy_class: String,
    pub retention_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalPreflightStatus {
    Allowed,
    Blocked,
    Degraded,
    OperatorRequired,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalPreflight {
    pub schema: String,
    pub status: TemporalPreflightStatus,
    pub scope: TemporalScope,
    pub claim_id: Option<String>,
    pub warnings: Vec<String>,
    pub blockers: Vec<String>,
    pub exact_next_action: String,
    pub recovery_tools: Vec<String>,
    pub authority_escalated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalClaimError {
    InvalidClaim,
    ScopeMismatch,
    StaleClaim,
    ExpiredClaim,
    AuthorityDenied,
    EvidenceRequired,
    OperatorConfirmationRequired,
    ForecastCannotBecomeCommitment,
}

pub fn authorize_claim(
    envelope: &TemporalClaimEnvelope,
    now: DateTime<Utc>,
) -> Result<(), TemporalClaimError> {
    validate_claim(&envelope.claim, None).map_err(|_| TemporalClaimError::InvalidClaim)?;
    if envelope.freshness_ms > envelope.stale_after_ms {
        return Err(TemporalClaimError::StaleClaim);
    }
    if envelope
        .claim
        .expires_at
        .is_some_and(|expiry| expiry <= now)
    {
        return Err(TemporalClaimError::ExpiredClaim);
    }
    if envelope.authority.evidence_required && envelope.claim.evidence_refs.is_empty() {
        return Err(TemporalClaimError::EvidenceRequired);
    }
    if envelope.authority.operator_confirmation_required && !envelope.claim.operator_confirmed {
        return Err(TemporalClaimError::OperatorConfirmationRequired);
    }
    match envelope.claim.kind {
        TemporalClaimKind::ExternalCommitment
            if !envelope.authority.may_commit_external_deadline =>
        {
            Err(TemporalClaimError::AuthorityDenied)
        }
        TemporalClaimKind::InternalReadinessTarget
            if !envelope.authority.may_set_internal_target =>
        {
            Err(TemporalClaimError::AuthorityDenied)
        }
        TemporalClaimKind::Estimate | TemporalClaimKind::Forecast
            if envelope.authority.may_commit_external_deadline
                && envelope.claim.status == TemporalClaimStatus::Canonical =>
        {
            Err(TemporalClaimError::ForecastCannotBecomeCommitment)
        }
        _ => Ok(()),
    }
}

pub fn revise_claim(
    previous: &TemporalClaim,
    mut next: TemporalClaim,
) -> Result<(TemporalClaim, TemporalClaim), TemporalClaimError> {
    next.claim_id = previous.claim_id.clone();
    next.revision = previous.revision + 1;
    next.supersedes_revision = Some(previous.revision);
    validate_claim(&next, Some(previous)).map_err(|_| TemporalClaimError::InvalidClaim)?;
    let mut superseded = previous.clone();
    superseded.status = TemporalClaimStatus::Superseded;
    Ok((superseded, next))
}

pub fn temporal_preflight(
    action_scope: &TemporalScope,
    envelope: Option<&TemporalClaimEnvelope>,
    now: DateTime<Utc>,
) -> TemporalPreflight {
    let recovery = vec![
        "focusa_project_verify".to_string(),
        "focusa_temporal_status".to_string(),
        "focusa_temporal_revise".to_string(),
    ];
    let Some(envelope) = envelope else {
        return TemporalPreflight {
            schema: "focusa.temporal_preflight.v1".into(),
            status: TemporalPreflightStatus::Allowed,
            scope: action_scope.clone(),
            claim_id: None,
            warnings: vec!["no deadline is set; no urgency was inferred".into()],
            blockers: Vec::new(),
            exact_next_action: "continue without fabricated deadline pressure".into(),
            recovery_tools: recovery,
            authority_escalated: false,
        };
    };
    if &envelope.claim.scope != action_scope {
        return TemporalPreflight {
            schema: "focusa.temporal_preflight.v1".into(),
            status: TemporalPreflightStatus::Blocked,
            scope: action_scope.clone(),
            claim_id: Some(envelope.claim.claim_id.clone()),
            warnings: Vec::new(),
            blockers: vec!["temporal claim belongs to a different project/workstream".into()],
            exact_next_action: "verify project_root + continuity_id before temporal action".into(),
            recovery_tools: recovery,
            authority_escalated: false,
        };
    }
    match authorize_claim(envelope, now) {
        Ok(()) => TemporalPreflight {
            schema: "focusa.temporal_preflight.v1".into(),
            status: TemporalPreflightStatus::Allowed,
            scope: action_scope.clone(),
            claim_id: Some(envelope.claim.claim_id.clone()),
            warnings: Vec::new(),
            blockers: Vec::new(),
            exact_next_action: "continue within existing permissions and evidence policy".into(),
            recovery_tools: recovery,
            authority_escalated: false,
        },
        Err(error) => TemporalPreflight {
            schema: "focusa.temporal_preflight.v1".into(),
            status: if error == TemporalClaimError::OperatorConfirmationRequired {
                TemporalPreflightStatus::OperatorRequired
            } else {
                TemporalPreflightStatus::Blocked
            },
            scope: action_scope.clone(),
            claim_id: Some(envelope.claim.claim_id.clone()),
            warnings: Vec::new(),
            blockers: vec![format!("{error:?}")],
            exact_next_action: "revise or re-authorize the temporal claim with evidence".into(),
            recovery_tools: recovery,
            authority_escalated: false,
        },
    }
}

pub fn derive_urgency(source: &TemporalClaim, now: DateTime<Utc>) -> Option<TemporalClaim> {
    if source.kind != TemporalClaimKind::ExternalCommitment
        || source.status != TemporalClaimStatus::Canonical
        || !source.operator_confirmed
        || source.target_at.is_none()
    {
        return None;
    }
    let target = source.target_at?;
    let remaining = target.signed_duration_since(now).num_milliseconds();
    let mut urgency = source.clone();
    urgency.claim_id = format!("{}:urgency", source.claim_id);
    urgency.revision = 1;
    urgency.kind = TemporalClaimKind::UrgencySignal;
    urgency.duration_ms = Some(remaining.max(0) as u64);
    urgency.target_at = Some(target);
    urgency.source = "derived_from_confirmed_commitment".into();
    urgency.source_ref = Some(source.claim_id.clone());
    urgency.operator_confirmed = false;
    urgency.confidence = if remaining <= 0 {
        TemporalConfidence::Verified
    } else {
        source.confidence
    };
    urgency.reason_code = Some(
        if remaining <= 0 {
            "deadline_reached"
        } else {
            "confirmed_deadline_remaining"
        }
        .into(),
    );
    Some(urgency)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim() -> TemporalClaim {
        let now = Utc::now();
        TemporalClaim {
            claim_id: "deadline".into(),
            revision: 1,
            scope: TemporalScope::project("/workspace/project", "main"),
            kind: TemporalClaimKind::ExternalCommitment,
            status: TemporalClaimStatus::Canonical,
            subject_ref: "release".into(),
            target_at: Some(now + chrono::Duration::hours(1)),
            duration_ms: None,
            timezone: "America/Los_Angeles".into(),
            source: "operator".into(),
            source_ref: None,
            operator_confirmed: true,
            confidence: TemporalConfidence::Verified,
            uncertainty: None,
            observed_at: now,
            effective_at: now,
            expires_at: None,
            supersedes_revision: None,
            evidence_refs: vec!["contract:release-date".into()],
            reason_code: None,
        }
    }

    fn envelope() -> TemporalClaimEnvelope {
        TemporalClaimEnvelope {
            schema: "focusa.temporal_claim_envelope.v1".into(),
            claim: claim(),
            authority: TemporalClaimAuthority {
                authority_class: TemporalAuthorityClass::Operator,
                authority_ref: "operator".into(),
                may_commit_external_deadline: true,
                may_set_internal_target: true,
                evidence_required: true,
                operator_confirmation_required: true,
            },
            freshness_ms: 0,
            stale_after_ms: 60_000,
            presentation_timezone: "America/Los_Angeles".into(),
            privacy_class: "project".into(),
            retention_until: None,
        }
    }

    #[test]
    fn no_claim_allows_work_without_fabricated_urgency() {
        let scope = claim().scope;
        let result = temporal_preflight(&scope, None, Utc::now());
        assert_eq!(result.status, TemporalPreflightStatus::Allowed);
        assert!(result.warnings[0].contains("no urgency"));
        assert!(!result.authority_escalated);
    }

    #[test]
    fn scope_mismatch_blocks_without_authority_escalation() {
        let mut other = claim().scope;
        other.continuity_id = "other".into();
        let result = temporal_preflight(&other, Some(&envelope()), Utc::now());
        assert_eq!(result.status, TemporalPreflightStatus::Blocked);
        assert!(!result.authority_escalated);
    }

    #[test]
    fn urgency_only_derives_from_confirmed_commitment() {
        assert!(derive_urgency(&claim(), Utc::now()).is_some());
        let mut forecast = claim();
        forecast.kind = TemporalClaimKind::Forecast;
        forecast.operator_confirmed = false;
        assert!(derive_urgency(&forecast, Utc::now()).is_none());
    }
}
