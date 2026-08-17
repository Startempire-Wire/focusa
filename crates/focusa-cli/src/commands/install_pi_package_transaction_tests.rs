//! Pi package activation transaction tests (issue #309).
//!
//! Proves: backups live outside the discovery root; activation, commit, and
//! rollback boundaries are explicit; retirement moves only verified
//! Focusa-owned legacy/backup/old entries; fault injection after activation
//! restores the exact prior package.

use super::*;
use crate::commands::pi_package::{
    activate_pi_package, commit_pi_activation, is_focusa_retired_entry_name, now_unix,
    package_identity_of, retire_focusa_packages, retired_root_for, rollback_pi_activation,
    CANONICAL_ENTRY, FAULT_AFTER_PI_ACTIVATION, PACKAGE_IDENTITY,
};
use crate::commands::update::phase_pi_package_apply;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::Mutex as AsyncMutex;

static PI_TEST_ENV_LOCK: AsyncMutex<()> = AsyncMutex::const_new(());

struct PiTestEnvVar {
    key: &'static str,
    previous: Option<OsString>,
}

impl PiTestEnvVar {
    fn with(key: &'static str, value: Option<&str>) -> Self {
        let previous = std::env::var_os(key);
        match value {
            Some(value) => unsafe {
                std::env::set_var(key, value);
            },
            None => unsafe {
                std::env::remove_var(key);
            },
        }
        Self { key, previous }
    }
}

impl Drop for PiTestEnvVar {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => unsafe {
                std::env::set_var(self.key, value);
            },
            None => unsafe {
                std::env::remove_var(self.key);
            },
        }
    }
}

fn unique_fixture(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("focusa-pi-txn-{label}-{}", uuid::Uuid::now_v7()))
}

fn write_focusa_package(dir: &Path, version: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(
        dir.join("package.json"),
        format!(r#"{{"name":"{PACKAGE_IDENTITY}","version":"{version}"}}"#),
    )
    .unwrap();
}

fn build_pi_extension_archive(fixture: &Path, version: &str) -> PathBuf {
    let package = fixture.join("package/pi-extension");
    write_focusa_package(&package, version);
    std::fs::write(package.join("index.ts"), format!("// focusa {version}\n")).unwrap();
    let archive = fixture.join(format!("focusa-pi-extension-v{version}.tar.gz"));
    let status = tar_command()
        .args(["-czf"])
        .arg(&archive)
        .args(["-C"])
        .arg(fixture.join("package"))
        .arg("pi-extension")
        .status()
        .unwrap();
    assert!(status.success(), "test archive creation failed");
    archive
}

/// Serves raw bytes (any GET path) so the OTA phase helper can download.
fn spawn_archive_fixture(bytes: Vec<u8>) -> (u16, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind archive fixture");
    listener
        .set_nonblocking(true)
        .expect("configure archive fixture listener");
    let port = listener.local_addr().expect("archive fixture port").port();
    let handle = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut request = [0_u8; 2048];
                    let _ = stream.read(&mut request);
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/gzip\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        bytes.len()
                    );
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.write_all(&bytes);
                    let _ = stream.flush();
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });
    (port, handle)
}

#[test]
fn retired_entry_name_patterns_cover_known_focusa_shapes_only() {
    for name in [
        "focusa-runtime",
        "focusa-runtime.legacy-0.9.143",
        ".focusa-backup-019f0000",
        "focusa-backup-1",
        "focusa-legacy-2",
        "focusa-old-3",
        "focusa.rollback",
        "focusa.disabled",
        "focusa-rollback-4",
        "focusa-disabled-5",
    ] {
        assert!(
            is_focusa_retired_entry_name(name),
            "{name} must be recognized as a Focusa-owned retired pattern"
        );
    }
    for name in [
        "guardian.ts",
        "uiai-engine.ts",
        "model-search-modal.ts",
        "ovh-model-switch.ts",
        "pi-web-access",
        "other-extension",
    ] {
        assert!(
            !is_focusa_retired_entry_name(name),
            "{name} must never match a Focusa retirement pattern"
        );
    }
}

