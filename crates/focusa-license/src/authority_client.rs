use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCodeStartRequest {
    pub request_id: Uuid,
    pub product: String,
    pub node_id: String,
    pub requested_features: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCodeChallenge {
    pub request_id: Uuid,
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_at_unix_ms: i64,
    pub interval_ms: u64,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DeviceCodePollResponse {
    AuthorizationPending,
    SlowDown,
    Authorized {
        signed_lease: String,
        refresh_credential: String,
    },
    Denied {
        reason_code: String,
    },
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityNodeSummary {
    pub node_id: String,
    pub label: String,
    pub active: bool,
    pub last_seen_at_unix_ms: Option<i64>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SensitiveCredential(String);

impl SensitiveCredential {
    pub fn new(value: String) -> Result<Self, AuthorityClientError> {
        if value.trim().is_empty() {
            return Err(AuthorityClientError::CredentialMissing);
        }
        Ok(Self(value))
    }

    pub fn expose_for_protected_store(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SensitiveCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SensitiveCredential([REDACTED])")
    }
}

impl std::fmt::Display for SensitiveCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct AuthorizedLeaseMaterial {
    pub signed_lease: String,
    pub refresh_credential: SensitiveCredential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollAction {
    Wait { until_unix_ms: i64 },
    Poll,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceAuthorizationStatus {
    Pending,
    Authorized,
    Denied,
    Expired,
}

#[derive(Clone)]
pub struct DeviceAuthorizationSession {
    challenge: DeviceCodeChallenge,
    status: DeviceAuthorizationStatus,
    next_poll_at_unix_ms: i64,
    poll_count: u32,
    max_polls: u32,
    material: Option<AuthorizedLeaseMaterial>,
    denial_reason: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthorityClientError {
    #[error("authority request identity is incomplete: {0}")]
    MissingIdentity(&'static str),
    #[error("authority origin is not HTTPS or loopback HTTP")]
    UnsafeAuthorityOrigin,
    #[error("device-code challenge does not match the request")]
    RequestMismatch,
    #[error("device-code poll attempted before the server interval")]
    PollTooEarly,
    #[error("device-code challenge expired")]
    ChallengeExpired,
    #[error("device-code poll budget exhausted")]
    PollBudgetExhausted,
    #[error("device authorization is terminal")]
    TerminalSession,
    #[error("authority returned an empty signed lease")]
    LeaseMissing,
    #[error("authority returned an empty refresh credential")]
    CredentialMissing,
    #[error("device authorization denied: {0}")]
    Denied(String),
}

impl DeviceCodeStartRequest {
    pub fn validate(&self) -> Result<(), AuthorityClientError> {
        for (value, field) in [(&self.product, "product"), (&self.node_id, "node_id")] {
            if value.trim().is_empty() {
                return Err(AuthorityClientError::MissingIdentity(field));
            }
        }
        Ok(())
    }
}

impl DeviceAuthorizationSession {
    pub fn new(
        request: &DeviceCodeStartRequest,
        challenge: DeviceCodeChallenge,
        observed_at_unix_ms: i64,
        max_polls: u32,
    ) -> Result<Self, AuthorityClientError> {
        request.validate()?;
        if challenge.request_id != request.request_id {
            return Err(AuthorityClientError::RequestMismatch);
        }
        for (value, field) in [
            (&challenge.device_code, "device_code"),
            (&challenge.user_code, "user_code"),
            (&challenge.verification_uri, "verification_uri"),
        ] {
            if value.trim().is_empty() {
                return Err(AuthorityClientError::MissingIdentity(field));
            }
        }
        validate_authority_origin(&challenge.verification_uri)?;
        if challenge.expires_at_unix_ms <= observed_at_unix_ms {
            return Err(AuthorityClientError::ChallengeExpired);
        }
        if max_polls == 0 {
            return Err(AuthorityClientError::PollBudgetExhausted);
        }
        let interval_ms = challenge.interval_ms.clamp(1_000, 60_000);
        Ok(Self {
            challenge,
            status: DeviceAuthorizationStatus::Pending,
            next_poll_at_unix_ms: observed_at_unix_ms + interval_ms as i64,
            poll_count: 0,
            max_polls,
            material: None,
            denial_reason: None,
        })
    }

    pub fn poll_action(
        &self,
        observed_at_unix_ms: i64,
    ) -> Result<PollAction, AuthorityClientError> {
        if self.status != DeviceAuthorizationStatus::Pending {
            return Ok(PollAction::Terminal);
        }
        if observed_at_unix_ms >= self.challenge.expires_at_unix_ms {
            return Ok(PollAction::Terminal);
        }
        if self.poll_count >= self.max_polls {
            return Err(AuthorityClientError::PollBudgetExhausted);
        }
        if observed_at_unix_ms < self.next_poll_at_unix_ms {
            return Ok(PollAction::Wait {
                until_unix_ms: self.next_poll_at_unix_ms,
            });
        }
        Ok(PollAction::Poll)
    }

    pub fn observe_poll(
        &mut self,
        response: DeviceCodePollResponse,
        observed_at_unix_ms: i64,
    ) -> Result<DeviceAuthorizationStatus, AuthorityClientError> {
        if self.status != DeviceAuthorizationStatus::Pending {
            return Err(AuthorityClientError::TerminalSession);
        }
        if observed_at_unix_ms < self.next_poll_at_unix_ms {
            return Err(AuthorityClientError::PollTooEarly);
        }
        if observed_at_unix_ms >= self.challenge.expires_at_unix_ms {
            self.status = DeviceAuthorizationStatus::Expired;
            return Err(AuthorityClientError::ChallengeExpired);
        }
        self.poll_count += 1;
        if self.poll_count > self.max_polls {
            return Err(AuthorityClientError::PollBudgetExhausted);
        }
        let base_interval = self.challenge.interval_ms.clamp(1_000, 60_000) as i64;
        match response {
            DeviceCodePollResponse::AuthorizationPending => {
                self.next_poll_at_unix_ms = observed_at_unix_ms + base_interval;
            }
            DeviceCodePollResponse::SlowDown => {
                self.next_poll_at_unix_ms = observed_at_unix_ms + base_interval + 5_000;
            }
            DeviceCodePollResponse::Authorized {
                signed_lease,
                refresh_credential,
            } => {
                if signed_lease.trim().is_empty() {
                    return Err(AuthorityClientError::LeaseMissing);
                }
                self.material = Some(AuthorizedLeaseMaterial {
                    signed_lease,
                    refresh_credential: SensitiveCredential::new(refresh_credential)?,
                });
                self.status = DeviceAuthorizationStatus::Authorized;
            }
            DeviceCodePollResponse::Denied { reason_code } => {
                self.denial_reason = Some(reason_code.clone());
                self.status = DeviceAuthorizationStatus::Denied;
                return Err(AuthorityClientError::Denied(reason_code));
            }
            DeviceCodePollResponse::Expired => {
                self.status = DeviceAuthorizationStatus::Expired;
                return Err(AuthorityClientError::ChallengeExpired);
            }
        }
        Ok(self.status)
    }

    pub fn material(&self) -> Option<&AuthorizedLeaseMaterial> {
        self.material.as_ref()
    }

    pub fn status(&self) -> DeviceAuthorizationStatus {
        self.status
    }
}

pub fn validate_authority_origin(value: &str) -> Result<(), AuthorityClientError> {
    let normalized = value.trim().to_ascii_lowercase();
    let safe = normalized.starts_with("https://")
        || normalized.starts_with("http://127.0.0.1:")
        || normalized.starts_with("http://localhost:")
        || normalized.starts_with("http://[::1]:");
    if !safe || normalized.contains('@') {
        return Err(AuthorityClientError::UnsafeAuthorityOrigin);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> DeviceCodeStartRequest {
        DeviceCodeStartRequest {
            request_id: Uuid::now_v7(),
            product: "focusa".into(),
            node_id: "node-1".into(),
            requested_features: vec!["core".into()],
        }
    }

    fn challenge(request: &DeviceCodeStartRequest) -> DeviceCodeChallenge {
        DeviceCodeChallenge {
            request_id: request.request_id,
            device_code: "device-secret".into(),
            user_code: "FOCUSA-1234".into(),
            verification_uri: "https://license.example.test/device".into(),
            expires_at_unix_ms: 100_000,
            interval_ms: 2_000,
        }
    }

    #[test]
    fn polling_is_bounded_and_slow_down_is_honored() {
        let request = request();
        let mut session =
            DeviceAuthorizationSession::new(&request, challenge(&request), 1_000, 3).unwrap();
        assert_eq!(
            session.poll_action(2_000).unwrap(),
            PollAction::Wait {
                until_unix_ms: 3_000
            }
        );
        session
            .observe_poll(DeviceCodePollResponse::SlowDown, 3_000)
            .unwrap();
        assert_eq!(
            session.poll_action(9_000).unwrap(),
            PollAction::Wait {
                until_unix_ms: 10_000
            }
        );
        session
            .observe_poll(
                DeviceCodePollResponse::Authorized {
                    signed_lease: "signed-envelope".into(),
                    refresh_credential: "refresh-secret".into(),
                },
                10_000,
            )
            .unwrap();
        assert_eq!(session.status(), DeviceAuthorizationStatus::Authorized);
        assert_eq!(
            session.material().unwrap().refresh_credential.to_string(),
            "[REDACTED]"
        );
        assert!(
            !format!("{:?}", session.material().unwrap().refresh_credential)
                .contains("refresh-secret")
        );
    }

    #[test]
    fn synthetic_authority_denials_and_products_are_exact() {
        for reason in ["wrong_product", "revoked", "refund", "node_limit"] {
            let request = request();
            let mut session =
                DeviceAuthorizationSession::new(&request, challenge(&request), 1_000, 3).unwrap();
            assert!(matches!(
                session.observe_poll(
                    DeviceCodePollResponse::Denied {
                        reason_code: reason.into()
                    },
                    3_000
                ),
                Err(AuthorityClientError::Denied(actual)) if actual == reason
            ));
        }
        for lease_kind in ["evaluation", "paid", "bundle"] {
            let request = request();
            let mut session =
                DeviceAuthorizationSession::new(&request, challenge(&request), 1_000, 3).unwrap();
            session
                .observe_poll(
                    DeviceCodePollResponse::Authorized {
                        signed_lease: format!("signed-{lease_kind}"),
                        refresh_credential: format!("refresh-{lease_kind}"),
                    },
                    3_000,
                )
                .unwrap();
            assert_eq!(
                session.material().unwrap().signed_lease,
                format!("signed-{lease_kind}")
            );
        }
    }

    #[test]
    fn wrong_request_origin_expiry_and_denial_fail_closed() {
        let request = request();
        let mut wrong = challenge(&request);
        wrong.request_id = Uuid::now_v7();
        assert!(matches!(
            DeviceAuthorizationSession::new(&request, wrong, 1_000, 3),
            Err(AuthorityClientError::RequestMismatch)
        ));
        let mut unsafe_origin = challenge(&request);
        unsafe_origin.verification_uri = "http://authority.example.test/device".into();
        assert!(matches!(
            DeviceAuthorizationSession::new(&request, unsafe_origin, 1_000, 3),
            Err(AuthorityClientError::UnsafeAuthorityOrigin)
        ));
        let mut session =
            DeviceAuthorizationSession::new(&request, challenge(&request), 1_000, 3).unwrap();
        assert!(matches!(
            session.observe_poll(
                DeviceCodePollResponse::Denied {
                    reason_code: "node_limit".into()
                },
                3_000
            ),
            Err(AuthorityClientError::Denied(reason)) if reason == "node_limit"
        ));
    }
}
