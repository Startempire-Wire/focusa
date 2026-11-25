//! Exact checkout identity and controller transaction regressions.
use super::*;

#[test]
fn inventory_requires_exact_root_and_origin() {
    let good =
        b"/work/project\n/work/project\nhttps://example.test/repo.git\n/work/project/.git\nbeads\n";
    assert!(
        parse_reply(
            "/work/project",
            "https://example.test/repo.git",
            Some("/work/project/.git"),
            good
        )
        .unwrap()
        .beads_present
    );
    assert!(
        parse_reply(
            "/work/project/child",
            "https://example.test/repo.git",
            None,
            good
        )
        .is_err()
    );
    assert!(
        parse_reply(
            "/work/project",
            "https://example.test/other.git",
            None,
            good
        )
        .is_err()
    );
    assert!(
        parse_reply(
            "/work/project",
            "https://example.test/repo.git",
            Some("/work/other/.git"),
            good
        )
        .is_err()
    );
    assert!(
        parse_reply(
            "/work/project",
            "https://example.test/repo.git",
            None,
            b"/work/project\n/work\nhttps://example.test/repo.git\n/work/project/.git\nbeads\n"
        )
        .is_err()
    );
    assert!(
        parse_reply(
            "/work/project",
            "https://example.test/repo.git",
            None,
            b"/work/project\n/work/project\nhttps://example.test/repo.git\n/work/project/.git\nabsent\nextra\n"
        )
        .is_err()
    );
}

