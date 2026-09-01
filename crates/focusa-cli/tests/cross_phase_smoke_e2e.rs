//! Runs the Spec124 cross-phase CLI smoke shell suite under cargo test.

use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");

struct IsolatedDaemon {
    child: Child,
    data_dir: PathBuf,
}

impl Drop for IsolatedDaemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.data_dir);
    }
}

fn daemon_binary(repo_root: &Path) -> PathBuf {
    if let Some(explicit) = std::env::var_os("FOCUSA_DAEMON_BIN") {
        return explicit.into();
    }
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join("target"));
    target
        .join("debug")
        .join(format!("focusa-daemon{}", std::env::consts::EXE_SUFFIX))
}

fn start_isolated_daemon(repo_root: &Path) -> (IsolatedDaemon, String) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve isolated daemon port");
    let port = listener
        .local_addr()
        .expect("reserved local address")
        .port();
    drop(listener);

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let data_dir = std::env::temp_dir().join(format!(
        "focusa-cross-phase-smoke-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&data_dir).expect("create isolated daemon data dir");

    let bind = format!("127.0.0.1:{port}");
    let base_url = format!("http://{bind}");
    let binary = daemon_binary(repo_root);
    assert!(
        binary.is_file(),
        "isolated daemon binary missing at {}; cargo test --workspace must build focusa-daemon",
        binary.display()
    );

    let child = Command::new(&binary)
        .env("FOCUSA_BIND", &bind)
        .env("FOCUSA_DATA_DIR", &data_dir)
        .env("FOCUSA_TEST_MODE", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start isolated Focusa daemon");
    let mut daemon = IsolatedDaemon { child, data_dir };

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if TcpStream::connect(&bind).is_ok() {
            break;
        }
        if let Some(status) = daemon.child.try_wait().expect("inspect isolated daemon") {
            panic!("isolated Focusa daemon exited before readiness: {status}");
        }
        assert!(
            Instant::now() < deadline,
            "isolated Focusa daemon did not bind {bind} within 10 seconds"
        );
        thread::sleep(Duration::from_millis(50));
    }

    (daemon, base_url)
}

fn post_json(base_url: &str, path: &str, body: serde_json::Value) -> serde_json::Value {
    tokio::runtime::Runtime::new()
        .expect("test HTTP runtime")
        .block_on(async {
            reqwest::Client::new()
                .post(format!("{base_url}{path}"))
                .json(&body)
                .send()
                .await
                .expect("send test request")
                .error_for_status()
                .expect("successful test response")
                .json()
                .await
                .expect("test JSON response")
        })
}

#[test]
fn spec124_cross_phase_cli_smoke_script_passes() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root");
    let script = repo_root.join("tests/spec_cli_cross_phase_smoke_test.sh");
    let (_daemon, base_url) = start_isolated_daemon(repo_root);
    let output = Command::new("bash")
        .arg(script)
        .env("FOCUSA_BIN", FOCUSA_BIN)
        .env("FOCUSA_BASE_URL", base_url)
        .current_dir(repo_root)
        .output()
        .expect("cross-phase smoke script should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "cross-phase smoke failed\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}"
    );
    assert!(stdout.contains("Spec124 CLI cross-phase smoke complete"));
}

