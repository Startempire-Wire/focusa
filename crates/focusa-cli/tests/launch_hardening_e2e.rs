//! Spec124 Order 11 launch-hardening regression coverage.

use std::fs;
use std::path::Path;
use std::process::Command;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("focusa repo root")
}

fn read_repo(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"))
}

fn run(args: &[&str]) -> (std::process::Output, String) {
    let output = Command::new(FOCUSA_BIN)
        .args(args)
        .output()
        .expect("focusa CLI should execute");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output, format!("{stdout}{stderr}"))
}

#[test]
fn route_parity_has_post_workpoint_resume_and_telemetry_snapshot() {
    let workpoint = read_repo("crates/focusa-api/src/routes/workpoint.rs");
    assert!(
        workpoint.contains(".route(\"/v1/workpoint/resume\", post(resume))"),
        "workpoint resume must remain POST-only route parity"
    );

    let telemetry = read_repo("crates/focusa-api/src/routes/telemetry.rs");
    assert!(
        telemetry.contains("/v1/telemetry/snapshot") && telemetry.contains("telemetry_snapshot"),
        "telemetry snapshot route must exist for TUI/menubar consumers"
    );

    let tui = read_repo("crates/focusa-cli/src/commands/tui.rs");
    assert!(
        tui.contains("/v1/workpoint/resume") && tui.contains("fetch_post"),
        "TUI headless path must POST workpoint resume"
    );
}

#[test]
fn stop_command_exposes_distinct_statuses() {
    let daemon = read_repo("crates/focusa-cli/src/commands/daemon.rs");
    let main = read_repo("crates/focusa-cli/src/main.rs");
    assert!(daemon.contains("StopOutcome::Stopped"));
    assert!(daemon.contains("StopOutcome::AlreadyStopped"));
    assert!(daemon.contains("authenticated exact-daemon shutdown request failed"));
    assert!(daemon.contains("daemon returned an invalid shutdown acceptance receipt"));
    assert!(daemon.contains("exact Focusa daemon instance still responds after shutdown timeout"));
    assert!(main.contains("already_stopped"));
    assert!(main.contains("Focusa daemon already stopped (no-op)"));
}

#[test]
fn uninstall_keep_flags_are_wired_into_dry_run_plan() {
    let (output, out) = run(&[
        "--json",
        "uninstall",
        "--dry-run",
        "--keep-data",
        "--keep-license",
        "--keep-path-modifications",
    ]);
    assert!(output.status.success(), "uninstall dry-run failed: {out}");
    assert!(
        out.contains("remove_install_artifacts"),
        "missing managed-software removal step: {out}"
    );
    assert!(
        !out.contains("\"name\": \"remove_install_root\""),
        "keep-data must not schedule full customer-data removal: {out}"
    );
    assert!(
        out.contains("--keep-data set"),
        "keep-data not reflected: {out}"
    );
    assert!(
        out.contains("--keep-license set"),
        "keep-license not reflected: {out}"
    );
    assert!(
        !out.contains("revert_path_"),
        "keep-path-modifications should remove PATH revert steps: {out}"
    );
}

#[test]
fn expired_pairing_rooms_are_cleaned_before_startup_rehydrate() {
    let persistence = read_repo("crates/focusa-core/src/runtime/persistence_sqlite.rs");
    assert!(
        persistence.contains("cleanup_expired_pairing_rooms")
            && persistence.contains(
                "DELETE FROM connect_sessions WHERE expires_at <= ?1 AND status != 'completed'"
            ),
        "persistence must delete expired incomplete pairing rooms"
    );

    let server = read_repo("crates/focusa-api/src/server.rs");
    assert!(
        server.contains("cleanup_expired_pairing_rooms")
            && server.contains("expired pairing rooms cleaned during startup rehydrate"),
        "server startup must run expired pairing-room cleanup before rehydrate"
    );
}

#[test]
fn focusa_no_decay_tick_is_documented_if_runtime_supported() {
    let runtime = read_repo("crates/focusa-core/src/runtime/daemon.rs");
    assert!(
        runtime.contains("FOCUSA_NO_DECAY_TICK"),
        "runtime currently supports the decay-tick escape hatch"
    );
    let docs = read_repo("docs/current/CLI_REFERENCE_CURRENT.md");
    assert!(
        docs.contains("FOCUSA_NO_DECAY_TICK=1") && docs.contains("disable the memory decay tick")
    );
}
