use focusa_core::daemon_dispatch::{
    DispatchStatus, MutationDispatchLedger, MutationEnvelope, WriterLeaseRegistry,
};
use focusa_core::daemon_multiplex::{
    DaemonHealth, DaemonRegistration, DaemonRegistryEvent, ProjectRouteKey, reduce_daemon_registry,
};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const PROJECTS: usize = 64;
const RETRIES: usize = 20;

fn route(index: usize) -> ProjectRouteKey {
    ProjectRouteKey {
        project_root: format!("/srv/host-{}/project-{}", index % 8, index),
        continuity_id: format!("continuity-{index}"),
        working_subpath_id: format!("working-subpath:feature-{index}"),
    }
}

#[test]
fn concurrent_remote_projects_have_bounded_exact_durable_dispatch() {
    let started = Instant::now();
    let mut events = Vec::new();
    for index in 0..PROJECTS {
        events.push(DaemonRegistryEvent::Enrolled {
            registration: DaemonRegistration {
                daemon_id: format!("daemon-{index}"),
                controller_id: "controller-stress".into(),
                endpoint: format!("https://host-{}.example.test", index % 8),
                auth_fingerprint: format!("sha256:daemon-{index}"),
                version: "0.9.143".into(),
                capabilities: BTreeSet::from(["mutation".into(), "evidence".into()]),
                allowed_native_sessions: BTreeSet::from([format!("session-{index}")]),
                health: DaemonHealth::Healthy,
                generation: 1,
            },
        });
        events.push(DaemonRegistryEvent::ScopeAssigned {
            daemon_id: format!("daemon-{index}"),
            generation: 1,
            route: route(index),
        });
    }
    let registry = Arc::new(reduce_daemon_registry(events.clone()));
    assert_eq!(registry.rejected_events, 0);

    let mut leases = WriterLeaseRegistry::default();
    for index in 0..PROJECTS {
        leases
            .acquire(
                &registry,
                route(index),
                &format!("daemon-{index}"),
                &format!("lease-{index}"),
                0,
                100_000,
            )
            .unwrap();
    }
    assert_eq!(
        leases.acquire(&registry, route(0), "daemon-0", "contender", 1, 100_000),
        Err(focusa_core::daemon_dispatch::DispatchError::WriterLeaseBusy)
    );

    let leases = Arc::new(leases);
    let ledger = Arc::new(Mutex::new(MutationDispatchLedger::default()));
    let mut workers = Vec::new();
    for index in 0..PROJECTS {
        let registry = Arc::clone(&registry);
        let leases = Arc::clone(&leases);
        let ledger = Arc::clone(&ledger);
        workers.push(std::thread::spawn(move || {
            let mutation = MutationEnvelope {
                mutation_id: format!("mutation-{index}"),
                route: route(index),
                writer_lease_id: format!("lease-{index}"),
                writer_lease_generation: 1,
                payload_digest: format!("sha256:project-{index}"),
                operation: "stress.checkpoint".into(),
            };
            for retry in 0..RETRIES {
                let receipt = ledger
                    .lock()
                    .unwrap()
                    .prepare(&registry, &leases, &mutation, retry as i64)
                    .unwrap()
                    .clone();
                assert_eq!(receipt.route, route(index));
                assert_eq!(receipt.daemon_id, format!("daemon-{index}"));
                assert_eq!(receipt.payload_digest, format!("sha256:project-{index}"));
            }
            ledger
                .lock()
                .unwrap()
                .settle_acknowledged(
                    &format!("mutation-{index}"),
                    &format!("effect:project-{index}"),
                )
                .unwrap();
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }

    let ledger = Arc::try_unwrap(ledger).unwrap().into_inner().unwrap();
    assert!(ledger.recovery_queue().is_empty());
    for index in 0..PROJECTS {
        let receipt = ledger.receipt(&format!("mutation-{index}")).unwrap();
        assert_eq!(receipt.status, DispatchStatus::Acknowledged);
        assert_eq!(receipt.daemon_id, format!("daemon-{index}"));
        assert_eq!(receipt.route, route(index));
        assert_eq!(
            receipt.effect_receipt_ref.as_deref(),
            Some(format!("effect:project-{index}").as_str())
        );
    }

    let restarted: MutationDispatchLedger =
        serde_json::from_slice(&serde_json::to_vec(&ledger).unwrap()).unwrap();
    assert_eq!(restarted, ledger);
    let replayed_registry = reduce_daemon_registry(
        serde_json::from_slice::<Vec<DaemonRegistryEvent>>(&serde_json::to_vec(&events).unwrap())
            .unwrap(),
    );
    assert_eq!(*registry, replayed_registry);

    let mut reconnect_events = events;
    reconnect_events.extend([
        DaemonRegistryEvent::HealthObserved {
            daemon_id: "daemon-0".into(),
            generation: 2,
            health: DaemonHealth::Offline,
            version: "0.9.143".into(),
            capabilities: BTreeSet::new(),
        },
        DaemonRegistryEvent::Enrolled {
            registration: DaemonRegistration {
                daemon_id: "daemon-0-reconnected".into(),
                controller_id: "controller-stress".into(),
                endpoint: "https://host-0-reconnected.example.test".into(),
                auth_fingerprint: "sha256:daemon-0-reconnected".into(),
                version: "0.9.143".into(),
                capabilities: BTreeSet::from(["mutation".into(), "evidence".into()]),
                allowed_native_sessions: BTreeSet::from(["session-0".into()]),
                health: DaemonHealth::Healthy,
                generation: 1,
            },
        },
        DaemonRegistryEvent::ScopeAssigned {
            daemon_id: "daemon-0-reconnected".into(),
            generation: 1,
            route: route(0),
        },
    ]);
    let reconnected_registry = reduce_daemon_registry(reconnect_events);
    assert_eq!(
        reconnected_registry.resolve(&route(0)).unwrap().daemon_id,
        "daemon-0-reconnected"
    );
    let mut leases = Arc::try_unwrap(leases).unwrap();
    let failover = leases
        .acquire(
            &reconnected_registry,
            route(0),
            "daemon-0-reconnected",
            "lease-0-reconnected",
            100_000,
            200_000,
        )
        .unwrap();
    assert_eq!(failover.generation, 2);
    assert!(started.elapsed() < Duration::from_secs(10));
}
