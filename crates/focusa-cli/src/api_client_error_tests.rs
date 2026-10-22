use super::*;
use serde_json::json;

#[test]
fn canonical_failure_preserves_cause_and_redacts_secrets() {
    let error = checked_api_body(
        json!({
            "schema": focusa_core::error_envelope::ERROR_ENVELOPE_SCHEMA,
            "status": "failed", "failure_class": "persistence_unavailable",
            "error": "Disk quota exceeded (os error 122)",
            "credentials": {"token": "must-not-appear"},
        }),
        "http://127.0.0.1/test",
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("Disk quota exceeded"));
    assert!(error.contains("persistence_unavailable"));
    assert!(!error.contains("must-not-appear"));
}

#[test]
fn ordinary_domain_status_and_failed_job_records_are_not_transport_failures() {
    for body in [
        json!({"status":"ok", "jobs":[]}),
        json!({"status":"ok", "job":{"status":"failed"}}),
        json!({"status":"blocked", "schema":"domain.status.v1"}),
    ] {
        assert_eq!(checked_api_body(body.clone(), "test").unwrap(), body);
    }
}

fn one_reply(body: Value) -> (String, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let worker = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        && std::time::Instant::now() < deadline =>
                {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("synthetic HTTP accept did not complete: {error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        loop {
            let mut buffer = [0; 1024];
            let read = stream.read(&mut buffer).unwrap();
            assert!(read > 0);
            request.extend_from_slice(&buffer[..read]);
            assert!(request.len() < 16_384);
            if let Some(end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                let header = String::from_utf8_lossy(&request[..end]);
                let length = header
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if request.len() >= end + 4 + length {
                    break;
                }
            }
        }
        let body = body.to_string();
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
    });
    (format!("http://{address}"), worker)
}

#[tokio::test]
async fn get_and_post_reject_real_http_200_storage_failure_envelopes() {
    for post in [false, true] {
        let (base, worker) = one_reply(focusa_core::error_envelope::persistence_error(
            "synthetic background job",
            "Disk quota exceeded (os error 122)",
        ));
        let client = ApiClient {
            base,
            client: ClientBuilder::new()
                .no_proxy()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap(),
        };
        let result = if post {
            client.post("/job", &json!({"name":"test"})).await
        } else {
            client.get("/jobs").await
        };
        worker.join().unwrap();
        let error = result.unwrap_err().to_string();
        assert!(error.contains("API_RESPONSE_ERROR"));
        assert!(error.contains("Disk quota exceeded"));
        assert!(!error.contains("no jobs array"));
        assert!(!error.contains("did not return a job record"));
    }
}