#[tokio::test]
async fn commit_rechecks_revocation_and_rolls_back_failed_effects() {
    use super::super::{BINDING_SCHEMA, BindingStatus, upsert_binding};

    let now = chrono::Utc::now().to_rfc3339();
    let mut attested: RemoteWorkspaceBinding = serde_json::from_value(serde_json::json!({
        "schema": BINDING_SCHEMA, "binding_id": "test-binding",
        "controller": {"daemon_identity": "controller", "controller_origin": "fixture"},
        "project": {"project_id": "demo", "repo_remote": "https://example.test/demo.git"},
        "transport": {"kind": "ssh", "host": "example.test", "user": "owner", "port": 22,
            "verified_at": now, "verification_evidence": ["fixture:probe"]},
        "roots": {"canonical_remote_root": "/work/demo"},
        "session": {"continuity_id": "demo-work", "principal": "demo-owner"},
        "state": {"status": BindingStatus::Verified}
    }))
    .unwrap();
    let root = std::env::temp_dir().join(format!(
        "focusa-remote-genesis-test-{}",
        uuid::Uuid::now_v7()
    ));
    let persistence =
        crate::runtime::persistence_sqlite::SqlitePersistence::new(&crate::types::FocusaConfig {
            data_dir: root.to_string_lossy().into_owned(),
            ..Default::default()
        })
        .unwrap();
    let mut ledger = rusqlite::Connection::open(root.join("focusa.sqlite")).unwrap();
    upsert_binding(&ledger, &attested).unwrap();
    ledger
        .execute("CREATE TABLE effects (id INTEGER)", [])
        .unwrap();
    let inspection = RemoteWorkspaceInspection {
        attested: attested.clone(),
        inventory: RemoteWorkspaceInventory {
            beads_present: true,
            beads_export: String::new(),
            git_directory: "/work/demo/.git".into(),
        },
    };
    let scope = RemoteOnboardingScope {
        binding_id: "test-binding",
        controller: &inspection.attested.controller,
        project: &inspection.attested.project,
        host: "example.test",
        user: "owner",
        port: 22,
        remote_root: "/work/demo",
        working_subpath: None,
        worktree_identity: None,
        continuity_id: "demo-work",
        principal: "demo-owner",
    };
    let max_age = chrono::TimeDelta::minutes(5);
    // Generic registry writes still accept candidate observations. A caller
    // claiming "verified" there has not performed a controller SSH probe.
    assert!(
        super::super::load_controller_attestation(
            &ledger,
            &scope,
            &inspection.attested,
            chrono::Utc::now(),
            max_age
        )
        .unwrap()
        .is_none()
    );

    assert!(
        inspection
            .commit(
                &mut ledger,
                &scope,
                max_age,
                |tx, inventory| -> Result<()> {
                    assert!(inventory.beads_present);
                    tx.execute("INSERT INTO effects VALUES (1)", [])?;
                    anyhow::bail!("fixture: rejected effect")
                }
            )
            .is_err()
    );
    assert_eq!(
        ledger
            .query_row("SELECT count(*) FROM effects", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    inspection
        .commit(&mut ledger, &scope, max_age, |tx, _| {
            tx.execute("INSERT INTO effects VALUES (1)", [])?;
            Ok(())
        })
        .unwrap();
    use super::super::{adopt_remote_bootstrap, read_remote_bootstrap};
    assert_eq!(
        read_remote_bootstrap(&ledger, &scope, &attested, chrono::Utc::now(), max_age).unwrap(),
        None
    );
    let receipt =
        adopt_remote_bootstrap(&mut ledger, &scope, &inspection, max_age, "first").unwrap();
    assert_eq!(
        adopt_remote_bootstrap(&mut ledger, &scope, &inspection, max_age, "first").unwrap(),
        receipt
    );
    assert!(adopt_remote_bootstrap(&mut ledger, &scope, &inspection, max_age, "other").is_err());
    assert_eq!(
        read_remote_bootstrap(&ledger, &scope, &attested, chrono::Utc::now(), max_age).unwrap(),
        Some(receipt.clone())
    );
    let receipt_id = receipt.receipt_id;
    use super::super::{read_remote_genesis_intent, stage_remote_genesis};
    assert_eq!(
        read_remote_genesis_intent(
            &ledger,
            &scope,
            &attested,
            chrono::Utc::now(),
            max_age,
            None
        )
        .unwrap(),
        None
    );
    let plan = serde_json::json!({"first_task":"demo-1"});
    let reserve = |ledger: &mut rusqlite::Connection, key, plan: &serde_json::Value| {
        stage_remote_genesis(ledger, &scope, &inspection, max_age, receipt_id, key, plan)
    };
    let intent = reserve(&mut ledger, "genesis", &plan).unwrap();
    assert_eq!(reserve(&mut ledger, "genesis", &plan).unwrap(), intent);
    let load_fence = |key| {
        super::super::RemoteGenesisFence::load(
            &ledger,
            &scope,
            &inspection.attested,
            max_age,
            key,
            inspection.inventory(),
        )
    };
    assert!(load_fence("wrong").is_err());
    let (checkpoint_fence, frozen_plan) = load_fence("genesis").unwrap();
    assert_eq!(frozen_plan, plan);
    assert_eq!(receipt.git_directory, "/work/demo/.git");
    let replacement_inventory = RemoteWorkspaceInventory {
        beads_present: true,
        beads_export: String::new(),
        git_directory: "/work/git/worktrees/replacement".into(),
    };
    let error = super::super::RemoteGenesisFence::load(
        &ledger,
        &scope,
        &inspection.attested,
        max_age,
        "genesis",
        &replacement_inventory,
    )
    .err()
    .expect("same path and origin must not authorize a different linked checkout");
    assert_eq!(error.to_string(), "remote_genesis_worktree_changed");

    let (tampered_fence, _) = load_fence("genesis").unwrap();
    let (revoked_fence, _) = load_fence("genesis").unwrap();
    let (independent_fence, _) = load_fence("genesis").unwrap();
    let mut candidate = crate::types::FocusaState {
        version: 42,
        ..Default::default()
    };
    let projection = intent.projection_key();
    assert!(!crate::scope_safety::classify_project_root(&projection).is_safe());
    let workpoint_id = uuid::Uuid::now_v7();
    let hlt =
        crate::types::HltLedgerEntry::new(projection.clone(), "Deliver demo".into(), "fixture", 1)
            .with_scope(Some(scope.continuity_id.to_string()), None);
    candidate
        .trajectory
        .records
        .push(crate::types::TrajectoryProjectionRecord {
            trajectory_id: "fixture-trajectory".into(),
            project_root: Some(projection.clone()),
            continuity_id: Some(scope.continuity_id.into()),
            canonical: true,
            active_workpoint_id: Some(workpoint_id),
            long_term_goal: hlt.new_hlt.clone(),
            ..Default::default()
        });
    candidate
        .workpoint
        .records
        .push(crate::types::WorkpointRecord {
            workpoint_id,
            project_root: Some(projection),
            continuity_id: Some(scope.continuity_id.into()),
            canonical: true,
            status: crate::types::WorkpointStatus::Active,
            work_item_id: Some("demo-1".into()),
            next_slice: Some("Verify first action".into()),
            ..Default::default()
        });
    let mut ladder = crate::types::TrajectoryLadderEvent::from_hlt_ledger(&hlt);
    ladder.trajectory_id = "fixture-trajectory".into();
    let publication = super::super::RemoteGenesisPublication {
        trajectory_id: ladder.trajectory_id.clone(),
        workpoint_id,
        hlt,
        ladder: vec![ladder],
    };
    let (invalid_fence, _) = load_fence("genesis").unwrap();
    assert!(
        persistence
            .persist_remote_genesis_checkpoint(
                &[],
                &Default::default(),
                &invalid_fence.with_publication(publication.clone())
            )
            .is_err()
    );
    let actor = crate::runtime::persistence_actor::PersistenceActor::start(persistence.clone());
    actor
        .persist_remote_genesis_checkpoint(
            vec![],
            candidate.clone(),
            checkpoint_fence.with_publication(publication.clone()),
        )
        .await
        .unwrap();
    let loaded = persistence.load_state().unwrap().unwrap();
    assert_eq!(loaded.version, 42);
    let (stored_intent, stored_publication) = super::super::read_remote_genesis_publication(
        &ledger,
        &scope,
        &inspection.attested,
        chrono::Utc::now(),
        max_age,
        Some("genesis"),
    )
    .unwrap()
    .unwrap();
    assert!(
        stored_publication
            .unwrap()
            .verifies(&loaded, &stored_intent, scope.continuity_id)
    );
    let (replay_fence, _) = load_fence("genesis").unwrap();
    assert!(
        persistence
            .persist_remote_genesis_checkpoint(
                &[],
                &candidate,
                &replay_fence.with_publication(publication)
            )
            .is_err()
    );
    let wrong_host = RemoteOnboardingScope {
        host: "other.example.test",
        ..scope
    };
    assert!(
        super::super::read_remote_genesis_publication(
            &ledger,
            &wrong_host,
            &inspection.attested,
            chrono::Utc::now(),
            max_age,
            None
        )
        .is_err()
    );

    assert_eq!(
        read_remote_genesis_intent(
            &ledger,
            &scope,
            &attested,
            chrono::Utc::now(),
            max_age,
            Some("genesis")
        )
        .unwrap(),
        Some(intent.clone())
    );
    assert!(
        read_remote_genesis_intent(
            &ledger,
            &scope,
            &attested,
            chrono::Utc::now(),
            max_age,
            Some("wrong")
        )
        .is_err()
    );
    assert_eq!(
        read_remote_genesis_intent(
            &ledger,
            &scope,
            &attested,
            chrono::Utc::now(),
            max_age,
            None
        )
        .unwrap(),
        Some(intent.clone())
    );
    assert!(
        reserve(
            &mut ledger,
            "genesis",
            &serde_json::json!({"first_task":"demo-2"})
        )
        .is_err()
    );
    assert!(reserve(&mut ledger, "other", &plan).is_err());
    ledger.execute(
        "UPDATE remote_genesis_intents SET intent_json = json_set(intent_json, '$.plan.first_task', 'other')",
        [],
    ).unwrap();
    assert!(
        read_remote_genesis_intent(
            &ledger,
            &scope,
            &attested,
            chrono::Utc::now(),
            max_age,
            None
        )
        .is_err()
    );
    assert!(reserve(&mut ledger, "genesis", &plan).is_err());
    assert!(
        persistence
            .persist_remote_genesis_checkpoint(&[], &candidate, &tampered_fence)
            .is_err()
    );
    ledger.execute("UPDATE remote_genesis_intents SET intent_json = json_set(intent_json, '$.plan.first_task', 'demo-1')", []).unwrap();
    attested.revoke("fixture", &now);
    upsert_binding(&ledger, &attested).unwrap();
    let replacement = crate::types::FocusaState {
        version: 43,
        ..Default::default()
    };
    assert!(
        actor
            .persist_remote_genesis_checkpoint(vec![], replacement, revoked_fence)
            .await
            .is_err()
    );
    assert_eq!(persistence.load_state().unwrap().unwrap().version, 42);
    let independent = crate::types::FocusaState {
        version: 44,
        ..Default::default()
    };
    use crate::types::{EventLogEntry, FocusaEvent, SignalOrigin};
    let event = || {
        EventLogEntry::with_temporal(
            FocusaEvent::TrajectoryGoalDefined {
                trajectory: Default::default(),
            },
            SignalOrigin::Adapter,
            None,
            Default::default(),
        )
    };
    let (ordinary, rejected) = tokio::join!(
        actor.persist_checkpoint(vec![event()], independent),
        actor.persist_remote_genesis_checkpoint(vec![event()], candidate, independent_fence),
    );
    ordinary.unwrap();
    assert!(rejected.is_err());
    assert_eq!(persistence.load_state().unwrap().unwrap().version, 44);
    assert_eq!(
        ledger
            .query_row("SELECT count(*) FROM event_hash_chain", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );

    assert!(
        read_remote_genesis_intent(
            &ledger,
            &scope,
            &inspection.attested,
            chrono::Utc::now(),
            max_age,
            None
        )
        .is_err()
    );
    assert!(reserve(&mut ledger, "genesis", &plan).is_err());
    assert!(
        read_remote_bootstrap(
            &ledger,
            &scope,
            &inspection.attested,
            chrono::Utc::now(),
            max_age
        )
        .is_err()
    );
    assert!(
        inspection
            .commit(&mut ledger, &scope, max_age, |_, _| -> Result<()> {
                panic!("revoked binding reached an effect");
            })
            .is_err()
    );
}

#[test]
fn root_is_a_literal_shell_argument() {
    assert!(
        inventory_command("/work/a'b", "owner")
            .unwrap()
            .contains("cd '/work/a'\"'\"'b'")
    );
    assert!(inventory_command("/work/a\nb", "owner").is_err());
    assert!(inventory_command("/", "owner").is_err());
}
