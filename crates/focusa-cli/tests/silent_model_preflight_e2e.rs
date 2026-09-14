//! GitHub #132 typed model resolution/unsupported parity.

use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");

#[test]
fn luna_max_preflight_uses_versioned_route_and_preserves_typed_unsupported_result() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0u8; 8192];
        loop {
            let count = stream.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            bytes.extend_from_slice(&buffer[..count]);
            if bytes.windows(4).any(|window| window == b"\r\n\r\n")
                && String::from_utf8_lossy(&bytes).contains("Luna Max")
            {
                break;
            }
        }
        let request = String::from_utf8_lossy(&bytes);
        assert!(request.starts_with("POST /v1/providers/pi-runtime/models/preflight HTTP/1.1"));
        assert!(request.contains("\"model\":\"Luna Max\""));
        let body = json!({
            "ok": false,
            "status": "model_unsupported",
            "canonical": true,
            "advisory": false,
            "degraded": false,
            "stale": false,
            "failure_class": "unsupported_model",
            "retry": {"retryable": false, "after_ms": null, "idempotency_key_required": false},
            "side_effects": [],
            "evidence_refs": [],
            "receipt_refs": [],
            "next_tools": [],
            "recovery_hint": "Select a model from the server-owned catalog.",
            "misuse_hint": null,
            "data": null
        })
        .to_string();
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
    });
    let output = Command::new(FOCUSA_BIN)
        .args([
            "--json",
            "silent",
            "model",
            "preflight",
            "--model",
            "Luna Max",
        ])
        .env("FOCUSA_API_URL", format!("http://{address}"))
        .env("FOCUSA_API_TIMEOUT", "5")
        .output()
        .unwrap();
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["status"], "model_unsupported");
    assert_eq!(value["result"]["failure_class"], "unsupported_model");
    assert_eq!(value["result"]["canonical"], true);
}
