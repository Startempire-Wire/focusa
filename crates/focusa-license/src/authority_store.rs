//! Durable authority-lease state and production trust-root boundary.

use std::{collections::BTreeMap, path::Path};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::authority::{
    AuthorityLeaseVerifier, AuthorityVerificationError, EntitlementSnapshot,
    LeaseVerificationContext, SignedEnvelope,
};

pub const AUTHORITY_STATE_SCHEMA: &str = "focusa.authority_state.v1";
pub const AUTHORITY_STATE_FILE: &str = "authority-lease.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedAuthorityState {
    pub schema: String,
    pub key_set: SignedEnvelope,
    pub lease: SignedEnvelope,
    pub key_set_sequence: u64,
    pub last_validated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_after: Option<DateTime<Utc>>,
}

#[derive(Debug, Error)]
pub enum AuthorityStoreError {
    #[error("authority state is missing")]
    Missing,
    #[error("authority state cannot be read: {0}")]
    Read(String),
    #[error("authority state is invalid JSON")]
    InvalidJson,
    #[error("unsupported authority state schema: {0}")]
    UnsupportedSchema(String),
    #[error("production authority trust roots are not embedded")]
    MissingTrustRoots,
    #[error("test or local trust root is forbidden in a production trust set: {0}")]
    ForbiddenTrustRoot(String),
    #[error("invalid authority trust root: {0}")]
    InvalidTrustRoot(String),
    #[error(transparent)]
    Verification(#[from] AuthorityVerificationError),
}

impl PersistedAuthorityState {
    pub fn read(path: &Path) -> Result<Self, AuthorityStoreError> {
        if !path.exists() {
            return Err(AuthorityStoreError::Missing);
        }
        let raw =
            std::fs::read(path).map_err(|error| AuthorityStoreError::Read(error.to_string()))?;
        let state: Self =
            serde_json::from_slice(&raw).map_err(|_| AuthorityStoreError::InvalidJson)?;
        if state.schema != AUTHORITY_STATE_SCHEMA {
            return Err(AuthorityStoreError::UnsupportedSchema(state.schema));
        }
        Ok(state)
    }

    pub fn verify(
        &self,
        roots: &BTreeMap<String, VerifyingKey>,
        context: &LeaseVerificationContext,
    ) -> Result<EntitlementSnapshot, AuthorityStoreError> {
        let verifier = AuthorityLeaseVerifier::from_signed_key_set(
            &self.key_set,
            roots,
            context.now,
            Some(self.key_set_sequence),
        )?;
        Ok(verifier.verify_lease(&self.lease, context)?)
    }
}

/// Parse roots embedded at compile time by the trusted distribution build.
/// Runtime environment variables and local files are intentionally excluded.
pub fn embedded_production_trust_roots()
-> Result<BTreeMap<String, VerifyingKey>, AuthorityStoreError> {
    let raw = option_env!("FOCUSA_AUTHORITY_ROOT_KEYS_JSON").unwrap_or("");
    parse_production_trust_roots(raw)
}

/// Resolve durable state into the sole runtime entitlement projection.
/// Every read failure is fail-closed; callers never infer a tier locally.
pub fn resolve_authority_state(
    path: &Path,
    roots: Result<BTreeMap<String, VerifyingKey>, AuthorityStoreError>,
    context: &LeaseVerificationContext,
) -> EntitlementSnapshot {
    let state = match PersistedAuthorityState::read(path) {
        Ok(state) => state,
        Err(AuthorityStoreError::Missing) => {
            return EntitlementSnapshot::unactivated(
                &context.expected_product,
                &context.expected_node_id,
            );
        }
        Err(error) => {
            return EntitlementSnapshot::recovery_only(
                &context.expected_product,
                &context.expected_node_id,
                store_error_code(&error),
            );
        }
    };
    let roots = match roots {
        Ok(roots) => roots,
        Err(error) => {
            return EntitlementSnapshot::recovery_only(
                &context.expected_product,
                &context.expected_node_id,
                store_error_code(&error),
            );
        }
    };
    state.verify(&roots, context).unwrap_or_else(|error| {
        EntitlementSnapshot::recovery_only(
            &context.expected_product,
            &context.expected_node_id,
            store_error_code(&error),
        )
    })
}

pub fn parse_production_trust_roots(
    raw: &str,
) -> Result<BTreeMap<String, VerifyingKey>, AuthorityStoreError> {
    if raw.trim().is_empty() {
        return Err(AuthorityStoreError::MissingTrustRoots);
    }
    let encoded: BTreeMap<String, String> = serde_json::from_str(raw)
        .map_err(|_| AuthorityStoreError::InvalidTrustRoot("json".into()))?;
    if encoded.is_empty() {
        return Err(AuthorityStoreError::MissingTrustRoots);
    }
    encoded
        .into_iter()
        .map(|(key_id, value)| {
            let normalized = key_id.to_ascii_lowercase();
            if ["test", "fixture", "local", "dev", "example"]
                .iter()
                .any(|marker| normalized.contains(marker))
            {
                return Err(AuthorityStoreError::ForbiddenTrustRoot(key_id));
            }
            let decoded = BASE64
                .decode(value)
                .map_err(|_| AuthorityStoreError::InvalidTrustRoot(key_id.clone()))?;
            let bytes: [u8; 32] = decoded
                .try_into()
                .map_err(|_| AuthorityStoreError::InvalidTrustRoot(key_id.clone()))?;
            let key = VerifyingKey::from_bytes(&bytes)
                .map_err(|_| AuthorityStoreError::InvalidTrustRoot(key_id.clone()))?;
            Ok((key_id, key))
        })
        .collect()
}

fn store_error_code(error: &AuthorityStoreError) -> &'static str {
    match error {
        AuthorityStoreError::Missing => "authority_state_missing",
        AuthorityStoreError::Read(_) => "authority_state_unreadable",
        AuthorityStoreError::InvalidJson => "authority_state_invalid_json",
        AuthorityStoreError::UnsupportedSchema(_) => "authority_state_unsupported_schema",
        AuthorityStoreError::MissingTrustRoots => "authority_trust_roots_missing",
        AuthorityStoreError::ForbiddenTrustRoot(_) => "authority_trust_root_forbidden",
        AuthorityStoreError::InvalidTrustRoot(_) => "authority_trust_root_invalid",
        AuthorityStoreError::Verification(_) => "authority_lease_verification_failed",
    }
}
