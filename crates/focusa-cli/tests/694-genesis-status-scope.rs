//! Actual CLI transport against an owned loopback fixture, not a daemon or authority.
use serde_json::json;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

const FIXTURE_TOKEN: &str = "synthetic-controller-test-credential";

fn serve_json(body: serde_json::Value) -> (SocketAddr, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let encoded = body.to_string();
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "CLI never requested Genesis status"
                    );
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("fixture accept: {error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut request = Vec::new();
        while !request.windows(4).any(|part| part == b"\r\n\r\n") {
            let mut buf = [0; 1024];
            let count = stream.read(&mut buf).unwrap();
            assert!(
                count != 0 && request.len() < 8192,
                "invalid fixture request"
            );
            request.extend_from_slice(&buf[..count]);
        }
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", encoded.len(), encoded).unwrap();
        String::from_utf8(request).unwrap()
    });
    (address, server)
}

#[test]
fn genesis_status_forwards_scope_and_refuses_unqualified_legacy_readiness() {
    for mode in ["explicit", "inferred", "history", "legacy-server"] {
        let directory = std::env::temp_dir().join(format!(
            "focusa-cli-genesis-status-{}-{}-{mode}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let root = directory.canonicalize().unwrap();
        std::fs::write(
            root.join(".focusa-project.json"),
            json!({
                "schema":"focusa.project.v1", "project_id":"synthetic-cli-genesis-status",
                "project_root":root
            })
            .to_string(),
        )
        .unwrap();
        let scoped = mode != "history";
        let qualified = mode == "explicit" || mode == "inferred";
        let body = if qualified {
            json!({"schema":"focusa.project_genesis.v1", "status":"ready", "canonical":true,
                "status_projection_schema":"focusa.project_genesis_status.v1",
                "project_identity":{"project_root":root,"verified":true},
                "authority":{"scope":root,"continuity_id":"status-a"},
                "requested_scope":{"project_root":root,"continuity_id":"status-a"},
                "resolved_scope":{"project_root":root,"continuity_id":"status-a"},
                "scope_validation":{"status":"verified","scope_kind":"project_workstream_record","execution_admission":false}})
        } else if mode == "legacy-server" {
            json!({"schema":"focusa.project_genesis.v1", "status":"ready",
                "project_identity":{"project_root":root,"verified":true},
                "authority":{"scope":root,"continuity_id":"status-b"}})
        } else {
            json!({"schema":"focusa.project_genesis_status.v1", "status":"historical", "canonical":false,
                "resolved_scope":{"project_root":root,"continuity_id":"status-b"}})
        };
        let (address, server) = serve_json(body);
        let mut command = Command::new(env!("CARGO_BIN_EXE_focusa"));
        command
            .args(["--json", "project", "genesis", "status", "--project-root"])
            .arg(&root)
            .env("FOCUSA_API_URL", format!("http://{address}"))
            .env("FOCUSA_AUTH_TOKEN", FIXTURE_TOKEN)
            .env("FOCUSA_TEST_MODE", "1")
            .env("FOCUSA_DATA_DIR", root.join("data"))
            .env("FOCUSA_PROJECT_ROOT", &root)
            .env_remove("FOCUSA_CONTINUITY_ID");
        if mode == "inferred" {
            command.env("FOCUSA_CONTINUITY_ID", "status-a");
        } else if scoped {
            command.args(["--continuity-id", "status-a"]);
        }
        let output = command.output().unwrap();
        let request = server.join().unwrap();
        assert!(!request.to_ascii_lowercase().contains("authorization:"));
        assert!(
            output.status.success(),
            "{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let first_line = request.lines().next().unwrap();
        assert!(
            first_line.contains("/v1/project/genesis/status?"),
            "{first_line}"
        );
        assert_eq!(
            first_line.contains("continuity_id=status-a"),
            scoped,
            "{mode}: {first_line}"
        );
        assert_eq!(
            request
                .to_ascii_lowercase()
                .contains("x-scope-continuity-id: status-a"),
            scoped,
            "{mode}: {request}"
        );
        let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(response["canonical"], qualified, "{mode}: {response}");
        assert_eq!(
            response["status"],
            if qualified {
                "ready"
            } else if scoped {
                "unqualified_readiness"
            } else {
                "historical"
            }
        );
        assert_eq!(
            response["resolved_scope"]["continuity_id"],
            if qualified { "status-a" } else { "status-b" }
        );
        if scoped && !qualified {
            assert_eq!(response["requested_scope"]["continuity_id"], "status-a");
        }
    }
}

#[test]
fn remote_genesis_status_authenticates_and_checks_the_exact_receipt() {
    let root = std::env::temp_dir().join(format!("absent-remote-{}", uuid::Uuid::now_v7()));
    let root = root.to_str().unwrap();
    let ready = json!({
        "schema":"focusa.remote_genesis_status.v1", "status":"ready", "canonical":true,
        "binding_id":"binding-a", "project_root":root, "continuity_id":"status-a",
        "execution_admission":false,
        "readiness_receipt":{"workpoint_id":"first", "trajectory_id":"trajectory-a"},
        "trajectory":{"trajectory_id":"trajectory-a"},
        "first_workpoint":{"workpoint_id":"first", "candidate":false, "status":"active", "canonical":true,
            "remote_binding_id":"binding-a", "project_root":root, "continuity_id":"status-a"},
    });
    for (field, wrong) in [
        ("", json!(null)),
        ("/binding_id", json!("binding-b")),
        ("/first_workpoint/continuity_id", json!("status-b")),
        ("/readiness_receipt/workpoint_id", json!("other")),
        ("/trajectory/trajectory_id", json!("trajectory-b")),
        ("/execution_admission", json!(true)),
    ] {
        let mut reply = ready.clone();
        if !field.is_empty() {
            *reply.pointer_mut(field).unwrap() = wrong;
        }
        let (address, server) = serve_json(reply);
        let output = Command::new(env!("CARGO_BIN_EXE_focusa"))
            .args([
                "--json",
                "project",
                "genesis",
                "status",
                "--project-root",
                root,
                "--continuity-id",
                "status-a",
                "--remote-binding-id",
                "binding-a",
            ])
            .env("FOCUSA_API_URL", format!("http://{address}"))
            .env("FOCUSA_AUTH_TOKEN", FIXTURE_TOKEN)
            .env("FOCUSA_TEST_MODE", "1")
            .output()
            .unwrap();
        let request = server.join().unwrap();
        assert!(
            request
                .lines()
                .next()
                .unwrap()
                .contains("remote_binding_id=binding-a")
        );
        assert!(
            request
                .to_ascii_lowercase()
                .lines()
                .any(|line| line == format!("authorization: bearer {FIXTURE_TOKEN}"))
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains(FIXTURE_TOKEN));
        assert_eq!(
            output.status.success(),
            field.is_empty(),
            "{field}: {stderr}"
        );
        if field.is_empty() {
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
                ready
            );
        } else {
            assert!(stderr.contains("unqualified remote onboarding response"));
        }
    }
    assert!(!std::path::Path::new(root).exists());
}

#[test]
fn remote_genesis_requires_a_credential_and_literal_loopback_origin() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let absent_root = std::env::temp_dir().join(format!("absent-remote-{}", uuid::Uuid::now_v7()));
    for (origin, with_token, diagnostic) in [
        (
            format!("http://{address}"),
            false,
            "requires the configured FOCUSA_AUTH_TOKEN",
        ),
        (
            format!("http://localhost:{}", address.port()),
            true,
            "literal loopback controller origin",
        ),
        (
            format!("http://{address}/proxy"),
            true,
            "literal loopback controller origin",
        ),
    ] {
        let mut cli = Command::new(env!("CARGO_BIN_EXE_focusa"));
        cli.args(["--json", "project", "genesis", "status", "--project-root"])
            .arg(&absent_root)
            .args([
                "--continuity-id",
                "status-a",
                "--remote-binding-id",
                "binding-a",
            ])
            .env("FOCUSA_API_URL", origin)
            .env("FOCUSA_API_TIMEOUT", "1")
            .env("FOCUSA_TEST_MODE", "1");
        if with_token {
            cli.env("FOCUSA_AUTH_TOKEN", FIXTURE_TOKEN);
        } else {
            cli.env_remove("FOCUSA_AUTH_TOKEN");
        }
        let output = cli.output().unwrap();
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(diagnostic), "{stderr}");
        assert!(!stderr.contains(FIXTURE_TOKEN));
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
    }
}
