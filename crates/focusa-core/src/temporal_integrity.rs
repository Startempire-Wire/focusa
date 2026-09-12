use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::temporal::{TemporalEvent, temporal_event_digest};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalEventSignature {
    pub algorithm: String,
    pub key_id: String,
    pub public_key_base64: String,
    pub signature_base64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalIntegrityError {
    UnsupportedAlgorithm,
    InvalidPublicKey,
    InvalidSignature,
    DigestMismatch,
    MissingSignature,
    KeyIdMismatch,
    KeyStoreUnavailable,
    KeyStoreCorrupt,
}

pub fn load_or_create_temporal_signing_key() -> Result<(String, SigningKey), TemporalIntegrityError>
{
    // OS keystore is the primary custody path (see #598: per-target native
    // backends). Headless/container hosts without a usable keystore fall back
    // to a durable 0600 key file under the daemon data directory so Spec 137
    // temporal authority does not fail closed on every headless deployment.
    let entry = keyring::Entry::new("focusa-temporal-signing", "host-ed25519").ok();
    if let Some(entry) = entry {
        match entry.get_password() {
            Ok(encoded) => {
                let bytes: [u8; 32] = STANDARD
                    .decode(encoded)
                    .ok()
                    .and_then(|bytes| bytes.try_into().ok())
                    .ok_or(TemporalIntegrityError::KeyStoreCorrupt)?;
                let signing_key = SigningKey::from_bytes(&bytes);
                return Ok((temporal_key_id(&signing_key), signing_key));
            }
            Err(keyring::Error::NoEntry) => {
                let signing_key = generate_temporal_signing_key();
                let persisted = entry.set_password(&STANDARD.encode(signing_key.to_bytes()));
                return temporal_key_after_primary_write(
                    signing_key,
                    persisted,
                    load_or_create_temporal_signing_key_file,
                );
            }
            Err(_) => {}
        }
    }
    load_or_create_temporal_signing_key_file()
}

// Keep the persistence-result boundary testable without a host OS keyring.
// On failure, load the existing fallback identity before considering creation;
// never expose an ephemeral key or suppress a fallback custody error.
fn temporal_key_after_primary_write(
    signing_key: SigningKey,
    persisted: Result<(), keyring::Error>,
    fallback: impl FnOnce() -> Result<(String, SigningKey), TemporalIntegrityError>,
) -> Result<(String, SigningKey), TemporalIntegrityError> {
    match persisted {
        Ok(()) => Ok((temporal_key_id(&signing_key), signing_key)),
        Err(_) => fallback(),
    }
}

fn temporal_key_id(signing_key: &SigningKey) -> String {
    format!(
        "temporal-ed25519:{}",
        hex::encode(Sha256::digest(signing_key.verifying_key().as_bytes()))
    )
}

fn generate_temporal_signing_key() -> SigningKey {
    let mut secret = [0_u8; 32];
    OsRng.fill_bytes(&mut secret);
    SigningKey::from_bytes(&secret)
}

/// Headless-safe durable custody for the host temporal signing key.
///
/// Precedence: `FOCUSA_TEMPORAL_SIGNING_KEY_FILE` override, then
/// `FOCUSA_DATA_DIR/keys/temporal-signing-key.b64`, then the default Focusa
/// data directory. The file is created 0600 (unix) and written atomically.
fn temporal_signing_key_file_path() -> std::path::PathBuf {
    if let Ok(explicit) = std::env::var("FOCUSA_TEMPORAL_SIGNING_KEY_FILE") {
        let explicit = explicit.trim();
        if !explicit.is_empty() {
            return std::path::PathBuf::from(explicit);
        }
    }
    let base = std::env::var("FOCUSA_DATA_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(crate::types::default_focusa_data_dir()));
    base.join("keys").join("temporal-signing-key.b64")
}

fn load_or_create_temporal_signing_key_file() -> Result<(String, SigningKey), TemporalIntegrityError>
{
    let path = temporal_signing_key_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| TemporalIntegrityError::KeyStoreUnavailable)?;
    }
    match std::fs::read_to_string(&path) {
        Ok(encoded) => {
            let bytes: [u8; 32] = STANDARD
                .decode(encoded.trim())
                .ok()
                .and_then(|bytes| bytes.try_into().ok())
                .ok_or(TemporalIntegrityError::KeyStoreCorrupt)?;
            let signing_key = SigningKey::from_bytes(&bytes);
            Ok((temporal_key_id(&signing_key), signing_key))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let signing_key = generate_temporal_signing_key();
            store_temporal_signing_key_file(&signing_key)?;
            Ok((temporal_key_id(&signing_key), signing_key))
        }
        Err(_) => Err(TemporalIntegrityError::KeyStoreUnavailable),
    }
}

