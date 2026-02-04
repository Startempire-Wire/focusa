//! HTTP client for the Focusa daemon API.
//!
//! All CLI commands funnel through this module.
//! Default endpoint: http://127.0.0.1:8787

use reqwest::Client;
use serde_json::Value;

const DEFAULT_BASE: &str = "http://127.0.0.1:8787";

pub struct ApiClient {
    client: Client,
    base: String,
}

impl ApiClient {
    pub fn new() -> Self {
        let base = std::env::var("FOCUSA_API_URL")
            .unwrap_or_else(|_| DEFAULT_BASE.to_string());
        Self {
            client: Client::new(),
            base,
        }
    }

    pub async fn get(&self, path: &str) -> anyhow::Result<Value> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Cannot reach daemon at {}: {}", url, e))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {} from {}: {}", status, url, body);
        }
        Ok(resp.json().await?)
    }

    pub async fn post(&self, path: &str, body: &Value) -> anyhow::Result<Value> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Cannot reach daemon at {}: {}", url, e))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {} from {}: {}", status, url, body);
        }
        Ok(resp.json().await?)
    }
}
