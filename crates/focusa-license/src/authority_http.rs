use crate::authority_client::{
    AuthorityNodeSummary, DeviceCodeChallenge, DeviceCodePollResponse, DeviceCodeStartRequest,
    SensitiveCredential,
};
use reqwest::{Client, Url};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCodePollRequest {
    pub request_id: Uuid,
    pub device_code: String,
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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthorityHttpError {
    #[error("authority endpoint set must use one safe origin")]
    UnsafeEndpointSet,
    #[error("authority HTTP policy budget is invalid")]
    InvalidBudget,
    #[error("authority request failed: {0}")]
    Request(&'static str),
    #[error("authority returned HTTP status {0}")]
    HttpStatus(u16),
    #[error("authority response exceeded the byte budget")]
    ResponseTooLarge,
    #[error("authority response schema is invalid")]
    InvalidResponse,
    #[error("authority request identity is incomplete: {0}")]
    MissingIdentity(&'static str),
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
        self.post(&self.policy.endpoints.start, request, request.request_id)
            .await
    }

    pub async fn poll(
        &self,
        request: &DeviceCodePollRequest,
    ) -> Result<DeviceCodePollResponse, AuthorityHttpError> {
        if request.device_code.trim().is_empty() {
            return Err(AuthorityHttpError::MissingIdentity("device_code"));
        }
        self.post(&self.policy.endpoints.poll, request, request.request_id)
            .await
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
        self.post(&self.policy.endpoints.refresh, &request, request_id)
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
        self.post(&self.policy.endpoints.nodes, &request, request_id)
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
        self.post(&self.policy.endpoints.deactivate_node, &request, request_id)
            .await
    }

    async fn post<T: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        endpoint: &Url,
        request: &T,
        request_id: Uuid,
    ) -> Result<R, AuthorityHttpError> {
        let response = self
            .client
            .post(endpoint.clone())
            .header("Idempotency-Key", request_id.to_string())
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
        if !response.status().is_success() {
            return Err(AuthorityHttpError::HttpStatus(response.status().as_u16()));
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|_| AuthorityHttpError::Request("response_read"))?;
        if bytes.len() > self.policy.max_response_bytes {
            return Err(AuthorityHttpError::ResponseTooLarge);
        }
        serde_json::from_slice(&bytes).map_err(|_| AuthorityHttpError::InvalidResponse)
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
