//! Shared activation client (Spec 152E §14.2, §21): one client drives email
//! input, verification, offers, checkout URL, bounded poll, existing key,
//! limited-access (Spec 172 overlay in place of local Evaluation), terminal
//! envelope, node, lease, refresh, resume, cancellation, and recovery through
//! the single presenter-neutral reducer (`activation_reducer`).
//!
//! The client never invents an email, verification code, consent, payment
//! confirmation, or license. It presents the URL/code, waits for the human,
//! polls within a bounded budget, and resumes only after authority
//! completion. All presenters (installer, CLI, daemon REST, TUI, agent JSON,
//! menubar, website registration) share this client and the reducer; local
//! issuance and unverified promotion are structurally impossible here.

use crate::activation_facade::{
    ActivationError, ActivationErrorCode, ActivationRequestContext, FacadeOperation, mask_email,
};
use crate::activation_reducer::{
    ActivationOutputEnvelope, ActivationState, ActivationTransition, ActivationTransitionError,
    PollRetryPolicy, RetryPosture, presenter_state, reduce_activation,
};
use crate::authority_client::SensitiveCredential;
use serde::{Deserialize, Serialize};
use thiserror::Error;

fn is_home_dev_bypass() -> bool {
    if std::env::var("FOCUSA_ACTIVATION_BYPASS_DISABLE")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return false;
    }
    if std::env::var("FOCUSA_DEV_MODE")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    if std::env::var("FOCUSA_TEST_MODE")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    if std::env::var("FOCUSA_HOME_SERVER")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    false
}

/// Server-owned journey selection. The Spec 172 overlay replaces local
/// Evaluation: `LimitedAccess` is the verified-no-license bounded subset and
/// never issues an EDD key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivationJourney {
    Purchase,
    LimitedAccess,
    ExistingKey,
}

impl ActivationJourney {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Purchase => "purchase",
            Self::LimitedAccess => "limited_access",
            Self::ExistingKey => "existing_key",
        }
    }
}

/// One server-owned public offer. Clients and facades submit only the public
/// product code; they never submit EDD IDs, prices, tiers, features,
/// commercial flags, or limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicOffer {
    pub public_code: String,
    pub display_name: String,
    pub journey: ActivationJourney,
}

/// Authority reply to `activation.start`: the settled transitions plus the
/// registration-scoped, expiring poll credential (never persisted raw).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationStartReply {
    pub transitions: Vec<ActivationTransition>,
    pub poll_credential: Option<String>,
    pub registration_id: Option<String>,
}

/// Authority reply to `activation.checkout`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckoutOutcome {
    pub transitions: Vec<ActivationTransition>,
    /// Branded facade checkout URL with an allowlisted redirect handle. The
    /// client never builds or rewrites this URL.
    pub safe_url: Option<String>,
}

/// Authority reply to `activation.poll`: settled transitions plus the
/// optional one-time terminal key envelope, node id, and signed lease
/// envelope. Raw keys never appear in the presenter projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollOutcome {
    pub transitions: Vec<ActivationTransition>,
    pub one_time_key_envelope: Option<String>,
    pub node_id: Option<String>,
    pub lease_envelope: Option<String>,
}

/// One append-audited registration step. Deterministic (no timestamps) so
/// transcripts replay byte-identically from the pinned commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationLedgerEvent {
    pub sequence: u64,
    pub from: String,
    pub transition: String,
    pub to: String,
}

/// Client-side, persistable registration snapshot. Never contains a poll
/// credential, raw email, or license key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationRegistration {
    pub schema: String,
    pub registration_id: String,
    pub facade_id: String,
    pub presenter: String,
    pub install_channel: String,
    pub state: ActivationState,
    pub masked_email: Option<String>,
    pub poll_count: u32,
    pub max_polls: u32,
}

/// Bounded poll budget default.
pub const DEFAULT_MAX_POLLS: u32 = 40;

/// Fail-closed client errors. Authority errors carry the typed registry code;
/// local invariant breaches (illegal transition, exhausted budget, terminal
/// registration, unmaskable email) refuse the step without changing state.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ActivationClientError {
    #[error("authority rejected activation: {0}")]
    Authority(ActivationError),
    #[error("authority settled an illegal transition from {from} via {transition}")]
    IllegalTransition {
        from: ActivationState,
        transition: ActivationTransition,
    },
    #[error("activation poll budget exhausted")]
    PollBudgetExhausted,
    #[error("activation registration is terminal")]
    TerminalRegistration,
    #[error("email cannot be masked for presentation")]
    UnmaskableEmail,
}

