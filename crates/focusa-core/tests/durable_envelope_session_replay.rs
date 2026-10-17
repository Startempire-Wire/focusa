//! Durable consumer regression: event fields must survive envelope attribution.
use focusa_core::types::{EventLogEntry, FocusaEvent, SignalOrigin};
use uuid::Uuid;

#[test]
fn session_started_survives_the_exact_durable_json_value_roundtrip() {
    let session_id = Uuid::now_v7();
    let project_root = std::env::temp_dir()
        .join(format!("durable-scope-{}", Uuid::now_v7()))
        .to_string_lossy()
        .into_owned();
    let event = FocusaEvent::SessionStarted {
        session_id,
        adapter_id: Some("fixture-adapter".into()),
        workspace_id: None,
        project_root: Some(project_root.clone()),
        continuity_id: Some("fixture-continuity".into()),
    };
    let entry = EventLogEntry::captured(event, SignalOrigin::Adapter, None);
    let value = serde_json::to_value(&entry).unwrap();
    let replayed: EventLogEntry = serde_json::from_value(value)
        .expect("durable replay must not consume session_id before decoding the flattened event");
    assert!(
        replayed.session_id.is_none(),
        "payload identity must not become ingress attribution"
    );
    assert!(replayed.project_root.is_none());
    assert!(replayed.continuity_id.is_none());
    match replayed.event {
        FocusaEvent::SessionStarted {
            session_id: actual,
            project_root: root,
            continuity_id,
            ..
        } => {
            assert_eq!(actual, session_id);
            assert_eq!(root, Some(project_root));
            assert_eq!(continuity_id.as_deref(), Some("fixture-continuity"));
        }
        _ => panic!("durable event kind changed"),
    }
}

#[test]
fn overlapping_ingress_and_payload_identity_remain_distinct() {
    let payload_id = Uuid::now_v7();
    let ingress_id = Uuid::now_v7();
    let payload_root = std::env::temp_dir()
        .join(format!("payload-{}", Uuid::now_v7()))
        .to_string_lossy()
        .into_owned();
    let ingress_root = std::env::temp_dir()
        .join(format!("ingress-{}", Uuid::now_v7()))
        .to_string_lossy()
        .into_owned();
    let mut entry = EventLogEntry::captured(
        FocusaEvent::SessionStarted {
            session_id: payload_id,
            adapter_id: None,
            workspace_id: None,
            project_root: Some(payload_root.clone()),
            continuity_id: Some("payload-continuity".into()),
        },
        SignalOrigin::Adapter,
        None,
    );
    entry.session_id = Some(ingress_id);
    entry.project_root = Some(ingress_root.clone());
    entry.continuity_id = Some("ingress-continuity".into());
    let value = serde_json::to_value(&entry).unwrap();
    assert_eq!(
        value["session_id"],
        serde_json::to_value(payload_id).unwrap()
    );
    assert_eq!(
        value["event_log_context"]["schema"],
        "focusa.event_log_context.v1"
    );
    let replayed: EventLogEntry = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(replayed.session_id, Some(ingress_id));
    assert_eq!(replayed.project_root, Some(ingress_root));
    assert_eq!(
        replayed.continuity_id.as_deref(),
        Some("ingress-continuity")
    );
    match replayed.event {
        FocusaEvent::SessionStarted {
            session_id,
            project_root,
            ..
        } => {
            assert_eq!(session_id, payload_id);
            assert_eq!(project_root, Some(payload_root));
        }
        _ => panic!("event kind changed"),
    }
    for mutation in ["version", "foreign_attribute"] {
        let mut invalid = value.clone();
        if mutation == "version" {
            invalid["event_log_context"]["schema"] = serde_json::json!("unsupported");
        } else {
            invalid["event_log_context"]["attributes"]["origin"] = serde_json::json!("gui");
        }
        assert!(serde_json::from_value::<EventLogEntry>(invalid).is_err());
    }
}

#[test]
fn restored_legacy_session_keeps_id_without_recapturing_time() {
    let session_id = Uuid::now_v7();
    let entry = EventLogEntry::captured(
        FocusaEvent::SessionRestored { session_id },
        SignalOrigin::Daemon,
        None,
    );
    let mut value = serde_json::to_value(&entry).unwrap();
    value.as_object_mut().unwrap().remove("temporal");
    let replayed: EventLogEntry = serde_json::from_value(value).unwrap();
    assert!(replayed.session_id.is_none());
    assert_eq!(
        serde_json::to_value(replayed.temporal).unwrap(),
        serde_json::to_value(focusa_core::temporal_clock::TemporalActionEnvelope::default())
            .unwrap()
    );
    assert!(
        matches!(replayed.event,FocusaEvent::SessionRestored {session_id:actual} if actual==session_id)
    );
}
