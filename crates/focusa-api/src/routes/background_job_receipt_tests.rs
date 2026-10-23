#[tokio::test]
async fn receipt_shape_handlers_preserve_running_job_until_real_durable_completion() {
    let (state, _) = crate::routes::turn::tests::setup_state(None).await;
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let conn = rusqlite::Connection::open(&path).unwrap();
    focusa_core::background_job_store::ensure_schema(&conn).unwrap();
    let running = BackgroundJobRecord {
        schema: BACKGROUND_JOB_SCHEMA.into(),
        job_id: "receipt-shape-api".into(),
        name: "synthetic receipt shape".into(),
        command: "synthetic-only-not-executed".into(),
        cwd: state.config.data_dir.clone(),
        attachment: None,
        status: BackgroundJobStatus::Running,
        failure_class: None,
        exit_code: None,
        pid: None,
        process_start_token: None,
        log_path: std::path::Path::new(&state.config.data_dir)
            .join("synthetic.log")
            .to_string_lossy()
            .to_string(),
        started_at: now_iso(),
        completed_at: None,
        output_tail: String::new(),
    };
    focusa_core::background_job_store::upsert_job(&conn, &running).unwrap();
    let mut events = state.events_tx.subscribe();
    for (status, expected_cause) in [
        ("completed", "requires completed_at and exit_code"),
        ("failed", "requires completed_at and exit_code"),
        ("queued", "newer lifecycle state"),
        ("unknown-status", "newer lifecycle state"),
    ] {
        let Json(reply) = update_job(
            State(state.clone()),
            axum::extract::Path(running.job_id.clone()),
            Json(UpdateJobBody {
                status: Some(status.into()),
                pid: None,
                process_start_token: None,
            }),
        )
        .await;
        assert!(reply.get("job").is_none(), "false update success: {reply}");
        assert!(reply.to_string().contains(expected_cause), "{reply}");
        assert_eq!(
            focusa_core::background_job_store::load_job(&conn, &running.job_id).unwrap(),
            Some(running.clone())
        );
    }
    for status in ["queued", "running", "unknown-status"] {
        let Json(reply) = complete_job(
            State(state.clone()),
            axum::extract::Path(running.job_id.clone()),
            Json(CompleteJobBody {
                exit_code: 0,
                status: Some(status.into()),
                failure_class: None,
                output_tail: "not a terminal receipt".into(),
            }),
        )
        .await;
        assert!(
            reply.get("job").is_none(),
            "false completion success: {reply}"
        );
        assert!(
            reply
                .to_string()
                .contains("nonterminal background job cannot carry completion fields")
        );
        assert_eq!(
            focusa_core::background_job_store::load_job(&conn, &running.job_id).unwrap(),
            Some(running.clone())
        );
    }
    assert!(matches!(
        events.try_recv(),
        Err(tokio::sync::broadcast::error::TryRecvError::Empty)
    ));
    let Json(reply) = complete_job(
        State(state.clone()),
        axum::extract::Path(running.job_id.clone()),
        Json(CompleteJobBody {
            exit_code: 0,
            status: None,
            failure_class: None,
            output_tail: "durable terminal result".into(),
        }),
    )
    .await;
    assert_eq!(reply["status"], "completed");
    let fresh = rusqlite::Connection::open(path).unwrap();
    let saved = focusa_core::background_job_store::load_job(&fresh, &running.job_id)
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, BackgroundJobStatus::Completed);
    assert_eq!(saved.exit_code, Some(0));
    assert!(saved.completed_at.is_some());
    assert_eq!(saved.output_tail, "durable terminal result");
    assert_eq!(serde_json::to_value(saved).unwrap(), reply["job"]);
    let event: Value = serde_json::from_str(&events.try_recv().unwrap()).unwrap();
    assert_eq!(event, reply["completion_event"]);
    assert!(matches!(
        events.try_recv(),
        Err(tokio::sync::broadcast::error::TryRecvError::Empty)
    ));
}