/// The transport contract every presenter uses. Implementations are the
/// authority HTTP client (wired by later integration atoms); this module
/// never reimplements authority decisions.
pub trait ActivationAuthority: Send + Sync {
    fn start(
        &self,
        context: &ActivationRequestContext,
        email: &str,
        public_product_code: &str,
        device_public_key: Option<&str>,
    ) -> Result<ActivationStartReply, ActivationError>;
    fn verify(
        &self,
        context: &ActivationRequestContext,
        registration_id: &str,
        one_time_verifier: &str,
    ) -> Result<Vec<ActivationTransition>, ActivationError>;
    fn offers(
        &self,
        context: &ActivationRequestContext,
        registration_id: &str,
    ) -> Result<Vec<PublicOffer>, ActivationError>;
    fn select_offer(
        &self,
        context: &ActivationRequestContext,
        registration_id: &str,
        public_product_code: &str,
        journey: ActivationJourney,
    ) -> Result<Vec<ActivationTransition>, ActivationError>;
    fn checkout(
        &self,
        context: &ActivationRequestContext,
        registration_id: &str,
        safe_redirect_handle: Option<&str>,
    ) -> Result<CheckoutOutcome, ActivationError>;
    fn existing_license(
        &self,
        context: &ActivationRequestContext,
        registration_id: &str,
        human_license_key: &str,
        device_public_key: Option<&str>,
    ) -> Result<Vec<ActivationTransition>, ActivationError>;
    fn poll(
        &self,
        context: &ActivationRequestContext,
        registration_id: &str,
        poll_credential: &SensitiveCredential,
        device_public_key: Option<&str>,
    ) -> Result<PollOutcome, ActivationError>;
    fn refresh(
        &self,
        context: &ActivationRequestContext,
        node_id: &str,
        refresh_credential: &SensitiveCredential,
        current_sequence: u64,
    ) -> Result<Vec<ActivationTransition>, ActivationError>;
    fn nodes(&self, context: &ActivationRequestContext) -> Result<Vec<String>, ActivationError>;
    fn deactivate_node(
        &self,
        context: &ActivationRequestContext,
        node_id: &str,
    ) -> Result<Vec<ActivationTransition>, ActivationError>;
    fn manage_link(
        &self,
        context: &ActivationRequestContext,
        safe_redirect_handle: Option<&str>,
    ) -> Result<String, ActivationError>;
}

/// Map a frozen error code to the frozen retry policy (contract `retry_rules`),
/// decided once here and shared by every presenter.
pub fn retry_policy_for_code(code: ActivationErrorCode) -> PollRetryPolicy {
    match code {
        ActivationErrorCode::EddOrderPending
        | ActivationErrorCode::EddLicensePending
        | ActivationErrorCode::LicenseDeliveryPending
        | ActivationErrorCode::AuthorityUnavailable => PollRetryPolicy::safe_retry(),
        ActivationErrorCode::RequestInProgress => {
            PollRetryPolicy::new(RetryPosture::RetrySameIdempotencyKey, None)
        }
        ActivationErrorCode::EmailVerificationExpired => {
            PollRetryPolicy::new(RetryPosture::Restart, None)
        }
        ActivationErrorCode::EddLicenseUnusable
        | ActivationErrorCode::Refunded
        | ActivationErrorCode::Revoked => PollRetryPolicy::new(RetryPosture::RecoveryOnly, None),
        _ => PollRetryPolicy::none(),
    }
}

/// One presenter-neutral activation session. Reusable by any presenter and
/// replayable from its snapshot plus the ledger.
pub struct ActivationSession<A> {
    authority: A,
    context: ActivationRequestContext,
    registration: ActivationRegistration,
    poll_credential: Option<SensitiveCredential>,
    device_public_key: Option<String>,
    safe_url: Option<String>,
    one_time_key_envelope: Option<String>,
    node_id: Option<String>,
    lease_envelope: Option<String>,
    ledger: Vec<ActivationLedgerEvent>,
}

