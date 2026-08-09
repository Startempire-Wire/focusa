//! Durable authority-lease state and production trust-root boundary.

use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::authority::{
    AuthorityKeySet, AuthorityLeaseVerifier, AuthorityVerificationError, EntitlementSnapshot,
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
    #[error("authority state cannot be written atomically: {0}")]
    Write(String),
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

    pub fn from_verified_envelopes(
        key_set: SignedEnvelope,
        lease: SignedEnvelope,
        roots: &BTreeMap<String, VerifyingKey>,
        context: &LeaseVerificationContext,
    ) -> Result<(Self, EntitlementSnapshot), AuthorityStoreError> {
        let payload = BASE64
            .decode(&key_set.payload_b64)
            .map_err(|_| AuthorityStoreError::InvalidJson)?;
        let key_set_payload: AuthorityKeySet =
            serde_json::from_slice(&payload).map_err(|_| AuthorityStoreError::InvalidJson)?;
        let state = Self {
            schema: AUTHORITY_STATE_SCHEMA.into(),
            key_set,
            lease,
            key_set_sequence: key_set_payload.sequence,
            last_validated_at: context.now,
            refresh_after: None,
        };
        let snapshot = state.verify(roots, context)?;
        Ok((state, snapshot))
    }

    pub fn write_atomic(&self, path: &Path) -> Result<(), AuthorityStoreError> {
        let parent = path.parent().ok_or_else(|| {
            AuthorityStoreError::Write("authority state path has no parent".into())
        })?;
        std::fs::create_dir_all(parent)
            .map_err(|error| AuthorityStoreError::Write(error.to_string()))?;
        let temporary = temporary_state_path(path);
        let result = (|| {
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options
                .open(&temporary)
                .map_err(|error| AuthorityStoreError::Write(error.to_string()))?;
            let payload = serde_json::to_vec_pretty(self)
                .map_err(|error| AuthorityStoreError::Write(error.to_string()))?;
            file.write_all(&payload)
                .and_then(|_| file.write_all(b"\n"))
                .and_then(|_| file.sync_all())
                .map_err(|error| AuthorityStoreError::Write(error.to_string()))?;
            std::fs::rename(&temporary, path)
                .map_err(|error| AuthorityStoreError::Write(error.to_string()))?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result
    }
}

fn temporary_state_path(path: &Path) -> PathBuf {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(format!(".tmp-{}", uuid::Uuid::now_v7()));
    PathBuf::from(temporary)
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
        AuthorityStoreError::Write(_) => "authority_state_unwritable",
        AuthorityStoreError::InvalidJson => "authority_state_invalid_json",
        AuthorityStoreError::UnsupportedSchema(_) => "authority_state_unsupported_schema",
        AuthorityStoreError::MissingTrustRoots => "authority_trust_roots_missing",
        AuthorityStoreError::ForbiddenTrustRoot(_) => "authority_trust_root_forbidden",
        AuthorityStoreError::InvalidTrustRoot(_) => "authority_trust_root_invalid",
        AuthorityStoreError::Verification(_) => "authority_lease_verification_failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::{EntitlementState, LeaseVerificationContext};
    use serde::Deserialize;

    /// Frozen Spec 152 authority golden vector (signed key set + lease for
    /// `focusa` node `node-golden-001`, sequence 42, issued 2026-08-02,
    /// expires 2026-09-01). Proves that a raw binary / local source build is
    /// decided by the signed lease alone — never by installer provenance.
    const GOLDEN_VECTOR: &str =
        include_str!("../tests/fixtures/spec152-authority-golden-vector.json");

    #[derive(Deserialize)]
    struct GoldenVector {
        root_key_id: String,
        root_public_key_b64: String,
        key_set_envelope: SignedEnvelope,
        lease_envelope: SignedEnvelope,
    }

    fn golden_roots() -> Result<BTreeMap<String, VerifyingKey>, AuthorityStoreError> {
        let vector: GoldenVector =
            serde_json::from_str(GOLDEN_VECTOR).expect("golden vector parses");
        let bytes: [u8; 32] = BASE64
            .decode(vector.root_public_key_b64)
            .expect("root key base64")
            .try_into()
            .expect("root key length");
        Ok(BTreeMap::from([(
            vector.root_key_id,
            VerifyingKey::from_bytes(&bytes).expect("root public key"),
        )]))
    }

    fn golden_context() -> LeaseVerificationContext {
        LeaseVerificationContext {
            expected_product: "focusa".to_string(),
            expected_node_id: "node-golden-001".to_string(),
            now: "2026-08-03T00:00:00Z".parse().expect("fixture time"),
            minimum_sequence: Some(42),
            expected_previous_digest: Some(
                "sha256:4e738ca5563c06cfd0018299933d58db1dd8bf97f6973dc99bf6cdc64b5550bd"
                    .to_string(),
            ),
        }
    }

    fn temp_config_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "focusa-source-build-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::now_v7()
        ))
    }

    /// Spec 152E §14.3 first run: a source-built or manually copied client
    /// (no installer provenance, no installer receipts, no install root)
    /// resolves as unactivated and grants nothing until the universal
    /// authority activation flow issues a signed lease.
    #[test]
    fn source_build_first_run_without_installer_or_lease_grants_nothing() {
        let dir = temp_config_dir("first-run");
        std::fs::create_dir_all(&dir).unwrap();
        // A manually copied binary leaves only noise files; their presence
        // must never create an entitlement fallback.
        std::fs::write(dir.join("source-build-readme.txt"), "binary copied by hand").unwrap();
        let snapshot = resolve_authority_state(
            &dir.join(AUTHORITY_STATE_FILE),
            golden_roots(),
            &golden_context(),
        );
        assert_eq!(snapshot.state, EntitlementState::Unactivated);
        assert_eq!(snapshot.product, "focusa");
        assert_eq!(
            snapshot.recovery_reason.as_deref(),
            Some("authority_lease_missing")
        );
        // No capability grant follows from missing installer state.
        assert!(!snapshot.feature_enabled("mission_canvas"));
        assert!(snapshot.limits.is_empty());
        // The projection carries no install-channel-derived grant surface.
        let projected = serde_json::to_value(&snapshot).unwrap();
        assert!(projected.get("install_channel").is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Spec 152E §14.3 / acceptance matrix deletion proof: deleting installer
    /// state can neither unlock protected work nor change the runtime
    /// decision; deleting the signed authority lease itself locks the machine
    /// and never falls back to a local grant.
    #[test]
    fn deleting_installer_state_never_unlocks_and_deleting_the_lease_locks() {
        let dir = temp_config_dir("delete-matrix");
        std::fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join(AUTHORITY_STATE_FILE);
        let vector: GoldenVector =
            serde_json::from_str(GOLDEN_VECTOR).expect("golden vector parses");
        let state = PersistedAuthorityState {
            schema: AUTHORITY_STATE_SCHEMA.to_string(),
            key_set: vector.key_set_envelope,
            lease: vector.lease_envelope,
            key_set_sequence: 7,
            last_validated_at: golden_context().now,
            refresh_after: None,
        };
        std::fs::write(&state_path, serde_json::to_vec(&state).unwrap()).unwrap();

        // With the signed lease present the machine is Active — installer
        // provenance plays no role in the decision.
        let active = resolve_authority_state(&state_path, golden_roots(), &golden_context());
        assert_eq!(active.state, EntitlementState::Active);
        assert!(active.feature_enabled("mission_canvas"));

        // Deleting installer artifacts changes nothing: only the signed
        // authority lease governs the runtime decision.
        let receipt = dir.join("installer-receipt.json");
        std::fs::write(&receipt, "{}").unwrap();
        std::fs::remove_file(&receipt).unwrap();
        let still_active = resolve_authority_state(&state_path, golden_roots(), &golden_context());
        assert_eq!(still_active.state, EntitlementState::Active);

        // Deleting the lease itself locks the machine: missing durable state
        // never falls back to a local/self-issued grant.
        std::fs::remove_file(&state_path).unwrap();
        let locked = resolve_authority_state(&state_path, golden_roots(), &golden_context());
        assert_eq!(locked.state, EntitlementState::Unactivated);
        assert!(!locked.feature_enabled("mission_canvas"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// install_channel is advisory telemetry only: the durable-state decision
    /// has no channel input and consults no installer marker files.
    #[test]
    fn state_resolution_never_reads_install_channel_or_installer_files() {
        let dir = temp_config_dir("channel-neutral");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("install-focusa.marker"), "official installer").unwrap();
        std::fs::write(dir.join(".focusa-installed"), "install root marker").unwrap();
        let snapshot = resolve_authority_state(
            &dir.join(AUTHORITY_STATE_FILE),
            golden_roots(),
            &golden_context(),
        );
        assert_eq!(snapshot.state, EntitlementState::Unactivated);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
