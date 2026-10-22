// Real SQLite rejection, isolated fixture only; no production storage is touched.
#[tokio::test]
async fn rejected_context_write_preserves_memory_and_emits_no_success() {
    let (state, persistence) = crate::routes::turn::tests::setup_state(None).await;
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let key = WorkstreamKey::new(
        focusa_core::scoped_state::ScopeRef::project(
            "project:write-rejection",
            root,
            "Write rejection fixture",
            "fixture:write-rejection",
        )
        .unwrap(),
        "write-rejection-continuity",
    )
    .unwrap();
    {
        let mut current = state.focusa.write().await;
        current.work_loop.execution_scope = Some(key.clone());
        current.work_loop.execution_work_item_id = Some("fixture-write-rejection-task".into());
    }
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let connection = rusqlite::Connection::open(path).unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER fixture_reject_context_write BEFORE INSERT ON events
         BEGIN SELECT RAISE(ABORT, 'disk quota exceeded: isolated write-rejection fixture'); END;",
        )
        .unwrap();
    let before = serde_json::to_value(&*state.focusa.read().await).unwrap();
    let count_before = persistence.event_count().unwrap();
    let mut messages = state.events_tx.subscribe();
    let request = || {
        serde_json::from_value::<DecisionContextRequest>(json!({
            "current_ask":"Verify an actual rejected source-fixture write",
            "operator_steering_detected":true
        }))
        .unwrap()
    };
    let result = set_decision_context(
        WorkLoopScope(key.clone()),
        State(state.clone()),
        HeaderMap::new(),
        Json(request()),
    )
    .await;
    assert!(
        result.is_err(),
        "storage rejection was reported as canonical success: {result:?}"
    );
    let (status, Json(failure)) = result.unwrap_err();
    assert!(status.is_server_error(), "{failure}");
    assert!(
        failure.to_string().contains("disk quota exceeded"),
        "{failure}"
    );
    assert_eq!(failure["failure_class"], "persistence_unavailable");
    assert_eq!(failure["retry_posture"], "do_not_retry_unchanged");
    assert!(!failure.to_string().contains("no state was lost"));
    assert_eq!(
        serde_json::to_value(&*state.focusa.read().await).unwrap(),
        before
    );
    assert_eq!(persistence.event_count().unwrap(), count_before);
    assert!(matches!(
        messages.try_recv(),
        Err(tokio::sync::broadcast::error::TryRecvError::Empty)
    ));
    assert!(state.writer_claims.read().await.is_empty());
    connection
        .execute_batch("DROP TRIGGER fixture_reject_context_write;")
        .unwrap();
    let Json(accepted) = set_decision_context(
        WorkLoopScope(key),
        State(state.clone()),
        HeaderMap::new(),
        Json(request()),
    )
    .await
    .unwrap();
    assert_eq!(accepted["status"], "accepted");
    assert_eq!(accepted["writer_id"], Value::Null);
    assert_eq!(persistence.event_count().unwrap(), count_before + 1);
    assert_eq!(
        state
            .focusa
            .read()
            .await
            .work_loop
            .decision_context
            .current_ask
            .as_deref(),
        Some("Verify an actual rejected source-fixture write")
    );
    assert!(messages.try_recv().is_ok());
    assert!(state.writer_claims.read().await.is_empty());
}