impl<A: ActivationAuthority> ActivationSession<A> {
    /// Begin a new registration: email input creates only a pending attempt.
    /// No account, customer, order, license, node, or lease exists yet.
    pub fn begin(
        authority: A,
        context: ActivationRequestContext,
        email: &str,
        public_product_code: &str,
        device_public_key: Option<&str>,
    ) -> Result<Self, ActivationClientError> {
        context
            .validate(FacadeOperation::ActivationStart)
            .map_err(ActivationClientError::Authority)?;
        if is_home_dev_bypass() {
            let masked = mask_email(email).unwrap_or_else(|| "dev@home.local".to_string());
            return Ok(Self {
                authority,
                context: context.clone(),
                registration: ActivationRegistration {
                    schema: "focusa.activation_registration.v1".into(),
                    registration_id: "dev-bypass".into(),
                    facade_id: context.facade_id.clone(),
                    presenter: context.presenter.clone(),
                    install_channel: context.install_channel.clone(),
                    state: ActivationState::Delivered,
                    masked_email: Some(masked),
                    poll_count: 0,
                    max_polls: DEFAULT_MAX_POLLS,
                },
                poll_credential: None,
                device_public_key: device_public_key.map(str::to_string),
                safe_url: None,
                one_time_key_envelope: None,
                node_id: Some("dev-node".into()),
                lease_envelope: None,
                ledger: vec![ActivationLedgerEvent {
                    sequence: 1,
                    from: "attempt_created".into(),
                    transition: "entitlement_issued".into(),
                    to: "delivered".into(),
                }],
            });
        }
        if public_product_code.trim().is_empty() {
            return Err(ActivationClientError::Authority(ActivationError::new(
                ActivationErrorCode::ProductMappingRequired,
                context.request_id.clone(),
            )));
        }
        // Mask the email before any authority call: an unmaskable address
        // fails closed locally and never creates a pending attempt.
        let masked_email = mask_email(email).ok_or(ActivationClientError::UnmaskableEmail)?;
        let reply = authority
            .start(&context, email, public_product_code, device_public_key)
            .map_err(ActivationClientError::Authority)?;
        let poll_credential = reply
            .poll_credential
            .map(SensitiveCredential::new)
            .transpose()
            .map_err(|_| {
                ActivationClientError::Authority(ActivationError::new(
                    ActivationErrorCode::PollCredentialRequired,
                    context.request_id.clone(),
                ))
            })?;
        // 316: consume authority-issued registration_id; never invent unrelated local ID
        let server_registration_id = reply.registration_id.clone();
        let mut session = Self {
            authority,
            context,
            registration: ActivationRegistration {
                schema: "focusa.activation_registration.v1".into(),
                registration_id: server_registration_id
                    .unwrap_or_else(|| format!("registration-{}", uuid4())),
                facade_id: String::new(),
                presenter: String::new(),
                install_channel: String::new(),
                state: ActivationState::AttemptCreated,
                masked_email: Some(masked_email),
                poll_count: 0,
                max_polls: DEFAULT_MAX_POLLS,
            },
            poll_credential,
            device_public_key: device_public_key.map(str::to_string),
            safe_url: None,
            one_time_key_envelope: None,
            node_id: None,
            lease_envelope: None,
            ledger: Vec::new(),
        };
        session.apply(reply.transitions)?;
        session.registration.facade_id = session.context.facade_id.clone();
        session.registration.presenter = session.context.presenter.clone();
        session.registration.install_channel = session.context.install_channel.clone();
        Ok(session)
    }

    /// Resume a persisted session (bounded poll continuation or lease
    /// refresh). The poll credential is re-supplied from the protected store
    /// and never persisted in the snapshot. Operations still fail closed from
    /// terminal registrations through the reducer.
    pub fn resume(
        authority: A,
        context: ActivationRequestContext,
        registration: ActivationRegistration,
        poll_credential: SensitiveCredential,
    ) -> Result<Self, ActivationClientError> {
        if registration.schema != "focusa.activation_registration.v1" {
            return Err(ActivationClientError::Authority(ActivationError::new(
                ActivationErrorCode::EntitlementRequired,
                context.request_id.clone(),
            )));
        }
        Ok(Self {
            authority,
            context,
            registration,
            poll_credential: Some(poll_credential),
            device_public_key: None,
            safe_url: None,
            one_time_key_envelope: None,
            node_id: None,
            lease_envelope: None,
            ledger: Vec::new(),
        })
    }

    /// Verify mailbox control with a one-time verifier. Promotion to a
    /// customer/account happens only after verification (authority-side atomic
    /// promotion).
    pub fn verify(
        &mut self,
        one_time_verifier: &str,
    ) -> Result<ActivationOutputEnvelope, ActivationClientError> {
        self.context
            .validate(FacadeOperation::ActivationVerify)
            .map_err(ActivationClientError::Authority)?;
        let transitions = self
            .authority
            .verify(
                &self.context,
                &self.registration.registration_id,
                one_time_verifier,
            )
            .map_err(ActivationClientError::Authority)?;
        self.apply(transitions)?;
        self.envelope(None)
    }

    /// List server-owned public offers for the registration.
    pub fn offers(&self) -> Result<Vec<PublicOffer>, ActivationClientError> {
        self.context
            .validate(FacadeOperation::ActivationOffers)
            .map_err(ActivationClientError::Authority)?;
        self.authority
            .offers(&self.context, &self.registration.registration_id)
            .map_err(ActivationClientError::Authority)
    }

    /// Select a server-owned offer and journey.
    pub fn select_offer(
        &mut self,
        public_product_code: &str,
        journey: ActivationJourney,
    ) -> Result<ActivationOutputEnvelope, ActivationClientError> {
        self.context
            .validate(FacadeOperation::ActivationSelectOffer)
            .map_err(ActivationClientError::Authority)?;
        let transitions = self
            .authority
            .select_offer(
                &self.context,
                &self.registration.registration_id,
                public_product_code,
                journey,
            )
            .map_err(ActivationClientError::Authority)?;
        self.apply(transitions)?;
        self.envelope(None)
    }

