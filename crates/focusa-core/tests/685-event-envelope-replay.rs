//! Regression for #685: flattened event identity must survive envelope decoding.
use chrono::Utc;
use focusa_core::runtime::{daemon::Daemon, persistence_sqlite::SqlitePersistence};
use focusa_core::types::{
    EventLogEntry, FocusaConfig, FocusaEvent, FocusaState, SignalKind, SignalOrigin,
};
use std::sync::{Arc, atomic::AtomicU64};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

fn entry(event: FocusaEvent) -> EventLogEntry {
    EventLogEntry {
        id: Uuid::from_u128(1),
        timestamp: Utc::now(),
        temporal: Default::default(),
        event,
        correlation_id: Some("synthetic-replay-regression".into()),
        origin: SignalOrigin::Daemon,
        machine_id: Some("synthetic-machine".into()),
        instance_id: None,
        session_id: None,
        project_root: None,
        continuity_id: None,
        thread_id: None,
        is_observation: false,
    }
}

#[test]
fn session_started_survives_real_envelope_round_trip_with_and_without_attribution() {
    let sid = Uuid::from_u128(2);
    for attributed in [false, true] {
        let mut original = entry(FocusaEvent::SessionStarted {
            session_id: sid,
            adapter_id: Some("pi".into()),
            workspace_id: Some("synthetic-workspace".into()),
            project_root: Some("/tmp/focusa-replay-regression".into()),
            continuity_id: Some("synthetic-continuity".into()),
        });
        if attributed {
            original.session_id = Some(sid);
            original.project_root = Some("/tmp/focusa-replay-regression".into());
            original.continuity_id = Some("synthetic-continuity".into());
        }
        let expected_event = serde_json::to_value(&original.event).unwrap();
        let payload = serde_json::to_value(&original).unwrap();
        assert_eq!(payload["session_id"], sid.to_string());
        let decoded = serde_json::from_value::<EventLogEntry>(payload)
            .expect("stored session_id must reach both the event and envelope decoder");
        assert_eq!(
            serde_json::to_value(&decoded.event).unwrap(),
            expected_event
        );
        assert_eq!(decoded.session_id, Some(sid));
        assert_eq!(decoded.machine_id, original.machine_id);
        assert_eq!(decoded.correlation_id, original.correlation_id);
        assert_eq!(decoded.id, original.id);
        assert_eq!(decoded.timestamp, original.timestamp);
    }
}

#[test]
fn session_restored_survives_real_envelope_round_trip() {
    let sid = Uuid::from_u128(3);
    let mut original = entry(FocusaEvent::SessionRestored { session_id: sid });
    original.session_id = Some(sid);
    let decoded: EventLogEntry =
        serde_json::from_value(serde_json::to_value(&original).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(decoded.event).unwrap(),
        serde_json::to_value(original.event).unwrap()
    );
    assert_eq!(decoded.session_id, Some(sid));
}

#[test]
fn thread_created_survives_real_envelope_round_trip() {
    let tid = Uuid::from_u128(4);
    let mut original = entry(FocusaEvent::ThreadCreated {
        thread_id: tid,
        name: "synthetic-thread".into(),
        primary_intent: "replay exact event meaning".into(),
        owner_machine_id: Some("synthetic-machine".into()),
    });
    original.thread_id = Some(tid);
    let decoded: EventLogEntry =
        serde_json::from_value(serde_json::to_value(&original).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(decoded.event).unwrap(),
        serde_json::to_value(original.event).unwrap()
    );
    assert_eq!(decoded.thread_id, Some(tid));
}

#[test]
fn genuinely_missing_required_event_identity_is_not_fabricated() {
    let original = entry(FocusaEvent::SessionRestored {
        session_id: Uuid::from_u128(5),
    });
    let mut payload = serde_json::to_value(original).unwrap();
    payload.as_object_mut().unwrap().remove("session_id");
    let error = serde_json::from_value::<EventLogEntry>(payload).unwrap_err();
    assert!(error.to_string().contains("session_id"));
}

#[test]
fn legacy_flat_envelope_defaults_remain_compatible() {
    let original = entry(FocusaEvent::SessionRestored {
        session_id: Uuid::from_u128(6),
    });
    let mut payload = serde_json::to_value(original).unwrap();
    for field in ["temporal", "correlation_id", "machine_id", "is_observation"] {
        payload.as_object_mut().unwrap().remove(field);
    }
    let decoded: EventLogEntry = serde_json::from_value(payload).unwrap();
    assert_eq!(decoded.session_id, Some(Uuid::from_u128(6)));
    assert!(decoded.correlation_id.is_none());
    assert!(decoded.machine_id.is_none());
    assert!(!decoded.is_observation);
}

