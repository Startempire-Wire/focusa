use crate::{
    AdapterFuture, LettaAdapterError, LettaTransport, LettaTurnRequest, LettaTurnResponse,
};
use reqwest::{Client, Url};
use std::{sync::Arc, time::Duration};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct LettaEndpointPolicy {
    endpoint: Url,
    timeout: Duration,
    max_response_bytes: usize,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LettaEndpointError {
    #[error("Letta endpoint must use HTTPS or loopback HTTP")]
    InsecureEndpoint,
    #[error("Letta endpoint must not contain credentials, query, or fragment")]
    UnsafeEndpoint,
    #[error("Letta endpoint timeout is outside 1..=120 seconds")]
    InvalidTimeout,
    #[error("Letta endpoint response budget is outside 1024..=16777216 bytes")]
    InvalidResponseBudget,
    #[error("Letta endpoint URL is invalid")]
    InvalidUrl,
}

impl LettaEndpointPolicy {
    pub fn new(
        endpoint: &str,
        timeout: Duration,
        max_response_bytes: usize,
    ) -> Result<Self, LettaEndpointError> {
        let endpoint = Url::parse(endpoint).map_err(|_| LettaEndpointError::InvalidUrl)?;
        let loopback = endpoint
            .host_str()
            .is_some_and(|host| matches!(host, "127.0.0.1" | "::1" | "localhost"));
        if endpoint.scheme() != "https" && !(endpoint.scheme() == "http" && loopback) {
            return Err(LettaEndpointError::InsecureEndpoint);
        }
        if !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
        {
            return Err(LettaEndpointError::UnsafeEndpoint);
        }
        if !(1..=120).contains(&timeout.as_secs()) {
            return Err(LettaEndpointError::InvalidTimeout);
        }
        if !(1024..=16 * 1024 * 1024).contains(&max_response_bytes) {
            return Err(LettaEndpointError::InvalidResponseBudget);
        }
        Ok(Self {
            endpoint,
            timeout,
            max_response_bytes,
        })
    }

    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }
}

pub trait LettaCredentialProvider: Send + Sync {
    fn bearer_token<'a>(&'a self) -> AdapterFuture<'a, Result<String, LettaAdapterError>>;
}

pub struct HttpLettaTransport<C> {
    client: Client,
    policy: LettaEndpointPolicy,
    credentials: Arc<C>,
}

impl<C> HttpLettaTransport<C>
where
    C: LettaCredentialProvider,
{
    pub fn new(
        policy: LettaEndpointPolicy,
        credentials: Arc<C>,
    ) -> Result<Self, LettaAdapterError> {
        let client = Client::builder()
            .timeout(policy.timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| LettaAdapterError::Transport(format!("client_build:{error}")))?;
        Ok(Self {
            client,
            policy,
            credentials,
        })
    }
}

impl<C> LettaTransport for HttpLettaTransport<C>
where
    C: LettaCredentialProvider + 'static,
{
    fn send_turn<'a>(
        &'a self,
        request: &'a LettaTurnRequest,
    ) -> AdapterFuture<'a, Result<LettaTurnResponse, LettaAdapterError>> {
        Box::pin(async move {
            let token = self.credentials.bearer_token().await?;
            if token.trim().is_empty() {
                return Err(LettaAdapterError::Transport("credential_missing".into()));
            }
            let response = self
                .client
                .post(self.policy.endpoint.clone())
                .bearer_auth(token)
                .header("Idempotency-Key", &request.event_id)
                .json(request)
                .send()
                .await
                .map_err(|error| {
                    LettaAdapterError::Transport(format!("request_failed:{}", error.classify()))
                })?;
            if !response.status().is_success() {
                return Err(LettaAdapterError::Transport(format!(
                    "http_status:{}",
                    response.status().as_u16()
                )));
            }
            let bytes = response
                .bytes()
                .await
                .map_err(|_| LettaAdapterError::Transport("response_read_failed".into()))?;
            if bytes.len() > self.policy.max_response_bytes {
                return Err(LettaAdapterError::Transport("response_too_large".into()));
            }
            serde_json::from_slice(&bytes)
                .map_err(|_| LettaAdapterError::Transport("response_schema_invalid".into()))
        })
    }
}

trait ReqwestErrorClass {
    fn classify(&self) -> &'static str;
}

impl ReqwestErrorClass for reqwest::Error {
    fn classify(&self) -> &'static str {
        if self.is_timeout() {
            "timeout"
        } else if self.is_connect() {
            "connect"
        } else if self.is_request() {
            "request"
        } else if self.is_decode() {
            "decode"
        } else {
            "transport"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_policy_requires_pinned_safe_origin_and_budgets() {
        assert!(
            LettaEndpointPolicy::new(
                "https://letta.example.test/v1/focusa/turns",
                Duration::from_secs(30),
                1024 * 1024,
            )
            .is_ok()
        );
        assert!(
            LettaEndpointPolicy::new(
                "http://127.0.0.1:8283/v1/focusa/turns",
                Duration::from_secs(30),
                1024 * 1024,
            )
            .is_ok()
        );
        assert_eq!(
            LettaEndpointPolicy::new(
                "http://letta.example.test/v1/focusa/turns",
                Duration::from_secs(30),
                1024 * 1024,
            )
            .unwrap_err(),
            LettaEndpointError::InsecureEndpoint
        );
        assert_eq!(
            LettaEndpointPolicy::new(
                "https://user:secret@letta.example.test/v1/turns?token=bad",
                Duration::from_secs(30),
                1024 * 1024,
            )
            .unwrap_err(),
            LettaEndpointError::UnsafeEndpoint
        );
    }
}
