//! Spec143 Project Genesis isolated-daemon end-to-end proof.

use serde_json::Value;
use std::net::{TcpListener, TcpStream};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
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
    path_env: String,
}

impl IsolatedDaemon {
    fn restart(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.child = Command::new(&self.binary)
            .env("FOCUSA_BIND", &self.bind)
            .env("FOCUSA_DATA_DIR", &self.data_dir)
            .env("FOCUSA_TEST_MODE", "1")
            .env("PATH", &self.path_env)
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
    let fake_bin = data_dir.join("bin");
    std::fs::create_dir_all(&fake_bin).unwrap();
    let fake_bd = fake_bin.join("bd");
    std::fs::write(
        &fake_bd,
        r#"#!/usr/bin/env bash
set -euo pipefail
command="${1:-}"; shift || true
case "$command" in
  init)
    mkdir -p .beads
    : > .beads/issues.jsonl
    printf '{"status":"initialized"}\n'
    ;;
  create)
    title="${1:-task}"
    mkdir -p .beads
    touch .beads/issues.jsonl
    count=$(wc -l < .beads/issues.jsonl)
    id="fixture-$((count + 1))"
    printf '{"id":"%s","title":"%s","status":"open","priority":0}\n' "$id" "$title" >> .beads/issues.jsonl
    printf '{"id":"%s","title":"%s","status":"open"}\n' "$id" "$title"
    ;;
  dep)
    printf '{"status":"linked"}\n'
    ;;
  *)
    printf '{"status":"ok"}\n'
    ;;