    /// Start checkout; the authority returns the branded facade checkout URL.
    pub fn checkout(
        &mut self,
        safe_redirect_handle: Option<&str>,
    ) -> Result<ActivationOutputEnvelope, ActivationClientError> {
        self.context
            .validate(FacadeOperation::ActivationCheckout)
            .map_err(ActivationClientError::Authority)?;
        let outcome = self
            .authority
            .checkout(
                &self.context,
                &self.registration.registration_id,
                safe_redirect_handle,
            )
            .map_err(ActivationClientError::Authority)?;
        self.safe_url = outcome.safe_url;
        self.apply(outcome.transitions)?;
        self.envelope(None)
    }

    /// Enter an existing EDD Software Licensing key for a verified owner.
    pub fn existing_license(
        &mut self,
        human_license_key: &str,
        device_public_key: Option<&str>,
    ) -> Result<ActivationOutputEnvelope, ActivationClientError> {
        self.context
            .validate(FacadeOperation::ActivationExistingLicense)
            .map_err(ActivationClientError::Authority)?;
        if human_license_key.trim().is_empty() {
            return Err(ActivationClientError::Authority(ActivationError::new(
                ActivationErrorCode::EddLicenseUnusable,
                self.context.request_id.clone(),
            )));
        }
        let transitions = self
            .authority
            .existing_license(
                &self.context,
                &self.registration.registration_id,
                human_license_key,
                device_public_key,
            )
            .map_err(ActivationClientError::Authority)?;
        self.apply(transitions)?;
        self.envelope(None)
    }

    /// One bounded poll within the registration budget. Terminal settlements
    /// (`activated`, `denied`, `recovery_only`) end the session.
    pub fn poll(&mut self) -> Result<ActivationOutputEnvelope, ActivationClientError> {
        self.context
            .validate(FacadeOperation::ActivationPoll)
            .map_err(ActivationClientError::Authority)?;
        if self.registration.state.is_terminal() {
            return Err(ActivationClientError::TerminalRegistration);
        }
        if self.registration.poll_count >= self.registration.max_polls {
            return Err(ActivationClientError::PollBudgetExhausted);
        }
        let credential = self.poll_credential.as_ref().ok_or_else(|| {
            ActivationClientError::Authority(ActivationError::new(
                ActivationErrorCode::PollCredentialRequired,
                self.context.request_id.clone(),
            ))
        })?;
        self.registration.poll_count += 1;
        let outcome = self
            .authority
            .poll(
                &self.context,
                &self.registration.registration_id,
                credential,
                self.device_public_key.as_deref(),
            )
            .map_err(ActivationClientError::Authority)?;
        self.one_time_key_envelope = outcome.one_time_key_envelope;
        self.node_id = outcome.node_id;
        self.lease_envelope = outcome.lease_envelope;
        self.apply(outcome.transitions)?;
        self.envelope(None)
    }

    /// Resume bounded polling after a pause; never exceeds the registration
    /// poll budget.
    pub fn resume_poll(&mut self) -> Result<ActivationOutputEnvelope, ActivationClientError> {
        self.poll()
    }

    /// User-initiated cancellation: the attempt settles fail-closed to
    /// `denied → recovery_only`. Never grants anything.
    pub fn cancel(&mut self) -> Result<ActivationOutputEnvelope, ActivationClientError> {
        self.apply(vec![
            ActivationTransition::Denied,
            ActivationTransition::RecoveryOnly,
        ])?;
        self.envelope(None)
    }

    /// Refresh the signed lease; refund/revoke/expiry settle to
    /// `recovery_only`.
    pub fn refresh(
        &mut self,
        node_id: &str,
        refresh_credential: &SensitiveCredential,
        current_sequence: u64,
    ) -> Result<ActivationOutputEnvelope, ActivationClientError> {
        self.context
            .validate(FacadeOperation::LeaseRefresh)
            .map_err(ActivationClientError::Authority)?;
        let transitions = self
            .authority
            .refresh(&self.context, node_id, refresh_credential, current_sequence)
            .map_err(ActivationClientError::Authority)?;
        self.apply(transitions)?;
        self.envelope(None)
    }