fn store_temporal_signing_key_file(signing_key: &SigningKey) -> Result<(), TemporalIntegrityError> {
    let path = temporal_signing_key_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| TemporalIntegrityError::KeyStoreUnavailable)?;
    }
    let encoded = STANDARD.encode(signing_key.to_bytes());
    let temp_path = path.with_extension("b64.tmp");
    std::fs::write(&temp_path, encoded).map_err(|_| TemporalIntegrityError::KeyStoreUnavailable)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o600))
            .map_err(|_| TemporalIntegrityError::KeyStoreUnavailable)?;
    }
    std::fs::rename(&temp_path, &path).map_err(|_| TemporalIntegrityError::KeyStoreUnavailable)?;
    Ok(())
}

pub fn sign_temporal_event(
    event: &mut TemporalEvent,
    key_id: impl Into<String>,
    signing_key: &SigningKey,
) {
    event.signature = None;
    event.digest = temporal_event_digest(event);
    let signature = signing_key.sign(event.digest.as_bytes());
    event.signature = Some(TemporalEventSignature {
        algorithm: "ed25519".into(),
        key_id: key_id.into(),
        public_key_base64: STANDARD.encode(signing_key.verifying_key().as_bytes()),
        signature_base64: STANDARD.encode(signature.to_bytes()),
    });
}

pub fn verify_temporal_event_signature(
    event: &TemporalEvent,
    expected_key_id: Option<&str>,
) -> Result<(), TemporalIntegrityError> {
    if temporal_event_digest(event) != event.digest {
        return Err(TemporalIntegrityError::DigestMismatch);
    }
    let envelope = event
        .signature
        .as_ref()
        .ok_or(TemporalIntegrityError::MissingSignature)?;
    if envelope.algorithm != "ed25519" {
        return Err(TemporalIntegrityError::UnsupportedAlgorithm);
    }
    if expected_key_id.is_some_and(|expected| expected != envelope.key_id) {
        return Err(TemporalIntegrityError::KeyIdMismatch);
    }
    let public_key = STANDARD
        .decode(&envelope.public_key_base64)
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .and_then(|bytes: [u8; 32]| VerifyingKey::from_bytes(&bytes).ok())
        .ok_or(TemporalIntegrityError::InvalidPublicKey)?;
    let signature = STANDARD
        .decode(&envelope.signature_base64)
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .map(|bytes: [u8; 64]| Signature::from_bytes(&bytes))
        .ok_or(TemporalIntegrityError::InvalidSignature)?;
    public_key
        .verify(event.digest.as_bytes(), &signature)
        .map_err(|_| TemporalIntegrityError::InvalidSignature)
}

