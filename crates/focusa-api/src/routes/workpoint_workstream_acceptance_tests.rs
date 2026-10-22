// Isolated acceptance through real handlers and SQLite, never production authority.
mod workstream_acceptance {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use focusa_core::scoped_state::{AttachmentKey, ScopeRef, WorkstreamKey};
    use tower::ServiceExt;

    async fn request(
        app: &axum::Router,
        root: &str,
        continuity: &str,
        path: &str,
        body: Value,
    ) -> (StatusCode, Value) {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/json")
                    .header("x-scope-project-root", root)
                    .header("x-scope-continuity-id", continuity)
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), 1_048_576)
            .await
            .unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    fn attachment(root: &str, continuity: &str, session: &str) -> AttachmentKey {
        AttachmentKey::new(
            WorkstreamKey::new(
                ScopeRef::project(
                    "project:workstream-acceptance",
                    root,
                    "Isolated acceptance fixture",
                    "fixture:workstream-acceptance",
                )
                .unwrap(),
                continuity,
            )
            .unwrap(),
            "fixture-instance",
            session,
            format!("fixture-attachment-{continuity}"),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn two_workstreams_accept_checkpoint_resume_and_durable_job_receipts() {
        let directory = tempfile::Builder::new()
            .prefix("focusa-workstream-acceptance-")
            .tempdir()
            .unwrap();
        let root = directory.path().canonicalize().unwrap();
        let root = root.to_string_lossy().to_string();
        let (state, persistence) = crate::routes::turn::tests::setup_state(None).await;
        std::fs::write(
            directory.path().join(".focusa-project.json"),
            serde_json::to_vec(&json!({
                "schema":"focusa.project.v1", "project_id":"workstream-acceptance",
                "project_root":root
            }))
            .unwrap(),
        )
        .unwrap();
        let mut digests = std::collections::BTreeMap::new();
        for continuity in ["acceptance-a", "acceptance-b"] {
            let (mut goal, mut events) = crate::routes::project::tests::workstream_marker_fixture(
                &format!("trajectory:{continuity}"),
                continuity,
            );
            goal.project_root = Some(root.clone());
            goal.canonical = true;
            for event in &mut events {
                event.project_root = root.clone();
            }
            state.focusa.write().await.trajectory.records.push(goal);
            persistence
                .append_trajectory_ladder_events(&events)
                .unwrap();
            let (status, Json(result)) = crate::routes::project::trajectory_guard(
                ScopeContext {
                    project_root: Some(root.clone()),
                    continuity_id: Some(continuity.into()),
                    ..Default::default()
                },
                State(state.clone()),
                Json(crate::routes::project::TrajectoryGuardRequest {
                    action: Some("migrate".into()),
                    project_root: Some(root.clone()),
                    continuity_id: Some(continuity.into()),
                    confirm: Some(true),
                    ..Default::default()
                }),
            )
            .await;
            assert!(status.is_success(), "fixture migration: {result}");
            digests.insert(
                continuity.to_string(),
                result["trajectory_binding"]["ledger_digest"].clone(),
            );
            assert!(
                digests[continuity].is_string(),
                "missing canonical ledger digest: {result}"
            );
        }
        let backup_names = || {
            let mut names = std::fs::read_dir(directory.path())
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
                .filter(|name| name.contains("backup"))
                .collect::<Vec<_>>();
            names.sort();
            names
        };
        let backups_before = backup_names();
        let app = axum::Router::new()
            .merge(super::super::router())
            .merge(crate::routes::background_jobs::router())
            .with_state(state.clone());
        let mut receipts = Vec::new();
        let mut checkpoint_ids = Vec::new();
        let sessions = [Uuid::now_v7().to_string(), Uuid::now_v7().to_string()];
        for round in 0..3 {
            for (index, continuity) in ["acceptance-a", "acceptance-b"].into_iter().enumerate() {
                let session = &sessions[index];
                let (guard_status, Json(guard)) = crate::routes::project::trajectory_guard(
                    ScopeContext {
                        project_root: Some(root.clone()),
                        continuity_id: Some(continuity.into()),
                        ..Default::default()
                    },
                    State(state.clone()),
                    Json(crate::routes::project::TrajectoryGuardRequest {
                        action: Some("verify".into()),
                        project_root: Some(root.clone()),
                        continuity_id: Some(continuity.into()),
                        ..Default::default()
                    }),
                )
                .await;
                assert!(guard_status.is_success(), "verify {continuity}: {guard}");
                assert_eq!(
                    guard["trajectory_integrity_guard"]["status"], "READY",
                    "{guard}"
                );
                assert_eq!(
                    guard["trajectory_binding"]["ledger_digest"],
                    digests[continuity]
                );
                let task = format!("fixture-task-{continuity}");
                let (status, checkpoint) = request(
                    &app, &root, continuity, "/v1/workpoint/checkpoint",
                    json!({
                        "project_root":root, "continuity_id":continuity,
                        "work_item_id":task, "checkpoint_reason":"operator_checkpoint",
                        "session_id":session,
                        "mission":format!("Prepare isolated source verification for {continuity}"),
                        "next_slice":format!("Observe isolated source verification for {continuity}"),
                        "source_turn_id":format!("fixture:{continuity}:{round}"),
                        "action_intent":{
                            "action_type":"prepare_current_source", "target_ref":root,
                            "lifecycle_stage":"prepare"
                        },
                        "canonical":true, "promote":true,
                        "idempotency_key":format!("fixture:{continuity}:{round}")
                    }),
                ).await;
                assert!(
                    status.is_success(),
                    "checkpoint {continuity}/{round}: {checkpoint}"
                );
                assert_eq!(checkpoint["canonical"], true, "{checkpoint}");
                checkpoint_ids.push(checkpoint["workpoint_id"].clone());
                let (status, resumed) = request(
                    &app, &root, continuity, "/v1/workpoint/resume",
                    json!({"project_root":root,"continuity_id":continuity,"session_id":session,"mode":"full_json"}),
                ).await;
                assert!(status.is_success(), "resume {continuity}: {resumed}");
                assert_eq!(resumed["canonical"], true, "{resumed}");
                assert_eq!(
                    resumed["workpoint_id"], checkpoint["workpoint_id"],
                    "{resumed}"
                );
                let key = attachment(&root, continuity, session);
                let (status, created) = request(
                    &app,
                    &root,
                    continuity,
                    "/v1/background-jobs",
                    json!({
                        "name":format!("isolated acceptance {continuity}/{round}"),
                        "command":format!("printf {continuity}"), "cwd":root,
                        "attachment":key, "pid":std::process::id()
                    }),
                )
                .await;
                assert!(status.is_success(), "job admission {continuity}: {created}");
                let id = created["job"]["job_id"].as_str().expect("durable job id");
                // Only a real, bounded successful child produces the completion input.
                let output = std::process::Command::new("printf")
                    .arg(continuity)
                    .output()
                    .unwrap();
                assert!(output.status.success());
                let text = String::from_utf8(output.stdout).unwrap();
                assert_eq!(text, continuity);
                let (status, completed) = request(
                    &app,
                    &root,
                    continuity,
                    &format!("/v1/background-jobs/{id}/complete"),
                    json!({"exit_code":output.status.code().unwrap(), "output_tail":text}),
                )
                .await;
                assert!(status.is_success(), "completion {continuity}: {completed}");
                assert_eq!(completed["job"]["status"], "completed", "{completed}");
                assert_eq!(
                    completed["job"]["attachment"],
                    serde_json::to_value(&key).unwrap()
                );
                receipts.push((id.to_string(), continuity.to_string()));
            }
        }
        // Read durable results from a new database connection, not response echoes.
        let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
        let connection = rusqlite::Connection::open(path).unwrap();
        let mut durable_rows = Vec::new();
        for (id, continuity) in &receipts {
            let row = focusa_core::background_job_store::load_job(&connection, id)
                .unwrap()
                .unwrap();
            assert_eq!(
                row.status,
                focusa_core::background_jobs::BackgroundJobStatus::Completed
            );
            assert_eq!(
                row.attachment.as_ref().unwrap().workstream.continuity_id,
                *continuity
            );
            durable_rows.push(serde_json::to_value(row).unwrap());
        }
        assert_eq!(receipts.len(), 6);
        assert_eq!(checkpoint_ids.len(), 6);
        assert_eq!(backup_names(), backups_before);
        let (status, refusal) = request(
            &app,
            &root,
            "unknown-continuity",
            "/v1/background-jobs",
            json!({"name":"must refuse unknown scope","command":"printf forbidden", "cwd":root,
                "attachment":attachment(&root,"unknown-continuity",&Uuid::now_v7().to_string())}),
        )
        .await;
        assert!(!status.is_success(), "unknown scope admitted: {refusal}");
        if let Ok(path) = std::env::var("FOCUSA_WORKSTREAM_ACCEPTANCE_EVIDENCE") {
            std::fs::write(
                path,
                serde_json::to_vec_pretty(&json!({
                    "schema":"focusa.workstream.acceptance.source_fixture.v1",
                    "proof_scope":"isolated_source_fixture_not_production_authority",
                    "checkpoint_ids":checkpoint_ids,"goal_digests":digests,
                    "durable_terminal_jobs":durable_rows,"backup_names_unchanged":true,
                    "unknown_scope_refused":true
                }))
                .unwrap(),
            )
            .unwrap();
        }
    }
}