#[test]
fn activate_commit_cycle_keeps_discovery_root_clean() {
    let fixture = unique_fixture("commit");
    let extensions = fixture.join("extensions");
    let prior = extensions.join(CANONICAL_ENTRY);
    write_focusa_package(&prior, "0.9.151");
    std::fs::write(prior.join("prior-marker.txt"), "prior").unwrap();
    let unrelated = extensions.join("uiai-engine.ts");
    std::fs::create_dir_all(&extensions).unwrap();
    std::fs::write(&unrelated, "unrelated").unwrap();

    let staged = fixture.join("staged");
    write_focusa_package(&staged, "0.9.152");
    std::fs::write(staged.join("new-marker.txt"), "new").unwrap();

    let receipt = activate_pi_package(&staged, &extensions, "v0.9.152")
        .expect("activation must succeed");
    let retired_root = retired_root_for(&extensions);

    // Canonical target replaced, backup outside discovery, no hidden backups.
    assert_eq!(
        package_identity_of(&extensions.join(CANONICAL_ENTRY)).as_deref(),
        Some(PACKAGE_IDENTITY)
    );
    assert_eq!(
        std::fs::read_to_string(extensions.join(CANONICAL_ENTRY).join("new-marker.txt")).unwrap(),
        "new"
    );
    let backup = receipt
        .prior
        .as_ref()
        .expect("prior package must be recorded")
        .backup
        .clone();
    assert!(backup.starts_with(&retired_root));
    assert!(backup.exists());
    assert!(!backup.starts_with(&extensions), "backup must live outside discovery");
    let hidden_under_discovery = std::fs::read_dir(&extensions)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".focusa-")
        });
    assert!(!hidden_under_discovery, "no backups may remain under discovery");
    assert!(unrelated.exists(), "unrelated extensions stay untouched");

    commit_pi_activation(&receipt).expect("commit must succeed");
    assert!(!backup.exists(), "commit removes the prior backup");
    let _ = std::fs::remove_dir_all(fixture);
}

#[test]
fn rollback_restores_exact_prior_package_after_activation() {
    let fixture = unique_fixture("rollback");
    let extensions = fixture.join("extensions");
    let prior = extensions.join(CANONICAL_ENTRY);
    write_focusa_package(&prior, "0.9.151");
    std::fs::write(prior.join("prior-marker.txt"), "prior").unwrap();

    let staged = fixture.join("staged");
    write_focusa_package(&staged, "0.9.152");
    std::fs::write(staged.join("new-marker.txt"), "new").unwrap();

    let receipt = activate_pi_package(&staged, &extensions, "v0.9.152").unwrap();
    assert!(!extensions
        .join(CANONICAL_ENTRY)
        .join("prior-marker.txt")
        .exists());

    rollback_pi_activation(&receipt).expect("rollback must restore the prior package");
    assert_eq!(
        std::fs::read_to_string(extensions.join(CANONICAL_ENTRY).join("prior-marker.txt")).unwrap(),
        "prior",
        "rollback restores the exact prior package"
    );
    assert!(
        !extensions
            .join(CANONICAL_ENTRY)
            .join("new-marker.txt")
            .exists(),
        "the failed package must be discarded"
    );
    let leftover_focusa = std::fs::read_dir(&extensions)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("focusa"))
        .count();
    assert_eq!(leftover_focusa, 1, "exactly the canonical target remains");
    let _ = std::fs::remove_dir_all(fixture);
}

#[test]
fn retire_moves_only_verified_focusa_owned_entries_and_keeps_alias() {
    let fixture = unique_fixture("retire");
    let extensions = fixture.join("extensions");
    let canonical = extensions.join(CANONICAL_ENTRY);
    write_focusa_package(&canonical, "0.9.152");
    std::fs::create_dir_all(&extensions).unwrap();

    // Verified legacy copy of the Focusa package.
    write_focusa_package(&extensions.join("focusa-runtime.legacy-0.9.143"), "0.9.143");
    // Dot-prefixed backup of the Focusa package.
    write_focusa_package(&extensions.join(".focusa-backup-019f0000"), "0.9.140");
    // Unrelated package: same-looking dir name, different identity.
    let unrelated_dir = extensions.join("focusa-lookalike");
    std::fs::create_dir_all(&unrelated_dir).unwrap();
    std::fs::write(
        unrelated_dir.join("package.json"),
        r#"{"name":"someone-elses-extension","version":"1.0.0"}"#,
    )
    .unwrap();
    // Unrelated loose files.
    std::fs::write(extensions.join("guardian.ts"), "guardian").unwrap();
    // Compatibility alias resolving to the canonical target.
    #[cfg(unix)]
    std::os::unix::fs::symlink(CANONICAL_ENTRY, extensions.join("focusa-runtime")).unwrap();

    let retired_root = retired_root_for(&extensions);
    let retired = retire_focusa_packages(&extensions, &retired_root).unwrap();
    assert_eq!(retired.len(), 2, "only the two verified Focusa-owned entries retire");

    assert!(!extensions.join("focusa-runtime.legacy-0.9.143").exists());
    assert!(!extensions.join(".focusa-backup-019f0000").exists());
    let moved = retired.iter().all(|path| path.starts_with(&retired_root) && path.exists());
    assert!(moved, "retired packages are preserved outside discovery");
    assert!(
        unrelated_dir.join("package.json").exists(),
        "unrelated packages are never moved"
    );
    assert!(extensions.join("guardian.ts").exists(), "unrelated files stay");
    #[cfg(unix)]
    {
        let alias = std::fs::canonicalize(extensions.join("focusa-runtime")).unwrap();
        assert_eq!(alias, std::fs::canonicalize(&canonical).unwrap());
    }
    assert!(canonical.join("package.json").exists());
    let _ = std::fs::remove_dir_all(fixture);
}