pub fn verify_signed_temporal_chain(
    events: &[TemporalEvent],
    expected_key_id: Option<&str>,
) -> Result<(), TemporalIntegrityError> {
    for event in events {
        verify_temporal_event_signature(event, expected_key_id)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal::TemporalEvent;
    use ed25519_dalek::SigningKey;
    use rand_core::OsRng;

    fn create_test_event() -> TemporalEvent {
        TemporalEvent {
            event_id: "test-event-1".into(),
            sequence: 1,
            event_kind: crate::temporal::TemporalEventKind::ClaimCommitted,
            scope: crate::temporal::TemporalScope {
                project_root: "/tmp/test".into(),
                continuity_id: "test-continuity".into(),
                host_id: None,
                operator_id: None,
                workpoint_id: None,
                item_id: None,
                task_id: None,
            },
            claim: None,
            clock_sample: None,
            metadata: std::collections::BTreeMap::new(),
            signature: None,
            predecessor_digest: None,
            recorded_at: chrono::Utc::now(),
            idempotency_key: "test-key".into(),
            digest: String::new(),
        }
    }

    fn test_signing_key() -> (String, SigningKey) {
        let mut secret = [0_u8; 32];
        OsRng.fill_bytes(&mut secret);
        let key = SigningKey::from_bytes(&secret);
        let key_id = format!(
            "test-ed25519:{}",
            hex::encode(Sha256::digest(key.verifying_key().as_bytes()))
        );
        (key_id, key)
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let (key_id, key) = test_signing_key();
        let mut event = create_test_event();
        sign_temporal_event(&mut event, &key_id, &key);
        assert!(event.signature.is_some());
        assert!(!event.digest.is_empty());
        verify_temporal_event_signature(&event, Some(&key_id))
            .expect("valid signature should verify");
    }

    #[test]
    fn verify_rejects_wrong_key_id() {
        let (key_id, key) = test_signing_key();
        let mut event = create_test_event();
        sign_temporal_event(&mut event, &key_id, &key);
        let result = verify_temporal_event_signature(&event, Some("wrong-key-id"));
        assert!(matches!(result, Err(TemporalIntegrityError::KeyIdMismatch)));
    }

    #[test]
    fn verify_rejects_unsigned_event() {
        let event = create_test_event();
        let result = verify_temporal_event_signature(&event, None);
        // Unsigned event has empty digest; digest check fires first.
        assert!(result.is_err());
    }

    #[test]
    fn tampered_digest_fails_verification() {
        let (key_id, key) = test_signing_key();
        let mut event = create_test_event();
        sign_temporal_event(&mut event, &key_id, &key);
        event.digest = "tampered-digest".into();
        let result = verify_temporal_event_signature(&event, Some(&key_id));
        assert!(matches!(
            result,
            Err(TemporalIntegrityError::DigestMismatch)
        ));
    }

    #[test]
    fn sign_produces_deterministic_digest_for_same_event() {
        let (key_id, key) = test_signing_key();
        let event = create_test_event();
        let mut event1 = event.clone();
        let mut event2 = event1.clone();
        sign_temporal_event(&mut event1, &key_id, &key);
        sign_temporal_event(&mut event2, &key_id, &key);
        assert_eq!(
            event1.digest, event2.digest,
            "identical events should have same digest"
        );
    }

    #[test]
    fn sign_produces_different_digests_for_different_events() {
        let (key_id, key) = test_signing_key();
        let mut event1 = create_test_event();
        let mut event2 = create_test_event();
        event2.event_kind = crate::temporal::TemporalEventKind::TargetBreached;
        sign_temporal_event(&mut event1, &key_id, &key);
        sign_temporal_event(&mut event2, &key_id, &key);
        assert_ne!(event1.digest, event2.digest);
    }

    #[test]
    fn verify_chain_passes_for_all_valid_events() {
        let (key_id, key) = test_signing_key();
        let mut events: Vec<TemporalEvent> = (0..3)
            .map(|i| {
                let mut e = create_test_event();
                e.event_id = format!("event-{}", i);
                e
            })
            .collect();
        for event in &mut events {
            sign_temporal_event(event, &key_id, &key);
        }
        assert!(verify_signed_temporal_chain(&events, Some(&key_id)).is_ok());
    }

    #[test]
    fn verify_chain_fails_if_one_event_tampered() {
        let (key_id, key) = test_signing_key();
        let mut events: Vec<TemporalEvent> = (0..3)
            .map(|i| {
                let mut e = create_test_event();
                e.event_id = format!("event-{}", i);
                e
            })
            .collect();
        for event in &mut events {
            sign_temporal_event(event, &key_id, &key);
        }
        events[1].digest = "tampered".into();
        assert!(verify_signed_temporal_chain(&events, Some(&key_id)).is_err());
    }

    #[test]
    fn verify_without_key_id_accepts_any_valid_signature() {
        let (key_id, key) = test_signing_key();
        let mut event = create_test_event();
        sign_temporal_event(&mut event, &key_id, &key);
        verify_temporal_event_signature(&event, None)
            .expect("None key_id should accept any valid key");
    }

    /// Env-var mutations are process-global; keep file-fallback scenarios in
    /// one serialized test to avoid cross-test interference.
    #[test]
    fn file_keystore_fallback_creates_reloads_and_fails_closed_on_corruption() {
        use std::sync::{Mutex, MutexGuard, OnceLock};
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let lock: &'static Mutex<()> = ENV_LOCK.get_or_init(|| Mutex::new(()));
        let _guard: MutexGuard<()> = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

        let dir = std::env::temp_dir().join(format!(
            "focusa-temporal-keystore-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let key_path = dir.join("temporal-signing-key.b64");

        // SAFETY: env mutation serialized by ENV_LOCK; single-threaded test.
        // The file keystore functions are exercised directly so this test is
        // deterministic on hosts where the OS keystore is available (#598
        // native backends) and where it is not (headless CI).
        unsafe { std::env::set_var("FOCUSA_TEMPORAL_SIGNING_KEY_FILE", &key_path) };
        let first = load_or_create_temporal_signing_key_file().expect("file keystore create");
        let second = load_or_create_temporal_signing_key_file().expect("file keystore reload");
        assert_eq!(first.0, second.0, "key must be durable across calls");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&key_path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600, "key file must be owner-only");
        }

        std::fs::write(&key_path, "not-base64!!").unwrap();
        assert_eq!(
            load_or_create_temporal_signing_key_file().err(),
            Some(TemporalIntegrityError::KeyStoreCorrupt),
            "corrupt key material must fail closed"
        );

        unsafe { std::env::remove_var("FOCUSA_TEMPORAL_SIGNING_KEY_FILE") };
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod persistence_failure_regression_tests {
    use super::*;

    #[test]
    fn primary_and_fallback_failure_never_return_ephemeral_key() {
        for expected in [
            TemporalIntegrityError::KeyStoreUnavailable,
            TemporalIntegrityError::KeyStoreCorrupt,
        ] {
            let result = temporal_key_after_primary_write(
                generate_temporal_signing_key(),
                Err(keyring::Error::NoEntry),
                || Err(expected.clone()),
            );
            assert_eq!(result.err(), Some(expected));
        }
    }

    #[test]
    fn failed_primary_write_preserves_existing_fallback_identity() {
        let existing = generate_temporal_signing_key();
        let expected = temporal_key_id(&existing);
        let (id, key) = temporal_key_after_primary_write(
            generate_temporal_signing_key(),
            Err(keyring::Error::NoEntry),
            || Ok((expected.clone(), existing)),
        )
        .unwrap();
        assert_eq!(id, expected);
        assert_eq!(temporal_key_id(&key), expected);
    }

    #[test]
    fn successful_primary_write_does_not_invoke_fallback() {
        let key = generate_temporal_signing_key();
        let expected = temporal_key_id(&key);
        let (id, _) = temporal_key_after_primary_write(key, Ok(()), || {
            panic!("a successful primary write must not change custody")
        })
        .unwrap();
        assert_eq!(id, expected);
    }
}
