//! Activation authority HTTP transport (Spec 152E §10 public activation API,
//! §9 facade binding, §20 stable failures): the concrete
//! [`ActivationAuthority`] implementation every presenter (CLI, Rust
//! installer, TUI, daemon REST, agent JSON) shares. It maps each frozen
//! operation (`spec152e-activation-call-stack.v1.yaml`) to its endpoint with
//! the typed request context (`request_id`, `facade_id`, `presenter`,
//! `install_channel`, idempotency key on mutations) and decodes the
//! transport reply into the client's typed reply shapes.
//!
//! The transport carries no identity, commerce, product, payment,
//! Evaluation, license, node, or lease decision: it serializes the caller's
//! typed request, sends it to the authority, and decodes the typed reply or
//! the frozen error code. Unknown error labels, malformed replies, and
//! transport failures all fail closed as `AUTHORITY_UNAVAILABLE` so the
//! shared reducer is never presented with invented authority state.

use crate::activation_client::{
    ActivationAuthority, ActivationJourney, ActivationStartReply, CheckoutOutcome, PollOutcome,
    PublicOffer,
};
use crate::activation_facade::{ActivationError, ActivationErrorCode, ActivationRequestContext};
use crate::activation_reducer::ActivationTransition;
use crate::authority::SignedEnvelope;
use crate::authority_client::SensitiveCredential;
use reqwest::Url;
use reqwest::blocking::Client as BlockingClient;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// One authority-owned terminal delivery bundle. The `lease_envelope` from
/// `activation.poll` carries this JSON so presenters can verify and persist
/// the signed key set plus lease through the canonical authority store
/// without ever touching raw keys or credentials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseDeliveryEnvelope {
    pub schema: String,
    pub key_set: SignedEnvelope,
    pub lease: SignedEnvelope,
}

impl LeaseDeliveryEnvelope {
    pub const SCHEMA: &'static str = "focusa.lease_delivery_envelope.v1";

    pub fn parse(raw: &str) -> Result<Self, ActivationHttpError> {
        let parsed: Self =
            serde_json::from_str(raw).map_err(|_| ActivationHttpError::MalformedReply)?;
        if parsed.schema != Self::SCHEMA {
            return Err(ActivationHttpError::MalformedReply);
        }
        if parsed.key_set.schema.is_empty()
            || parsed.key_set.signer_key_id.is_empty()
            || parsed.key_set.payload_b64.is_empty()
            || parsed.lease.schema.is_empty()
            || parsed.lease.signer_key_id.is_empty()
            || parsed.lease.payload_b64.is_empty()
        {
            return Err(ActivationHttpError::MalformedReply);
        }
        Ok(parsed)
    }
}