esac
"#,
    )
    .unwrap();
    #[cfg(unix)]
    std::fs::set_permissions(&fake_bd, std::fs::Permissions::from_mode(0o755)).unwrap();
    let path_env = format!(
        "{}:{}",
        fake_bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
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
        .env("PATH", &path_env)
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
        path_env,
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

fn mutation_args(project_root: &str, confirm: bool) -> Vec<&str> {
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

fn bootstrap_args<'a>(project_root: &'a str, action: &'a str, confirm: bool) -> Vec<&'a str> {
    let mut args = vec![
        "project",
        "bootstrap",
        action,
        "--project-root",
        project_root,
        "--project-id",
        "bootstrap-e2e",
        "--canonical-name",
        "Bootstrap E2E",
        "--continuity-id",
        "bootstrap-e2e-continuity",
        "--idempotency-key",
        "bootstrap-e2e-key",
        "--discipline-profile",
        "standard_software_project",
        "--initialize-git",
        "true",
        "--initialize-task-provider",
        "true",
        "--hlt",
        "Ship the disciplined project",
        "--hlt-confirmed",
        "--specification-ref",
        "docs/01-bootstrap-e2e-spec.md",
        "--acceptance",
        "Project baseline is ready",
        "--acceptance",
        "First Workpoint is active",
        "--current-state",
        "Empty project",
        "--desired-end-state",
        "Disciplined project ready",
    ];
    if confirm {
        args.push("--confirm");
    }
    args.push("--json");
    args
}

#[test]
fn standard_bootstrap_is_previewable_local_only_idempotent_and_rollback_bounded() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let (daemon, base_url) = start_isolated_daemon(repo_root);
    let root = daemon.project_root.to_string_lossy().to_string();
    std::fs::remove_dir_all(&daemon.project_root).unwrap();

    let preview = json_output(&run(&base_url, &bootstrap_args(&root, "preview", false)));
    assert_eq!(find_string(&preview, "status"), Some("preview_ready"));
    assert!(!daemon.project_root.exists());

    let applied = json_output(&run(&base_url, &bootstrap_args(&root, "apply", true)));
    assert_eq!(find_string(&applied, "status"), Some("ready"));
    assert_eq!(find_string(&applied, "project_id"), Some("bootstrap-e2e"));
    assert_eq!(find_string(&applied, "identity_confidence"), Some("high"));
    assert_eq!(
        find_string(&applied, "marker_schema"),
        Some("focusa.project.v2")
    );
    let identity = json_output(&run(
        &base_url,
        &[
            "project",
            "identity",
            "--cwd",
            &root,
            "--project-root",
            &root,
            "--json",
        ],
    ));
    assert_eq!(find_string(&identity, "project_root"), Some(root.as_str()));
    let verification = json_output(&run(
        &base_url,
        &[
            "project",
            "verify",
            "--cwd",
            &root,
            "--project-root",
            &root,
            "--project-id",
            "bootstrap-e2e",
            "--canonical-name",
            "Bootstrap E2E",
            "--json",
        ],
    ));
    assert!(verification.to_string().contains("\"verified\":true"));
    assert!(daemon.project_root.join(".focusa-project.json").is_file());
    assert!(daemon.project_root.join(".git").is_dir());
    assert!(daemon.project_root.join(".beads").is_dir());
    assert!(daemon.project_root.join("docs").is_dir());
    let remotes = Command::new("git")
        .args(["remote"])
        .current_dir(&daemon.project_root)
        .output()
        .unwrap();
    assert!(
        remotes.stdout.is_empty(),
        "bootstrap must never create a remote"
    );

    let issues_path = daemon.project_root.join(".beads/issues.jsonl");
    let tasks_before_replay = std::fs::read_to_string(&issues_path)
        .unwrap()
        .lines()
        .count();
    let replay = json_output(&run(&base_url, &bootstrap_args(&root, "apply", true)));
    assert_eq!(find_string(&replay, "status"), Some("ready"));
    let tasks_after_replay = std::fs::read_to_string(&issues_path)
        .unwrap()
        .lines()
        .count();
    assert_eq!(tasks_after_replay, tasks_before_replay);

    let mut rollback_args = bootstrap_args(&root, "repair", true);
    let json_index = rollback_args.len() - 1;
    rollback_args.insert(json_index, "--repair-action");
    rollback_args.insert(json_index + 1, "rollback");
    let rollback = json_output(&run(&base_url, &rollback_args));
    assert_eq!(find_string(&rollback, "status"), Some("rolled_back"));
    assert!(!daemon.project_root.join(".git").exists());
    assert!(!daemon.project_root.join(".beads").exists());
    assert!(
        daemon
            .project_root
            .join(".focusa/bootstrap/receipt.json")
            .is_file()
    );
}

#[test]
fn temporal_authority_preserves_no_deadline_and_forecasts_from_observed_history() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let (mut daemon, base_url) = start_isolated_daemon(repo_root);
    let root = daemon.project_root.to_string_lossy().to_string();
    let continuity = "temporal-e2e";

    let empty = json_output(&run(
        &base_url,
        &[
            "temporal",
            "status",
            "--project-root",
            &root,
            "--continuity-id",
            continuity,
            "--json",
        ],
    ));
    assert!(empty.to_string().contains("\"deadline_status\":\"none\""));
    assert!(empty.to_string().contains("without fabricated urgency"));

    let committed = json_output(&run(
        &base_url,
        &[
            "temporal",
            "commit",
            "--project-root",
            &root,
            "--continuity-id",
            continuity,
            "--idempotency-key",
            "deadline-1",
            "--claim-id",
            "release-deadline",
            "--kind",
            "external_commitment",
            "--subject-ref",
            "release",
            "--target-at",
            "2030-08-01T17:00:00Z",
            "--timezone",
            "America/Los_Angeles",
            "--source",
            "operator",
            "--operator-confirmed",
            "--confidence",
            "verified",
            "--evidence-ref",
            "contract:release-date",
            "--confirm",
            "--json",
        ],
    ));
    assert_eq!(find_string(&committed, "status"), Some("completed"));

    for (index, duration) in [1000_u64, 2000, 3000].into_iter().enumerate() {
        let duration = duration.to_string();
        let key = format!("build-{index}");
        let evidence = format!("run:{index}");
        let observed = json_output(&run(
            &base_url,
            &[
                "temporal",
                "observe",
                "--project-root",
                &root,
                "--continuity-id",
                continuity,
                "--idempotency-key",
                &key,
                "--phase",
                "build",
                "--duration-ms",
                &duration,
                "--evidence-ref",
                &evidence,
                "--json",
            ],
        ));
        assert_eq!(find_string(&observed, "status"), Some("completed"));
    }
    daemon.restart();
    let forecast = json_output(&run(
        &base_url,
        &[
            "temporal",
            "forecast",
            "--project-root",
            &root,
            "--continuity-id",
            continuity,
            "--phase",
            "build",
            "--json",
        ],
    ));
    assert!(forecast.to_string().contains("empirical_nearest_rank"));
    assert!(forecast.to_string().contains("\"p50_ms\":2000"));
}
