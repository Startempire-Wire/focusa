//! Spec138 source access, privacy, adversarial-content, quarantine, and retention gates.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceAccessClass {
    Public,
    Licensed,
    Private,
    Sensitive,
    PublicSummaryOnly,
    Prohibited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyClass {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSecurityPolicy {
    pub policy_id: String,
    pub version: u64,
    pub maximum_manipulation_risk: u8,
    pub maximum_prompt_injection_risk: u8,
    pub encryption_required_for: Vec<PrivacyClass>,
    pub sanitization_required: bool,
    pub legal_hold_policy_ref: String,
    pub deletion_policy_ref: String,
    pub audit_export_policy_ref: String,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceIngestionRequest {
    pub request_id: String,
    pub source_ref: String,
    pub access_class: SourceAccessClass,
    pub privacy_class: PrivacyClass,
    pub access_authority_ref: String,
    pub license_terms_ref: Option<String>,
    pub raw_content_requested: bool,
    pub sanitized: bool,
    pub encrypted: bool,
    pub manipulation_risk: u8,
    pub prompt_injection_risk: u8,
    pub poisoning_indicator_refs: Vec<String>,
    pub high_consequence_use: bool,
    pub retention_policy_ref: String,
    pub legal_hold_ref: Option<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestionDisposition {
    Accept,
    PublicSummaryOnly,
    Quarantine,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSecurityDecision {
    pub decision_id: String,
    pub request_id: String,
    pub disposition: IngestionDisposition,
    pub reason_codes: Vec<String>,
    pub least_privilege_scope_refs: Vec<String>,
    pub retention_policy_ref: String,
    pub legal_hold_ref: Option<String>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
    pub decided_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSecurityAuditExport {
    pub export_id: String,
    pub policy_ref: String,
    pub decision_refs: Vec<String>,
    pub least_privilege_scope_refs: Vec<String>,
    pub encrypted: bool,
    pub generated_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceSecurityError {
    MissingIdentity,
    MissingAuthority,
    MissingLicense,
    MissingEvidence,
    MissingReceipt,
    MissingRetention,
    SanitizationRequired,
    EncryptionRequired,
    RawSummarySourceProhibited,
    InvalidRisk,
    HighConsequenceUnknownBlocked,
    EmptyAuditExport,
}

#[allow(clippy::too_many_arguments)]
pub fn build_security_audit_export(
    export_id: impl Into<String>,
    policy: &SourceSecurityPolicy,
    decision_refs: Vec<String>,
    least_privilege_scope_refs: Vec<String>,
    encrypted: bool,
    evidence_refs: Vec<String>,
    receipt_ref: impl Into<String>,
    now: DateTime<Utc>,
) -> Result<SourceSecurityAuditExport, SourceSecurityError> {
    let export_id = export_id.into();
    let receipt_ref = receipt_ref.into();
    if export_id.trim().is_empty() || policy.audit_export_policy_ref.trim().is_empty() {
        return Err(SourceSecurityError::MissingIdentity);
    }
    if decision_refs.is_empty() || least_privilege_scope_refs.is_empty() {
        return Err(SourceSecurityError::EmptyAuditExport);
    }
    if !encrypted {
        return Err(SourceSecurityError::EncryptionRequired);
    }
    if evidence_refs.is_empty() {
        return Err(SourceSecurityError::MissingEvidence);
    }
    if receipt_ref.trim().is_empty() {
        return Err(SourceSecurityError::MissingReceipt);
    }
    Ok(SourceSecurityAuditExport {
        export_id,
        policy_ref: policy.audit_export_policy_ref.clone(),
        decision_refs,
        least_privilege_scope_refs,
        encrypted,
        generated_at: now,
        evidence_refs,
        receipt_ref,
    })
}

pub fn evaluate_source_ingestion(
    decision_id: impl Into<String>,
    request: &SourceIngestionRequest,
    policy: &SourceSecurityPolicy,
    least_privilege_scope_refs: Vec<String>,
    receipt_ref: impl Into<String>,
    now: DateTime<Utc>,
) -> Result<SourceSecurityDecision, SourceSecurityError> {
    if request.request_id.trim().is_empty()
        || request.source_ref.trim().is_empty()
        || policy.policy_id.trim().is_empty()
        || policy.version == 0
    {
        return Err(SourceSecurityError::MissingIdentity);
    }
    if request.access_authority_ref.trim().is_empty() || least_privilege_scope_refs.is_empty() {
        return Err(SourceSecurityError::MissingAuthority);
    }
    if matches!(request.access_class, SourceAccessClass::Licensed)
        && request
            .license_terms_ref
            .as_deref()
            .is_none_or(str::is_empty)
    {
        return Err(SourceSecurityError::MissingLicense);
    }
    if request.evidence_refs.is_empty() || policy.evidence_refs.is_empty() {
        return Err(SourceSecurityError::MissingEvidence);
    }
    if request.retention_policy_ref.trim().is_empty()
        || request.retention_policy_ref != policy.deletion_policy_ref
            && request.retention_policy_ref != policy.legal_hold_policy_ref
    {
        return Err(SourceSecurityError::MissingRetention);
    }
    let receipt_ref = receipt_ref.into();
    if receipt_ref.trim().is_empty() || policy.receipt_ref.trim().is_empty() {
        return Err(SourceSecurityError::MissingReceipt);
    }
    if request.manipulation_risk > 100 || request.prompt_injection_risk > 100 {
        return Err(SourceSecurityError::InvalidRisk);
    }
    if policy.sanitization_required && !request.sanitized {
        return Err(SourceSecurityError::SanitizationRequired);
    }
    if policy
        .encryption_required_for
        .contains(&request.privacy_class)
        && !request.encrypted
    {
        return Err(SourceSecurityError::EncryptionRequired);
    }
    if matches!(request.access_class, SourceAccessClass::PublicSummaryOnly)
        && request.raw_content_requested
    {
        return Err(SourceSecurityError::RawSummarySourceProhibited);
    }
    let high_risk = request.manipulation_risk > policy.maximum_manipulation_risk
        || request.prompt_injection_risk > policy.maximum_prompt_injection_risk
        || !request.poisoning_indicator_refs.is_empty();
    let (disposition, reasons) = if matches!(request.access_class, SourceAccessClass::Prohibited) {
        (
            IngestionDisposition::Reject,
            vec!["source_access_prohibited".into()],
        )
    } else if high_risk {
        if request.high_consequence_use {
            (
                IngestionDisposition::Reject,
                vec!["high_consequence_adversarial_source".into()],
            )
        } else {
            (
                IngestionDisposition::Quarantine,
                vec!["source_risk_above_policy".into()],
            )
        }
    } else if matches!(request.access_class, SourceAccessClass::PublicSummaryOnly) {
        (
            IngestionDisposition::PublicSummaryOnly,
            vec!["public_summary_boundary".into()],
        )
    } else {
        (
            IngestionDisposition::Accept,
            vec!["source_controls_satisfied".into()],
        )
    };
    let mut evidence_refs = request.evidence_refs.clone();
    evidence_refs.extend(policy.evidence_refs.clone());
    evidence_refs.sort();
    evidence_refs.dedup();
    Ok(SourceSecurityDecision {
        decision_id: decision_id.into(),
        request_id: request.request_id.clone(),
        disposition,
        reason_codes: reasons,
        least_privilege_scope_refs,
        retention_policy_ref: request.retention_policy_ref.clone(),
        legal_hold_ref: request.legal_hold_ref.clone(),
        evidence_refs,
        receipt_ref,
        decided_at: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn policy() -> SourceSecurityPolicy {
        SourceSecurityPolicy {
            policy_id: "security:v1".into(),
            version: 1,
            maximum_manipulation_risk: 20,
            maximum_prompt_injection_risk: 20,
            encryption_required_for: vec![PrivacyClass::Confidential, PrivacyClass::Restricted],
            sanitization_required: true,
            legal_hold_policy_ref: "retention:hold".into(),
            deletion_policy_ref: "retention:delete".into(),
            audit_export_policy_ref: "audit:export".into(),
            evidence_refs: vec!["evidence:policy".into()],
            receipt_ref: "receipt:policy".into(),
        }
    }
    fn request() -> SourceIngestionRequest {
        SourceIngestionRequest {
            request_id: "request".into(),
            source_ref: "source".into(),
            access_class: SourceAccessClass::Public,
            privacy_class: PrivacyClass::Public,
            access_authority_ref: "authority".into(),
            license_terms_ref: None,
            raw_content_requested: false,
            sanitized: true,
            encrypted: false,
            manipulation_risk: 0,
            prompt_injection_risk: 0,
            poisoning_indicator_refs: vec![],
            high_consequence_use: false,
            retention_policy_ref: "retention:delete".into(),
            legal_hold_ref: None,
            evidence_refs: vec!["evidence:source".into()],
        }
    }
    #[test]
    fn adversarial_sources_quarantine_or_reject_high_consequence_use() {
        let mut risky = request();
        risky.prompt_injection_risk = 90;
        assert_eq!(
            evaluate_source_ingestion(
                "decision",
                &risky,
                &policy(),
                vec!["scope:read".into()],
                "receipt",
                Utc::now()
            )
            .unwrap()
            .disposition,
            IngestionDisposition::Quarantine
        );
        risky.high_consequence_use = true;
        assert_eq!(
            evaluate_source_ingestion(
                "decision",
                &risky,
                &policy(),
                vec!["scope:read".into()],
                "receipt",
                Utc::now()
            )
            .unwrap()
            .disposition,
            IngestionDisposition::Reject
        );
    }
    #[test]
    fn audit_exports_require_encryption_scope_evidence_and_receipt() {
        assert_eq!(
            build_security_audit_export(
                "export",
                &policy(),
                vec!["decision:1".into()],
                vec!["scope:audit".into()],
                false,
                vec!["evidence:audit".into()],
                "receipt:audit",
                Utc::now(),
            ),
            Err(SourceSecurityError::EncryptionRequired)
        );
        assert!(
            build_security_audit_export(
                "export",
                &policy(),
                vec!["decision:1".into()],
                vec!["scope:audit".into()],
                true,
                vec!["evidence:audit".into()],
                "receipt:audit",
                Utc::now(),
            )
            .is_ok()
        );
    }

    #[test]
    fn public_summary_and_private_encryption_boundaries_fail_closed() {
        let mut summary = request();
        summary.access_class = SourceAccessClass::PublicSummaryOnly;
        summary.raw_content_requested = true;
        assert_eq!(
            evaluate_source_ingestion(
                "decision",
                &summary,
                &policy(),
                vec!["scope:read".into()],
                "receipt",
                Utc::now()
            ),
            Err(SourceSecurityError::RawSummarySourceProhibited)
        );
        let mut private = request();
        private.privacy_class = PrivacyClass::Restricted;
        assert_eq!(
            evaluate_source_ingestion(
                "decision",
                &private,
                &policy(),
                vec!["scope:read".into()],
                "receipt",
                Utc::now()
            ),
            Err(SourceSecurityError::EncryptionRequired)
        );
    }
}