#[test]
fn activation_failure_restores_prior_destination() {
    let fixture = unique_fixture("activate-fail");
    let extensions = fixture.join("extensions");
    let prior = extensions.join(CANONICAL_ENTRY);
    write_focusa_package(&prior, "0.9.151");
    std::fs::write(prior.join("prior-marker.txt"), "prior").unwrap();

    let missing_staged = fixture.join("does-not-exist");
    let error = activate_pi_package(&missing_staged, &extensions, "v0.9.152")
        .expect_err("activating a missing staged package must fail");
    assert!(error.to_string().contains("activate Pi extension"));

    assert_eq!(
        std::fs::read_to_string(prior.join("prior-marker.txt")).unwrap(),
        "prior",
        "the prior destination must be restored"
    );
    let hidden_under_discovery = std::fs::read_dir(&extensions)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".focusa-")
        });
    assert!(!hidden_under_discovery);
    let _ = std::fs::remove_dir_all(fixture);
}

#[tokio::test]
async fn update_phase_fault_after_activation_rolls_back_prior_package() {
    let _env_guard = PI_TEST_ENV_LOCK.lock().await;
    let fixture = unique_fixture("ota-fault");
    let extensions = fixture.join("extensions");
    let prior = extensions.join(CANONICAL_ENTRY);
    write_focusa_package(&prior, "0.9.151");
    std::fs::write(prior.join("prior-marker.txt"), "prior").unwrap();
    let state = fixture.join("state");
    let stage = fixture.join("stage");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::create_dir_all(&stage).unwrap();

    let archive = build_pi_extension_archive(&fixture, "0.9.152");
    let bytes = std::fs::read(&archive).unwrap();
    let expected_sha256 = format!("{:x}", Sha256::digest(&bytes));
    let (port, server) = spawn_archive_fixture(bytes);
    let fake_bin = fixture.join("bin");
    std::fs::create_dir_all(&fake_bin).unwrap();
    let npm = fake_bin.join("npm");
    std::fs::write(&npm, "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&npm, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let _fault = PiTestEnvVar::with(FAULT_AFTER_PI_ACTIVATION, Some("1"));
    let _path = PiTestEnvVar::with("PATH", Some(fake_bin.to_string_lossy().as_ref()));

    let error = phase_pi_package_apply(
        &state,
        &stage,
        &extensions,
        &format!("http://127.0.0.1:{port}/pi-extension.tar.gz"),
        &expected_sha256,
        "0.9.152",
    )
    .await
    .expect_err("the injected fault must fail the update after Pi activation");
    assert!(
        error.to_string().contains("injected fault after Pi extension activation"),
        "unexpected error: {error}"
    );

    // The transaction state must contain the typed receipt.
    let receipt_path = state.join("pi-extension-activation.json");
    assert!(receipt_path.exists(), "receipt must be persisted before the fault");
    let receipt: crate::commands::pi_package::PiActivationReceipt =
        serde_json::from_slice(&std::fs::read(&receipt_path).unwrap()).unwrap();
    assert_eq!(receipt.schema, "focusa.pi_activation_receipt.v1");
    assert!(receipt.prior.is_some(), "the prior package must be recorded");

    // The apply failure path must restore the exact prior package.
    rollback_pi_activation(&receipt).expect("rollback must restore the prior package");
    assert_eq!(
        std::fs::read_to_string(extensions.join(CANONICAL_ENTRY).join("prior-marker.txt")).unwrap(),
        "prior"
    );
    assert!(!extensions
        .join(CANONICAL_ENTRY)
        .join("index.ts")
        .exists());
    let _ = server.join();
    let _ = std::fs::remove_dir_all(fixture);
}

#[test]
fn receipt_round_trips_through_json() {
    let receipt = crate::commands::pi_package::PiActivationReceipt {
        schema: "focusa.pi_activation_receipt.v1".to_string(),
        destination: PathBuf::from("/tmp/extensions/focusa"),
        version: "0.9.152".to_string(),
        prior: Some(crate::commands::pi_package::PriorPiPackage {
            backup: PathBuf::from("/tmp/retired-extensions/backups/focusa-backup-x"),
            sha256: "abc".to_string(),
        }),
        retired: vec![PathBuf::from("/tmp/retired-extensions/focusa-runtime.legacy-0.9.143-x")],
        activated_at: now_unix(),
    };
    let encoded = serde_json::to_vec_pretty(&receipt).unwrap();
    let decoded: crate::commands::pi_package::PiActivationReceipt =
        serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded.schema, receipt.schema);
    assert_eq!(decoded.destination, receipt.destination);
    assert_eq!(decoded.prior.unwrap().backup, receipt.prior.unwrap().backup);
}
