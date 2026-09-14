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
    let port = free_port();
    let data_dir = std::env::temp_dir().join(format!(
        "focusa-exact-shutdown-{label}-{}",
        uuid::Uuid::now_v7()
    ));
    std::fs::create_dir_all(&data_dir).unwrap();
    let log_path = data_dir.join("daemon.log");
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_focusa-daemon"))
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
