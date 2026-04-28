//! HTTP client for the Focusa daemon API.
//!
//! All CLI commands funnel through this module.
//! Default endpoint: http://127.0.0.1:8787

use reqwest::{Client, ClientBuilder};
use serde_json::Value;
use std::process::{Command, Stdio};
use std::time::Duration;

const DEFAULT_BASE: &str = "http://127.0.0.1:8787";
const DEFAULT_TIMEOUT_SECS: u64 = 2;

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
            .expect("Failed to build HTTP client");

        Self { client, base }
    }

    pub fn base_url(&self) -> &str {
        &self.base
    }

    pub fn http_client(&self) -> &Client {
        &self.client
    }

    pub async fn get(&self, path: &str) -> anyhow::Result<Value> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|err| classify_reqwest_error(err, &url))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("[API_HTTP_ERROR] status={} url={} body={}", status, url, body);
        }
        resp.json()
            .await
            .map_err(|err| classify_reqwest_error(err, &url))
    }

    pub async fn post(&self, path: &str, body: &Value) -> anyhow::Result<Value> {
        self.post_with_headers(path, body, &[]).await
    }

    pub async fn post_with_headers(&self, path: &str, body: &Value, headers: &[(&str, &str)]) -> anyhow::Result<Value> {
        let url = format!("{}{}", self.base, path);
        let mut req = self.client.post(&url).json(body);
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
            anyhow::bail!("[API_HTTP_ERROR] status={} url={} body={}", status, url, body);
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

        // Use curl for a truly synchronous HTTP request
        let _ = Command::new("curl")
            .args([
                "-s",
                "-X",
                "POST",
                "-H",
                "Content-Type: application/json",
                "-d",
                body_json.as_str(),
                "-m",
                &timeout_secs.to_string(),
                &url,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output();
    }
}
