//! Signed temporal event persistence and strict idempotent replay checks.

use axum::{Json, http::StatusCode};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use focusa_core::temporal::{TemporalClaim, TemporalEvent, TemporalLedger};
use serde_json::Value;

use super::temporal::{ApiFailure, fail};

pub(crate) fn temporal_signing_key()
-> Result<(String, ed25519_dalek::SigningKey), (StatusCode, Json<Value>)> {
    match (
        std::env::var("FOCUSA_TEMPORAL_SIGNING_KEY_ID").ok(),
        std::env::var("FOCUSA_TEMPORAL_SIGNING_KEY").ok(),
    ) {
        (Some(key_id), Some(encoded)) => {
            let bytes: [u8; 32] = STANDARD
                .decode(encoded)
                .ok()
                .and_then(|bytes| bytes.try_into().ok())
                .ok_or_else(|| {
                    fail(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "temporal_signing_key_invalid",
                        "temporal signing key must be base64-encoded 32-byte Ed25519 material",
                    )
                })?;
            Ok((key_id, ed25519_dalek::SigningKey::from_bytes(&bytes)))
        }
        (None, None) => focusa_core::temporal_integrity::load_or_create_temporal_signing_key()
            .map_err(|error| {
                fail(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "temporal_signing_key_unavailable",
                    format!("host temporal signing key unavailable: {error:?}"),
                )
            }),
        _ => Err(fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "temporal_signing_key_incomplete",
            "set both temporal signing key environment variables or neither",
        )),
    }
}

fn replay_claim_matches(prior: Option<&TemporalClaim>, draft: Option<&TemporalClaim>) -> bool {
    match (prior, draft) {
        (None, None) => true,
        (Some(prior), Some(draft)) => {
            let mut prior = prior.clone();
            let draft = draft.clone();
            prior.observed_at = draft.observed_at;
            prior.effective_at = draft.effective_at;
            prior == draft
        }
        _ => false,
    }
}

pub(super) fn idempotent_replay_matches(
    existing: &[TemporalEvent],
    drafts: &[TemporalEvent],
) -> bool {
    existing.len() == drafts.len()
        && existing.iter().zip(drafts).all(|(prior, draft)| {
            prior.event_kind == draft.event_kind
                && prior.scope == draft.scope
                && replay_claim_matches(prior.claim.as_ref(), draft.claim.as_ref())
                && prior.metadata == draft.metadata
        })
}

pub(super) fn append_signed_events(
    ledger: &TemporalLedger,
    idempotency_key: &str,
    events: Vec<TemporalEvent>,
) -> Result<Vec<TemporalEvent>, ApiFailure> {
    let replay = ledger
        .read_all()
        .map_err(|error| {
            fail(
                StatusCode::CONFLICT,
                "temporal_ledger_invalid",
                format!("{error:?}"),
            )
        })?
        .into_iter()
        .filter(|event| event.idempotency_key == idempotency_key)
        .collect::<Vec<_>>();
    if !replay.is_empty() {
        return if idempotent_replay_matches(&replay, &events) {
            Ok(replay)
        } else {
            Err(fail(
                StatusCode::CONFLICT,
                "idempotency_payload_mismatch",
                "idempotency key was already committed with a different temporal mutation",
            ))
        };
    }
    let (key_id, signing_key) = temporal_signing_key()?;
    ledger
        .append_signed_batch(idempotency_key, events, &key_id, &signing_key)
        .map_err(|error| {
            fail(
                StatusCode::PRECONDITION_FAILED,
                "temporal_ledger_append_failed",
                format!("{error:?}"),
            )
        })
}