#[test]
fn detached_background_job_reuses_one_durable_row() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root");
    let (_daemon, base_url) = start_isolated_daemon(repo_root);
    let portable_cwd = std::env::temp_dir().to_string_lossy().into_owned();
    let name = format!("detach-e2e-{}", std::process::id());
    let dispatched = Command::new(FOCUSA_BIN)
        .args([
            "bg",
            "--json",
            "run",
            "--detach",
            "--name",
            &name,
            "--cwd",
            &portable_cwd,
            "--",
            FOCUSA_BIN,
            "--version",
        ])
        .env("FOCUSA_API_URL", &base_url)
        .output()
        .expect("dispatch detached background job");
    assert!(
        dispatched.status.success(),
        "dispatch failed status={}\nSTDOUT:\n{}\nSTDERR:\n{}",
        dispatched.status,
        String::from_utf8_lossy(&dispatched.stdout),
        String::from_utf8_lossy(&dispatched.stderr)
    );
    let receipt: serde_json::Value =
        serde_json::from_slice(&dispatched.stdout).expect("versioned dispatch receipt");
    assert_eq!(
        receipt["schema"],
        focusa_core::background_jobs::BACKGROUND_JOB_DISPATCH_SCHEMA
    );
    let job_id = receipt["job_id"]
        .as_str()
        .filter(|value| !value.is_empty())
        .expect("durable job id");

    let waited = Command::new(FOCUSA_BIN)
        .args([
            "bg",
            "--json",
            "wait",
            "--job",
            job_id,
            "--timeout-ms",
            "10000",
        ])
        .env("FOCUSA_API_URL", &base_url)
        .output()
        .expect("wait for detached background job");
    assert!(
        waited.status.success(),
        "wait failed: {}",
        String::from_utf8_lossy(&waited.stderr)
    );
    let wait_result: serde_json::Value =
        serde_json::from_slice(&waited.stdout).expect("wait result");
    assert_eq!(wait_result["status"], "done");
    assert_eq!(wait_result["job"]["job_id"], job_id);
    assert_eq!(wait_result["job"]["status"], "completed");
    assert_eq!(
        wait_result["completion_event"]["event_type"],
        focusa_core::background_jobs::BACKGROUND_JOB_COMPLETION_EVENT
    );
    assert_eq!(wait_result["completion_event"]["job_id"], job_id);

    let listed = Command::new(FOCUSA_BIN)
        .args(["bg", "--json", "list"])
        .env("FOCUSA_API_URL", &base_url)
        .output()
        .expect("list background jobs");
    assert!(listed.status.success());
    let list_result: serde_json::Value =
        serde_json::from_slice(&listed.stdout).expect("background job list");
    let matching = list_result["jobs"]
        .as_array()
        .expect("jobs array")
        .iter()
        .filter(|job| job["name"] == name)
        .collect::<Vec<_>>();
    assert_eq!(
        matching.len(),
        1,
        "detached monitor created a duplicate row"
    );
    assert_eq!(matching[0]["job_id"], job_id);

    let lost_name = format!("detach-launch-failed-e2e-{}", std::process::id());
    let lost_dispatch = Command::new(FOCUSA_BIN)
        .args([
            "bg",
            "--json",
            "run",
            "--detach",
            "--name",
            &lost_name,
            "--cwd",
            &portable_cwd,
            "--",
            "/definitely-not-a-focusa-command-390",
        ])
        .env("FOCUSA_API_URL", &base_url)
        .output()
        .expect("dispatch command that the monitor cannot spawn");
    assert!(lost_dispatch.status.success());
    let lost_receipt: serde_json::Value =
        serde_json::from_slice(&lost_dispatch.stdout).expect("launch-failed dispatch receipt");
    let lost_job_id = lost_receipt["job_id"]
        .as_str()
        .filter(|value| !value.is_empty())
        .expect("launch-failed durable job id");

    let lost_wait = Command::new(FOCUSA_BIN)
        .args([
            "bg",
            "--json",
            "wait",
            "--job",
            lost_job_id,
            "--timeout-ms",
            "10000",
        ])
        .env("FOCUSA_API_URL", &base_url)
        .output()
        .expect("wait for launch-failed background job");
    assert!(lost_wait.status.success());
    let lost_result: serde_json::Value =
        serde_json::from_slice(&lost_wait.stdout).expect("launch-failed wait result");
    assert_eq!(lost_result["status"], "done");
    assert_eq!(lost_result["job"]["job_id"], lost_job_id);
    assert_eq!(lost_result["job"]["status"], "failed");
    assert_eq!(lost_result["job"]["failure_class"], "launch_failed");
    assert_eq!(lost_result["job"]["exit_code"], 126);
    assert!(lost_result["job"]["completed_at"].is_string());
    assert!(
        lost_result["job"]["output_tail"]
            .as_str()
            .unwrap()
            .contains("[launch_failed:command_spawn]")
    );
    assert_eq!(
        lost_result["completion_event"]["event_type"],
        focusa_core::background_jobs::BACKGROUND_JOB_COMPLETION_EVENT
    );
    assert_eq!(lost_result["completion_event"]["job_id"], lost_job_id);
    assert_eq!(lost_result["completion_event"]["status"], "failed");
    assert_eq!(
        lost_result["completion_event"]["failure_class"],
        "launch_failed"
    );

    let direct_name = format!("direct-launch-failed-e2e-{}", std::process::id());
    let direct = Command::new(FOCUSA_BIN)
        .args([
            "bg",
            "--json",
            "run",
            "--name",
            &direct_name,
            "--cwd",
            &portable_cwd,
            "--",
            "/definitely-not-a-focusa-command-391",
        ])
        .env("FOCUSA_API_URL", &base_url)
        .output()
        .expect("run command that cannot spawn");
    assert!(
        !direct.status.success(),
        "unspawnable command must fail CLI"
    );
    let direct_failure: serde_json::Value =
        serde_json::from_slice(&direct.stdout).expect("structured direct launch error");
    assert_eq!(direct_failure["status"], "blocked");
    let direct_error = direct_failure["details"]["raw_error"]
        .as_str()
        .unwrap()
        .to_ascii_lowercase();
    assert!(
        direct_error.contains("no such file")
            || direct_error.contains("not found")
            || direct_error.contains("cannot find the file"),
        "unexpected platform spawn error: {direct_error}"
    );

    let listed = Command::new(FOCUSA_BIN)
        .args(["bg", "--json", "list"])
        .env("FOCUSA_API_URL", &base_url)
        .output()
        .expect("list direct launch failure");
    assert!(listed.status.success());
    let list_result: serde_json::Value =
        serde_json::from_slice(&listed.stdout).expect("background job list");
    let direct_job = list_result["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["name"] == direct_name)
        .expect("direct launch failure durable row");
    assert_eq!(direct_job["status"], "failed");
    assert_eq!(direct_job["failure_class"], "launch_failed");
    assert_eq!(direct_job["exit_code"], 126);
}

