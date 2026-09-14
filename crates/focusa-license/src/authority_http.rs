use crate::authority_client::{
    AuthorityNodeSummary, DeviceCodeChallenge, DeviceCodePollResponse, DeviceCodeStartRequest,
    SensitiveCredential,
};
use reqwest::{Client, Response, Url};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{collections::BTreeSet, time::Duration};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthorityEndpointSet {
    pub start: Url,
    pub poll: Url,
    pub refresh: Url,
    pub nodes: Url,
    pub deactivate_node: Url,
}

#[derive(Debug, Clone)]
pub struct AuthorityHttpPolicy {
    pub endpoints: AuthorityEndpointSet,
    pub timeout: Duration,
    pub max_response_bytes: usize,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCodePollRequest {
    pub request_id: Uuid,
    pub device_code: String,
}

impl std::fmt::Debug for DeviceCodePollRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DeviceCodePollRequest")
            .field("request_id", &self.request_id)
            .field("device_code", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseRefreshRequest {
    pub request_id: Uuid,
    pub product: String,
    pub node_id: String,
    pub refresh_credential: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseRefreshResponse {
    pub signed_lease: String,
    pub refresh_credential: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeListRequest {
    pub request_id: Uuid,
    pub product: String,
    pub node_id: String,
    pub refresh_credential: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeListResponse {
    pub nodes: Vec<AuthorityNodeSummary>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeactivateNodeRequest {
    pub request_id: Uuid,
    pub product: String,
    pub requesting_node_id: String,
    pub target_node_id: String,
    pub refresh_credential: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct AuthorityErrorEnvelope {
    error: String,
    #[serde(default)]
    request_id: Option<Uuid>,
    #[serde(default)]
    retry_after_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityFailureDisposition {
    RetryPoll,
    SlowDown,
    RestartAuthorization,
    CorrectProduct,
    RecoveryOnly,
    ManageNodes,
    AuthorityUnavailable,
    Denied,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthorityHttpError {
    #[error("authority endpoint set must use one safe origin")]
    UnsafeEndpointSet,
    #[error("authority HTTP policy budget is invalid")]
    InvalidBudget,
    #[error("authority request failed: {0}")]
    Request(&'static str),
    #[error("authority rejected request: status={status} code={code}")]
    AuthorityRejected {
        status: u16,
        code: String,
        retry_after_ms: Option<u64>,
    },
    #[error("authority response request correlation mismatched")]
    RequestCorrelationMismatch,
    #[error("authority response exceeded the byte budget")]
    ResponseTooLarge,
    #[error("authority response schema is invalid")]
    InvalidResponse,
    #[error("authority request identity is incomplete: {0}")]
    MissingIdentity(&'static str),
}

impl AuthorityHttpError {
    pub fn disposition(&self) -> AuthorityFailureDisposition {
        match self {
            Self::AuthorityRejected { code, .. } => match code.as_str() {
                "AUTHORIZATION_PENDING" => AuthorityFailureDisposition::RetryPoll,
                "SLOW_DOWN" => AuthorityFailureDisposition::SlowDown,
                "AUTHORIZATION_EXPIRED" => AuthorityFailureDisposition::RestartAuthorization,
                "WRONG_PRODUCT" => AuthorityFailureDisposition::CorrectProduct,
                "LEASE_REVOKED" | "LICENSE_REFUNDED" => AuthorityFailureDisposition::RecoveryOnly,
                "NODE_LIMIT_EXHAUSTED" => AuthorityFailureDisposition::ManageNodes,
                "AUTHORITY_UNAVAILABLE" => AuthorityFailureDisposition::AuthorityUnavailable,
                _ => AuthorityFailureDisposition::Denied,
            },
            Self::Request("timeout" | "connect" | "transport") => {
                AuthorityFailureDisposition::AuthorityUnavailable
            }
            _ => AuthorityFailureDisposition::Denied,
        }
    }
}

impl AuthorityHttpPolicy {
    pub fn validate(&self) -> Result<(), AuthorityHttpError> {
        if !(1..=120).contains(&self.timeout.as_secs())
            || !(1024..=16 * 1024 * 1024).contains(&self.max_response_bytes)
        {
            return Err(AuthorityHttpError::InvalidBudget);
        }
        let endpoints = [
            &self.endpoints.start,
            &self.endpoints.poll,
            &self.endpoints.refresh,
            &self.endpoints.nodes,
            &self.endpoints.deactivate_node,
        ];
        let origins = endpoints
            .iter()
            .map(|url| {
                (
                    url.scheme().to_string(),
                    url.host_str().unwrap_or("").to_ascii_lowercase(),
                    url.port_or_known_default(),
                )
            })
            .collect::<BTreeSet<_>>();
        if origins.len() != 1
            || endpoints.iter().any(|url| {
                let loopback = url
                    .host_str()
                    .is_some_and(|host| matches!(host, "127.0.0.1" | "::1" | "localhost"));
                (url.scheme() != "https" && !(url.scheme() == "http" && loopback))
                    || !url.username().is_empty()
                    || url.password().is_some()
                    || url.query().is_some()
                    || url.fragment().is_some()
            })
        {
            return Err(AuthorityHttpError::UnsafeEndpointSet);
        }
        Ok(())
    }
}

pub struct AuthorityHttpClient {
    client: Client,
    policy: AuthorityHttpPolicy,
}

impl AuthorityHttpClient {
    pub fn new(policy: AuthorityHttpPolicy) -> Result<Self, AuthorityHttpError> {
        policy.validate()?;
        let client = Client::builder()
            .timeout(policy.timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| AuthorityHttpError::Request("client_build"))?;
        Ok(Self { client, policy })
    }

    pub async fn start(
        &self,
        request: &DeviceCodeStartRequest,
    ) -> Result<DeviceCodeChallenge, AuthorityHttpError> {
        self.post(
            &self.policy.endpoints.start,
            request,
            request.request_id,
            "device_code_start",
        )
        .await
    }

    pub async fn poll(
        &self,
        request: &DeviceCodePollRequest,
    ) -> Result<DeviceCodePollResponse, AuthorityHttpError> {
        if request.device_code.trim().is_empty() {
            return Err(AuthorityHttpError::MissingIdentity("device_code"));
        }
        match self
            .post(
                &self.policy.endpoints.poll,
                request,
                request.request_id,
                "device_code_poll",
            )
            .await
        {
            Err(error) if error.disposition() == AuthorityFailureDisposition::RetryPoll => {
                Ok(DeviceCodePollResponse::AuthorizationPending)
            }
            Err(error) if error.disposition() == AuthorityFailureDisposition::SlowDown => {
                Ok(DeviceCodePollResponse::SlowDown)
            }
            Err(error)
                if error.disposition() == AuthorityFailureDisposition::RestartAuthorization =>
            {
                Ok(DeviceCodePollResponse::Expired)
            }
            Err(AuthorityHttpError::AuthorityRejected { code, .. }) => {
                Ok(DeviceCodePollResponse::Denied { reason_code: code })
            }
            result => result,
        }
    }

    pub async fn refresh(
        &self,
        request_id: Uuid,
        product: &str,
        node_id: &str,
        credential: &SensitiveCredential,
    ) -> Result<LeaseRefreshResponse, AuthorityHttpError> {
        require_identity(product, "product")?;
        require_identity(node_id, "node_id")?;
        let request = LeaseRefreshRequest {
            request_id,
            product: product.into(),
            node_id: node_id.into(),
            refresh_credential: credential.expose_for_protected_store().into(),
        };
        self.post(
            &self.policy.endpoints.refresh,
            &request,
            request_id,
            "lease_refresh",
        )
        .await
    }

    pub async fn nodes(
        &self,
        request_id: Uuid,
        product: &str,
        node_id: &str,
        credential: &SensitiveCredential,
    ) -> Result<NodeListResponse, AuthorityHttpError> {
        require_identity(product, "product")?;
        require_identity(node_id, "node_id")?;
        let request = NodeListRequest {
            request_id,
            product: product.into(),
            node_id: node_id.into(),
            refresh_credential: credential.expose_for_protected_store().into(),
        };
        self.post(
            &self.policy.endpoints.nodes,
            &request,
            request_id,
            "node_list",
        )
        .await
    }

    pub async fn deactivate_node(
        &self,
        request_id: Uuid,
        product: &str,
        requesting_node_id: &str,
        target_node_id: &str,
        credential: &SensitiveCredential,
    ) -> Result<NodeListResponse, AuthorityHttpError> {
        require_identity(product, "product")?;
        require_identity(requesting_node_id, "requesting_node_id")?;
        require_identity(target_node_id, "target_node_id")?;
        let request = DeactivateNodeRequest {
            request_id,
            product: product.into(),
            requesting_node_id: requesting_node_id.into(),
            target_node_id: target_node_id.into(),
            refresh_credential: credential.expose_for_protected_store().into(),
        };
        self.post(
            &self.policy.endpoints.deactivate_node,
            &request,
            request_id,
            "node_deactivate",
        )
        .await
    }

    async fn post<T: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        endpoint: &Url,
        request: &T,
        request_id: Uuid,
        operation: &'static str,
    ) -> Result<R, AuthorityHttpError> {
        let response = self
            .client
            .post(endpoint.clone())
            .header("Idempotency-Key", request_id.to_string())
            .header("X-Request-Id", request_id.to_string())
            .header("X-Focusa-Operation", operation)
            .json(request)
            .send()
            .await
            .map_err(|error| {
                AuthorityHttpError::Request(if error.is_timeout() {
                    "timeout"
                } else if error.is_connect() {
                    "connect"
                } else {
                    "transport"
                })
            })?;
        let status = response.status();
        let bytes = read_bounded_response(response, self.policy.max_response_bytes).await?;
        if !status.is_success() {
            return Err(authority_rejection(status.as_u16(), request_id, &bytes));
        }
        serde_json::from_slice(&bytes).map_err(|_| AuthorityHttpError::InvalidResponse)
    }
}

async fn read_bounded_response(
    mut response: Response,
    max_response_bytes: usize,
) -> Result<Vec<u8>, AuthorityHttpError> {
    if response
        .content_length()
        .is_some_and(|length| length > max_response_bytes as u64)
    {
        return Err(AuthorityHttpError::ResponseTooLarge);
    }
    let mut body = Vec::with_capacity(
        response
            .content_length()
            .unwrap_or_default()
            .min(max_response_bytes as u64) as usize,
    );
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| AuthorityHttpError::Request("response_read"))?
    {
        if body
            .len()
            .checked_add(chunk.len())
            .is_none_or(|length| length > max_response_bytes)
        {
            return Err(AuthorityHttpError::ResponseTooLarge);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn authority_rejection(status: u16, request_id: Uuid, body: &[u8]) -> AuthorityHttpError {
    let envelope = serde_json::from_slice::<AuthorityErrorEnvelope>(body).ok();
    if envelope
        .as_ref()
        .and_then(|value| value.request_id)
        .is_some_and(|value| value != request_id)
    {
        return AuthorityHttpError::RequestCorrelationMismatch;
    }
    let code = envelope
        .as_ref()
        .map(|value| value.error.trim())
        .filter(|value| {
            (3..=64).contains(&value.len())
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        })
        .map(str::to_string)
        .unwrap_or_else(|| format!("AUTHORITY_HTTP_STATUS_{status}"));
    AuthorityHttpError::AuthorityRejected {
        status,
        code,
        retry_after_ms: envelope
            .and_then(|value| value.retry_after_ms)
            .map(|value| value.min(60_000)),
    }
}

fn require_identity(value: &str, field: &'static str) -> Result<(), AuthorityHttpError> {
    if value.trim().is_empty() {
        Err(AuthorityHttpError::MissingIdentity(field))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoints(origin: &str) -> AuthorityEndpointSet {
        AuthorityEndpointSet {
            start: Url::parse(&format!("{origin}/device/start")).unwrap(),
            poll: Url::parse(&format!("{origin}/device/poll")).unwrap(),
            refresh: Url::parse(&format!("{origin}/lease/refresh")).unwrap(),
            nodes: Url::parse(&format!("{origin}/nodes")).unwrap(),
            deactivate_node: Url::parse(&format!("{origin}/nodes/deactivate")).unwrap(),
        }
    }

    #[test]
    fn poll_request_debug_redacts_device_credential() {
        let request = DeviceCodePollRequest {
            request_id: Uuid::nil(),
            device_code: "device-secret".to_string(),
        };
        let rendered = format!("{request:?}");
        assert!(rendered.contains("[REDACTED]"));
        assert!(!rendered.contains("device-secret"));
    }

    #[test]
    fn authority_errors_are_stable_correlated_and_retry_bounded() {
        let request_id = Uuid::now_v7();
        let body = serde_json::to_vec(&serde_json::json!({
            "error": "NODE_LIMIT_EXHAUSTED",
            "request_id": request_id,
            "retry_after_ms": 90_000,
        }))
        .unwrap();
        assert_eq!(
            authority_rejection(409, request_id, &body),
            AuthorityHttpError::AuthorityRejected {
                status: 409,
                code: "NODE_LIMIT_EXHAUSTED".to_string(),
                retry_after_ms: Some(60_000),
            }
        );

        let mismatched = serde_json::to_vec(&serde_json::json!({
            "error": "AUTHORIZATION_PENDING",
            "request_id": Uuid::now_v7(),
        }))
        .unwrap();
        assert_eq!(
            authority_rejection(409, request_id, &mismatched),
            AuthorityHttpError::RequestCorrelationMismatch
        );
        assert_eq!(
            authority_rejection(503, request_id, b"not-json"),
            AuthorityHttpError::AuthorityRejected {
                status: 503,
                code: "AUTHORITY_HTTP_STATUS_503".to_string(),
                retry_after_ms: None,
            }
        );
        for (code, expected) in [
            (
                "AUTHORIZATION_PENDING",
                AuthorityFailureDisposition::RetryPoll,
            ),
            ("SLOW_DOWN", AuthorityFailureDisposition::SlowDown),
            (
                "AUTHORIZATION_EXPIRED",
                AuthorityFailureDisposition::RestartAuthorization,
            ),
            ("WRONG_PRODUCT", AuthorityFailureDisposition::CorrectProduct),
            ("LEASE_REVOKED", AuthorityFailureDisposition::RecoveryOnly),
            (
                "LICENSE_REFUNDED",
                AuthorityFailureDisposition::RecoveryOnly,
            ),
            (
                "NODE_LIMIT_EXHAUSTED",
                AuthorityFailureDisposition::ManageNodes,
            ),
            (
                "AUTHORITY_UNAVAILABLE",
                AuthorityFailureDisposition::AuthorityUnavailable,
            ),
        ] {
            assert_eq!(
                AuthorityHttpError::AuthorityRejected {
                    status: 409,
                    code: code.to_string(),
                    retry_after_ms: None,
                }
                .disposition(),
                expected
            );
        }
    }

    #[test]
    fn endpoints_are_same_origin_safe_and_bounded() {
        let valid = AuthorityHttpPolicy {
            endpoints: endpoints("https://authority.example.test"),
            timeout: Duration::from_secs(30),
            max_response_bytes: 1024 * 1024,
        };
        assert!(valid.validate().is_ok());

        let mut mixed = valid.clone();
        mixed.endpoints.poll = Url::parse("https://foreign.example.test/poll").unwrap();
        assert_eq!(mixed.validate(), Err(AuthorityHttpError::UnsafeEndpointSet));

        let insecure = AuthorityHttpPolicy {
            endpoints: endpoints("http://authority.example.test"),
            timeout: Duration::from_secs(30),
            max_response_bytes: 1024 * 1024,
        };
        assert_eq!(
            insecure.validate(),
            Err(AuthorityHttpError::UnsafeEndpointSet)
        );
    }
}
