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

pub struct ApiClient {
    client: Client,
    base: String,
}

impl ApiClient {
    pub fn new() -> Self {
        let base = std::env::var("FOCUSA_API_URL").unwrap_or_else(|_| DEFAULT_BASE.to_string());

        let timeout = std::env::var("FOCUSA_API_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_TIMEOUT_SECS);

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
        let resp = self.client.get(&url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {} from {}: {}", status, url, body);
        }
        Ok(resp.json().await?)
    }

    pub async fn post(&self, path: &str, body: &Value) -> anyhow::Result<Value> {
        let url = format!("{}{}", self.base, path);
        let resp = self.client.post(&url).json(body).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {} from {}: {}", status, url, body);
        }
        Ok(resp.json().await?)
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
