//! One real pipe test complements the existing writer/error unit tests (#511).
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn bg_list_exits_cleanly_when_consumer_closes_stdout() {
    for json_mode in [false, true] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "CLI never requested fixture");
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("fixture accept: {error}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let size = stream.read(&mut buffer).unwrap();
                assert!(size > 0 && request.len() < 16_384);
                request.extend_from_slice(&buffer[..size]);
            }
            assert!(request.starts_with(b"GET /v1/background-jobs "));
            // Synthetic terminal job: no reconciliation or production writes.
            let body = serde_json::json!({"jobs":[{
                "job_id":"fixture", "status":"completed", "name":"x".repeat(1024 * 1024)
            }]})
            .to_string();
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
        });
        let mut command = Command::new(env!("CARGO_BIN_EXE_focusa"));
        if json_mode {
            command.arg("--json");
        }
        let mut child = command
            .args(["bg", "list"])
            .env("FOCUSA_API_URL", format!("http://{address}"))
            .env("FOCUSA_API_TIMEOUT", "5")
            .env("NO_PROXY", "127.0.0.1")
            .env("no_proxy", "127.0.0.1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        // Close the actual OS pipe before printing begins, not a mocked writer.
        drop(child.stdout.take());
        let deadline = Instant::now() + Duration::from_secs(15);
        while child.try_wait().unwrap().is_none() {
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("bg list hung after downstream closed stdout");
            }
            thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        server.join().unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "json={json_mode}: {stderr}");
        assert!(
            !stderr.contains("panicked") && !stderr.contains("stack backtrace"),
            "{stderr}"
        );
    }
}