#[test]
fn required_envelope_identity_is_not_defaulted() {
    let original = entry(FocusaEvent::SessionRestored {
        session_id: Uuid::from_u128(7),
    });
    let mut payload = serde_json::to_value(original).unwrap();
    payload.as_object_mut().unwrap().remove("id");
    assert!(
        serde_json::from_value::<EventLogEntry>(payload)
            .unwrap_err()
            .to_string()
            .contains("missing field `id`")
    );
}

#[test]
fn actual_sqlite_producer_and_daemon_restart_replay_preserve_scope_and_history() {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            let root = std::env::temp_dir().join(format!("focusa-685-replay-{}", Uuid::now_v7()));
            std::fs::create_dir(&root).unwrap();
            let mut config = FocusaConfig::default();
            config.data_dir = root.to_string_lossy().into_owned();
            let persistence = SqlitePersistence::new(&config).unwrap();
            persistence.save_state(&FocusaState::default()).unwrap();
            let sid = Uuid::from_u128(8);
            let mut started = entry(FocusaEvent::SessionStarted {
                session_id: sid,
                adapter_id: Some("pi".into()),
                workspace_id: Some("synthetic-workspace".into()),
                project_root: Some("/tmp/focusa-replay-regression".into()),
                continuity_id: Some("synthetic-continuity".into()),
            });
            started.session_id = Some(sid);
            started.project_root = Some("/tmp/focusa-replay-regression".into());
            started.continuity_id = Some("synthetic-continuity".into());
            persistence.append_event(&started).unwrap();
            let mut thread = entry(FocusaEvent::ThreadCreated {
                thread_id: Uuid::from_u128(9),
                name: "synthetic-thread".into(),
                primary_intent: "restart replay proof".into(),
                owner_machine_id: Some("synthetic-machine".into()),
            });
            thread.id = Uuid::from_u128(10);
            // Creation has no caller thread: the event's new thread ID is not
            // permission to mutate a pre-existing caller-owned thread.
            persistence.append_event(&thread).unwrap();
            assert!(
                persistence.durable_events_after(0, 10).unwrap()[1]
                    .thread_id
                    .is_none()
            );
            let before: Vec<_> = persistence
                .durable_events_after(0, 10)
                .unwrap()
                .into_iter()
                .map(|record| (record.sequence, record.event_id, record.payload))
                .collect();
            assert_eq!(before.len(), 2);
            drop(persistence);
            let shared = Arc::new(RwLock::new(FocusaState::default()));
            let daemon = Daemon::new(
                config.clone(),
                shared.clone(),
                Arc::new(Mutex::new(())),
                Arc::new(AtomicU64::new(0)),
            )
            .expect("actual startup replay must decode session and thread events");
            let restored = shared.try_read().unwrap().clone();
            assert_eq!(restored.version, 2);
            let session = restored.session.as_ref().unwrap();
            assert_eq!(session.session_id, sid);
            assert_eq!(
                session.project_root.as_deref(),
                Some("/tmp/focusa-replay-regression")
            );
            assert_eq!(
                session.continuity_id.as_deref(),
                Some("synthetic-continuity")
            );
            let persistence = daemon.persistence();
            let after: Vec<_> = persistence
                .durable_events_after(0, 10)
                .unwrap()
                .into_iter()
                .map(|record| (record.sequence, record.event_id, record.payload))
                .collect();
            assert_eq!(
                after, before,
                "replay must not rewrite event payloads or sequence"
            );
            persistence.save_state(&restored).unwrap();
            drop(persistence);
            drop(daemon);
            let shared = Arc::new(RwLock::new(FocusaState::default()));
            let _daemon = Daemon::new(
                config.clone(),
                shared.clone(),
                Arc::new(Mutex::new(())),
                Arc::new(AtomicU64::new(0)),
            )
            .unwrap();
            let settled = shared.try_read().unwrap();
            assert_eq!(
                settled.version, 2,
                "checkpointed events must not be applied twice"
            );
            assert_eq!(settled.session.as_ref().unwrap().session_id, sid);
            drop(settled);
            let persistence = _daemon.persistence();
            let mut foreign = entry(FocusaEvent::IntuitionSignalObserved {
                signal_id: Uuid::from_u128(11),
                signal_type: SignalKind::Warning,
                severity: "info".into(),
                summary: "synthetic ownership rejection".into(),
                related_frame_id: None,
            });
            foreign.id = Uuid::from_u128(12);
            foreign.thread_id = Some(Uuid::from_u128(9));
            foreign.machine_id = Some("foreign-machine".into());
            persistence.append_event(&foreign).unwrap();
            drop(_daemon);
            let error = Daemon::new(
                config,
                Arc::new(RwLock::new(FocusaState::default())),
                Arc::new(Mutex::new(())),
                Arc::new(AtomicU64::new(0)),
            )
            .err()
            .expect("foreign caller thread must still be rejected");
            assert!(format!("{error:#}").to_lowercase().contains("owner"));
            assert_eq!(
                persistence
                    .load_state_with_event_sequence()
                    .unwrap()
                    .unwrap()
                    .0
                    .version,
                2
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
