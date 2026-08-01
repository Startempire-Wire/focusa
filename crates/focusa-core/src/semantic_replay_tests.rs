use crate::runtime::persistence_sqlite::SqlitePersistence;
use crate::semantic_pair::*;
use crate::semantic_replay::*;
use crate::types::FocusaConfig;
use std::collections::BTreeMap;

fn item(id: &str) -> SemanticItem {
    SemanticItem {
        id: id.into(),
        statement: id.into(),
        status: "open".into(),
        artifact_refs: vec![],
        attributes: BTreeMap::new(),
    }
}

fn stream() -> Vec<SemanticEventEnvelope> {
    let first = SemanticEventEnvelope::new(
        "event-0",
        "pair-1",
        0,
        "2026-01-01T00:00:00Z",
        GENESIS_HASH,
        SemanticPairEvent::PairCreated {
            builder_attempt: BuilderAttempt {
                attempt_id: "attempt".into(),
                builder: "builder".into(),
                started_at: "2026-01-01T00:00:00Z".into(),
            },
            builder_context: BuilderContext::default(),
            snapshot: ImmutableSnapshot {
                snapshot_id: "snapshot".into(),
                captured_at: "2026-01-01T00:00:00Z".into(),
                content_hash: "sha256:snapshot".into(),
                artifact_refs: vec![],
            },
        },
    )
    .unwrap();
    let second = SemanticEventEnvelope::new(
        "event-1",
        "pair-1",
        1,
        "2026-01-01T00:00:01Z",
        first.hash.clone(),
        SemanticPairEvent::ObligationAdded(item("obligation")),
    )
    .unwrap();
    vec![first, second]
}

#[test]
fn replay_is_deterministic_and_equivalent_after_serialization() {
    let events = stream();
    let first = replay(&events).unwrap();
    let encoded = serde_json::to_vec(&events).unwrap();
    let decoded: Vec<SemanticEventEnvelope> = serde_json::from_slice(&encoded).unwrap();
    let second = replay(&decoded).unwrap();
    assert_eq!(first.aggregate, second.aggregate);
    assert_eq!(first.head_hash, second.head_hash);
}

#[test]
fn tampering_is_detected() {
    let mut events = stream();
    if let SemanticPairEvent::ObligationAdded(item) = &mut events[1].event {
        item.statement = "tampered".into();
    }
    assert!(matches!(
        replay(&events),
        Err(ReplayError::HashMismatch { .. })
    ));
}

#[test]
fn duplicates_and_out_of_order_events_are_rejected() {
    let mut duplicate = stream();
    duplicate[1].event_id = duplicate[0].event_id.clone();
    duplicate[1].hash = duplicate[1].computed_hash().unwrap();
    assert!(matches!(
        replay(&duplicate),
        Err(ReplayError::DuplicateEvent(_))
    ));

    let mut out_of_order = stream();
    out_of_order[1].sequence = 3;
    assert!(matches!(
        replay(&out_of_order),
        Err(ReplayError::OutOfOrder { .. })
    ));
}

#[test]
fn sqlite_batch_append_is_atomic_on_invalid_suffix() {
    let root = std::env::temp_dir().join(format!("focusa-semantic-{}", uuid::Uuid::now_v7()));
    let mut config = FocusaConfig::default();
    config.data_dir = root.to_string_lossy().into_owned();
    let persistence = SqlitePersistence::new(&config).unwrap();
    let events = stream();
    persistence
        .append_semantic_pair_events("pair-1", &events[..1])
        .unwrap();

    let mut invalid = events[1].clone();
    invalid.sequence = 9;
    assert!(persistence
        .append_semantic_pair_events("pair-1", &[events[1].clone(), invalid])
        .is_err());
    assert_eq!(
        persistence
            .load_semantic_pair_events("pair-1")
            .unwrap()
            .len(),
        1
    );
    let _ = std::fs::remove_dir_all(root);
}
