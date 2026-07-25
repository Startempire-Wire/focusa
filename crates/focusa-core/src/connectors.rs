//! Typed, provider-neutral connector primitives for Mission Canvas.
//!
//! Connectors never own canonical project state. They expose bounded capability,
//! health, retry, and rate-policy behavior and return redacted envelopes.

use async_trait::async_trait;
use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorCapability {
    pub capability_id: String,
    pub operation: String,
    pub side_effecting: bool,
    pub permission_scope: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorHealthStatus {
    Ready,
    Degraded,
    Offline,
    Unauthorized,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorHealth {
    pub status: ConnectorHealthStatus,
    pub checked_at: String,
    pub latency_ms: u64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorRetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
    pub maximum_backoff_ms: u64,
    pub retry_statuses: Vec<u16>,
}

impl Default for ConnectorRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 100,
            maximum_backoff_ms: 2_000,
            retry_statuses: vec![408, 425, 429, 500, 502, 503, 504],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorRatePolicy {
    pub requests_per_second: u32,
    pub burst: u32,
}

impl Default for ConnectorRatePolicy {
    fn default() -> Self {
        Self {
            requests_per_second: 5,
            burst: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorDescriptor {
    pub connector_id: String,
    pub connector_kind: String,
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    pub health_url: String,
    pub allowed_origin: String,
    pub capabilities: Vec<ConnectorCapability>,
    pub retry_policy: ConnectorRetryPolicy,
    pub rate_policy: ConnectorRatePolicy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectorResult {
    pub schema: String,
    pub connector_id: String,
    pub capability_id: String,
    pub status: String,
    pub value: Value,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorErrorEnvelope {
    pub schema: String,
    pub connector_id: String,
    pub status: String,
    pub failure_class: String,
    pub message: String,
    pub retriable: bool,
    pub retry_after_ms: Option<u64>,
}

#[async_trait]
pub trait ConnectorAdapter: Send + Sync {
    fn descriptor(&self) -> &ConnectorDescriptor;
    async fn health(&self) -> ConnectorHealth;
    async fn execute(
        &self,
        capability_id: &str,
        method: Method,
        url: &str,
        body: Option<Value>,
    ) -> Result<ConnectorResult, ConnectorErrorEnvelope>;
}

pub struct HttpJsonConnector {
    descriptor: ConnectorDescriptor,
    client: Client,
    last_request: Arc<Mutex<Option<Instant>>>,
}

impl HttpJsonConnector {
    #[allow(clippy::result_large_err)]
    pub fn new(descriptor: ConnectorDescriptor) -> Result<Self, ConnectorErrorEnvelope> {
        if descriptor.connector_id.trim().is_empty()
            || descriptor.project_root.trim().is_empty()
            || descriptor.continuity_id.trim().is_empty()
            || descriptor.attachment_id.trim().is_empty()
            || descriptor.rate_policy.requests_per_second == 0
        {
            return Err(error(
                &descriptor.connector_id,
                "invalid_config",
                "Connector scope and rate policy are required",
                false,
                None,
            ));
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|_| {
                error(
                    &descriptor.connector_id,
                    "client_init",
                    "Connector HTTP client initialization failed",
                    false,
                    None,
                )
            })?;
        Ok(Self {
            descriptor,
            client,
            last_request: Arc::new(Mutex::new(None)),
        })
    }

    async fn apply_rate_policy(&self) {
        let interval =
            Duration::from_secs_f64(1.0 / self.descriptor.rate_policy.requests_per_second as f64);
        let mut last = self.last_request.lock().await;
        if let Some(previous) = *last {
            let elapsed = previous.elapsed();
            if elapsed < interval {
                tokio::time::sleep(interval - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }

    fn capability(&self, capability_id: &str) -> Option<&ConnectorCapability> {
        self.descriptor
            .capabilities
            .iter()
            .find(|item| item.capability_id == capability_id)
    }

    fn origin_allowed(&self, url: &str) -> bool {
        if !url.starts_with(&self.descriptor.allowed_origin) {
            return false;
        }
        let remainder = &url[self.descriptor.allowed_origin.len()..];
        remainder.is_empty() || remainder.starts_with('/') || remainder.starts_with('?')
    }
}

#[async_trait]
impl ConnectorAdapter for HttpJsonConnector {
    fn descriptor(&self) -> &ConnectorDescriptor {
        &self.descriptor
    }

    async fn health(&self) -> ConnectorHealth {
        let started = Instant::now();
        let response = self.client.get(&self.descriptor.health_url).send().await;
        let (status, message) = match response {
            Ok(value) if value.status().is_success() => (
                ConnectorHealthStatus::Ready,
                "Connector health check passed",
            ),
            Ok(value)
                if value.status() == StatusCode::UNAUTHORIZED
                    || value.status() == StatusCode::FORBIDDEN =>
            {
                (
                    ConnectorHealthStatus::Unauthorized,
                    "Connector authorization required",
                )
            }
            Ok(_) => (
                ConnectorHealthStatus::Degraded,
                "Connector health check returned a non-success status",
            ),
            Err(_) => (
                ConnectorHealthStatus::Offline,
                "Connector health check failed",
            ),
        };
        ConnectorHealth {
            status,
            checked_at: chrono::Utc::now().to_rfc3339(),
            latency_ms: started.elapsed().as_millis() as u64,
            message: message.into(),
        }
    }

    async fn execute(
        &self,
        capability_id: &str,
        method: Method,
        url: &str,
        body: Option<Value>,
    ) -> Result<ConnectorResult, ConnectorErrorEnvelope> {
        let capability = self.capability(capability_id).ok_or_else(|| {
            error(
                &self.descriptor.connector_id,
                "capability_missing",
                "Connector capability is not declared",
                false,
                None,
            )
        })?;
        if capability.operation != method.as_str() {
            return Err(error(
                &self.descriptor.connector_id,
                "method_denied",
                "Connector method is outside the declared capability",
                false,
                None,
            ));
        }
        if !self.origin_allowed(url) {
            return Err(error(
                &self.descriptor.connector_id,
                "origin_denied",
                "Connector target is outside the allowed origin",
                false,
                None,
            ));
        }
        let policy = &self.descriptor.retry_policy;
        let mut backoff = policy.initial_backoff_ms;
        for attempt in 1..=policy.max_attempts.max(1) {
            self.apply_rate_policy().await;
            let mut request = self.client.request(method.clone(), url);
            if let Some(value) = body.clone() {
                request = request.json(&value);
            }
            match request.send().await {
                Ok(response) if response.status().is_success() => {
                    let value = response.json::<Value>().await.map_err(|_| {
                        error(
                            &self.descriptor.connector_id,
                            "invalid_response",
                            "Connector response was not valid JSON",
                            false,
                            None,
                        )
                    })?;
                    return Ok(ConnectorResult {
                        schema: "focusa.connector_result.v1".into(),
                        connector_id: self.descriptor.connector_id.clone(),
                        capability_id: capability_id.into(),
                        status: "completed".into(),
                        value,
                        evidence_refs: vec![format!(
                            "connector:{}:{}",
                            self.descriptor.connector_id, capability_id
                        )],
                    });
                }
                Ok(response) => {
                    let code = response.status().as_u16();
                    let retriable =
                        policy.retry_statuses.contains(&code) && attempt < policy.max_attempts;
                    if !retriable {
                        return Err(error(
                            &self.descriptor.connector_id,
                            "http_status",
                            "Connector returned a non-success status",
                            false,
                            None,
                        ));
                    }
                }
                Err(_) if attempt >= policy.max_attempts => {
                    return Err(error(
                        &self.descriptor.connector_id,
                        "transport",
                        "Connector request failed",
                        false,
                        None,
                    ));
                }
                Err(_) => {}
            }
            tokio::time::sleep(Duration::from_millis(backoff)).await;
            backoff = (backoff.saturating_mul(2)).min(policy.maximum_backoff_ms);
        }
        Err(error(
            &self.descriptor.connector_id,
            "retry_exhausted",
            "Connector retry policy exhausted",
            false,
            None,
        ))
    }
}

fn error(
    connector_id: &str,
    class: &str,
    message: &str,
    retriable: bool,
    retry_after_ms: Option<u64>,
) -> ConnectorErrorEnvelope {
    ConnectorErrorEnvelope {
        schema: "focusa.connector_error.v1".into(),
        connector_id: connector_id.into(),
        status: "blocked".into(),
        failure_class: class.into(),
        message: message.into(),
        retriable,
        retry_after_ms,
    }
}
