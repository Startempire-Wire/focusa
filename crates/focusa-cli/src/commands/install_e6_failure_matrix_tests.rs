use super::*;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Mutex as AsyncMutex;

static TEST_ENV_LOCK: AsyncMutex<()> = AsyncMutex::const_new(());

struct TestEnvVar {
    key: &'static str,
    previous: Option<OsString>,
}

impl TestEnvVar {
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

impl Drop for TestEnvVar {
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

#[derive(Clone, Default)]
struct TestEventCollector {
    events: Arc<Mutex<Vec<InstallEvent>>>,
}

impl TestEventCollector {
    fn events(&self) -> Vec<InstallEvent> {
        self.events
            .lock()
            .expect("event collector mutex poisoned")
            .clone()
    }
}

impl InstallEventSink for TestEventCollector {
    fn emit(&self, event: InstallEvent) {
        self.events
            .lock()
            .expect("event collector mutex poisoned")
            .push(event);
    }
}

fn spawn_http_fixture(body: String) -> (u16, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind isolated local fixture");
    listener
        .set_nonblocking(true)
        .expect("configure local fixture listener");
    let port = listener
        .local_addr()
        .expect("read local fixture port")
        .port();
    let handle = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut request = [0_u8; 1024];
                    let n = stream.read(&mut request).unwrap_or_default();
                    let request_line = String::from_utf8_lossy(&request[..n]);
                    let serve_checksum = request_line.starts_with("GET /SHA256SUMS.txt");
                    let payload = if serve_checksum { body } else { String::new() };
                    let status = if serve_checksum {
                        "200 OK"
                    } else {
                        "404 Not Found"
                    };
                    let response = format!(
                        "HTTP/1.1 {status}\r\n\
                         Content-Type: text/plain\r\n\
                         Content-Length: {}\r\n\
                         Connection: close\r\n\
                         \r\n\
                         {}",
                        payload.len(),
                        payload
                    );
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(_) => break,
            }
        }
    });
    (port, handle)
}

#[tokio::test]
async fn phase_pi_extension_download_returns_none_when_pi_binary_is_missing() {
    let _env_guard = TEST_ENV_LOCK.lock().await;
    let fixture = std::env::temp_dir().join(format!(
        "focusa-pi-extension-missing-{}",
        uuid::Uuid::now_v7()
    ));
    let install_root = fixture.join("install-root");
    let empty_path = fixture.join("empty-bin");
    std::fs::create_dir_all(&empty_path).unwrap();
    let _path = TestEnvVar::with("PATH", Some(empty_path.to_string_lossy().as_ref()));

    let result = phase_pi_extension_download(
        Channel::Stable,
        None,
        None,
        &install_root,
        &NullEventSink,
        &CancellationToken::new(),
    )
    .await
    .expect("pi is absent so no extension download is attempted");

    let download_skipped = result.is_none();
    let share_created = install_root.join("share").exists();
    assert!(download_skipped);
    assert!(!share_created);
    println!("E6_PI_ABSENT download_skipped={download_skipped} share_created={share_created}");
    let _ = std::fs::remove_dir_all(fixture);
}

