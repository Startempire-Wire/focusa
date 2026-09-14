//! Actual CLI-to-HTTP proof: evidence linking must carry licensed request identity.
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::time::{Duration, Instant};

#[test]
fn evidence_link_sends_generated_and_explicit_idempotency_keys() {
    for explicit in [None, Some("evidence-retry-123")] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            let (mut stream, _) = loop {
                match listener.accept() {
                    Ok(connection) => break connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "CLI never sent its request");
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("accept: {error}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let (headers, body) = loop {
                let mut buffer = [0; 4096];
                let count = stream.read(&mut buffer).unwrap();
                assert!(count > 0, "incomplete request");
                bytes.extend_from_slice(&buffer[..count]);
                assert!(bytes.len() < 65536, "unexpectedly large request");
                if let Some(end) = bytes.windows(4).position(|v| v == b"\r\n\r\n") {
                    let headers = String::from_utf8(bytes[..end].to_vec()).unwrap();
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse().unwrap())
                        })
                        .unwrap();
                    if bytes.len() >= end + 4 + length {
                        let body: serde_json::Value =
                            serde_json::from_slice(&bytes[end + 4..end + 4 + length]).unwrap();
                        break (headers, body);
                    }
                }
            };
            assert!(headers.starts_with("POST /v1/workpoint/evidence/link HTTP/1.1"));
            let values: Vec<_> = headers
                .lines()
                .filter_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("idempotency-key")
                        .then(|| value.trim())
                })
                .collect();
            assert_eq!(
                values.len(),
                1,
                "exactly one idempotency header is required"
            );
            match explicit {
                Some(key) => assert_eq!(values[0], key),
                None => {
                    uuid::Uuid::parse_str(values[0]).expect("generated UUID key");
                }
            }
            assert_eq!(body["idempotency_key"], values[0]);
            assert_eq!(body["workpoint_id"], "11111111-1111-4111-8111-111111111111");
            assert!(
                headers
                    .to_ascii_lowercase()
                    .contains("x-focusa-writer-id: focusa-cli")
            );
            let response = r#"{"status":"completed"}"#;
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response.len(), response).unwrap();
        });
        let home =
            std::env::temp_dir().join(format!("focusa-evidence-test-{}", uuid::Uuid::now_v7()));
        let mut command = Command::new(env!("CARGO_BIN_EXE_focusa"));
        command
            .current_dir(std::env::temp_dir())
            .env("HOME", &home)
            .env("FOCUSA_API_URL", format!("http://{address}"))
            .env("FOCUSA_API_TIMEOUT", "5")
            .env_remove("FOCUSA_TEST_MODE")
            .args([
                "workpoint",
                "evidence-link",
                "--workpoint-id",
                "11111111-1111-4111-8111-111111111111",
                "--target-ref",
                "test:transport",
                "--result",
                "verified",
                "--json",
            ]);
        if let Some(key) = explicit {
            command.args(["--idempotency-key", key]);
        }
        let output = command.output().unwrap();
        server.join().expect("strict HTTP receiver");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn evidence_link_help_exposes_retry_key() {
    let output = Command::new(env!("CARGO_BIN_EXE_focusa"))
        .args(["workpoint", "evidence-link", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("--idempotency-key"));
}
