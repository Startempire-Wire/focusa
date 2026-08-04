use letta_adapter::arbitration::{
    ArbitrationError, CognitiveOwner, OwnershipAcquireRequest, RuntimeAuthorityScope,
    RuntimeOwnershipRegistry,
};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

fn scope(session: &str) -> RuntimeAuthorityScope {
    RuntimeAuthorityScope {
        project_root: "/project".into(),
        continuity_id: "continuity".into(),
        workpoint_id: "workpoint-1".into(),
        native_session_id: session.into(),
    }
}

#[test]
fn concurrent_adapters_cannot_duplicate_steal_or_cross_session_ownership() {
    let registry = Arc::new(Mutex::new(RuntimeOwnershipRegistry::default()));
    let mut workers = Vec::new();
    for index in 0..16 {
        let registry = Arc::clone(&registry);
        workers.push(std::thread::spawn(move || {
            registry.lock().unwrap().acquire(OwnershipAcquireRequest {
                lease_id: format!("lease-{index}"),
                scope: scope("session-1"),
                owner: if index % 2 == 0 {
                    CognitiveOwner::Pi
                } else {
                    CognitiveOwner::Letta
                },
                adapter_instance_id: format!("adapter-{index}"),
                epoch_id: Uuid::now_v7(),
                now_unix_ms: 0,
                expires_at_unix_ms: 100,
            })
        }));
    }
    let results = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert!(
        results
            .iter()
            .filter(|result| matches!(result, Err(ArbitrationError::CompetingOwner)))
            .count()
            >= 15
    );

    let mut registry = Arc::try_unwrap(registry).unwrap().into_inner().unwrap();
    let original = registry.leases[0].clone();
    let event_key = registry.authorize_turn(&original, "event-1", 1).unwrap();
    assert_eq!(
        registry.authorize_turn(&original, "event-1", 2),
        Err(ArbitrationError::DuplicateTurn)
    );
    let mut foreign_session = original.clone();
    foreign_session.scope = scope("session-foreign");
    assert_eq!(
        registry.authorize_turn(&foreign_session, "event-2", 2),
        Err(ArbitrationError::ForeignLease)
    );
    registry
        .authorize_uiai_client_tool(&original, &event_key, 2)
        .unwrap();
    assert_eq!(
        registry.authorize_uiai_client_tool(&original, "foreign-parent", 2),
        Err(ArbitrationError::ForeignLease)
    );

    let handed = registry
        .handoff(
            &original,
            "lease-handoff",
            CognitiveOwner::Letta,
            "adapter-letta",
            Uuid::now_v7(),
            200,
        )
        .unwrap();
    assert_eq!(handed.generation, original.generation + 1);
    assert_eq!(
        registry.authorize_turn(&original, "event-stolen", 3),
        Err(ArbitrationError::ForeignLease)
    );
    registry.authorize_turn(&handed, "event-2", 3).unwrap();

    let restarted: RuntimeOwnershipRegistry =
        serde_json::from_slice(&serde_json::to_vec(&registry).unwrap()).unwrap();
    assert_eq!(restarted, registry);
}
