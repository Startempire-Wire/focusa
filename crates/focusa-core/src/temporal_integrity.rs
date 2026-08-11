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
    let entry = keyring::Entry::new("focusa-temporal-signing", "host-ed25519")
        .map_err(|_| TemporalIntegrityError::KeyStoreUnavailable)?;
    let signing_key = match entry.get_password() {
        Ok(encoded) => {
            let bytes: [u8; 32] = STANDARD
                .decode(encoded)
                .ok()
                .and_then(|bytes| bytes.try_into().ok())
                .ok_or(TemporalIntegrityError::KeyStoreCorrupt)?;
            SigningKey::from_bytes(&bytes)
        }
        Err(keyring::Error::NoEntry) => {
            let mut secret = [0_u8; 32];
            OsRng.fill_bytes(&mut secret);
            let key = SigningKey::from_bytes(&secret);
            entry
                .set_password(&STANDARD.encode(key.to_bytes()))
                .map_err(|_| TemporalIntegrityError::KeyStoreUnavailable)?;
            key
        }
        Err(_) => return Err(TemporalIntegrityError::KeyStoreUnavailable),
    };
    let key_id = format!(
        "temporal-ed25519:{}",
        hex::encode(Sha256::digest(signing_key.verifying_key().as_bytes()))
    );
    Ok((key_id, signing_key))
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
        let key_id = format!("test-ed25519:{}", hex::encode(Sha256::digest(key.verifying_key().as_bytes())));
        (key_id, key)
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let (key_id, key) = test_signing_key();
        let mut event = create_test_event();
        sign_temporal_event(&mut event, &key_id, &key);
        assert!(event.signature.is_some());
        assert!(!event.digest.is_empty());
        verify_temporal_event_signature(&event, Some(&key_id)).expect("valid signature should verify");
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
        assert!(matches!(result, Err(TemporalIntegrityError::DigestMismatch)));
    }

    #[test]
    fn sign_produces_deterministic_digest_for_same_event() {
        let (key_id, key) = test_signing_key();
        let event = create_test_event();
        let mut event1 = event.clone();
        let mut event2 = event1.clone();
        sign_temporal_event(&mut event1, &key_id, &key);
        sign_temporal_event(&mut event2, &key_id, &key);
        assert_eq!(event1.digest, event2.digest, "identical events should have same digest");
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
        let mut events: Vec<TemporalEvent> = (0..3).map(|i| {
            let mut e = create_test_event();
            e.event_id = format!("event-{}", i);
            e
        }).collect();
        for event in &mut events {
            sign_temporal_event(event, &key_id, &key);
        }
        assert!(verify_signed_temporal_chain(&events, Some(&key_id)).is_ok());
    }

    #[test]
    fn verify_chain_fails_if_one_event_tampered() {
        let (key_id, key) = test_signing_key();
        let mut events: Vec<TemporalEvent> = (0..3).map(|i| {
            let mut e = create_test_event();
            e.event_id = format!("event-{}", i);
            e
        }).collect();
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
        verify_temporal_event_signature(&event, None).expect("None key_id should accept any valid key");
    }
}
