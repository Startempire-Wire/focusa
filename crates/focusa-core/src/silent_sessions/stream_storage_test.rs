use std::fs;

use chrono::{TimeZone, Utc};
use serde_json::json;

use crate::{
    runtime::persistence_sqlite::SqlitePersistence,
    silent_sessions::{
        CanonicalStreamEvent, ConfigRevisionId, EVENT_SCHEMA_VERSION, ObservationProvenance,
        OutputChannel, RedactionReport, SecureStreamStore, SilentSession, SilentSessionAuthority,
        SilentSessionEvent, SilentSessionEventId, SilentSessionRunId, StreamRecoveryAction,
        StreamStorageError, append_reducer_event_and_project,
    },
    types::FocusaConfig,
};

fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("focusa-stream-{}", uuid::Uuid::now_v7()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn fixture() -> (
    std::path::PathBuf,
    SqlitePersistence,
    SilentSession,
    SilentSessionRunId,
) {
    let dir = temp_dir();
    let config = FocusaConfig {
        data_dir: dir.join("data").to_string_lossy().into_owned(),
        ..FocusaConfig::default()
    };
    let persistence = SqlitePersistence::new(&config).unwrap();
    let session = SilentSession::draft(
        SilentSessionAuthority::new(
            crate::test_support::absolute_path_string("silent-stream-project"),
            "continuity-stream",
        )
        .unwrap(),
        "stream-proof",
        "prove durable streams",
        ConfigRevisionId::new(),
        Utc.with_ymd_and_hms(2026, 7, 17, 15, 0, 0).unwrap(),
    )
    .unwrap();
    let mut created = SilentSessionEvent {
        event_schema_version: EVENT_SCHEMA_VERSION,
        id: SilentSessionEventId::new(),
        silent_session_id: session.id,
        run_id: None,
        sequence: 1,
        kind: "session.created".into(),
        payload: json!({}),
        idempotency_key: "stream-session-created".into(),
        previous_event_hash: None,
        event_hash: String::new(),
        occurred_at: session.created_at,
    };
    append_reducer_event_and_project(&persistence, &mut created, &session).unwrap();
    (dir, persistence, session, SilentSessionRunId::new())
}

fn stream_event(
    session: &SilentSession,
    run_id: SilentSessionRunId,
    sequence: u64,
    channel: OutputChannel,
) -> CanonicalStreamEvent {
    CanonicalStreamEvent {
        schema: "focusa.silent_session_event.v1".into(),
        event_id: SilentSessionEventId::new(),
        session_id: session.id,
        run_id,
        seq: sequence,
        occurred_at: session.created_at + chrono::Duration::seconds(sequence as i64),
        observed_at: session.created_at + chrono::Duration::seconds(sequence as i64),
        kind: match channel {
            OutputChannel::Stdout => "stream.stdout",
            OutputChannel::Stderr => "stream.stderr",
            _ => "agent.working",
        }
        .into(),
        source: "harness:test".into(),
        provenance: ObservationProvenance::RuntimeObserved,
        canonical: false,
        channel,
        payload: json!({"text": format!("line-{sequence}")}),
        artifact_refs: Vec::new(),
        correlation_id: SilentSessionEventId::new(),
        redaction: RedactionReport {
            applied: true,
            classes: Vec::new(),
        },
    }
}

#[test]
fn chunks_are_secure_durable_resumable_and_idempotent() {
    let (dir, persistence, session, run_id) = fixture();
    let root = dir.join("streams");
    let store = SecureStreamStore::new(&root, persistence.clone()).unwrap();
    let events = vec![
        stream_event(&session, run_id, 1, OutputChannel::Stdout),
        stream_event(&session, run_id, 2, OutputChannel::Stdout),
    ];
    let published = store
        .publish_chunk(session.id, run_id, OutputChannel::Stdout, 0, &events)
        .unwrap();
    assert!(!published.replayed);
    assert!(published.manifest.compressed_bytes > 0);
    let chunk_path = root.join(&published.manifest.chunk_ref);
    let sidecar_path = chunk_path.with_extension("manifest.json");
    assert!(chunk_path.is_file());
    assert!(sidecar_path.is_file());
    let sidecar: crate::silent_sessions::StreamChunkManifest =
        serde_json::from_slice(&fs::read(&sidecar_path).unwrap()).unwrap();
    assert_eq!(sidecar, published.manifest);

    let replayed = store
        .publish_chunk(session.id, run_id, OutputChannel::Stdout, 0, &events)
        .unwrap();
    assert!(replayed.replayed);
    assert_eq!(replayed.manifest.chunk_hash, published.manifest.chunk_hash);

    let restarted = SecureStreamStore::new(&root, persistence).unwrap();
    let (first, cursor) = restarted
        .read_after(session.id, run_id, OutputChannel::Stdout, None, 1)
        .unwrap();
    assert_eq!(first[0].seq, 1);
    let (second, _) = restarted
        .read_after(
            session.id,
            run_id,
            OutputChannel::Stdout,
            cursor.as_deref(),
            10,
        )
        .unwrap();
    assert_eq!(
        second.iter().map(|event| event.seq).collect::<Vec<_>>(),
        vec![2]
    );
    let (independent, _) = restarted
        .read_after(session.id, run_id, OutputChannel::Stdout, None, 10)
        .unwrap();
    assert_eq!(independent.len(), 2);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(root.join(&published.manifest.chunk_ref))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}

#[test]
fn redaction_and_monotonic_sequence_are_fail_closed() {
    let (dir, persistence, session, run_id) = fixture();
    let store = SecureStreamStore::new(dir.join("streams"), persistence).unwrap();
    let mut unredacted = stream_event(&session, run_id, 1, OutputChannel::Stderr);
    unredacted.redaction.applied = false;
    assert!(
        store
            .publish_chunk(session.id, run_id, OutputChannel::Stderr, 0, &[unredacted])
            .is_err()
    );

    let duplicate_sequence = vec![
        stream_event(&session, run_id, 2, OutputChannel::Stderr),
        stream_event(&session, run_id, 2, OutputChannel::Stderr),
    ];
    assert!(matches!(
        store.publish_chunk(
            session.id,
            run_id,
            OutputChannel::Stderr,
            0,
            &duplicate_sequence
        ),
        Err(StreamStorageError::InvalidEventOrder)
    ));

    let first = vec![stream_event(&session, run_id, 1, OutputChannel::Stdout)];
    store
        .publish_chunk(session.id, run_id, OutputChannel::Stdout, 0, &first)
        .unwrap();
    let skipped_chunk = vec![stream_event(&session, run_id, 2, OutputChannel::Stdout)];
    assert!(matches!(
        store.publish_chunk(session.id, run_id, OutputChannel::Stdout, 2, &skipped_chunk),
        Err(StreamStorageError::IndexPositionMismatch)
    ));
    let overlapping_run_sequence = vec![stream_event(
        &session,
        run_id,
        1,
        OutputChannel::SystemDiagnostics,
    )];
    assert!(matches!(
        store.publish_chunk(
            session.id,
            run_id,
            OutputChannel::SystemDiagnostics,
            0,
            &overlapping_run_sequence
        ),
        Err(StreamStorageError::IndexPositionMismatch)
    ));
}

#[test]
fn corruption_and_cross_run_cursor_are_rejected() {
    let (dir, persistence, session, run_id) = fixture();
    let root = dir.join("streams");
    let store = SecureStreamStore::new(&root, persistence).unwrap();
    let events = vec![stream_event(&session, run_id, 1, OutputChannel::Stdout)];
    let published = store
        .publish_chunk(session.id, run_id, OutputChannel::Stdout, 0, &events)
        .unwrap();
    let (_, cursor) = store
        .read_after(session.id, run_id, OutputChannel::Stdout, None, 1)
        .unwrap();
    assert!(matches!(
        store.read_after(
            session.id,
            SilentSessionRunId::new(),
            OutputChannel::Stdout,
            cursor.as_deref(),
            1
        ),
        Err(StreamStorageError::CursorRunMismatch)
    ));

    fs::write(root.join(published.manifest.chunk_ref), b"tampered").unwrap();
    assert!(matches!(
        store.read_after(session.id, run_id, OutputChannel::Stdout, None, 10),
        Err(StreamStorageError::ChecksumMismatch)
    ));
}

#[test]
fn rotator_flushes_durably_and_disconnects_slow_subscriber() {
    use crate::silent_sessions::{RotationPolicy, StreamRotator};

    let (dir, persistence, session, run_id) = fixture();
    let store = SecureStreamStore::new(dir.join("streams"), persistence).unwrap();
    let mut rotator = StreamRotator::new(
        store,
        session.id,
        run_id,
        RotationPolicy {
            max_event_count: 1,
            ..RotationPolicy::default()
        },
    );
    rotator.fanout.subscribe("fast", 10);
    rotator.fanout.subscribe("slow", 1);
    rotator
        .push(
            stream_event(&session, run_id, 1, OutputChannel::Stdout),
            session.created_at,
        )
        .unwrap();
    assert_eq!(
        rotator
            .push(
                stream_event(&session, run_id, 2, OutputChannel::Stdout),
                session.created_at
            )
            .unwrap()
            .len(),
        1
    );
    rotator.complete().unwrap();
    assert_eq!(rotator.fanout.drain("fast").len(), 2);
    assert!(rotator.fanout.state("slow").unwrap().disconnected);
}

#[test]
fn recovery_rebuilds_a_missing_index_from_a_verified_sidecar() {
    let (dir, persistence, session, run_id) = fixture();
    let root = dir.join("streams");
    let store = SecureStreamStore::new(&root, persistence.clone()).unwrap();
    let published = store
        .publish_chunk(
            session.id,
            run_id,
            OutputChannel::Stdout,
            0,
            &[stream_event(&session, run_id, 1, OutputChannel::Stdout)],
        )
        .unwrap();
    persistence
        .with_connection_mut(|connection| {
            connection.execute(
                "DELETE FROM silent_session_control_stream_indexes WHERE silent_session_id=?1 AND run_id=?2",
                rusqlite::params![session.id.to_string(), run_id.to_string()],
            )?;
            Ok(())
        })
        .unwrap();
    let report = store.recover_registered_run(session.id, run_id).unwrap();
    assert!(!report.degraded, "{report:?}");
    assert_eq!(report.events[0].action, StreamRecoveryAction::IndexRebuilt);
    let (events, _) = store
        .read_after(session.id, run_id, OutputChannel::Stdout, None, 10)
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(
        report.events[0].artifact_ref,
        format!(
            "{}.manifest.json",
            published.manifest.chunk_ref.trim_end_matches(".fss")
        )
    );
}

#[test]
fn recovery_quarantines_corrupt_registered_artifacts_and_marks_degraded() {
    let (dir, persistence, session, run_id) = fixture();
    let root = dir.join("streams");
    let store = SecureStreamStore::new(&root, persistence).unwrap();
    let published = store
        .publish_chunk(
            session.id,
            run_id,
            OutputChannel::Stdout,
            0,
            &[stream_event(&session, run_id, 1, OutputChannel::Stdout)],
        )
        .unwrap();
    let chunk = root.join(&published.manifest.chunk_ref);
    let sidecar = chunk.with_extension("manifest.json");
    fs::write(&sidecar, b"not-json").unwrap();
    let report = store.recover_registered_run(session.id, run_id).unwrap();
    assert!(report.degraded);
    assert_eq!(report.events[0].action, StreamRecoveryAction::Quarantined);
    assert!(!chunk.exists());
    assert!(!sidecar.exists());
    let quarantine = root
        .join(session.id.to_string())
        .join(run_id.to_string())
        .join("recovery/quarantine");
    assert_eq!(fs::read_dir(quarantine).unwrap().count(), 2);
}

#[cfg(unix)]
#[test]
fn symlink_stream_root_is_rejected() {
    use std::os::unix::fs::symlink;

    let (dir, persistence, _session, _run_id) = fixture();
    let actual = dir.join("actual");
    fs::create_dir(&actual).unwrap();
    let link = dir.join("streams-link");
    symlink(&actual, &link).unwrap();
    assert!(matches!(
        SecureStreamStore::new(link.join("nested"), persistence),
        Err(StreamStorageError::UnsafePath(_))
    ));
}
