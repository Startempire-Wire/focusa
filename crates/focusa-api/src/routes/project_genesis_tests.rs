use super::*;

fn test_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("focusa-genesis-{label}-{}", Uuid::now_v7()));
    fs::create_dir_all(&root).unwrap();
    root
}

fn complete_request(root: &Path) -> ProjectGenesisRequest {
    ProjectGenesisRequest {
        project_root: root.to_string_lossy().to_string(),
        continuity_id: "genesis-test-continuity".into(),
        idempotency_key: "genesis-test-key".into(),
        hlt: Some("Ship verified product capability".into()),
        hlt_confirmed: Some(true),
        desired_end_state: Some("Acceptance is proven end to end".into()),
        current_state: Some("Capability is absent".into()),
        specification_ref: Some(
            "docs/143-focusa-master-release-cycle-trajectory-genesis-flow-implementation-spec.md"
                .into(),
        ),
        acceptance_criteria: vec!["First Workpoint is active".into()],
        task_provider: Some("alternate_test_adapter".into()),
        tasks: vec![GenesisTaskInput {
            id: Some("task-ready".into()),
            title: "Activate first Workpoint".into(),
            status: Some("open".into()),
            priority: Some(0),
            ..GenesisTaskInput::default()
        }],
        ..ProjectGenesisRequest::default()
    }
}

