use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Output};
use std::thread;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");

fn fake_api() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake Focusa API");
    let address = listener.local_addr().expect("fake API address");
    thread::spawn(move || {
        for incoming in listener.incoming().take(16) {
            let Ok(mut stream) = incoming else { break };
            let mut request = [0_u8; 8192];
            let _ = stream.read(&mut request);
            let body = b"{}";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.write_all(body);
        }
    });
    format!("http://{address}")
}

fn run_wrap(verbose: bool) -> Output {
    let mut command = Command::new(FOCUSA_BIN);
    command.env("FOCUSA_API_URL", fake_api());
    if verbose {
        command.arg("--verbose");
    }
    command
        .args(["wrap", "--", "/bin/true"])
        .output()
        .expect("focusa wrap should execute")
}

#[test]
fn normal_wrap_stderr_has_no_debug_diagnostics() {
    let output = run_wrap(false);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "normal wrap failed: {stderr}");
    assert!(
        !stderr.contains("[DEBUG]"),
        "unexpected debug output: {stderr}"
    );
}

#[test]
fn verbose_wrap_retains_explicit_debug_diagnostics() {
    let output = run_wrap(true);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "verbose wrap failed: {stderr}");
    for marker in ["[DEBUG] Mode:", "[DEBUG] Turn ID:", "[DEBUG] Running:"] {
        assert!(stderr.contains(marker), "missing {marker} in: {stderr}");
    }
}