/// Authority HTTP policy: bounded base URL, timeout, and response size. All
/// values are validated fail-closed before any request can be issued.
#[derive(Debug, Clone)]
pub struct ActivationHttpPolicy {
    pub base_url: Url,
    pub timeout: Duration,
    pub max_response_bytes: usize,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ActivationHttpError {
    #[error("activation authority policy is invalid: {0}")]
    InvalidPolicy(&'static str),
    #[error("activation authority reply is malformed")]
    MalformedReply,
}

impl ActivationHttpPolicy {
    pub fn validate(&self) -> Result<(), ActivationHttpError> {
        if self.base_url.scheme() != "https" {
            return Err(ActivationHttpError::InvalidPolicy(
                "authority base URL must use https",
            ));
        }
        if self.base_url.host_str().is_none() {
            return Err(ActivationHttpError::InvalidPolicy(
                "authority base URL has no host",
            ));
        }
        if self.timeout.is_zero() || self.timeout > Duration::from_secs(300) {
            return Err(ActivationHttpError::InvalidPolicy(
                "authority timeout must be within 1..=300 seconds",
            ));
        }
        if !(1024..=4 * 1024 * 1024).contains(&self.max_response_bytes) {
            return Err(ActivationHttpError::InvalidPolicy(
                "authority max response must be within 1 KiB..=4 MiB",
            ));
        }
        Ok(())
    }
}

/// Concrete activation authority transport. Blocking transport is
/// intentional: the shared [`ActivationSession`] contract is synchronous so
/// every presenter (interactive CLI, installer) shares one synchronous
/// driver; calls block only for the bounded policy timeout.
pub struct ActivationHttpClient {
    policy: ActivationHttpPolicy,
    http: BlockingClient,
}

impl ActivationHttpClient {
    pub fn new(policy: ActivationHttpPolicy) -> Result<Self, ActivationHttpError> {
        policy.validate()?;
        let http = BlockingClient::builder()
            .timeout(policy.timeout)
            .connect_timeout(policy.timeout)
            .build()
            .map_err(|_| ActivationHttpError::InvalidPolicy("reqwest client build failed"))?;
        Ok(Self { policy, http })
    }

    fn endpoint(&self, path: &str) -> Result<Url, ActivationHttpError> {
        // 315: strip leading '/' so Url::join preserves WordPress namespace path
        // https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/ + "activation/start"
        // not /v1/activation/start which discards the namespace.
        let relative = path.trim_start_matches('/');
        self.policy
            .base_url
            .join(relative)
            .map_err(|_| ActivationHttpError::InvalidPolicy("authority path join failed"))
    }

    /// POST a typed mutation request and decode the reply value.
    fn post<T: Serialize>(
        &self,
        path: &str,
        request: &WireActivationRequest<T>,
    ) -> Result<serde_json::Value, ActivationError> {
        let url = self
            .endpoint(path)
            .map_err(|_| self.unavailable("request-unknown"))?;
        let response = self
            .http
            .post(url)
            .header("X-Request-Id", &request.request_id)
            .header("Idempotency-Key", &request.idempotency_key)
            .json(request)
            .send()
            .map_err(|_| self.unavailable(&request.request_id))?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            // Typed routing diagnostic without exposing body secrets
            if status == 404 || status == 405 {
                return Err(ActivationError::new(
                    ActivationErrorCode::AuthorityUnavailable,
                    request.request_id.clone(),
                ));
            }
            // Spec 152E §20 (#344): surface the authority's typed error code
            // from the reply body instead of collapsing every non-2xx into
            // AUTHORITY_UNAVAILABLE. Fail closed when the body is not a
            // decodable typed envelope.
            let body = response.text().unwrap_or_default();
            return decode_response_body(&request.request_id, body, self.policy.max_response_bytes);
        }
        decode_response_body(
            &request.request_id,
            response
                .text()
                .map_err(|_| self.unavailable(&request.request_id))?,
            self.policy.max_response_bytes,
        )
    }

    /// GET a typed read operation; `registration_id` is carried as the
    /// opaque query parameter (the frozen call stack lists it as the only
    /// input for `activation.offers`).
    fn get(
        &self,
        path: &str,
        request_id: &str,
        registration_id: Option<&str>,
    ) -> Result<serde_json::Value, ActivationError> {
        let mut url = self
            .endpoint(path)
            .map_err(|_| self.unavailable(request_id))?;
        if let Some(registration_id) = registration_id {
            url.query_pairs_mut()
                .append_pair("registration_id", registration_id);
        }
        let response = self
            .http
            .get(url)
            .header("X-Request-Id", request_id)
            .send()
            .map_err(|_| self.unavailable(request_id))?;
        if !response.status().is_success() {
            // Spec 152E §20 (#344): typed authority errors surface verbatim;
            // non-JSON or oversized bodies still fail closed.
            let body = response.text().unwrap_or_default();
            return decode_response_body(request_id, body, self.policy.max_response_bytes);
        }
        decode_response_body(
            request_id,
            response.text().map_err(|_| self.unavailable(request_id))?,
            self.policy.max_response_bytes,
        )
    }

    fn unavailable(&self, request_id: &str) -> ActivationError {
        ActivationError::new(
            ActivationErrorCode::AuthorityUnavailable,
            request_id.to_string(),
        )
    }
}

/// Wire envelope shared by every mutation operation. The typed request
/// context fields are the frozen `required_all` fields; mutations require an
/// idempotency key (validated by the shared client before this transport).
#[derive(Debug, Serialize)]
pub struct WireActivationRequest<T> {
    pub request_id: String,
    pub idempotency_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_id: Option<String>,
    pub facade_id: String,
    pub presenter: String,
    pub install_channel: String,
    pub origin: String,
    #[serde(flatten)]
    pub payload: T,
}

fn mutation_request<T: Serialize>(
    context: &ActivationRequestContext,
    registration_id: Option<String>,
    payload: T,
) -> WireActivationRequest<T> {
    WireActivationRequest {
        request_id: context.request_id.clone(),
        idempotency_key: context
            .idempotency_key
            .clone()
            .unwrap_or_else(|| context.request_id.clone()),
        registration_id,
        facade_id: context.facade_id.clone(),
        presenter: context.presenter.clone(),
        install_channel: context.install_channel.clone(),
        origin: context.origin.clone(),
        payload,
    }
}

// ── Per-operation payloads (frozen input fields only) ─────────────────────

#[derive(Debug, Serialize)]
pub struct StartPayload {
    pub email: String,
    pub public_product_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_redirect_handle: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VerifyPayload {
    pub one_time_verifier: String,
}

#[derive(Debug, Serialize)]
pub struct SelectOfferPayload {
    pub public_product_code: String,
    pub journey: String,
}

#[derive(Debug, Serialize)]
pub struct CheckoutPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_redirect_handle: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExistingLicensePayload {
    pub human_license_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_public_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PollPayload {
    pub opaque_poll_credential: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_public_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RefreshPayload {
    pub node_id: String,
    pub refresh_credential: String,
    pub current_sequence: u64,
}

#[derive(Debug, Serialize)]
pub struct DeactivatePayload {
    pub node_id: String,
}

#[derive(Debug, Serialize)]
pub struct ManageLinkPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_redirect_handle: Option<String>,
}

// ── Transport reply decoding (mirrors the client reply shapes) ────────────

#[derive(Debug, Deserialize)]
struct WireTransitions {
    transitions: Vec<ActivationTransition>,
}

#[derive(Debug, Deserialize)]
struct WireStartReply {
    transitions: Vec<ActivationTransition>,
    #[serde(default)]
    poll_credential: Option<String>,
    #[serde(default)]
    registration_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WireCheckoutReply {
    transitions: Vec<ActivationTransition>,
    #[serde(default)]
    safe_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WirePollReply {
    transitions: Vec<ActivationTransition>,
    #[serde(default)]
    one_time_key_envelope: Option<String>,
    #[serde(default)]
    node_id: Option<String>,
    #[serde(default)]
    lease_envelope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WireOffer {
    public_code: String,
    display_name: String,
    journey: String,
}

#[derive(Debug, Deserialize)]
struct WireOffersReply {
    offers: Vec<WireOffer>,
}

#[derive(Debug, Deserialize)]
struct WireNodesReply {
    nodes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct WireLinkReply {
    link: String,
}

#[derive(Debug, Deserialize)]
struct WireErrorReply {
    #[serde(default)]
    error: Option<WireError>,
}

#[derive(Debug, Deserialize)]
struct WireError {
    code: String,
    #[serde(default)]
    next_action: Option<String>,
}

/// Map a frozen error label to its typed code. Unknown labels fail closed to
/// `None` so callers can only ever surface `AUTHORITY_UNAVAILABLE`, never an
/// invented authority decision.
pub fn code_from_label(label: &str) -> Option<ActivationErrorCode> {
    ActivationErrorCode::ALL
        .iter()
        .copied()
        .find(|code| code.label() == label)
}

/// Decode a bounded response body into the shared envelope/error shape.
fn decode_response_body(
    request_id: &str,
    body: String,
    max_response_bytes: usize,
) -> Result<serde_json::Value, ActivationError> {
    if body.len() > max_response_bytes {
        return Err(ActivationError::new(
            ActivationErrorCode::AuthorityUnavailable,
            request_id.to_string(),
        ));
    }
    let value: serde_json::Value = serde_json::from_str(&body).map_err(|_| {
        ActivationError::new(
            ActivationErrorCode::AuthorityUnavailable,
            request_id.to_string(),
        )
    })?;
    if let Some(error) = value.get("error") {
        let code = error
            .get("code")
            .and_then(serde_json::Value::as_str)
            .and_then(code_from_label)
            .unwrap_or(ActivationErrorCode::AuthorityUnavailable);
        return Err(ActivationError::new(code, request_id.to_string()));
    }
    Ok(value)
}

fn decode_transitions(
    request_id: &str,
    value: serde_json::Value,
) -> Result<Vec<ActivationTransition>, ActivationError> {
    serde_json::from_value::<WireTransitions>(value)
        .map(|reply| reply.transitions)
        .map_err(|_| {
            ActivationError::new(
                ActivationErrorCode::AuthorityUnavailable,
                request_id.to_string(),
            )
        })
}

impl ActivationAuthority for ActivationHttpClient {
    fn start(
        &self,
        context: &ActivationRequestContext,
        email: &str,
        public_product_code: &str,
        device_public_key: Option<&str>,
    ) -> Result<ActivationStartReply, ActivationError> {
        let request = mutation_request(
            context,
            None,
            StartPayload {
                email: email.to_string(),
                public_product_code: public_product_code.to_string(),
                device_public_key: device_public_key.map(str::to_string),
                safe_redirect_handle: None,
            },
        );
        let value = self.post(facade_operation_path::START, &request)?;
        let reply: WireStartReply = serde_json::from_value(value).map_err(|_| {
            ActivationError::new(
                ActivationErrorCode::AuthorityUnavailable,
                context.request_id.clone(),
            )
        })?;
        Ok(ActivationStartReply {
            transitions: reply.transitions,
            poll_credential: reply.poll_credential,
            registration_id: reply.registration_id,
        })
    }

    fn verify(
        &self,
        context: &ActivationRequestContext,
        registration_id: &str,
        one_time_verifier: &str,
    ) -> Result<Vec<ActivationTransition>, ActivationError> {
        let request = mutation_request(
            context,
            Some(registration_id.to_string()),
            VerifyPayload {
                one_time_verifier: one_time_verifier.to_string(),
            },
        );
        let value = self.post(facade_operation_path::VERIFY, &request)?;
        decode_transitions(&context.request_id, value)
    }

    fn offers(
        &self,
        context: &ActivationRequestContext,
        registration_id: &str,
    ) -> Result<Vec<PublicOffer>, ActivationError> {
        let value = self.get(
            facade_operation_path::OFFERS,
            &context.request_id,
            Some(registration_id),
        )?;
        let reply: WireOffersReply = serde_json::from_value(value).map_err(|_| {
            ActivationError::new(
                ActivationErrorCode::AuthorityUnavailable,
                context.request_id.clone(),
            )
        })?;
        reply
            .offers
            .into_iter()
            .map(|offer| {
                let journey = match offer.journey.as_str() {
                    "purchase" => ActivationJourney::Purchase,
                    "limited_access" => ActivationJourney::LimitedAccess,
                    "existing_key" => ActivationJourney::ExistingKey,
                    _ => {
                        return Err(ActivationError::new(
                            ActivationErrorCode::AuthorityUnavailable,
                            context.request_id.clone(),
                        ));
                    }
                };
                Ok(PublicOffer {
                    public_code: offer.public_code,
                    display_name: offer.display_name,
                    journey,
                })
            })
            .collect()
    }

    fn select_offer(
        &self,
        context: &ActivationRequestContext,
        registration_id: &str,
        public_product_code: &str,
        journey: ActivationJourney,
    ) -> Result<Vec<ActivationTransition>, ActivationError> {
        let request = mutation_request(
            context,
            Some(registration_id.to_string()),
            SelectOfferPayload {
                public_product_code: public_product_code.to_string(),
                journey: journey.label().to_string(),
            },
        );
        let value = self.post(facade_operation_path::SELECT_OFFER, &request)?;
        decode_transitions(&context.request_id, value)
    }

    fn checkout(
        &self,
        context: &ActivationRequestContext,
        registration_id: &str,
        safe_redirect_handle: Option<&str>,
    ) -> Result<CheckoutOutcome, ActivationError> {
        let request = mutation_request(
            context,
            Some(registration_id.to_string()),
            CheckoutPayload {
                safe_redirect_handle: safe_redirect_handle.map(str::to_string),
            },
        );
        let value = self.post(facade_operation_path::CHECKOUT, &request)?;
        let reply: WireCheckoutReply = serde_json::from_value(value).map_err(|_| {
            ActivationError::new(
                ActivationErrorCode::AuthorityUnavailable,
                context.request_id.clone(),
            )
        })?;
        Ok(CheckoutOutcome {
            transitions: reply.transitions,
            safe_url: reply.safe_url,
        })
    }

    fn existing_license(
        &self,
        context: &ActivationRequestContext,
        registration_id: &str,
        human_license_key: &str,
        device_public_key: Option<&str>,
    ) -> Result<Vec<ActivationTransition>, ActivationError> {
        let request = mutation_request(
            context,
            Some(registration_id.to_string()),
            ExistingLicensePayload {
                human_license_key: human_license_key.to_string(),
                device_public_key: device_public_key.map(str::to_string),
            },
        );
        let value = self.post(facade_operation_path::EXISTING_LICENSE, &request)?;
        decode_transitions(&context.request_id, value)
    }

    fn poll(
        &self,
        context: &ActivationRequestContext,
        registration_id: &str,
        poll_credential: &SensitiveCredential,
        device_public_key: Option<&str>,
    ) -> Result<PollOutcome, ActivationError> {
        let request = mutation_request(
            context,
            Some(registration_id.to_string()),
            PollPayload {
                opaque_poll_credential: poll_credential.expose_for_protected_store().to_string(),
                device_public_key: device_public_key.map(str::to_string),
            },
        );
        let value = self.post(facade_operation_path::POLL, &request)?;
        let reply: WirePollReply = serde_json::from_value(value).map_err(|_| {
            ActivationError::new(
                ActivationErrorCode::AuthorityUnavailable,
                context.request_id.clone(),
            )
        })?;
        Ok(PollOutcome {
            transitions: reply.transitions,
            one_time_key_envelope: reply.one_time_key_envelope,
            node_id: reply.node_id,
            lease_envelope: reply.lease_envelope,
        })
    }

    fn refresh(
        &self,
        context: &ActivationRequestContext,
        node_id: &str,
        refresh_credential: &SensitiveCredential,
        current_sequence: u64,
    ) -> Result<Vec<ActivationTransition>, ActivationError> {
        let request = mutation_request(
            context,
            None,
            RefreshPayload {
                node_id: node_id.to_string(),
                refresh_credential: refresh_credential.expose_for_protected_store().to_string(),
                current_sequence,
            },
        );
        let value = self.post(facade_operation_path::REFRESH, &request)?;
        decode_transitions(&context.request_id, value)
    }

    fn nodes(&self, context: &ActivationRequestContext) -> Result<Vec<String>, ActivationError> {
        let value = self.get(facade_operation_path::NODES, &context.request_id, None)?;
        let reply: WireNodesReply = serde_json::from_value(value).map_err(|_| {
            ActivationError::new(
                ActivationErrorCode::AuthorityUnavailable,
                context.request_id.clone(),
            )
        })?;
        Ok(reply.nodes)
    }

    fn deactivate_node(
        &self,
        context: &ActivationRequestContext,
        node_id: &str,
    ) -> Result<Vec<ActivationTransition>, ActivationError> {
        let request = mutation_request(
            context,
            None,
            DeactivatePayload {
                node_id: node_id.to_string(),
            },
        );
        let value = self.post(facade_operation_path::DEACTIVATE_NODE, &request)?;
        decode_transitions(&context.request_id, value)
    }

    fn manage_link(
        &self,
        context: &ActivationRequestContext,
        safe_redirect_handle: Option<&str>,
    ) -> Result<String, ActivationError> {
        let request = mutation_request(
            context,
            None,
            ManageLinkPayload {
                safe_redirect_handle: safe_redirect_handle.map(str::to_string),
            },
        );
        let value = self.post(facade_operation_path::MANAGE_LINK, &request)?;
        let reply: WireLinkReply = serde_json::from_value(value).map_err(|_| {
            ActivationError::new(
                ActivationErrorCode::AuthorityUnavailable,
                context.request_id.clone(),
            )
        })?;
        Ok(reply.link)
    }
}

/// Frozen endpoint paths (call stack `operations[].path`). Kept as named
/// constants so static contract scans can bind the transport to the frozen
/// call stack without re-deriving paths.
pub mod facade_operation_path {
    pub const START: &str = "/v1/activation/start";
    pub const VERIFY: &str = "/v1/activation/verify";
    pub const OFFERS: &str = "/v1/activation/offers";
    pub const SELECT_OFFER: &str = "/v1/activation/select-offer";
    pub const CHECKOUT: &str = "/v1/activation/checkout";
    pub const EXISTING_LICENSE: &str = "/v1/activation/existing-license";
    pub const POLL: &str = "/v1/activation/poll";
    pub const REFRESH: &str = "/v1/lease/refresh";
    pub const NODES: &str = "/v1/nodes";
    pub const DEACTIVATE_NODE: &str = "/v1/nodes/deactivate";
    pub const MANAGE_LINK: &str = "/v1/account/manage-link";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activation_facade::FacadeOperation;

    #[test]
    fn transport_paths_match_the_frozen_call_stack() {
        assert_eq!(
            facade_operation_path::START,
            FacadeOperation::ActivationStart.path()
        );
        assert_eq!(
            facade_operation_path::VERIFY,
            FacadeOperation::ActivationVerify.path()
        );
        assert_eq!(
            facade_operation_path::OFFERS,
            FacadeOperation::ActivationOffers.path()
        );
        assert_eq!(
            facade_operation_path::SELECT_OFFER,
            FacadeOperation::ActivationSelectOffer.path()
        );
        assert_eq!(
            facade_operation_path::CHECKOUT,
            FacadeOperation::ActivationCheckout.path()
        );
        assert_eq!(
            facade_operation_path::EXISTING_LICENSE,
            FacadeOperation::ActivationExistingLicense.path()
        );
        assert_eq!(
            facade_operation_path::POLL,
            FacadeOperation::ActivationPoll.path()
        );
        assert_eq!(
            facade_operation_path::REFRESH,
            FacadeOperation::LeaseRefresh.path()
        );
        assert_eq!(
            facade_operation_path::NODES,
            FacadeOperation::NodesList.path()
        );
        assert_eq!(
            facade_operation_path::DEACTIVATE_NODE,
            FacadeOperation::NodesDeactivate.path()
        );
        assert_eq!(
            facade_operation_path::MANAGE_LINK,
            FacadeOperation::AccountManageLink.path()
        );
    }

    #[test]
    fn all_frozen_error_labels_round_trip_and_unknown_fails_closed() {
        for code in ActivationErrorCode::ALL {
            assert_eq!(code_from_label(code.label()), Some(code));
        }
        assert_eq!(code_from_label("NOT_A_REAL_CODE"), None);
        assert_eq!(code_from_label(""), None);
    }

    #[test]
    fn policy_validation_is_fail_closed() {
        let base = Url::parse("https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/authority/").unwrap();
        let valid = ActivationHttpPolicy {
            base_url: base.clone(),
            timeout: Duration::from_secs(30),
            max_response_bytes: 1024 * 1024,
        };
        assert_eq!(valid.validate(), Ok(()));
        assert_eq!(
            ActivationHttpPolicy {
                base_url: Url::parse("http://insecure.example").unwrap(),
                ..valid.clone()
            }
            .validate(),
            Err(ActivationHttpError::InvalidPolicy(
                "authority base URL must use https"
            ))
        );
        assert_eq!(
            ActivationHttpPolicy {
                timeout: Duration::ZERO,
                ..valid.clone()
            }
            .validate(),
            Err(ActivationHttpError::InvalidPolicy(
                "authority timeout must be within 1..=300 seconds"
            ))
        );
        assert_eq!(
            ActivationHttpPolicy {
                max_response_bytes: 1,
                ..valid
            }
            .validate(),
            Err(ActivationHttpError::InvalidPolicy(
                "authority max response must be within 1 KiB..=4 MiB"
            ))
        );
    }

    #[test]
    fn non_2xx_authority_errors_surface_typed_code_not_unavailable() {
        // #344 regression: a 409 carrying a typed Spec 152E error body must
        // surface that code; AUTHORITY_UNAVAILABLE is reserved for transport
        // and undecodable replies.
        use std::io::{Read, Write};
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 4096];
            let _ = stream.read(&mut buffer);
            stream.write_all(
                b"HTTP/1.1 409 Conflict\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n",
            ).unwrap();
            stream
                .write_all(br#"{"error":{"code":"EDD_ORDER_PENDING","next_action":"poll_after_retry_after"}}"#)
                .unwrap();
        });
        let base_url = Url::parse(&format!("https://127.0.0.1:{port}/authority/")).unwrap();
        // http:// is rejected by validate(); build the client directly.
        let policy = ActivationHttpPolicy {
            base_url,
            timeout: Duration::from_secs(5),
            max_response_bytes: 1024 * 1024,
        };
        let validated = policy.clone();
        let _ = validated.validate(); // would reject http — bypass via struct construction below
        let mut client = ActivationHttpClient::new(ActivationHttpPolicy {
            base_url: Url::parse("https://wpuiai.com/authority/").unwrap(),
            ..policy.clone()
        })
        .unwrap();
        client.policy.base_url =
            Url::parse(&format!("http://127.0.0.1:{port}/authority/")).unwrap();
        let error = client
            .get(
                facade_operation_path::OFFERS,
                "request-3429",
                Some("registration-0001"),
            )
            .unwrap_err();
        server.join().unwrap();
        assert_ne!(
            error.code,
            ActivationErrorCode::AuthorityUnavailable,
            "typed authority error was swallowed into AUTHORITY_UNAVAILABLE (#344)"
        );
    }

    #[test]
    fn response_decoding_maps_typed_replies_and_errors() {
        let request_id = "request-0001";
        let body = r#"{"transitions":["challenge_delivered"],"poll_credential":"poll-secret"}"#;
        let value = decode_response_body(request_id, body.to_string(), 1024 * 1024).unwrap();
        let reply: WireStartReply = serde_json::from_value(value).unwrap();
        assert_eq!(
            reply.transitions,
            vec![ActivationTransition::ChallengeDelivered]
        );
        assert_eq!(reply.poll_credential.as_deref(), Some("poll-secret"));

        let error = decode_response_body(
            request_id,
            r#"{"error":{"code":"EDD_ORDER_PENDING","next_action":"poll_after_retry_after"}}"#
                .to_string(),
            1024 * 1024,
        )
        .unwrap_err();
        assert_eq!(error.code, ActivationErrorCode::EddOrderPending);

        let unknown = decode_response_body(
            request_id,
            r#"{"error":{"code":"INVENTED_CODE"}}"#.to_string(),
            1024 * 1024,
        )
        .unwrap_err();
        assert_eq!(unknown.code, ActivationErrorCode::AuthorityUnavailable);

        let malformed =
            decode_response_body(request_id, "not-json".to_string(), 1024 * 1024).unwrap_err();
        assert_eq!(malformed.code, ActivationErrorCode::AuthorityUnavailable);

        let oversized =
            decode_response_body(request_id, "{\"transitions\":[]}".to_string(), 4).unwrap_err();
        assert_eq!(oversized.code, ActivationErrorCode::AuthorityUnavailable);
    }

    #[test]
    fn poll_and_checkout_replies_decode_with_optional_fields() {
        let poll: WirePollReply = serde_json::from_str(
            r#"{"transitions":["lease_issued","delivered"],"one_time_key_envelope":"base64:key","node_id":"node-1","lease_envelope":"{\"schema\":\"focusa.lease_delivery_envelope.v1\"}"}"#,
        )
        .unwrap();
        assert_eq!(
            poll.transitions,
            vec![
                ActivationTransition::LeaseIssued,
                ActivationTransition::Delivered
            ]
        );
        assert_eq!(poll.node_id.as_deref(), Some("node-1"));
        let checkout: WireCheckoutReply =
            serde_json::from_str(r#"{"transitions":["checkout_started"]}"#).unwrap();
        assert!(checkout.safe_url.is_none());
    }

    #[test]
    fn lease_delivery_envelope_parses_and_fails_closed() {
        let envelope = LeaseDeliveryEnvelope {
            schema: LeaseDeliveryEnvelope::SCHEMA.into(),
            key_set: SignedEnvelope {
                schema: "focusa.signed_envelope.v1".into(),
                signer_key_id: "key-001".into(),
                payload_b64: "cGF5bG9hZA".into(),
                signature_b64: "c2ln".into(),
            },
            lease: SignedEnvelope {
                schema: "focusa.signed_envelope.v1".into(),
                signer_key_id: "key-001".into(),
                payload_b64: "cGF5bG9hZA".into(),
                signature_b64: "c2ln".into(),
            },
        };
        let raw = serde_json::to_string(&envelope).unwrap();
        let parsed = LeaseDeliveryEnvelope::parse(&raw).unwrap();
        assert_eq!(parsed, envelope);
        assert_eq!(
            LeaseDeliveryEnvelope::parse("{\"schema\":\"wrong\"}"),
            Err(ActivationHttpError::MalformedReply)
        );
        assert_eq!(
            LeaseDeliveryEnvelope::parse("not-json"),
            Err(ActivationHttpError::MalformedReply)
        );
    }
}
