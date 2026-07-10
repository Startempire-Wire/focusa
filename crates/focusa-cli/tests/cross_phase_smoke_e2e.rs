//! Runs the Spec124 cross-phase CLI smoke shell suite under cargo test.

use std::path::Path;
use std::process::Command;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");

#[test]
fn spec124_cross_phase_cli_smoke_script_passes() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root");
    let script = repo_root.join("tests/spec_cli_cross_phase_smoke_test.sh");
    let output = Command::new("bash")
        .arg(script)
        .env("FOCUSA_BIN", FOCUSA_BIN)
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
