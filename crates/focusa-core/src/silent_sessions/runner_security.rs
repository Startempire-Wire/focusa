use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{
    ApprovalId, AuthorizationDecision, ControlAuditId, RunnerCommandId, SilentSessionAction,
    SilentSessionId, SilentSessionRunId,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunnerIdentity {
    pub principal_id: String,
    pub os_user: String,
    pub socket_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunnerCommandClaims {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub runner: RunnerIdentity,
    pub action_digest: String,
    pub nonce: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthenticatedRunnerCommand {
    pub command_id: RunnerCommandId,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub runner_principal_id: String,
    pub runner_os_user: String,
    pub socket_scope: String,
    pub action_digest: String,
    pub payload_hash: String,
    pub nonce: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub auth_tag: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RunnerAuthenticationError {
    #[error("runner command identity or socket scope mismatch")]
    IdentityMismatch,
    #[error("runner command is not currently valid")]
    Expired,
    #[error("runner command nonce was already consumed")]
    Replay,
    #[error("runner command authentication tag mismatch")]
    InvalidTag,
    #[error("runner command payload hash does not match")]
    PayloadMismatch,
    #[error("runner command key must not be empty")]
    EmptyKey,
    #[error("runner command signing payload serialization failed")]
    Serialization,
}

impl AuthenticatedRunnerCommand {
    pub fn issue(
        claims: RunnerCommandClaims,
        payload: &[u8],
        key: &[u8],
    ) -> Result<Self, RunnerAuthenticationError> {
        if key.is_empty() {
            return Err(RunnerAuthenticationError::EmptyKey);
        }
        let mut command = Self {
            command_id: RunnerCommandId::new(),
            session_id: claims.session_id,
            run_id: claims.run_id,
            runner_principal_id: claims.runner.principal_id,
            runner_os_user: claims.runner.os_user,
            socket_scope: claims.runner.socket_scope,
            action_digest: claims.action_digest,
            payload_hash: hex::encode(Sha256::digest(payload)),
            nonce: claims.nonce,
            issued_at: claims.issued_at,
            expires_at: claims.expires_at,
            auth_tag: String::new(),
        };
        command.auth_tag = hmac_sha256_hex(key, &command.signing_bytes()?);
        Ok(command)
    }

    pub fn authenticate_payload(
        &self,
        runner: &RunnerIdentity,
        now: DateTime<Utc>,
        key: &[u8],
        consumed_nonces: &mut BTreeSet<String>,
        payload: &[u8],
    ) -> Result<(), RunnerAuthenticationError> {
        if self.payload_hash != hex::encode(Sha256::digest(payload)) {
            return Err(RunnerAuthenticationError::PayloadMismatch);
        }
        self.authenticate(runner, now, key, consumed_nonces)
    }

    pub fn authenticate(
        &self,
        runner: &RunnerIdentity,
        now: DateTime<Utc>,
        key: &[u8],
        consumed_nonces: &mut BTreeSet<String>,
    ) -> Result<(), RunnerAuthenticationError> {
        if key.is_empty() {
            return Err(RunnerAuthenticationError::EmptyKey);
        }
        if self.runner_principal_id != runner.principal_id
            || self.runner_os_user != runner.os_user
            || self.socket_scope != runner.socket_scope
        {
            return Err(RunnerAuthenticationError::IdentityMismatch);
        }
        if now < self.issued_at || now > self.expires_at {
            return Err(RunnerAuthenticationError::Expired);
        }
        if consumed_nonces.contains(&self.nonce) {
            return Err(RunnerAuthenticationError::Replay);
        }
        let expected = hmac_sha256_hex(key, &self.signing_bytes()?);
        if !constant_time_eq(expected.as_bytes(), self.auth_tag.as_bytes()) {
            return Err(RunnerAuthenticationError::InvalidTag);
        }
        consumed_nonces.insert(self.nonce.clone());
        Ok(())
    }

    fn signing_bytes(&self) -> Result<Vec<u8>, RunnerAuthenticationError> {
        serde_json::to_vec(&(
            self.command_id,
            self.session_id,
            self.run_id,
            &self.runner_principal_id,
            &self.runner_os_user,
            &self.socket_scope,
            &self.action_digest,
            &self.payload_hash,
            &self.nonce,
            self.issued_at,
            self.expires_at,
        ))
        .map_err(|_| RunnerAuthenticationError::Serialization)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlAuditInput {
    pub audit_id: ControlAuditId,
    pub occurred_at: DateTime<Utc>,
    pub actor: String,
    pub action: SilentSessionAction,
    pub project_root: String,
    pub continuity_id: String,
    pub session_id: Option<SilentSessionId>,
    pub run_id: Option<SilentSessionRunId>,
    pub approval_id: Option<ApprovalId>,
    pub decision: AuthorizationDecision,
    pub details: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedactedControlAuditRecord {
    pub audit_id: ControlAuditId,
    pub occurred_at: DateTime<Utc>,
    pub actor: String,
    pub action: SilentSessionAction,
    pub project_root: String,
    pub continuity_id: String,
    pub session_id: Option<SilentSessionId>,
    pub run_id: Option<SilentSessionRunId>,
    pub approval_id: Option<ApprovalId>,
    pub decision: AuthorizationDecision,
    pub redacted_details: Value,
    pub redaction_classes: Vec<String>,
}

pub fn redact_control_audit(input: ControlAuditInput) -> RedactedControlAuditRecord {
    let mut classes = BTreeSet::new();
    let redacted_details = redact_value(None, input.details, &mut classes);
    RedactedControlAuditRecord {
        audit_id: input.audit_id,
        occurred_at: input.occurred_at,
        actor: input.actor,
        action: input.action,
        project_root: input.project_root,
        continuity_id: input.continuity_id,
        session_id: input.session_id,
        run_id: input.run_id,
        approval_id: input.approval_id,
        decision: input.decision,
        redacted_details,
        redaction_classes: classes.into_iter().collect(),
    }
}

fn redact_value(key: Option<&str>, value: Value, classes: &mut BTreeSet<String>) -> Value {
    if let Some(key) = key {
        if !key.to_ascii_lowercase().ends_with("_ref") {
            if let Some(class) = redaction_class(key) {
                classes.insert(class.into());
                return Value::String(format!("<redacted:{class}>"));
            }
        }
    }
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    let redacted = redact_value(Some(&key), value, classes);
                    (key, redacted)
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(|value| redact_value(None, value, classes))
                .collect(),
        ),
        Value::String(value) if value.contains("BEGIN PRIVATE KEY") => {
            classes.insert("private_key_material".into());
            Value::String("<redacted:private_key_material>".into())
        }
        other => other,
    }
}

fn redaction_class(key: &str) -> Option<&'static str> {
    let normalized = key.to_ascii_lowercase().replace('-', "_");
    if normalized == "authorization" || normalized == "auth_header" {
        Some("auth_header")
    } else if normalized.contains("bearer")
        || normalized == "token"
        || normalized.ends_with("_token")
    {
        Some("bearer_token")
    } else if normalized.contains("provider_credential") || normalized.contains("api_key") {
        Some("provider_credential")
    } else if normalized.contains("private_key") {
        Some("private_key_material")
    } else if normalized.contains("secret") || normalized.contains("password") {
        Some("secret_value")
    } else {
        None
    }
}

fn hmac_sha256_hex(key: &[u8], message: &[u8]) -> String {
    const BLOCK: usize = 64;
    let mut key_block = [0_u8; BLOCK];
    let material = if key.len() > BLOCK {
        Sha256::digest(key).to_vec()
    } else {
        key.to_vec()
    };
    key_block[..material.len()].copy_from_slice(&material);
    let mut inner_pad = [0x36_u8; BLOCK];
    let mut outer_pad = [0x5c_u8; BLOCK];
    for index in 0..BLOCK {
        inner_pad[index] ^= key_block[index];
        outer_pad[index] ^= key_block[index];
    }
    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(message);
    let inner_hash = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner_hash);
    hex::encode(outer.finalize())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0_u8;
    for (left, right) in left.iter().zip(right) {
        difference |= left ^ right;
    }
    difference == 0
}
