//! Runs the Spec128 installer/update runtime shell suite under cargo test.

use std::path::Path;
use std::process::Command;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");

#[test]
fn spec128_installer_update_runtime_script_passes() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root");
    let script = repo_root.join("tests/spec128_update_runtime_test.sh");
    let output = Command::new("bash")
        .arg(script)
        .env("FOCUSA_BIN", FOCUSA_BIN)
        .current_dir(repo_root)
        .output()
        .expect("Spec128 runtime script should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Spec128 runtime suite failed\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}"
    );
    assert!(stdout.contains("Spec128 installer/update runtime suite complete"));
}
