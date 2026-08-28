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
            "/tmp",
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
}