#[test]
fn missing_hlt_enters_one_bounded_impasse() {
    let root = test_root("hlt-impasse");
    let mut request = complete_request(&root);
    request.hlt = None;
    request.hlt_confirmed = Some(false);
    let packet = build_staged_packet(&root, &request, None);
    assert_eq!(packet["status"], "hlt_impasse");
    assert_eq!(packet["hlt_status"], "missing_required");
    assert_eq!(
        packet["next_action"],
        "answer one concise HLT intent question"
    );
    assert!(
        packet["missing_links"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "hlt")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn complete_alternate_provider_chain_stages_first_workpoint() {
    let root = test_root("alternate-provider");
    let request = complete_request(&root);
    let packet = build_staged_packet(&root, &request, None);
    assert_eq!(packet["status"], "staged");
    assert_eq!(
        packet["task_provider_and_task_graph"]["provider"],
        "alternate_test_adapter"
    );
    assert_eq!(packet["first_workpoint_candidate"]["id"], "task-ready");
    assert!(packet["missing_links"].as_array().unwrap().is_empty());
    assert_eq!(packet["authority"]["operator_steering_precedence"], true);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn brownfield_beads_inventory_is_adopted_without_duplication() {
    let root = test_root("beads");
    fs::create_dir_all(root.join(".beads")).unwrap();
    fs::write(
        root.join(".beads/issues.jsonl"),
        concat!(
            "{\"id\":\"done\",\"title\":\"Done\",\"status\":\"closed\",\"priority\":0}\n",
            "{\"id\":\"ready\",\"title\":\"Ready task\",\"status\":\"open\",\"priority\":1}\n"
        ),
    )
    .unwrap();
    let mut request = complete_request(&root);
    request.task_provider = None;
    request.tasks.clear();
    let packet = build_staged_packet(&root, &request, None);
    assert_eq!(packet["task_provider_and_task_graph"]["provider"], "beads");
    assert_eq!(
        packet["task_provider_and_task_graph"]["tasks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(packet["first_workpoint_candidate"]["id"], "ready");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn numbered_spec_and_acceptance_are_discovered_for_ambient_bootstrap() {
    let root = test_root("spec-discovery");
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(
        root.join("docs/01-product-genesis-spec.md"),
        "# Product spec\n\n## Acceptance\n\n1. First Workpoint is active.\n2. Readiness receipt is durable.\n",
    )
    .unwrap();
    let mut request = complete_request(&root);
    request.specification_ref = None;
    request.acceptance_criteria.clear();
    let packet = build_staged_packet(&root, &request, None);
    assert_eq!(packet["status"], "staged");
    assert_eq!(
        packet["specification_and_acceptance"]["specification_ref"],
        "docs/01-product-genesis-spec.md"
    );
    assert_eq!(
        packet["specification_and_acceptance"]["acceptance_criteria"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn ids_are_stable_for_idempotent_resume() {
    let root = test_root("idempotency");
    let first = stable_id("genesis", &root, "same-key");
    let second = stable_id("genesis", &root, "same-key");
    assert_eq!(first, second);
    assert_eq!(
        stable_uuid(&root, "same-key"),
        stable_uuid(&root, "same-key")
    );
    assert_ne!(
        stable_uuid(&root, "same-key"),
        stable_uuid(&root, "other-key")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn coordination_conflict_uses_plain_language_and_confirmed_takeover() {
    let root = test_root("coordination-conflict");
    fs::write(
        root.join(".focusa-project.json"),
        serde_json::to_vec(&json!({
            "genesis_binding": {"status": "ready", "continuity_id": "other-continuity"}
        }))
        .unwrap(),
    )
    .unwrap();
    let request = complete_request(&root);
    let (status, Json(body)) = existing_readiness_gate(&root, &request).unwrap_err();
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["status"], "coordination_conflict");
    assert_eq!(body["choices"].as_array().unwrap().len(), 4);
    assert!(!body.to_string().contains("writer lease"));

    let mut takeover = request;
    takeover.takeover = Some(true);
    takeover.confirm = Some(true);
    assert!(existing_readiness_gate(&root, &takeover).unwrap().is_none());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn staged_genesis_enforces_continuity_ownership_after_reconnect() {
    let root = test_root("staged-ownership");
    let owner = complete_request(&root);
    let mut packet = build_staged_packet(&root, &owner, None);
    initialize_crist_state(&root, &mut packet).unwrap();
    write_json_atomic(&packet_path(&root), &packet).unwrap();

    let same_owner_start = existing_genesis_guard(&root, &owner, true)
        .unwrap()
        .expect("same-owner start must return the staged packet idempotently");
    assert_eq!(
        same_owner_start["ownership"]["continuity_id"],
        owner.continuity_id
    );
    assert!(
        existing_genesis_guard(&root, &owner, false)
            .unwrap()
            .is_none()
    );

    let mut other = complete_request(&root);
    other.continuity_id = "other-continuity".to_string();
    let (status, Json(body)) = existing_genesis_guard(&root, &other, false).unwrap_err();
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["existing_continuity_id"], owner.continuity_id);
    assert_eq!(body["requested_continuity_id"], "other-continuity");

    other.takeover = Some(true);
    other.confirm = Some(true);
    assert!(
        existing_genesis_guard(&root, &other, false)
            .unwrap()
            .is_none()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn crist_state_and_operating_profile_survive_disk_reconnect() {
    let root = test_root("crist-reconnect");
    let request = complete_request(&root);
    let mut packet = build_staged_packet(&root, &request, None);
    initialize_crist_state(&root, &mut packet).unwrap();
    assert_eq!(packet["crist_stage"], "context_collecting");
    assert_eq!(packet["revision"], 2);
    assert_eq!(
        packet["resolved_project_operating_profile"]["crist_state"]["stage"],
        "context_collecting"
    );
    assert_eq!(packet["ownership"]["continuity_id"], request.continuity_id);

    let receipt = record_crist_transition(&root, &mut packet, "context_ready", "accept_context")
        .expect("the next canonical C.R.I.S.T. transition must be accepted");
    assert_eq!(receipt["outcome"], "accepted");
    write_json_atomic(&packet_path(&root), &packet).unwrap();

    let reconnected = read_json(&packet_path(&root)).expect("persisted C.R.I.S.T. state");
    assert_eq!(reconnected["crist_stage"], "context_ready");
    assert_eq!(reconnected["revision"], 3);
    assert_eq!(
        reconnected["resolved_project_operating_profile"]["crist_state"]["stage"],
        "context_ready"
    );
    assert_eq!(
        reconnected["transition_receipts"].as_array().unwrap().len(),
        3
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_crist_transition_fails_closed_with_durable_receipt() {
    let root = test_root("crist-invalid-transition");
    let request = complete_request(&root);
    let mut packet = build_staged_packet(&root, &request, None);
    initialize_crist_state(&root, &mut packet).unwrap();
    let receipt = record_crist_transition(&root, &mut packet, "operational", "skip_to_operational")
        .expect_err("non-adjacent C.R.I.S.T. transition must fail closed");
    assert_eq!(receipt["outcome"], "rejected");
    assert_eq!(receipt["reason_code"], "invalid_crist_transition");
    let second_receipt =
        record_crist_transition(&root, &mut packet, "operational", "skip_to_operational")
            .expect_err("each repeated invalid attempt must fail closed");
    assert_ne!(receipt["receipt_id"], second_receipt["receipt_id"]);
    assert_eq!(second_receipt["attempt"], 4);
    for durable_receipt in [receipt, second_receipt] {
        let receipt_id = durable_receipt["receipt_id"].as_str().unwrap();
        assert!(
            root.join(".focusa/project-genesis/transition-receipts")
                .join(format!("{receipt_id}.json"))
                .is_file()
        );
    }
    assert_eq!(packet["crist_stage"], "context_collecting");
    assert_eq!(packet["revision"], 2);
    write_json_atomic(&packet_path(&root), &packet).unwrap();

    let reconnected = read_json(&packet_path(&root)).expect("receipt journal persists");
    assert_eq!(reconnected["crist_stage"], "context_collecting");
    assert_eq!(reconnected["revision"], 2);
    assert_eq!(
        reconnected["transition_receipts"][2]["reason_code"],
        "invalid_crist_transition"
    );
    assert_eq!(
        reconnected["transition_receipts"].as_array().unwrap().len(),
        4
    );
    assert_eq!(reconnected["receipts"].as_array().unwrap().len(), 5);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn marker_is_committed_only_after_ready_packet_in_source_contract() {
    let source = include_str!("project_genesis.rs");
    let ready = source
        .find("packet[\"status\"] = json!(\"ready\")")
        .unwrap();
    let packet_write = source[ready..]
        .find("write_json_atomic(&packet_path(&root), &packet)")
        .unwrap()
        + ready;
    let marker_write = source[packet_write..]
        .find("write_json_atomic(&marker_path, &marker)")
        .unwrap()
        + packet_write;
    assert!(ready < packet_write && packet_write < marker_write);
    assert!(source.contains("FocusaEvent::TrajectoryGoalDefined"));
    assert!(source.contains("FocusaEvent::WorkpointCheckpointPromoted"));
}
