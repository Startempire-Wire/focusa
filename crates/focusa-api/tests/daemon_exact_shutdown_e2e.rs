use focusa_core::daemon_lifecycle::{
    DaemonLockRecord, DaemonProcessIdentity, DaemonShutdownRequest,
};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

struct TestDaemon {
    child: Child,
    data_dir: PathBuf,
    log_path: PathBuf,
    base_url: String,
}

impl Drop for TestDaemon {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            if let Err(error) = self.child.kill() {
                eprintln!("test daemon cleanup signal failed: {error}");
            }
            if let Err(error) = self.child.wait() {
                eprintln!("test daemon cleanup wait failed: {error}");
            }
        }
        if self.data_dir.exists()
            && let Err(error) = std::fs::remove_dir_all(&self.data_dir)
        {
            eprintln!("test daemon fixture cleanup failed: {error}");
        }
    }
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn spawn_daemon(label: &str) -> TestDaemon {
    let data_dir = std::env::temp_dir().join(format!(
        "focusa-exact-shutdown-{label}-{}",
        uuid::Uuid::now_v7()
    ));
    spawn_daemon_at(data_dir)
}

fn spawn_daemon_at(data_dir: PathBuf) -> TestDaemon {
    let port = free_port();
    std::fs::create_dir_all(&data_dir).unwrap();
    let log_path = data_dir.join("daemon.log");
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .unwrap();
    let home = data_dir.join("fixture-home");
    std::fs::create_dir_all(&home).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_focusa-daemon"))
        .env_clear()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("FOCUSA_TEST_MODE", "1")
        .env("FOCUSA_BIND", format!("127.0.0.1:{port}"))
        .env("FOCUSA_DATA_DIR", &data_dir)
        .env("RUST_LOG", "focusa=info")
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    TestDaemon {
        child,
        data_dir,
        log_path,
        base_url: format!("http://127.0.0.1:{port}"),
    }
}