#[tokio::test]
async fn malformed_pi_extension_archive_rejects_and_keeps_existing_destination() {
    let _env_guard = TEST_ENV_LOCK.lock().await;
    let fixture = std::env::temp_dir().join(format!(
        "focusa-pi-extension-malformed-{}",
        uuid::Uuid::now_v7()
    ));
    let package = fixture.join("package/malformed");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(package.join("README.md"), "malformed payload").unwrap();
    let archive = fixture.join("focusa-pi-extension-vtest.tar.gz");
    assert!(
        std::process::Command::new("tar")
            .args(["-czf"])
            .arg(&archive)
            .arg("-C")
            .arg(fixture.join("package"))
            .arg("malformed")
            .status()
            .unwrap()
            .success()
    );

    let extensions = fixture.join("extensions");
    let destination = extensions.join("focusa");
    std::fs::create_dir_all(&destination).unwrap();
    let preserved_marker = destination.join("preserve.txt");
    std::fs::write(&preserved_marker, "keep").unwrap();

    let _extension_dir = TestEnvVar::with(
        "FOCUSA_PI_EXT_DIR",
        Some(extensions.to_string_lossy().as_ref()),
    );
    let asset = InstalledAsset {
        name: "focusa-pi-extension-vtest.tar.gz".to_string(),
        version: "vtest".to_string(),
        triple: "all".to_string(),
        sha256: String::new(),
        install_path: archive.display().to_string(),
    };
    let error = integrate_pi_extension(&asset, &fixture, None, None)
        .expect_err("malformed Pi archive must fail");
    let destination_preserved = preserved_marker.is_file()
        && std::fs::read_to_string(&preserved_marker)
            .map(|value| value == "keep")
            .ok()
            .unwrap_or(false);
    let package_json_absent = !destination.join("package.json").exists();
    let error_message = error.to_string();
    assert!(error_message.contains("Pi extension archive contains unsafe or incomplete paths"));
    assert!(destination_preserved);
    assert!(package_json_absent);
    println!(
        "E6_PI_FAILURE_SAFE destination_preserved={destination_preserved} package_json_absent={package_json_absent}"
    );
    let _ = std::fs::remove_dir_all(fixture);
}