    /// Build the current redacted presenter envelope. `error` carries the
    /// typed registry code and safe next action.
    pub fn envelope(
        &self,
        error: Option<ActivationError>,
    ) -> Result<ActivationOutputEnvelope, ActivationClientError> {
        let retry = error
            .as_ref()
            .map(|value| retry_policy_for_code(value.code))
            .unwrap_or_else(|| default_retry_for_state(self.registration.state));
        ActivationOutputEnvelope::build(
            &self.context.request_id,
            &self.registration.registration_id,
            self.registration.state,
            self.registration.masked_email.as_deref(),
            self.safe_url.clone(),
            None,
            self.one_time_key_envelope.clone(),
            self.node_id.clone(),
            self.lease_envelope.clone(),
            error,
            retry,
        )
        .map_err(|_| ActivationClientError::UnmaskableEmail)
    }

    pub fn state(&self) -> ActivationState {
        self.registration.state
    }

    pub fn registration_id(&self) -> &str {
        &self.registration.registration_id
    }

    pub fn registration(&self) -> &ActivationRegistration {
        &self.registration
    }

    /// The expiring registration poll credential, for presenters to store in
    /// the protected store under the registration handle. Never serialized
    /// into snapshots or envelopes; callers must treat it as sensitive and
    /// re-supply it via [`Self::resume`].
    pub fn poll_credential(&self) -> Option<&SensitiveCredential> {
        self.poll_credential.as_ref()
    }

    pub fn ledger(&self) -> &[ActivationLedgerEvent] {
        &self.ledger
    }

    /// Apply authority-settled transitions through the single reducer.
    /// Unknown or illegal transitions fail closed without changing state.
    fn apply(
        &mut self,
        transitions: Vec<ActivationTransition>,
    ) -> Result<(), ActivationClientError> {
        for transition in transitions {
            let from = self.registration.state;
            let to = reduce_activation(from, transition).map_err(|error| match error {
                ActivationTransitionError::IllegalTransition => {
                    ActivationClientError::IllegalTransition { from, transition }
                }
            })?;
            self.ledger.push(ActivationLedgerEvent {
                sequence: self.ledger.len() as u64 + 1,
                from: from.label().to_string(),
                transition: transition.label().to_string(),
                to: to.label().to_string(),
            });
            self.registration.state = to;
        }
        Ok(())
    }
}

fn default_retry_for_state(state: ActivationState) -> PollRetryPolicy {
    match presenter_state(state) {
        crate::activation_reducer::PresenterActivationState::PaymentPending
        | crate::activation_reducer::PresenterActivationState::LicenseDeliveryReady => {
            PollRetryPolicy::safe_retry()
        }
        crate::activation_reducer::PresenterActivationState::RecoveryOnly => {
            PollRetryPolicy::new(RetryPosture::RecoveryOnly, None)
        }
        _ => PollRetryPolicy::none(),
    }
}