async fn health(client: &reqwest::Client, daemon: &TestDaemon) -> DaemonProcessIdentity {
    for _ in 0..100 {
        if let Ok(response) = client
            .get(format!("{}/v1/health", daemon.base_url))
            .send()
            .await
            && response.status().is_success()
        {
            let value: serde_json::Value = response.json().await.unwrap();
            return serde_json::from_value(value["daemon"].clone()).unwrap();
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("test daemon did not become ready");
}

async fn exact_shutdown(
    client: &reqwest::Client,
    daemon: &TestDaemon,
    identity: &DaemonProcessIdentity,
) {
    let lock = DaemonLockRecord::parse(
        &std::fs::read_to_string(daemon.data_dir.join("focusa-daemon.lock")).unwrap(),
    )
    .unwrap();
    assert_eq!(lock.pid, identity.pid);
    assert_eq!(lock.start_token, identity.start_token);
    let response = client
        .post(format!("{}/v1/shutdown", daemon.base_url))
        .bearer_auth(lock.shutdown_token)
        .json(&DaemonShutdownRequest::new(
            identity.pid,
            identity.start_token.clone(),
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::ACCEPTED);
}

async fn wait_for_exit(daemon: &mut TestDaemon) {
    for _ in 0..100 {
        if daemon.child.try_wait().unwrap().is_some() {
            let log = std::fs::read_to_string(&daemon.log_path).unwrap();
            assert!(
                log.contains("Focusa daemon shutdown persistence flush complete"),
                "daemon exited without the durable shutdown flush receipt:\n{log}"
            );
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let log = std::fs::read_to_string(&daemon.log_path).unwrap_or_default();
    panic!("exact daemon did not exit after accepted shutdown:\n{log}");
}

#[tokio::test]
async fn stopping_one_exact_daemon_preserves_the_other_instance() {
    let client = reqwest::Client::new();
    let mut daemon_a = spawn_daemon("a");
    let mut daemon_b = spawn_daemon("b");
    let identity_a = health(&client, &daemon_a).await;
    let identity_b = health(&client, &daemon_b).await;
    assert_ne!(identity_a.pid, identity_b.pid);
    assert_ne!(identity_a.start_token, identity_b.start_token);

    exact_shutdown(&client, &daemon_a, &identity_a).await;
    wait_for_exit(&mut daemon_a).await;

    let identity_b_after = health(&client, &daemon_b).await;
    assert_eq!(identity_b_after, identity_b);
    assert!(daemon_b.child.try_wait().unwrap().is_none());
    assert!(!daemon_a.data_dir.join("focusa-daemon.lock").exists());
    assert!(daemon_b.data_dir.join("focusa-daemon.lock").exists());

    exact_shutdown(&client, &daemon_b, &identity_b).await;
    wait_for_exit(&mut daemon_b).await;
}

#[tokio::test]
async fn fresh_daemon_reads_native_full_and_incremental_temporal_work_item_restore() {
    use focusa_core::runtime::backup::{
        BACKUP_POLICY_SCHEMA, BackupPolicy, create_full_generation,
    };
    use focusa_core::runtime::backup_contracts::TemporalWorkItemRestoreCoverage;
    use focusa_core::runtime::backup_incremental::create_incremental_generation;
    use focusa_core::runtime::backup_restore::restore_generation;
    use focusa_core::runtime::persistence_sqlite::SqlitePersistence;
    use focusa_core::runtime::temporal_work_items::WorkItem;
    use focusa_core::types::FocusaConfig;

    let fixture = tempfile::tempdir().unwrap();
    let source = fixture.path().join("source");
    let store = SqlitePersistence::new(&FocusaConfig {
        data_dir: source.to_string_lossy().into_owned(),
        ..Default::default()
    })
    .unwrap();
    let policy = BackupPolicy {
        schema: BACKUP_POLICY_SCHEMA.into(),
        enabled: true,
        backup_root: fixture.path().join("backups"),
        rpo_seconds: 900,
        rto_seconds: 7_200,
        full_interval_seconds: 3_600,
        incremental_interval_seconds: 900,
        keep_hourly: 24,
        keep_daily: 14,
        keep_weekly: 8,
        keep_monthly: 12,
        restore_interval_seconds: 604_800,
        local_required: true,
        off_host_required: true,
        off_host_remote: None,
        min_free_bytes: 0,
        min_free_percent: 0,
        max_concurrent_operations: 1,
        compression: "zstd".into(),
        compression_level: 1,
        incremental_strategy: "experimental_full_snapshot_chunks_v0".into(),
        policy_digest: "isolated-test-policy".into(),
    };
    let project = fixture.path().join("project");
    std::fs::create_dir(&project).unwrap();
    let root = project.to_str().unwrap();
    let continuity = "isolated-restore-drill";
    let now = chrono::Utc::now();
    let first = WorkItem {
        item_id: "first-item".into(),
        project_root: root.into(),
        continuity_id: continuity.into(),
        title: "First item".into(),
        status: "in_progress".into(),
        revision: 2,
        estimate_ms: Some(60_000),
        created_at: now,
        started_at: Some(now),
        paused_at: None,
        completed_at: None,
        evidence_refs: vec!["evidence:started".into()],
        last_reason: None,
        idempotency_keys: vec!["create-first".into(), "start-first".into()],
    };
    let mut snapshot = store.read_temporal_work_items(root, continuity).unwrap();
    snapshot.items.push(first.clone());
    assert!(store.commit_temporal_work_items(snapshot).unwrap());
    let db = store.database_path();
    let full = create_full_generation(&db, &policy, "full", "test").unwrap();
    let mut snapshot = store.read_temporal_work_items(root, continuity).unwrap();
    snapshot.items[0].status = "completed".into();
    snapshot.items[0].revision = 3;
    snapshot.items[0].completed_at = Some(now);
    snapshot.items[0]
        .evidence_refs
        .push("evidence:completed".into());
    snapshot.items[0]
        .idempotency_keys
        .push("complete-first".into());
    snapshot.items.push(WorkItem {
        item_id: "second-item".into(),
        title: "Second item".into(),
        status: "planned".into(),
        revision: 1,
        started_at: None,
        evidence_refs: vec!["evidence:planned".into()],
        idempotency_keys: vec!["create-second".into()],
        ..first.clone()
    });
    let latest = snapshot.items.clone();
    assert!(store.commit_temporal_work_items(snapshot).unwrap());
    let incremental = create_incremental_generation(&db, &policy, "incremental", "test").unwrap();
    drop(store);
    std::fs::remove_dir_all(source).unwrap();

    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    for (generation, expected) in [(full, vec![first]), (incremental, latest)] {
        let cold_dir = fixture.path().join(&generation.slot_id);
        let receipt = restore_generation(
            &policy.backup_root,
            &generation.generation_id,
            &cold_dir.join("focusa.sqlite"),
            policy.rto_seconds,
        )
        .unwrap();
        assert_eq!(
            receipt.temporal_work_item_coverage,
            TemporalWorkItemRestoreCoverage::CanonicalSqliteV1
        );
        assert!(!cold_dir.join("temporal-work-items").exists());
        let mut daemon = spawn_daemon_at(cold_dir);
        let identity = health(&client, &daemon).await;
        let url = format!("{}/v1/workpoint/items", daemon.base_url);
        for (continuity_id, items) in [(continuity, expected), ("different-continuity", vec![])] {
            let actual: serde_json::Value = client
                .get(&url)
                .query(&[("project_root", root), ("continuity_id", continuity_id)])
                .send()
                .await
                .unwrap()
                .error_for_status()
                .unwrap()
                .json()
                .await
                .unwrap();
            assert_eq!(actual["canonical"], true);
            assert_eq!(actual["items"], serde_json::to_value(items).unwrap());
        }
        exact_shutdown(&client, &daemon, &identity).await;
        wait_for_exit(&mut daemon).await;
    }
}