#[tokio::test]
async fn delegate_service_render_windows_target_returns_warning_outcome() {
    let fixture = std::env::temp_dir().join(format!(
        "focusa-window-service-warning-{}",
        uuid::Uuid::now_v7()
    ));
    let bin_dir = fixture.join("bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    std::fs::write(bin_dir.join("focusa-daemon"), b"#!/bin/sh\necho daemon\n").unwrap();

    let outcome = delegate_service_render(InstallTarget::WindowsX64, &bin_dir, false)
        .await
        .expect("windows target should return warning outcome");

    let warning_message = match outcome {
        ServiceRegistrationOutcome::Warning(message) => {
            assert_eq!(
                message,
                "Windows service registration is unavailable in this installer build"
            );
            message
        }
        ServiceRegistrationOutcome::Registered(message) => {
            panic!("unexpected registered outcome on windows target: {message}")
        }
    };
    println!("E6_SERVICE_WARNING outcome=warning message=\"{warning_message}\"");
    let _ = std::fs::remove_dir_all(fixture);
}

#[test]
fn phase_atomic_cleanup_preserves_new_install_content_and_removes_stash() {
    let install_root = std::env::temp_dir().join(format!(
        "focusa-atomic-cleanup-success-{}",
        uuid::Uuid::now_v7()
    ));
    let stash_path = install_root.with_extension("stash");

    std::fs::create_dir_all(install_root.join("bin")).unwrap();
    std::fs::write(install_root.join("bin/focusa"), b"previous").unwrap();
    assert!(phase_atomic_stash(&install_root, &stash_path).unwrap());

    // Simulate a successful upgrade that already replaced install content.
    std::fs::create_dir_all(install_root.join("bin")).unwrap();
    std::fs::write(install_root.join("bin/focusa"), b"new").unwrap();
    std::fs::create_dir_all(install_root.join("share")).unwrap();
    std::fs::write(
        install_root.join("share").join("agent-context"),
        b"new agent context payload",
    )
    .unwrap();

    phase_atomic_cleanup(&stash_path).unwrap();
    let stash_removed = !stash_path.exists();
    let focusa_content = std::fs::read_to_string(install_root.join("bin/focusa"))
        .expect("focusa binary should still exist after upgrade cleanup");
    let share_payload_present = install_root.join("share").join("agent-context").is_file();
    assert!(stash_removed);
    assert_eq!(focusa_content, "new");
    assert!(share_payload_present);
    println!(
        "E6_UPGRADE_CLEANUP stash_removed={stash_removed} focusa_content={focusa_content} share_payload_present={share_payload_present}"
    );
    let _ = std::fs::remove_dir_all(&install_root);
}

#[test]
fn cancellation_result_restores_stash_with_prior_state_and_emits_rollback_events() {
    let install_root =
        std::env::temp_dir().join(format!("focusa-cancel-rollback-{}", uuid::Uuid::now_v7()));
    let stash_path = install_root.with_extension("stash");

    std::fs::create_dir_all(install_root.join("bin")).unwrap();
    std::fs::write(install_root.join("bin/focusa"), b"prior-install").unwrap();
    std::fs::write(install_root.join("bin/marker"), b"kept").unwrap();
    assert!(phase_atomic_stash(&install_root, &stash_path).unwrap());

    // Write partial install content that should be discarded during rollback.
    std::fs::create_dir_all(install_root.join("bin")).unwrap();
    std::fs::write(install_root.join("bin/focusa"), b"partial").unwrap();
    std::fs::write(install_root.join("bin/partial"), b"discard me").unwrap();

    let collector = TestEventCollector::default();
    let error = cancellation_result::<()>(&install_root, &stash_path, true, &collector)
        .expect_err("cancelled install should fail and rollback");
    assert!(error.to_string().contains("prior installation restored"));

    let events = collector.events();
    assert_eq!(events.len(), 3);
    assert!(matches!(
        &events[0],
        InstallEvent::PhaseFailed {
            phase: InstallPhase::Finalize,
            message: _,
            recovery_hint: Some(hint),
        } if hint == "staged downloads were removed before rollback"
    ));
    assert!(matches!(
        &events[1],
        InstallEvent::RollbackStarted { reason } if reason == "installation cancelled by operator"
    ));
    assert!(matches!(&events[2], InstallEvent::RollbackSucceeded));
    let restored_focusa = std::fs::read_to_string(install_root.join("bin/focusa")).unwrap();
    let marker_present = install_root.join("bin/marker").is_file();
    let partial_discarded = !install_root.join("bin/partial").exists();
    let stash_removed = !stash_path.exists();
    let event_sequence = events
        .iter()
        .map(|event| match event {
            InstallEvent::PhaseFailed { .. } => "PhaseFailed",
            InstallEvent::RollbackStarted { .. } => "RollbackStarted",
            InstallEvent::RollbackSucceeded => "RollbackSucceeded",
            _ => "Other",
        })
        .collect::<Vec<_>>();
    assert_eq!(restored_focusa, "prior-install");
    assert!(marker_present);
    assert!(partial_discarded);
    assert!(stash_removed);
    println!(
        "E6_CANCELLATION_ROLLBACK restored_content={restored_focusa} marker_present={marker_present} partial_discarded={partial_discarded} stash_removed={stash_removed} event_sequence={:?}",
        event_sequence
    );
    let _ = std::fs::remove_dir_all(&install_root);
}

#[tokio::test]
async fn verify_checksum_rejects_mismatched_hash_from_local_http_fixture() {
    let _env_guard = TEST_ENV_LOCK.lock().await;
    let fixture =
        std::env::temp_dir().join(format!("focusa-checksum-mismatch-{}", uuid::Uuid::now_v7()));
    let payload = b"checksum fixture payload";
    let asset_name = "focusa-vtest-x86_64-unknown-linux-musl";
    let asset_path = fixture.join(asset_name);
    std::fs::create_dir_all(&fixture).unwrap();
    std::fs::write(&asset_path, payload).unwrap();
    let real = hex::encode(Sha256::digest(payload));
    let mismatch = if real.as_bytes()[0] == b'a' {
        format!("b{}", &real[1..])
    } else {
        format!("a{}", &real[1..])
    };
    let sha_body = format!("{}  {}\n", mismatch, asset_name);
    let (port, server) = spawn_http_fixture(sha_body);
    let _release_base = TestEnvVar::with(
        "FOCUSA_RELEASE_BASE_URL",
        Some(&format!("http://127.0.0.1:{port}")),
    );
    let asset = InstalledAsset {
        name: asset_name.to_string(),
        version: "vtest".to_string(),
        triple: "x86_64-unknown-linux-musl".to_string(),
        sha256: String::new(),
        install_path: asset_path.display().to_string(),
    };
    let error = verify_checksum(&asset)
        .await
        .expect_err("mismatched checksum must be rejected");
    let mismatch_detected = error.to_string().contains("checksum mismatch");
    let server_joined = server.join().is_ok();
    assert!(mismatch_detected);
    assert!(server_joined);
    println!(
        "E6_INTEGRITY_FAILURE mismatch_detected={mismatch_detected} server_joined={server_joined} asset={}",
        asset.name
    );
    let _ = std::fs::remove_dir_all(fixture);
}