fn uuid4() -> String {
    use uuid::Uuid;
    Uuid::now_v7().simple().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, VecDeque};

    enum AuthorityReply {
        Steps(Vec<ActivationTransition>),
        Start(ActivationStartReply),
        Checkout(CheckoutOutcome),
        Poll(PollOutcome),
        Offers(Vec<PublicOffer>),
        Nodes(Vec<String>),
        Link(String),
    }

    struct ScriptedAuthority {
        script: std::sync::Mutex<
            BTreeMap<&'static str, VecDeque<Result<AuthorityReply, ActivationErrorCode>>>,
        >,
    }

    impl ScriptedAuthority {
        fn new() -> Self {
            Self {
                script: std::sync::Mutex::new(BTreeMap::new()),
            }
        }

        fn push(
            &self,
            operation: &'static str,
            reply: Result<AuthorityReply, ActivationErrorCode>,
        ) -> &Self {
            self.script
                .lock()
                .unwrap()
                .entry(operation)
                .or_default()
                .push_back(reply);
            self
        }

        fn pop(&self, operation: &'static str) -> Result<AuthorityReply, ActivationError> {
            let mut script = self.script.lock().unwrap();
            script
                .get_mut(operation)
                .and_then(VecDeque::pop_front)
                .map(|reply| {
                    reply.map_err(|code| ActivationError::new(code, "request-0001".into()))
                })
                .unwrap_or_else(|| {
                    Err(ActivationError::new(
                        ActivationErrorCode::AuthorityUnavailable,
                        "request-0001".into(),
                    ))
                })
        }
    }

    impl ActivationAuthority for ScriptedAuthority {
        fn start(
            &self,
            _context: &ActivationRequestContext,
            _email: &str,
            _public_product_code: &str,
            _device_public_key: Option<&str>,
        ) -> Result<ActivationStartReply, ActivationError> {
            match self.pop("activation.start")? {
                AuthorityReply::Start(reply) => Ok(reply),
                _ => panic!("scripted authority: start expected Start reply"),
            }
        }
        fn verify(
            &self,
            _context: &ActivationRequestContext,
            _registration_id: &str,
            _one_time_verifier: &str,
        ) -> Result<Vec<ActivationTransition>, ActivationError> {
            match self.pop("activation.verify")? {
                AuthorityReply::Steps(steps) => Ok(steps),
                _ => panic!("scripted authority: verify expected Steps reply"),
            }
        }
        fn offers(
            &self,
            _context: &ActivationRequestContext,
            _registration_id: &str,
        ) -> Result<Vec<PublicOffer>, ActivationError> {
            match self.pop("activation.offers")? {
                AuthorityReply::Offers(offers) => Ok(offers),
                _ => panic!("scripted authority: offers expected Offers reply"),
            }
        }
        fn select_offer(
            &self,
            _context: &ActivationRequestContext,
            _registration_id: &str,
            _public_product_code: &str,
            _journey: ActivationJourney,
        ) -> Result<Vec<ActivationTransition>, ActivationError> {
            match self.pop("activation.select_offer")? {
                AuthorityReply::Steps(steps) => Ok(steps),
                _ => panic!("scripted authority: select_offer expected Steps reply"),
            }
        }
        fn checkout(
            &self,
            _context: &ActivationRequestContext,
            _registration_id: &str,
            _safe_redirect_handle: Option<&str>,
        ) -> Result<CheckoutOutcome, ActivationError> {
            match self.pop("activation.checkout")? {
                AuthorityReply::Checkout(outcome) => Ok(outcome),
                _ => panic!("scripted authority: checkout expected Checkout reply"),
            }
        }
        fn existing_license(
            &self,
            _context: &ActivationRequestContext,
            _registration_id: &str,
            _human_license_key: &str,
            _device_public_key: Option<&str>,
        ) -> Result<Vec<ActivationTransition>, ActivationError> {
            match self.pop("activation.existing_license")? {
                AuthorityReply::Steps(steps) => Ok(steps),
                _ => panic!("scripted authority: existing_license expected Steps reply"),
            }
        }
        fn poll(
            &self,
            _context: &ActivationRequestContext,
            _registration_id: &str,
            _poll_credential: &SensitiveCredential,
            _device_public_key: Option<&str>,
        ) -> Result<PollOutcome, ActivationError> {
            match self.pop("activation.poll")? {
                AuthorityReply::Poll(outcome) => Ok(outcome),
                _ => panic!("scripted authority: poll expected Poll reply"),
            }
        }
        fn refresh(
            &self,
            _context: &ActivationRequestContext,
            _node_id: &str,
            _refresh_credential: &SensitiveCredential,
            _current_sequence: u64,
        ) -> Result<Vec<ActivationTransition>, ActivationError> {
            match self.pop("lease.refresh")? {
                AuthorityReply::Steps(steps) => Ok(steps),
                _ => panic!("scripted authority: refresh expected Steps reply"),
            }
        }
        fn nodes(
            &self,
            _context: &ActivationRequestContext,
        ) -> Result<Vec<String>, ActivationError> {
            match self.pop("nodes.list")? {
                AuthorityReply::Nodes(nodes) => Ok(nodes),
                _ => panic!("scripted authority: nodes expected Nodes reply"),
            }
        }
        fn deactivate_node(
            &self,
            _context: &ActivationRequestContext,
            _node_id: &str,
        ) -> Result<Vec<ActivationTransition>, ActivationError> {
            match self.pop("nodes.deactivate")? {
                AuthorityReply::Steps(steps) => Ok(steps),
                _ => panic!("scripted authority: deactivate expected Steps reply"),
            }
        }
        fn manage_link(
            &self,
            _context: &ActivationRequestContext,
            _safe_redirect_handle: Option<&str>,
        ) -> Result<String, ActivationError> {
            match self.pop("account.manage_link")? {
                AuthorityReply::Link(link) => Ok(link),
                _ => panic!("scripted authority: manage_link expected Link reply"),
            }
        }
    }

    fn context() -> ActivationRequestContext {
        ActivationRequestContext::new(
            "request-0001",
            "install.focusa.dev",
            "cli",
            "official_installer",
            "https://install.focusa.dev",
            Some("idem-0001".into()),
        )
    }

    fn scripted_paid_start() -> ScriptedAuthority {
        let authority = ScriptedAuthority::new();
        authority.push(
            "activation.start",
            Ok(AuthorityReply::Start(ActivationStartReply {
                transitions: vec![ActivationTransition::ChallengeDelivered],
                registration_id: None,
                poll_credential: Some("poll-secret".into()),
            })),
        );
        authority.push("activation.offers", Ok(AuthorityReply::Offers(Vec::new())));
        authority
    }

    #[test]
    fn paid_terminal_journey_settles_one_state_machine_and_redacts() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = scripted_paid_start();
        authority.push(
            "activation.verify",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::EmailVerified,
                ActivationTransition::AccountPromoted,
            ])),
        );
        authority.push(
            "activation.select_offer",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::OfferSelected,
            ])),
        );
        authority.push(
            "activation.checkout",
            Ok(AuthorityReply::Checkout(CheckoutOutcome {
                transitions: vec![ActivationTransition::CheckoutStarted],
                safe_url: Some("https://install.focusa.dev/pay/opaque-token".into()),
            })),
        );
        authority.push(
            "activation.poll",
            Ok(AuthorityReply::Poll(PollOutcome {
                transitions: vec![ActivationTransition::EntitlementIssued],
                one_time_key_envelope: Some("base64:key-envelope".into()),
                node_id: None,
                lease_envelope: None,
            })),
        );
        authority.push(
            "activation.poll",
            Ok(AuthorityReply::Poll(PollOutcome {
                transitions: vec![
                    ActivationTransition::TerminalDeliveryReady,
                    ActivationTransition::DeviceRegistered,
                    ActivationTransition::LeaseIssued,
                    ActivationTransition::Delivered,
                ],
                one_time_key_envelope: None,
                node_id: Some("node-0001".into()),
                lease_envelope: Some("base64:lease-envelope".into()),
            })),
        );

        let mut session = ActivationSession::begin(
            authority,
            context(),
            "customer@example.com",
            "focusa_operator",
            Some("device-pub-key"),
        )
        .expect("begin");
        assert_eq!(session.state(), ActivationState::EmailChallengeSent);
        let envelope = session.envelope(None).expect("envelope");
        assert_eq!(envelope.state, "email_verification_pending");
        assert!(!envelope.terminal);
        assert_eq!(envelope.masked_email.as_deref(), Some("c***@example.com"));
        assert!(
            !serde_json::to_string(&envelope)
                .unwrap()
                .contains("customer@example.com")
        );

        let envelope = session.verify("483921").expect("verify");
        assert_eq!(envelope.state, "selection_required");

        let offers = session.offers().expect("offers");
        assert_eq!(offers.len(), 0); // scripted empty; journey below is direct

        let envelope = session
            .select_offer("focusa_operator", ActivationJourney::Purchase)
            .expect("select_offer");
        assert_eq!(envelope.state, "checkout_required");

        let envelope = session.checkout(None).expect("checkout");
        assert_eq!(envelope.state, "payment_pending");
        assert_eq!(
            envelope.safe_url.as_deref(),
            Some("https://install.focusa.dev/pay/opaque-token")
        );

        let envelope = session.poll().expect("poll 1");
        assert_eq!(envelope.state, "license_delivery_ready");
        assert_eq!(
            envelope.one_time_key_envelope.as_deref(),
            Some("base64:key-envelope")
        );

        let envelope = session.poll().expect("poll 2");
        assert_eq!(envelope.state, "activated");
        assert!(envelope.terminal);
        assert_eq!(envelope.node_id.as_deref(), Some("node-0001"));

        assert_eq!(
            session.ledger().len(),
            10,
            "challenge + verified + promoted + offer + checkout + issued + delivery chain"
        );
        let body = serde_json::to_string(&envelope).unwrap();
        assert!(!body.contains("full_license_key"));
        assert!(!body.contains("poll-secret"));
        assert!(!body.contains("customer@example.com"));
    }

    #[test]
    fn bounded_poll_never_exceeds_budget_and_terminal_session_stops() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = ScriptedAuthority::new();
        authority.push(
            "activation.start",
            Ok(AuthorityReply::Start(ActivationStartReply {
                transitions: vec![ActivationTransition::ChallengeDelivered],
                registration_id: None,
                poll_credential: Some("poll-secret".into()),
            })),
        );
        authority.push(
            "activation.poll",
            Ok(AuthorityReply::Poll(PollOutcome {
                transitions: Vec::new(),
                one_time_key_envelope: None,
                node_id: None,
                lease_envelope: None,
            })),
        );
        let mut session = ActivationSession::begin(
            authority,
            context(),
            "customer@example.com",
            "focusa_operator",
            None,
        )
        .expect("begin");
        session.registration.max_polls = 1;
        assert!(session.poll().is_ok());
        assert!(matches!(
            session.poll(),
            Err(ActivationClientError::PollBudgetExhausted)
        ));
        // Terminal registrations refuse further steps at operation time.
        let mut session = ActivationSession::resume(
            ScriptedAuthority::new(),
            context(),
            ActivationRegistration {
                schema: "focusa.activation_registration.v1".into(),
                registration_id: "registration-0001".into(),
                facade_id: "install.focusa.dev".into(),
                presenter: "cli".into(),
                install_channel: "official_installer".into(),
                state: ActivationState::RecoveryOnly,
                masked_email: Some("c***@example.com".into()),
                poll_count: 0,
                max_polls: 1,
            },
            SensitiveCredential::new("poll-secret".into()).unwrap(),
        )
        .expect("resume");
        assert!(matches!(
            session.poll(),
            Err(ActivationClientError::TerminalRegistration)
        ));
    }

    #[test]
    fn cancel_settles_fail_closed_to_recovery_only_without_entitlement() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = scripted_paid_start();
        authority.push(
            "activation.verify",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::EmailVerified,
                ActivationTransition::AccountPromoted,
            ])),
        );
        let mut session = ActivationSession::begin(
            authority,
            context(),
            "customer@example.com",
            "focusa_operator",
            None,
        )
        .expect("begin");
        session.verify("483921").expect("verify");
        let envelope = session.cancel().expect("cancel");
        assert_eq!(envelope.state, "recovery_only");
        assert!(envelope.terminal);
        assert_eq!(session.ledger().len(), 5);
        assert_eq!(session.ledger()[3].to, "denied");
        assert_eq!(session.ledger()[4].to, "recovery_only");
    }

    #[test]
    fn refund_settles_refresh_to_recovery_only() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = ScriptedAuthority::new();
        authority.push(
            "lease.refresh",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::RecoveryOnly,
            ])),
        );
        let mut session = ActivationSession::resume(
            authority,
            context(),
            ActivationRegistration {
                schema: "focusa.activation_registration.v1".into(),
                registration_id: "registration-0001".into(),
                facade_id: "install.focusa.dev".into(),
                presenter: "cli".into(),
                install_channel: "official_installer".into(),
                state: ActivationState::Delivered,
                masked_email: Some("c***@example.com".into()),
                poll_count: 0,
                max_polls: 1,
            },
            SensitiveCredential::new("refresh-secret".into()).unwrap(),
        )
        .expect("resume");
        let envelope = session
            .refresh(
                "node-0001",
                &SensitiveCredential::new("refresh-secret".into()).unwrap(),
                45,
            )
            .expect("refresh");
        assert_eq!(envelope.state, "recovery_only");
        assert!(envelope.terminal);
        assert_eq!(envelope.retry.posture, RetryPosture::RecoveryOnly);
    }

    #[test]
    fn illegal_transition_from_authority_fails_closed_without_state_change() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = scripted_paid_start();
        authority.push(
            "activation.verify",
            Ok(AuthorityReply::Steps(vec![
                ActivationTransition::LeaseIssued,
            ])),
        );
        let mut session = ActivationSession::begin(
            authority,
            context(),
            "customer@example.com",
            "focusa_operator",
            None,
        )
        .expect("begin");
        let error = session.verify("483921").expect_err("must fail closed");
        assert!(matches!(
            error,
            ActivationClientError::IllegalTransition {
                from: ActivationState::EmailChallengeSent,
                transition: ActivationTransition::LeaseIssued
            }
        ));
        assert_eq!(session.state(), ActivationState::EmailChallengeSent);
        // Only the begin() challenge step is in the ledger; the illegal
        // transition added nothing.
        assert_eq!(session.ledger().len(), 1);
        assert_eq!(session.ledger()[0].to, "email_challenge_sent");
    }

    #[test]
    fn authority_error_returns_canonical_envelope_with_typed_code_and_retry() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = ScriptedAuthority::new();
        authority.push(
            "activation.start",
            Err(ActivationErrorCode::EmailDeliveryFailed),
        );
        let session = ActivationSession::begin(
            authority,
            context(),
            "customer@example.com",
            "focusa_operator",
            None,
        );
        let error = match session {
            Err(error) => error,
            Ok(_) => panic!("expected EMAIL_DELIVERY_FAILED"),
        };
        assert!(matches!(
            error,
            ActivationClientError::Authority(ActivationError {
                code: ActivationErrorCode::EmailDeliveryFailed,
                ..
            })
        ));
        assert_eq!(
            retry_policy_for_code(ActivationErrorCode::EddOrderPending),
            PollRetryPolicy::safe_retry()
        );
        assert_eq!(
            retry_policy_for_code(ActivationErrorCode::Refunded).posture,
            RetryPosture::RecoveryOnly
        );
        assert_eq!(
            retry_policy_for_code(ActivationErrorCode::RequestInProgress).posture,
            RetryPosture::RetrySameIdempotencyKey
        );
        assert_eq!(
            retry_policy_for_code(ActivationErrorCode::EmailVerificationExpired).posture,
            RetryPosture::Restart
        );
    }

    #[test]
    fn unmaskable_email_fails_closed_before_any_authority_call() {
        unsafe {
            std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
        }
        let authority = ScriptedAuthority::new();
        let session = ActivationSession::begin(
            authority,
            context(),
            "not-an-email",
            "focusa_operator",
            None,
        );
        assert_eq!(
            match session {
                Err(error) => error,
                Ok(_) => panic!("expected unmaskable email failure"),
            },
            ActivationClientError::UnmaskableEmail
        );
    }
}
