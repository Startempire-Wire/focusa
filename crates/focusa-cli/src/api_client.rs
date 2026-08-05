//! HTTP client for the Focusa daemon API.
//!
//! All CLI commands funnel through this module.
//! Default endpoint: http://127.0.0.1:8787

use reqwest::{Client, ClientBuilder};
use serde_json::Value;

fn body_idempotency_key(body: &Value) -> Option<&str> {
    [
        "idempotency_key",
        "idempotencyKey",
        "request_id",
        "requestId",
    ]
    .iter()
    .find_map(|key| body.get(*key).and_then(Value::as_str))
    .filter(|value| !value.trim().is_empty())
}

fn redact_error_json(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                let sensitive = ["secret", "token", "credential", "authorization", "api_key"]
                    .iter()
                    .any(|needle| key.to_ascii_lowercase().contains(needle));
                if sensitive {
                    *value = Value::String("[REDACTED]".into());
                } else {
                    redact_error_json(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(redact_error_json),
        _ => {}
    }
}
use std::process::{Command, Stdio};
use std::time::Duration;

const DEFAULT_BASE: &str = "http://127.0.0.1:8787";
const DEFAULT_TIMEOUT_SECS: u64 = 10;

fn classify_reqwest_error(err: reqwest::Error, url: &str) -> anyhow::Error {
    if err.is_timeout() {
        anyhow::anyhow!("[API_TIMEOUT] url={} reason={}", url, err)
    } else if err.is_connect() {
        anyhow::anyhow!("[API_CONNECT_ERROR] url={} reason={}", url, err)
    } else if err.is_decode() {
        anyhow::anyhow!("[API_DECODE_ERROR] url={} reason={}", url, err)
    } else {
        anyhow::anyhow!("[API_REQUEST_ERROR] url={} reason={}", url, err)
    }
}

pub struct ApiClient {
    client: Client,
    base: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self::with_timeout_secs(DEFAULT_TIMEOUT_SECS)
    }

    pub fn with_timeout_secs(default_timeout_secs: u64) -> Self {
        let base = std::env::var("FOCUSA_API_URL")
            .or_else(|_| std::env::var("FOCUSA_BASE_URL"))
            .unwrap_or_else(|_| DEFAULT_BASE.to_string());

        let timeout = std::env::var("FOCUSA_API_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default_timeout_secs);

        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout))
            .build()
            .unwrap_or_else(|err| {
                eprintln!(
                    "[API_CLIENT_INIT_FALLBACK] failed to build configured HTTP client; using reqwest default client: {err}"
                );
                Client::new()
            });

        Self { client, base }
    }

    pub fn base_url(&self) -> &str {
        &self.base
    }

    pub fn http_client(&self) -> &Client {
        &self.client
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HLT Ledger API — Spec98/99: scope-bounded, no singleton
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get HLT history for a project (scope-bounded by project_root + continuity_id).
    #[allow(dead_code)]
    pub async fn get_hlt_history(
        &self,
        project_root: &str,
        continuity_id: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Value> {
        let mut path = format!(
            "/v1/hlt/history?project_root={}",
            urlencoding::encode(project_root)
        );
        if let Some(cid) = continuity_id {
            path.push_str(&format!("&continuity_id={}", urlencoding::encode(cid)));
        }
        path.push_str(&format!("&limit={}", limit));
        self.get(&path).await
    }

    /// Get current HLT from trajectory view (scope-bounded).
    #[allow(dead_code)]
    pub async fn get_trajectory_view(
        &self,
        project_root: &str,
        continuity_id: Option<&str>,
        session_id: Option<&str>,
    ) -> anyhow::Result<Value> {
        let mut path = format!(
            "/v1/trajectory/view?project_root={}",
            urlencoding::encode(project_root)
        );
        if let Some(cid) = continuity_id {
            path.push_str(&format!("&continuity_id={}", urlencoding::encode(cid)));
        }
        if let Some(sid) = session_id {
            path.push_str(&format!("&session_id={}", urlencoding::encode(sid)));
        }
        self.get(&path).await
    }

    pub async fn get(&self, path: &str) -> anyhow::Result<Value> {
        self.get_with_headers(path, &[]).await
    }

    pub async fn get_text_with_headers(
        &self,
        path: &str,
        headers: &[(&str, &str)],
    ) -> anyhow::Result<(u16, String)> {
        let url = format!("{}{}", self.base, path);
        let mut request = self.client.get(&url);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let resp = request
            .send()
            .await
            .map_err(|err| classify_reqwest_error(err, &url))?;
        let status = resp.status().as_u16();
        let body = resp
            .text()
            .await
            .map_err(|err| classify_reqwest_error(err, &url))?;
        Ok((status, body))
    }

    pub async fn get_probe(&self, path: &str) -> anyhow::Result<(u16, Value)> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|err| classify_reqwest_error(err, &url))?;
        let status = resp.status().as_u16();
        let value = resp
            .json()
            .await
            .map_err(|err| classify_reqwest_error(err, &url))?;
        Ok((status, value))
    }

    pub async fn get_scoped(
        &self,
        path: &str,
        project_root: &str,
        continuity_id: &str,
    ) -> anyhow::Result<Value> {
        self.get_with_headers(
            path,
            &[
                ("x-scope-project-root", project_root),
                ("x-scope-continuity-id", continuity_id),
            ],
        )
        .await
    }

    pub async fn get_with_headers(
        &self,
        path: &str,
        headers: &[(&str, &str)],
    ) -> anyhow::Result<Value> {
        let url = format!("{}{}", self.base, path);
        let mut request = self.client.get(&url);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let resp = request
            .send()
            .await
            .map_err(|err| classify_reqwest_error(err, &url))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let safe_body = serde_json::from_str::<Value>(&body)
                .map(|mut value| {
                    redact_error_json(&mut value);
                    value.to_string()
                })
                .unwrap_or_else(|_| "[REDACTED_ERROR_BODY]".to_string());
            anyhow::bail!(
                "[API_HTTP_ERROR] status={} url={} body={}",
                status,
                url,
                safe_body
            );
        }
        resp.json()
            .await
            .map_err(|err| classify_reqwest_error(err, &url))
    }

    pub async fn post(&self, path: &str, body: &Value) -> anyhow::Result<Value> {
        self.post_with_headers(path, body, &[]).await
    }

    pub async fn post_with_headers(
        &self,
        path: &str,
        body: &Value,
        headers: &[(&str, &str)],
    ) -> anyhow::Result<Value> {
        let url = format!("{}{}", self.base, path);
        let mut req = self.client.post(&url).json(body);
        let explicit_idempotency = headers
            .iter()
            .any(|(key, _)| key.eq_ignore_ascii_case("Idempotency-Key"));
        if !explicit_idempotency {
            if let Some(key) = body_idempotency_key(body) {
                req = req.header("Idempotency-Key", key.trim());
            }
        }
        for (key, value) in headers {
            req = req.header(*key, *value);
        }
        let resp = req
            .send()
            .await
            .map_err(|err| classify_reqwest_error(err, &url))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let safe_body = serde_json::from_str::<Value>(&body)
                .map(|mut value| {
                    redact_error_json(&mut value);
                    value.to_string()
                })
                .unwrap_or_else(|_| "[REDACTED_ERROR_BODY]".to_string());
            anyhow::bail!(
                "[API_HTTP_ERROR] status={} url={} body={}",
                status,
                url,
                safe_body
            );
        }
        resp.json()
            .await
            .map_err(|err| classify_reqwest_error(err, &url))
    }

    /// Blocking POST using curl - for use before process exit.
    /// Uses curl since tokio's runtime may not complete spawned tasks before exit.
    pub fn post_blocking(&self, path: &str, body: &Value, timeout_secs: u64) {
        let url = format!("{}{}", self.base, path);
        let body_json = body.to_string();

        // Use curl for a truly synchronous HTTP request.
        let timeout = timeout_secs.to_string();
        let mut args = vec![
            "-s".to_string(),
            "-X".to_string(),
            "POST".to_string(),
            "-H".to_string(),
            "Content-Type: application/json".to_string(),
        ];
        if let Some(key) = body_idempotency_key(body) {
            args.extend(["-H".to_string(), format!("Idempotency-Key: {}", key.trim())]);
        }
        args.extend(["-d".to_string(), body_json, "-m".to_string(), timeout, url]);
        let _ = Command::new("curl")
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[tokio::test]
    async fn scoped_get_sends_project_and_continuity_headers() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock daemon");
        let address = listener.local_addr().expect("mock daemon address");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept scoped request");
            let mut buffer = [0u8; 4096];
            let read = stream.read(&mut buffer).expect("read scoped request");
            let request = String::from_utf8_lossy(&buffer[..read]).to_ascii_lowercase();
            assert!(request.starts_with("get /v1/work-loop/status?summary_only=true"));
            assert!(request.contains("x-scope-project-root: /tmp/focusa-project"));
            assert!(request.contains("x-scope-continuity-id: focusa-test-continuity"));
            let body = r#"{"status":"completed"}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .expect("write mock response");
        });
        let api = ApiClient {
            client: Client::new(),
            base: format!("http://{address}"),
        };
        let response = api
            .get_scoped(
                "/v1/work-loop/status?summary_only=true",
                "/tmp/focusa-project",
                "focusa-test-continuity",
            )
            .await
            .expect("scoped request succeeds");
        assert_eq!(response["status"], "completed");
        server.join().expect("mock daemon joins");
    }
}
