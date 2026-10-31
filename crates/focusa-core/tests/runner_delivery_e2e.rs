//! End-to-end proof for GitHub #679: a signed launch must reach a real runner
//! process, input must reach the harness's stdin, and queries must round-trip.
//!
//! Run with the built runner binary:
//!   FOCUSA_SESSION_RUNNER_BIN=<target>/debug/focusa-session-runner \
//!     cargo test -p focusa-core --test runner_delivery_e2e -- --nocapture
#![cfg(unix)]

use focusa_core::runtime::persistence_sqlite::SqlitePersistence;
use focusa_core::silent_sessions::{
    dispatch_input, dispatch_launch, dispatch_query, LaunchPlan, LaunchTrustPolicy, MissionDelivery,
    RunnerPaths, SilentSessionId, SilentSessionRunId,
};
use focusa_core::types::FocusaConfig;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("focusa-e2e-{name}-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn owner_uid(path: &Path) -> u32 {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(path).unwrap().uid()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn signed_launch_input_and_query_reach_a_real_process() {
    let Some(runner_bin) = std::env::var_os("FOCUSA_SESSION_RUNNER_BIN").map(PathBuf::from) else {
        eprintln!("skipped: set FOCUSA_SESSION_RUNNER_BIN to the built focusa-session-runner");
        return;
    };
    assert!(runner_bin.is_file(), "runner binary missing: {}", runner_bin.display());

    let root = scratch("delivery");
    let workspace = root.join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    let data_dir = root.join("data");
    std::fs::create_dir_all(root.join("runners")).unwrap();

    let mut config = FocusaConfig::default();
    config.data_dir = data_dir.display().to_string();
    let persistence = SqlitePersistence::new(&config).expect("persistence");

    let mission = "Mission for the #679 delivery proof.";
    let mission_file = root.join("mission.md");
    {
        // The runner refuses mission artifacts readable by anyone else.
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&mission_file)
            .unwrap();
        file.write_all(mission.as_bytes()).unwrap();
    }
    let mission_sha = hex::encode(Sha256::digest(mission.as_bytes()));

    let session_id = SilentSessionId::new();
    let run_id = SilentSessionRunId::new();
    let os_user = std::env::var("USER").unwrap_or_else(|_| "wirebot".into());
    let uid = owner_uid(&root);

    let paths = RunnerPaths::resolve(&data_dir, &format!("session-{session_id}"), &format!("run-{run_id}"))
        .expect("paths");
    let _ = paths;

    let plan = LaunchPlan {
        executable: "/bin/cat".into(),
        argv: Vec::new(),
        cwd: workspace.display().to_string(),
        os_user: os_user.clone(),
        mission_delivery: MissionDelivery::Stdin {
            artifact_path: mission_file.display().to_string(),
            sha256: mission_sha,
        },
        adapter_id: "generic_pty".into(),
        adapter_version: "generic_pty.v1".into(),
        config_revision_id: "cfgrev-e2e".into(),
        model_binding: "e2e/model".into(),
        thinking_level: "default".into(),
        bootstrap_packet_ref: "bootstrap/e2e".into(),
        trust: LaunchTrustPolicy {
            project_verified: true,
            workspace_verified: true,
            operator_approved: true,
            context_authority_allowed: true,
            trust_preflight_passed: true,
            required_noninteractive_flag: None,
        },
    };

    let receipt = dispatch_launch(
        &persistence,
        &data_dir,
        session_id,
        run_id,
        &plan,
        uid,
        Duration::from_secs(15),
    )
    .await
    .expect("launch must reach the runner");

    let pid = receipt.pid.expect("a new runner must report its pid");
    assert!(pid > 0, "runner pid must be real");
    assert!(Path::new(&receipt.socket).exists(), "runner socket must exist");

    dispatch_input(
        &persistence,
        &data_dir,
        session_id,
        run_id,
        &os_user,
        "hello from the #679 e2e proof",
        uid,
        Duration::from_secs(10),
    )
    .await
    .expect("input must be delivered to the harness");

    dispatch_query(
        &persistence,
        &data_dir,
        session_id,
        run_id,
        &os_user,
        uid,
        Duration::from_secs(10),
    )
    .await
    .expect("query must round-trip");

    unsafe {
        libc::kill(-(pid as i32), libc::SIGKILL);
        libc::kill(pid as i32, libc::SIGKILL);
    }
    std::fs::remove_dir_all(&root).ok();
}
