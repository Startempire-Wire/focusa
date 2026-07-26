//! Spec143 Project Genesis isolated-daemon end-to-end proof.

use serde_json::Value;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");

struct IsolatedDaemon {
    child: Child,
    data_dir: PathBuf,
    project_root: PathBuf,
    bind: String,
    binary: PathBuf,
}

impl IsolatedDaemon {
    fn restart(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.child = Command::new(&self.binary)
            .env("FOCUSA_BIND", &self.bind)
            .env("FOCUSA_DATA_DIR", &self.data_dir)
            .env("FOCUSA_TEST_MODE", "1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while TcpStream::connect(&self.bind).is_err() {
            if let Some(status) = self.child.try_wait().unwrap() {
                panic!("restarted isolated daemon exited: {status}");
            }
            assert!(
                Instant::now() < deadline,
                "restarted daemon readiness timeout"
            );
            thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Drop for IsolatedDaemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.data_dir);
        let _ = std::fs::remove_dir_all(&self.project_root);
    }
}

fn daemon_binary(repo_root: &Path) -> PathBuf {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join("target"));
    target
        .join("debug")
        .join(format!("focusa-daemon{}", std::env::consts::EXE_SUFFIX))
}

fn start_isolated_daemon(repo_root: &Path) -> (IsolatedDaemon, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let data_dir = std::env::temp_dir().join(format!(
        "focusa-genesis-daemon-{}-{nonce}",
        std::process::id()
    ));
    let project_root = std::env::temp_dir().join(format!(
        "focusa-genesis-project-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::create_dir_all(&project_root).unwrap();
    std::fs::write(
        project_root.join(".focusa-project.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema": "focusa.project.v2",
            "project_id": "genesis-e2e",
            "canonical_name": "Genesis E2E",
            "project_root": project_root,
        }))
        .unwrap(),
    )
    .unwrap();
    let bind = format!("127.0.0.1:{port}");
    let base_url = format!("http://{bind}");
    let binary = daemon_binary(repo_root);
    assert!(
        binary.is_file(),
        "daemon fixture missing: {}",
        binary.display()
    );
    let child = Command::new(&binary)
        .env("FOCUSA_BIND", &bind)
        .env("FOCUSA_DATA_DIR", &data_dir)
        .env("FOCUSA_TEST_MODE", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut daemon = IsolatedDaemon {
        child,
        data_dir,
        project_root,
        bind: bind.clone(),
        binary,
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    while TcpStream::connect(&bind).is_err() {
        if let Some(status) = daemon.child.try_wait().unwrap() {
            panic!("isolated daemon exited: {status}");
        }
        assert!(
            Instant::now() < deadline,
            "isolated daemon readiness timeout"
        );
        thread::sleep(Duration::from_millis(50));
    }
    (daemon, base_url)
}

fn run(base_url: &str, args: &[&str]) -> Output {
    let output = Command::new(FOCUSA_BIN)
        .args(args)
        .env("FOCUSA_BASE_URL", base_url)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "focusa {:?} failed\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn json_output(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "invalid JSON output ({error}): {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn find_string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    match value {
        Value::Object(map) => map
            .get(key)
            .and_then(Value::as_str)
            .or_else(|| map.values().find_map(|value| find_string(value, key))),
        Value::Array(values) => values.iter().find_map(|value| find_string(value, key)),
        _ => None,
    }
}

fn mutation_args<'a>(project_root: &'a str, confirm: bool) -> Vec<&'a str> {
    let mut args = vec![
        "project",
        "genesis",
        "start",
        "--project-root",
        project_root,
        "--continuity-id",
        "genesis-e2e-continuity",
        "--idempotency-key",
        "genesis-e2e-key",
        "--hlt",
        "Ship the verified Genesis journey",
        "--hlt-confirmed",
        "--specification-ref",
        "docs/143-focusa-master-release-cycle-trajectory-genesis-flow-implementation-spec.md",
        "--acceptance",
        "First Workpoint is active",
        "--current-state",
        "Genesis incomplete",
        "--desired-end-state",
        "Project ready",
        "--allow-task-decomposition",
    ];
    if confirm {
        args.push("--confirm");
    }
    args.push("--json");
    args
}

#[test]
fn genesis_commits_first_workpoint_atomically_and_replays_idempotently() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let (mut daemon, base_url) = start_isolated_daemon(repo_root);
    let root = daemon.project_root.to_string_lossy().to_string();

    let staged = json_output(&run(&base_url, &mutation_args(&root, false)));
    assert_eq!(find_string(&staged, "status"), Some("staged"));

    let mut commit_args = mutation_args(&root, true);
    commit_args[2] = "commit";
    let committed = json_output(&run(&base_url, &commit_args));
    assert_eq!(find_string(&committed, "status"), Some("ready"));
    let first_id = find_string(&committed, "workpoint_id").unwrap().to_string();

    let replay = json_output(&run(&base_url, &commit_args));
    assert_eq!(find_string(&replay, "status"), Some("ready"));
    assert_eq!(
        find_string(&replay, "workpoint_id"),
        Some(first_id.as_str())
    );

    let status = json_output(&run(
        &base_url,
        &[
            "project",
            "genesis",
            "status",
            "--project-root",
            &root,
            "--json",
        ],
    ));
    assert_eq!(find_string(&status, "status"), Some("ready"));

    daemon.restart();
    let warm_status = json_output(&run(
        &base_url,
        &[
            "project",
            "genesis",
            "status",
            "--project-root",
            &root,
            "--json",
        ],
    ));
    assert_eq!(
        find_string(&warm_status, "workpoint_id"),
        Some(first_id.as_str())
    );

    let marker: Value = serde_json::from_slice(
        &std::fs::read(daemon.project_root.join(".focusa-project.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(marker["genesis_binding"]["status"], "ready");
    assert_eq!(marker["genesis_binding"]["workpoint_id"], first_id);
}
