//! Actual CLI transport against an owned loopback fixture, not a daemon or authority.
use serde_json::json;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

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
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
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
            let mut bytes = [0; 8192];
            let count = stream.read(&mut bytes).unwrap();
            let request = String::from_utf8_lossy(&bytes[..count]).to_string();
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", encoded.len(), encoded).unwrap();
            request
        });
        let mut command = Command::new(env!("CARGO_BIN_EXE_focusa"));
        command
            .args(["--json", "project", "genesis", "status", "--project-root"])
            .arg(&root)
            .env("FOCUSA_API_URL", format!("http://{address}"))
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
