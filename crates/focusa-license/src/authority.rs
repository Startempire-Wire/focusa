//! Canonical Spec 152 authority-lease verification.
//!
//! The authority owns policy and signs exact canonical payload bytes. Focusa
//! only verifies those bytes and projects a bounded entitlement snapshot.

use std::collections::BTreeMap;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use thiserror::Error;

const LEASE_DOMAIN: &[u8] = b"FOCUSA-AUTHORITY-LEASE-V1\0";
const KEY_SET_DOMAIN: &[u8] = b"FOCUSA-AUTHORITY-KEY-SET-V1\0";
pub const LEASE_SCHEMA: &str = "focusa.authority_lease.v1";
pub const KEY_SET_SCHEMA: &str = "focusa.authority_key_set.v1";
pub const ENVELOPE_SCHEMA: &str = "focusa.signed_envelope.v1";


fn node_ids_equivalent(received: &str, expected: &str) -> bool {
    fn normalize(value: &str) -> Option<&str> {
        let normalized = value.strip_prefix("node-").unwrap_or(value);
        uuid::Uuid::parse_str(normalized).ok().map(|_| normalized)
    }
    match (normalize(received), normalize(expected)) {
        (Some(received), Some(expected)) => received == expected,
        _ => received == expected,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityKeyStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityPublicKey {
    pub key_id: String,
    pub public_key_b64: String,
    pub status: AuthorityKeyStatus,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityKeySet {
    pub schema: String,
    pub sequence: u64,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub keys: Vec<AuthorityPublicKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedEnvelope {
    pub schema: String,
    pub signer_key_id: String,
    /// Base64 of the exact canonical JSON bytes signed by the authority.
    pub payload_b64: String,
    pub signature_b64: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityLeaseStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityLeasePayload {
    pub schema: String,
    pub lease_id: String,
    pub product: String,
    pub subject_id: String,
    pub node_id: String,
    pub sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_lease_digest: Option<String>,
    pub issued_at: DateTime<Utc>,
    pub not_before: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offline_grace_until: Option<DateTime<Utc>>,
    pub authority_key_id: String,
    pub status: AuthorityLeaseStatus,
    #[serde(default)]
    pub features: BTreeMap<String, bool>,
    #[serde(default)]
    pub limits: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitlementState {
    Unactivated,
    Active,
    OfflineGrace,
    RecoveryOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitlementSnapshot {
    pub state: EntitlementState,
    pub product: String,
    pub node_id: String,
    /// Account UUID the signed lease was issued to (Spec 152E §7.1 / §15
    /// lease `subject_id`). Same-account UIAI activation routes the Focusa
    /// parent and the independent UIAI grant through one EDD account; a
    /// verified lease always carries it, synthetic snapshots may omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<String>,
    pub lease_id: Option<String>,
    pub sequence: Option<u64>,
    pub lease_digest: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub offline_grace_until: Option<DateTime<Utc>>,
    pub features: BTreeMap<String, bool>,
    pub limits: BTreeMap<String, u64>,
    pub recovery_reason: Option<String>,
}

impl EntitlementSnapshot {
    pub fn unactivated(product: impl Into<String>, node_id: impl Into<String>) -> Self {
        Self {
            state: EntitlementState::Unactivated,
            product: product.into(),
            node_id: node_id.into(),
            subject_id: None,
            lease_id: None,
            sequence: None,
            lease_digest: None,
            expires_at: None,
            offline_grace_until: None,
            features: BTreeMap::new(),
            limits: BTreeMap::new(),
            recovery_reason: Some("authority_lease_missing".to_string()),
        }
    }

    pub fn recovery_only(
        product: impl Into<String>,
        node_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            state: EntitlementState::RecoveryOnly,
            recovery_reason: Some(reason.into()),
            ..Self::unactivated(product, node_id)
        }
    }

    pub fn feature_enabled(&self, feature: &str) -> bool {
        matches!(
            self.state,
            EntitlementState::Active | EntitlementState::OfflineGrace
        ) && self.features.get(feature).copied().unwrap_or(false)
    }

    pub fn limit(&self, limit: &str) -> Option<u64> {
        matches!(
            self.state,
            EntitlementState::Active | EntitlementState::OfflineGrace
        )
        .then(|| self.limits.get(limit).copied())
        .flatten()
    }
}

#[derive(Debug, Clone)]
pub struct LeaseVerificationContext {
    pub expected_product: String,
    pub expected_node_id: String,
    pub now: DateTime<Utc>,
    pub minimum_sequence: Option<u64>,
    pub expected_previous_digest: Option<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuthorityVerificationError {
    #[error("unsupported envelope schema: {0}")]
    UnsupportedEnvelopeSchema(String),
    #[error("unsupported payload schema: {0}")]
    UnsupportedPayloadSchema(String),
    #[error("invalid base64 in {0}")]
    InvalidBase64(&'static str),
    #[error("invalid JSON payload")]
    InvalidPayload,
    #[error("unknown signing key: {0}")]
    UnknownKey(String),
    #[error("signing key is revoked: {0}")]
    RevokedKey(String),
    #[error("signing key is outside its validity window: {0}")]
    KeyOutsideValidity(String),
    #[error("signature verification failed")]
    InvalidSignature,
    #[error("authority key id does not match envelope signer")]
    AuthorityKeyMismatch,
    #[error("wrong product: expected {expected}, received {actual}")]
    WrongProduct { expected: String, actual: String },
    #[error("wrong node: expected {expected}, received {actual}")]
    WrongNode { expected: String, actual: String },
    #[error("lease sequence is stale: minimum {minimum}, received {actual}")]
    StaleSequence { minimum: u64, actual: u64 },
    #[error("previous lease digest mismatch")]
    PreviousDigestMismatch,
    #[error("lease is not yet valid")]
    NotYetValid,
    #[error("lease is expired")]
    Expired,
    #[error("lease is revoked")]
    RevokedLease,
    #[error("authority key set is expired")]
    ExpiredKeySet,
    #[error("authority key set has no usable keys")]
    EmptyKeySet,
}

#[derive(Debug, Clone)]
pub struct AuthorityLeaseVerifier {
    key_set: AuthorityKeySet,
}

impl AuthorityLeaseVerifier {
    pub fn from_signed_key_set(
        envelope: &SignedEnvelope,
        trusted_roots: &BTreeMap<String, VerifyingKey>,
        now: DateTime<Utc>,
        minimum_sequence: Option<u64>,
    ) -> Result<Self, AuthorityVerificationError> {
        let root = trusted_roots.get(&envelope.signer_key_id).ok_or_else(|| {
            AuthorityVerificationError::UnknownKey(envelope.signer_key_id.clone())
        })?;
        let (payload_bytes, key_set) =
            verify_and_parse::<AuthorityKeySet>(envelope, root, KEY_SET_DOMAIN, KEY_SET_SCHEMA)?;
        let _ = payload_bytes;
        if key_set.expires_at < now {
            return Err(AuthorityVerificationError::ExpiredKeySet);
        }
        if minimum_sequence.is_some_and(|minimum| key_set.sequence < minimum) {
            return Err(AuthorityVerificationError::StaleSequence {
                minimum: minimum_sequence.unwrap_or_default(),
                actual: key_set.sequence,
            });
        }
        if !key_set
            .keys
            .iter()
            .any(|key| key.status == AuthorityKeyStatus::Active)
        {
            return Err(AuthorityVerificationError::EmptyKeySet);
        }
        Ok(Self { key_set })
    }

    pub fn verify_lease(
        &self,
        envelope: &SignedEnvelope,
        context: &LeaseVerificationContext,
    ) -> Result<EntitlementSnapshot, AuthorityVerificationError> {
        if envelope.schema != ENVELOPE_SCHEMA {
            return Err(AuthorityVerificationError::UnsupportedEnvelopeSchema(
                envelope.schema.clone(),
            ));
        }
        let key = self
            .key_set
            .keys
            .iter()
            .find(|candidate| candidate.key_id == envelope.signer_key_id)
            .ok_or_else(|| {
                AuthorityVerificationError::UnknownKey(envelope.signer_key_id.clone())
            })?;
        if key.status == AuthorityKeyStatus::Revoked {
            return Err(AuthorityVerificationError::RevokedKey(key.key_id.clone()));
        }
        if context.now < key.not_before || context.now > key.not_after {
            return Err(AuthorityVerificationError::KeyOutsideValidity(
                key.key_id.clone(),
            ));
        }
        let verifying_key = decode_verifying_key(&key.public_key_b64)?;
        let (payload_bytes, payload) = verify_and_parse::<AuthorityLeasePayload>(
            envelope,
            &verifying_key,
            LEASE_DOMAIN,
            LEASE_SCHEMA,
        )?;
        if payload.authority_key_id != envelope.signer_key_id {
            return Err(AuthorityVerificationError::AuthorityKeyMismatch);
        }
        if payload.product != context.expected_product {
            return Err(AuthorityVerificationError::WrongProduct {
                expected: context.expected_product.clone(),
                actual: payload.product,
            });
        }
        // `node-<uuid>` is a legacy serialization of the same UUID. Accept
        // equivalence only when both values reduce to a valid UUID; all other
        // node bindings remain exact and fail closed.
        if !node_ids_equivalent(&payload.node_id, &context.expected_node_id) {
            return Err(AuthorityVerificationError::WrongNode {
                expected: context.expected_node_id.clone(),
                actual: payload.node_id,
            });
        }
        if let Some(minimum) = context.minimum_sequence {
            if payload.sequence < minimum {
                return Err(AuthorityVerificationError::StaleSequence {
                    minimum,
                    actual: payload.sequence,
                });
            }
        }
        if let Some(expected) = &context.expected_previous_digest {
            if payload.previous_lease_digest.as_ref() != Some(expected) {
                return Err(AuthorityVerificationError::PreviousDigestMismatch);
            }
        }
        if payload.status == AuthorityLeaseStatus::Revoked {
            return Err(AuthorityVerificationError::RevokedLease);
        }
        if context.now < payload.not_before {
            return Err(AuthorityVerificationError::NotYetValid);
        }
        let state = if context.now <= payload.expires_at {
            EntitlementState::Active
        } else if payload
            .offline_grace_until
            .is_some_and(|grace_until| context.now <= grace_until)
        {
            EntitlementState::OfflineGrace
        } else {
            return Err(AuthorityVerificationError::Expired);
        };
        let lease_digest = format!("sha256:{:x}", Sha256::digest(&payload_bytes));
        Ok(EntitlementSnapshot {
            state,
            product: payload.product,
            node_id: payload.node_id,
            subject_id: Some(payload.subject_id),
            lease_id: Some(payload.lease_id),
            sequence: Some(payload.sequence),
            lease_digest: Some(lease_digest),
            expires_at: Some(payload.expires_at),
            offline_grace_until: payload.offline_grace_until,
            features: payload.features,
            limits: payload.limits,
            recovery_reason: None,
        })
    }

    pub fn resolve(
        &self,
        envelope: Option<&SignedEnvelope>,
        context: &LeaseVerificationContext,
    ) -> EntitlementSnapshot {
        let Some(envelope) = envelope else {
            return EntitlementSnapshot::unactivated(
                &context.expected_product,
                &context.expected_node_id,
            );
        };
        self.verify_lease(envelope, context)
            .unwrap_or_else(|error| {
                EntitlementSnapshot::recovery_only(
                    &context.expected_product,
                    &context.expected_node_id,
                    error_code(&error),
                )
            })
    }
}

fn verify_and_parse<T: DeserializeOwned>(
    envelope: &SignedEnvelope,
    key: &VerifyingKey,
    domain: &[u8],
    expected_schema: &str,
) -> Result<(Vec<u8>, T), AuthorityVerificationError> {
    if envelope.schema != ENVELOPE_SCHEMA {
        return Err(AuthorityVerificationError::UnsupportedEnvelopeSchema(
            envelope.schema.clone(),
        ));
    }
    let payload = BASE64
        .decode(&envelope.payload_b64)
        .map_err(|_| AuthorityVerificationError::InvalidBase64("payload"))?;
    let signature_bytes = BASE64
        .decode(&envelope.signature_b64)
        .map_err(|_| AuthorityVerificationError::InvalidBase64("signature"))?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| AuthorityVerificationError::InvalidSignature)?;
    let signed = [domain, payload.as_slice()].concat();
    key.verify(&signed, &signature)
        .map_err(|_| AuthorityVerificationError::InvalidSignature)?;
    let value: serde_json::Value =
        serde_json::from_slice(&payload).map_err(|_| AuthorityVerificationError::InvalidPayload)?;
    let schema = value
        .get("schema")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    if schema != expected_schema {
        return Err(AuthorityVerificationError::UnsupportedPayloadSchema(
            schema.to_string(),
        ));
    }
    let typed =
        serde_json::from_value(value).map_err(|_| AuthorityVerificationError::InvalidPayload)?;
    Ok((payload, typed))
}

fn decode_verifying_key(encoded: &str) -> Result<VerifyingKey, AuthorityVerificationError> {
    let bytes = BASE64
        .decode(encoded)
        .map_err(|_| AuthorityVerificationError::InvalidBase64("public_key"))?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| AuthorityVerificationError::InvalidPayload)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| AuthorityVerificationError::InvalidPayload)
}

fn error_code(error: &AuthorityVerificationError) -> &'static str {
    match error {
        AuthorityVerificationError::UnsupportedEnvelopeSchema(_) => "unsupported_envelope_schema",
        AuthorityVerificationError::UnsupportedPayloadSchema(_) => "unsupported_payload_schema",
        AuthorityVerificationError::InvalidBase64(_) => "invalid_base64",
        AuthorityVerificationError::InvalidPayload => "invalid_payload",
        AuthorityVerificationError::UnknownKey(_) => "unknown_key",
        AuthorityVerificationError::RevokedKey(_) => "revoked_key",
        AuthorityVerificationError::KeyOutsideValidity(_) => "key_outside_validity",
        AuthorityVerificationError::InvalidSignature => "invalid_signature",
        AuthorityVerificationError::AuthorityKeyMismatch => "authority_key_mismatch",
        AuthorityVerificationError::WrongProduct { .. } => "wrong_product",
        AuthorityVerificationError::WrongNode { .. } => "wrong_node",
        AuthorityVerificationError::StaleSequence { .. } => "stale_sequence",
        AuthorityVerificationError::PreviousDigestMismatch => "previous_digest_mismatch",
        AuthorityVerificationError::NotYetValid => "not_yet_valid",
        AuthorityVerificationError::Expired => "expired",
        AuthorityVerificationError::RevokedLease => "revoked_lease",
        AuthorityVerificationError::ExpiredKeySet => "expired_key_set",
        AuthorityVerificationError::EmptyKeySet => "empty_key_set",
    }
}

#[cfg(test)]
mod tests {
    use super::node_ids_equivalent;

    #[test]
    fn node_ids_equivalent_accepts_legacy_prefix() {
        let id = "01a040ac-a798-7ae3-ac22-d310a87a3aa8";
        assert!(node_ids_equivalent(id, &format!("node-{id}")));
        assert!(node_ids_equivalent(&format!("node-{id}"), id));
        assert!(node_ids_equivalent(id, id));
    }

    #[test]
    fn node_ids_equivalent_rejects_mismatch() {
        let a = "01a040ac-a798-7ae3-ac22-d310a87a3aa8";
        let b = "02a040ac-a798-7ae3-ac22-d310a87a3aa8";
        assert!(!node_ids_equivalent(a, b));
        assert!(!node_ids_equivalent(&format!("node-{a}"), &format!("node-{b}")));
    }
}
