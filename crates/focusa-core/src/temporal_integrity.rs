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