#[test]
fn stale_queued_creator_reconciles_through_normal_completion() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root");
    let (_daemon, base_url) = start_isolated_daemon(repo_root);
    let portable_cwd = std::env::temp_dir().to_string_lossy().into_owned();
    let name = format!("stale-queued-e2e-{}", std::process::id());
    let created = post_json(
        &base_url,
        "/v1/background-jobs",
        serde_json::json!({
            "name": name,
            "command": "never-started",
            "cwd": portable_cwd,
            "pid": u32::MAX,
        }),
    );
    let job_id = created["job"]["job_id"].as_str().expect("created job id");
    assert_eq!(created["job"]["status"], "queued");

    let waited = Command::new(FOCUSA_BIN)
        .args([
            "bg",
            "--json",
            "wait",
            "--job",
            job_id,
            "--timeout-ms",
            "10000",
        ])
        .env("FOCUSA_API_URL", &base_url)
        .output()
        .expect("reconcile stale queued creator");
    assert!(
        waited.status.success(),
        "stale reconciliation failed: {}",
        String::from_utf8_lossy(&waited.stderr)
    );
    let result: serde_json::Value =
        serde_json::from_slice(&waited.stdout).expect("stale reconciliation receipt");
    assert_eq!(result["status"], "done");
    assert_eq!(result["job"]["status"], "failed");
    assert_eq!(result["job"]["failure_class"], "launch_failed");
    assert_eq!(result["job"]["exit_code"], 126);
    assert!(result["job"]["completed_at"].is_string());
    assert_eq!(
        result["completion_event"]["event_type"],
        focusa_core::background_jobs::BACKGROUND_JOB_COMPLETION_EVENT
    );
    assert_eq!(result["completion_event"]["job_id"], job_id);
    assert_eq!(result["completion_event"]["failure_class"], "launch_failed");
}
