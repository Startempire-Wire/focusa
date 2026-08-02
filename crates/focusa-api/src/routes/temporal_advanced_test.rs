use std::collections::BTreeMap;

use chrono::Utc;
use focusa_core::temporal::{TemporalEvent, TemporalEventKind, TemporalScope};
use serde_json::{Value, json};
use uuid::Uuid;

use super::temporal_advanced::unattested_legacy_digests;

fn event(
    kind: TemporalEventKind,
    digest: &str,
    metadata: BTreeMap<String, Value>,
) -> TemporalEvent {
    TemporalEvent {
        event_id: Uuid::now_v7().to_string(),
        sequence: 1,
        event_kind: kind,
        scope: TemporalScope::project("/project", "continuity"),
        claim: None,
        clock_sample: None,
        metadata,
        signature: None,
        predecessor_digest: None,
        recorded_at: Utc::now(),
        idempotency_key: "key".into(),
        digest: digest.into(),
    }
}

#[test]
fn signature_migration_excludes_already_attested_legacy_events() {
    let legacy = event(
        TemporalEventKind::ClaimCommitted,
        "sha256:legacy",
        BTreeMap::new(),
    );
    let mut metadata = BTreeMap::new();
    metadata.insert("legacy_event_digests".into(), json!(["sha256:legacy"]));
    let attestation = event(
        TemporalEventKind::LegacySignatureAttestation,
        "sha256:attestation",
        metadata,
    );
    assert!(unattested_legacy_digests(&[legacy, attestation]).is_empty());
}
