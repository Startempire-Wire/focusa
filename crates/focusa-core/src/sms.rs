//! Transport-neutral SMS/communications broker contracts (Plan 180).
//!
//! The connector owns provider/browser state. Focusa owns capability grants,
//! scopes, challenge handles, and value-free audit. OTP values and connector
//! credentials are intentionally absent from every public type in this module.

use serde::{Deserialize, Serialize};

pub const SMS_GRANT_SCHEMA: &str = "focusa.sms_grant.v1";
pub const SMS_CHALLENGE_SCHEMA: &str = "focusa.sms_otp_challenge.v1";
pub const SMS_HEALTH_SCHEMA: &str = "focusa.sms_health.v1";
pub const SMS_AUDIT_SCHEMA: &str = "focusa.sms_audit.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SmsCapability {
    Health,
    Checkpoint,
    Revoke,
    ReadOtp,
    InjectOtp,
    ListThreads,
    ReadThread,
    Search,
    Send,
    Events,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SmsScope {
    pub connector_id: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub challenge_handle: Option<String>,
    #[serde(default)]
    pub target_handle: Option<String>,
    #[serde(default)]
    pub thread_handles: Vec<String>,
    #[serde(default)]
    pub recipient_handles: Vec<String>,
    #[serde(default)]
    pub not_before: Option<String>,
    #[serde(default)]
    pub not_after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmsGrant {
    pub schema: String,
    pub grant_id: String,
    pub consumer_ref: String,
    pub capabilities: Vec<SmsCapability>,
    pub scope: SmsScope,
    pub granted_at: String,
    pub expires_at: String,
    pub use_count_allowed: u32,
    pub use_count_used: u32,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmsOtpChallenge {
    pub schema: String,
    pub challenge_handle: String,
    pub provider: String,
    pub connector_id: String,
    pub target_handle: String,
    pub consumer_ref: String,
    pub requested_at: String,
    pub expires_at: String,
    pub baseline_handle: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmsConnectorHealth {
    pub schema: String,
    pub connector_id: String,
    pub connector_kind: String,
    pub status: String,
    pub checkpoint_generation: u64,
    pub checkpoint_status: String,
    pub restored_at: Option<String>,
    pub last_probe_at: String,
    pub capabilities: Vec<SmsCapability>,
    #[serde(default)]
    pub failure_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmsThreadSummary {
    pub thread_handle: String,
    pub display_name: Option<String>,
    pub participant_handles: Vec<String>,
    pub unread_count: u64,
    pub last_message_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmsMessage {
    pub message_handle: String,
    pub thread_handle: String,
    pub direction: String,
    pub sender_handle: Option<String>,
    pub recipient_handles: Vec<String>,
    pub body: String,
    pub sent_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmsSendRequest {
    pub recipient_handles: Vec<String>,
    pub body: String,
    pub idempotency_key: String,
    pub grant_id: String,
    pub consumer_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmsAuditRecord {
    pub schema: String,
    pub audit_id: String,
    pub action: String,
    pub consumer_ref: String,
    pub grant_id: String,
    pub connector_id: String,
    pub target_handle: Option<String>,
    pub occurred_at: String,
    pub status: String,
    pub failure_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmsAuthorizationVerdict {
    pub allowed: bool,
    pub reasons: Vec<String>,
}

/// Fail-closed capability check shared by every adapter.
pub fn authorize_sms_action(
    grant: &SmsGrant,
    capability: SmsCapability,
    consumer_ref: &str,
    connector_id: &str,
    now: &str,
) -> SmsAuthorizationVerdict {
    let mut reasons = Vec::new();
    if grant.schema != SMS_GRANT_SCHEMA {
        reasons.push("unsupported grant schema".into());
    }
    if grant.status != "active" {
        reasons.push("grant is not active".into());
    }
    if grant.consumer_ref != consumer_ref {
        reasons.push("consumer mismatch".into());
    }
    if grant.scope.connector_id != connector_id {
        reasons.push("connector mismatch".into());
    }
    if !grant.capabilities.contains(&capability) {
        reasons.push("capability missing".into());
    }
    if grant.expires_at.as_str() <= now {
        reasons.push("grant expired".into());
    }
    if grant.use_count_used >= grant.use_count_allowed {
        reasons.push("grant exhausted".into());
    }
    SmsAuthorizationVerdict {
        allowed: reasons.is_empty(),
        reasons,
    }
}

/// OTP injection requires exact provider/challenge/target binding in addition
/// to the ordinary capability verdict. An OTP grant never implies thread/send.
pub fn authorize_otp_injection(
    grant: &SmsGrant,
    challenge: &SmsOtpChallenge,
    consumer_ref: &str,
    now: &str,
) -> SmsAuthorizationVerdict {
    let mut verdict = authorize_sms_action(
        grant,
        SmsCapability::InjectOtp,
        consumer_ref,
        &challenge.connector_id,
        now,
    );
    if grant.scope.provider.as_deref() != Some(challenge.provider.as_str()) {
        verdict.reasons.push("provider mismatch".into());
    }
    if grant.scope.challenge_handle.as_deref() != Some(challenge.challenge_handle.as_str()) {
        verdict.reasons.push("challenge mismatch".into());
    }
    if grant.scope.target_handle.as_deref() != Some(challenge.target_handle.as_str()) {
        verdict.reasons.push("target mismatch".into());
    }
    if challenge.consumer_ref != consumer_ref || challenge.status != "waiting" {
        verdict.reasons.push("challenge is not eligible".into());
    }
    if challenge.expires_at.as_str() <= now {
        verdict.reasons.push("challenge expired".into());
    }
    verdict.allowed = verdict.reasons.is_empty();
    verdict
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grant(capability: SmsCapability) -> SmsGrant {
        SmsGrant {
            schema: SMS_GRANT_SCHEMA.into(),
            grant_id: "grant-1".into(),
            consumer_ref: "agent-1".into(),
            capabilities: vec![capability],
            scope: SmsScope {
                connector_id: "sms-1".into(),
                provider: Some("github.com".into()),
                challenge_handle: Some("challenge-1".into()),
                target_handle: Some("target-1".into()),
                ..Default::default()
            },
            granted_at: "2026-08-29T00:00:00Z".into(),
            expires_at: "2026-08-29T01:00:00Z".into(),
            use_count_allowed: 1,
            use_count_used: 0,
            status: "active".into(),
        }
    }

    #[test]
    fn otp_grant_never_implies_message_access() {
        let grant = grant(SmsCapability::InjectOtp);
        assert!(
            authorize_sms_action(
                &grant,
                SmsCapability::InjectOtp,
                "agent-1",
                "sms-1",
                "2026-08-29T00:30:00Z"
            )
            .allowed
        );
        assert!(
            !authorize_sms_action(
                &grant,
                SmsCapability::ReadThread,
                "agent-1",
                "sms-1",
                "2026-08-29T00:30:00Z"
            )
            .allowed
        );
        assert!(
            !authorize_sms_action(
                &grant,
                SmsCapability::Send,
                "agent-1",
                "sms-1",
                "2026-08-29T00:30:00Z"
            )
            .allowed
        );
    }

    #[test]
    fn otp_injection_is_exactly_bound() {
        let grant = grant(SmsCapability::InjectOtp);
        let mut challenge = SmsOtpChallenge {
            schema: SMS_CHALLENGE_SCHEMA.into(),
            challenge_handle: "challenge-1".into(),
            provider: "github.com".into(),
            connector_id: "sms-1".into(),
            target_handle: "target-1".into(),
            consumer_ref: "agent-1".into(),
            requested_at: "2026-08-29T00:00:00Z".into(),
            expires_at: "2026-08-29T00:35:00Z".into(),
            baseline_handle: "baseline-redacted".into(),
            status: "waiting".into(),
        };
        assert!(
            authorize_otp_injection(&grant, &challenge, "agent-1", "2026-08-29T00:30:00Z").allowed
        );
        challenge.target_handle = "wrong".into();
        assert!(
            !authorize_otp_injection(&grant, &challenge, "agent-1", "2026-08-29T00:30:00Z").allowed
        );
    }
}
