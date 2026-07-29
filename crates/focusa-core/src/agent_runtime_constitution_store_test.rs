use crate::agent_runtime_constitution::{
    RuntimeConstitutionEvent, RuntimeConstitutionLifecycleState, RuntimeConstitutionVersion,
};
use crate::agent_runtime_constitution_store::*;
use chrono::Utc;
use rusqlite::Connection;

#[test]
fn migration_creates_every_spec140_persistence_surface() {
    let mut connection = Connection::open_in_memory().unwrap();
    migrate_runtime_constitution_schema(&mut connection).unwrap();
    for table in [
        "runtime_constitutions",
        "instruction_sources",
        "instruction_claims",
        "instruction_conflicts",
        "instruction_resolutions",
        "operating_contracts",
        "prompt_assembly_plans",
        "prompt_variants",
        "prompt_evaluations",
        "skill_activation_plans",
        "tool_routing_plans",
        "enforcement_plans",
        "validation_matrices",
        "contract_impact_assessments",
        "delivery_manifests",
        "runtime_constitution_events",
    ] {
        let found: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(found, 1, "missing {table}");
    }
}

#[test]
fn append_is_hash_chained_and_idempotent() {
    let mut connection = Connection::open_in_memory().unwrap();
    migrate_runtime_constitution_schema(&mut connection).unwrap();
    let version = RuntimeConstitutionVersion {
        version: "1".into(),
        parent_version: None,
        content_sha256: "a".repeat(64),
        lifecycle: RuntimeConstitutionLifecycleState::Draft,
        created_at: Utc::now(),
    };
    let event = RuntimeConstitutionEvent::RuntimeConstitutionDrafted(version);
    let first = append_runtime_constitution_event(
        &mut connection,
        "event-1",
        "constitution-1",
        "draft-v1",
        &event,
    )
    .unwrap();
    let replay = append_runtime_constitution_event(
        &mut connection,
        "event-retry",
        "constitution-1",
        "draft-v1",
        &event,
    )
    .unwrap();
    assert_eq!(first.event_id, replay.event_id);
    assert_eq!(first.event_hash, replay.event_hash);
    assert_eq!(first.sequence, 1);
    assert_eq!(first.previous_event_hash, None);
}
