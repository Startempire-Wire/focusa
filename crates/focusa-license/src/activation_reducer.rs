//! Shared activation state reducer (Spec 152E §5 universal registration state
//! machine, Spec 172 overlay, and the frozen
//! `spec152e-activation-internal.v1.json` contract).
//!
//! This module is presenter-neutral: every registration state, transition, and
//! terminal settlement is decided exactly once here, and presenters receive
//! only a redacted rendering projection (`ActivationOutputEnvelope` /
//! `PresenterActivationState`). No presenter, installer, facade, or local
//! runtime may reimplement identity, product, payment, Evaluation, license,
//! node, or lease decisions.
//!
//! Unknown states, illegal transitions, terminal re-entry, and unmasked output
//! fail closed. The reducer accepts no caller-controlled product, price,
//! grant, feature, limit, or entitlement input.

use crate::activation_facade::{ActivationError, ActivationErrorCode, mask_email};
use serde::{Deserialize, Serialize};

/// Canonical registration states from the frozen Spec 152E contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivationState {
    AttemptCreated,
    EmailChallengeSent,
    EmailVerified,
    AccountPromoted,
    OfferSelected,
    CheckoutPending,
    LimitedAccessReview,
    ExistingKeyReview,
    EntitlementIssued,
    TerminalDeliveryReady,
    DeviceRegistered,
    LeaseIssued,
    Delivered,
    Expired,
    Denied,
    Refunded,
    Revoked,
    Superseded,
    RecoveryOnly,
}

impl std::fmt::Display for ActivationState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.label())
    }
}

impl std::fmt::Display for ActivationTransition {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.label())
    }
}

impl std::fmt::Display for PresenterActivationState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.label())
    }
}

impl ActivationState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::AttemptCreated => "attempt_created",
            Self::EmailChallengeSent => "email_challenge_sent",
            Self::EmailVerified => "email_verified",
            Self::AccountPromoted => "account_promoted",
            Self::OfferSelected => "offer_selected",
            Self::CheckoutPending => "checkout_pending",
            Self::LimitedAccessReview => "limited_access_review",
            Self::ExistingKeyReview => "existing_key_review",
            Self::EntitlementIssued => "entitlement_issued",
            Self::TerminalDeliveryReady => "terminal_delivery_ready",
            Self::DeviceRegistered => "device_registered",
            Self::LeaseIssued => "lease_issued",
            Self::Delivered => "delivered",
            Self::Expired => "expired",
            Self::Denied => "denied",
            Self::Refunded => "refunded",
            Self::Revoked => "revoked",
            Self::Superseded => "superseded",
            Self::RecoveryOnly => "recovery_only",
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Delivered
                | Self::Expired
                | Self::Denied
                | Self::Refunded
                | Self::Revoked
                | Self::Superseded
                | Self::RecoveryOnly
        )
    }
}

/// Typed events the authority may settle onto a registration. Each event is a
/// first-class enum member so call sites receive a typed transition rather
/// than a stringly-typed step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivationTransition {
    ChallengeDelivered,
    EmailVerified,
    AccountPromoted,
    OfferSelected,
    CheckoutStarted,
    LimitedAccessChosen,
    ExistingKeyChosen,
    EntitlementIssued,
    TerminalDeliveryReady,
    DeviceRegistered,
    LeaseIssued,
    Delivered,
    Expired,
    Denied,
    Refunded,
    Revoked,
    Superseded,
    RecoveryOnly,
}

impl ActivationTransition {
    pub const fn label(self) -> &'static str {
        match self {
            Self::ChallengeDelivered => "challenge_delivered",
            Self::EmailVerified => "email_verified",
            Self::AccountPromoted => "account_promoted",
            Self::OfferSelected => "offer_selected",
            Self::CheckoutStarted => "checkout_started",
            Self::LimitedAccessChosen => "limited_access_chosen",
            Self::ExistingKeyChosen => "existing_key_chosen",
            Self::EntitlementIssued => "entitlement_issued",
            Self::TerminalDeliveryReady => "terminal_delivery_ready",
            Self::DeviceRegistered => "device_registered",
            Self::LeaseIssued => "lease_issued",
            Self::Delivered => "delivered",
            Self::Expired => "expired",
            Self::Denied => "denied",
            Self::Refunded => "refunded",
            Self::Revoked => "revoked",
            Self::Superseded => "superseded",
            Self::RecoveryOnly => "recovery_only",
        }
    }
}

/// Fail-closed reducer errors. Unknown or illegal transitions are never
/// silently dropped or normalized into a grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActivationTransitionError {
    /// The event is not allowed from the current registration state.
    IllegalTransition,
}

/// Pure, deterministic, presenter-neutral reducer for one registration.
///
/// This is the single source of truth for the frozen Spec 152E state machine:
/// exactly the 48 contract transitions succeed and every other
/// `(state, event)` pair fails closed. Timestamps, polling, credentials, and
/// rendering live outside this function.
pub const fn reduce_activation(
    state: ActivationState,
    transition: ActivationTransition,
) -> Result<ActivationState, ActivationTransitionError> {
    use ActivationState as S;
    use ActivationTransition as T;
    let next = match (state, transition) {
        (S::AttemptCreated, T::ChallengeDelivered) => S::EmailChallengeSent,
        (S::EmailChallengeSent, T::EmailVerified) => S::EmailVerified,
        (S::EmailVerified, T::AccountPromoted) => S::AccountPromoted,
        (S::AccountPromoted, T::OfferSelected) => S::OfferSelected,
        (S::AccountPromoted, T::LimitedAccessChosen) => S::LimitedAccessReview,
        (S::AccountPromoted, T::ExistingKeyChosen) => S::ExistingKeyReview,
        (S::OfferSelected, T::CheckoutStarted) => S::CheckoutPending,
        (S::OfferSelected, T::LimitedAccessChosen) => S::LimitedAccessReview,
        (S::OfferSelected, T::ExistingKeyChosen) => S::ExistingKeyReview,
        (S::CheckoutPending, T::EntitlementIssued) => S::EntitlementIssued,
        (S::LimitedAccessReview, T::DeviceRegistered) => S::DeviceRegistered,
        (S::ExistingKeyReview, T::EntitlementIssued) => S::EntitlementIssued,
        (S::EntitlementIssued, T::TerminalDeliveryReady) => S::TerminalDeliveryReady,
        (S::EntitlementIssued, T::DeviceRegistered) => S::DeviceRegistered,
        (S::TerminalDeliveryReady, T::DeviceRegistered) => S::DeviceRegistered,
        (S::DeviceRegistered, T::LeaseIssued) => S::LeaseIssued,
        (S::LeaseIssued, T::Delivered) => S::Delivered,
        (S::LeaseIssued, T::Superseded) => S::Superseded,
        (S::Delivered, T::Superseded) => S::Superseded,

        (S::AttemptCreated | S::EmailChallengeSent | S::CheckoutPending, T::Expired) => S::Expired,
        (S::AttemptCreated, T::Denied)
        | (S::EmailChallengeSent, T::Denied)
        | (S::EmailVerified, T::Denied)
        | (S::AccountPromoted, T::Denied)
        | (S::OfferSelected, T::Denied)
        | (S::CheckoutPending, T::Denied)
        | (S::LimitedAccessReview, T::Denied)
        | (S::ExistingKeyReview, T::Denied)
        | (S::DeviceRegistered, T::Denied) => S::Denied,
        (S::EntitlementIssued | S::TerminalDeliveryReady | S::DeviceRegistered | S::LeaseIssued | S::Delivered, T::Refunded) => S::Refunded,
        (S::EntitlementIssued | S::TerminalDeliveryReady | S::DeviceRegistered | S::LeaseIssued | S::Delivered, T::Revoked) => S::Revoked,
        (S::LeaseIssued | S::Delivered, T::RecoveryOnly) => S::RecoveryOnly,
        (S::Expired | S::Denied | S::Refunded | S::Revoked | S::Superseded, T::RecoveryOnly) => {
            S::RecoveryOnly
        }
        _ => return Err(ActivationTransitionError::IllegalTransition),
    };
    Ok(next)
}

/// Presenter rendering states from the frozen contract. Presenters may render
/// these states and their links; they may not reimplement transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenterActivationState {
    EmailRequired,
    EmailVerificationPending,
    EmailVerified,
    SelectionRequired,
    CheckoutRequired,
    PaymentPending,
    LicenseDeliveryReady,
    Activated,
    Denied,
    RecoveryOnly,
}

impl PresenterActivationState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::EmailRequired => "email_required",
            Self::EmailVerificationPending => "email_verification_pending",
            Self::EmailVerified => "email_verified",
            Self::SelectionRequired => "selection_required",
            Self::CheckoutRequired => "checkout_required",
            Self::PaymentPending => "payment_pending",
            Self::LicenseDeliveryReady => "license_delivery_ready",
            Self::Activated => "activated",
            Self::Denied => "denied",
            Self::RecoveryOnly => "recovery_only",
        }
    }
}

/// Pure rendering projection: registration state → presenter state. Rendering
/// only; the reducer above remains the only decision authority.
pub const fn presenter_state(state: ActivationState) -> PresenterActivationState {
    use ActivationState as S;
    use PresenterActivationState as P;
    match state {
        S::AttemptCreated => P::EmailRequired,
        S::EmailChallengeSent => P::EmailVerificationPending,
        S::EmailVerified => P::EmailVerified,
        S::AccountPromoted | S::LimitedAccessReview | S::ExistingKeyReview => {
            P::SelectionRequired
        }
        S::OfferSelected => P::CheckoutRequired,
        S::CheckoutPending => P::PaymentPending,
        S::EntitlementIssued | S::TerminalDeliveryReady | S::DeviceRegistered => {
            P::LicenseDeliveryReady
        }
        S::LeaseIssued | S::Delivered => P::Activated,
        S::Expired | S::Denied => P::Denied,
        S::Refunded | S::Revoked | S::Superseded | S::RecoveryOnly => P::RecoveryOnly,
    }
}

/// Retry postures from the frozen `Retry` schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryPosture {
    None,
    SafeRetry,
    RetrySameIdempotencyKey,
    Restart,
    RecoveryOnly,
}

impl RetryPosture {
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::SafeRetry => "safe_retry",
            Self::RetrySameIdempotencyKey => "retry_same_idempotency_key",
            Self::Restart => "restart",
            Self::RecoveryOnly => "recovery_only",
        }
    }
}

/// Bounded poll/retry policy. `retry_after_seconds` is clamped to the frozen
/// 1..=30 window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PollRetryPolicy {
    pub posture: RetryPosture,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<u32>,
}

impl PollRetryPolicy {
    pub const DEFAULT_RETRY_AFTER_SECONDS: u32 = 3;
    pub const MAXIMUM_RETRY_AFTER_SECONDS: u32 = 30;

    pub fn new(posture: RetryPosture, retry_after_seconds: Option<u32>) -> Self {
        Self {
            posture,
            retry_after_seconds: retry_after_seconds
                .map(|value| value.clamp(1, Self::MAXIMUM_RETRY_AFTER_SECONDS)),
        }
    }

    pub fn none() -> Self {
        Self::new(RetryPosture::None, None)
    }

    pub fn safe_retry() -> Self {
        Self::new(
            RetryPosture::SafeRetry,
            Some(Self::DEFAULT_RETRY_AFTER_SECONDS),
        )
    }
}

/// Redacted error section of the canonical output envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationErrorEnvelope {
    pub code: String,
    pub next_action: String,
}

/// Canonical, presenter-safe output envelope. The struct has no field for any
/// forbidden value (`email`, `normalized_email`, `raw_email`,
/// `full_license_key`, poll/verification hashes, server credentials, signing
/// keys, card data, or internal EDD records), so those values cannot be
/// serialized by construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationOutputEnvelope {
    pub schema: String,
    pub request_id: String,
    pub registration_id: String,
    pub state: String,
    pub terminal: bool,
    pub retry: PollRetryPolicy,
    pub next_action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub masked_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_delivery_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_time_key_envelope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lease_envelope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ActivationErrorEnvelope>,
}

/// Fail-closed envelope construction errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActivationEnvelopeError {
    MissingIdentity(&'static str),
    /// A masked email must match the frozen `^[^@]*\*[^@]*@[^@]+$` pattern;
    /// raw or unmaskable emails are rejected.
    UnmaskedEmail,
}

const ENVELOPE_SCHEMA: &str = "focusa.activation.response.v1";

impl ActivationOutputEnvelope {
    /// Build a redacted envelope. `masked_email` must already be masked (the
    /// caller obtains it via `mask_email`); raw emails fail closed.
    pub fn build(
        request_id: &str,
        registration_id: &str,
        state: ActivationState,
        masked_email: Option<&str>,
        safe_url: Option<String>,
        verification_delivery_status: Option<String>,
        one_time_key_envelope: Option<String>,
        node_id: Option<String>,
        lease_envelope: Option<String>,
        error: Option<ActivationError>,
        retry: PollRetryPolicy,
    ) -> Result<Self, ActivationEnvelopeError> {
        if request_id.trim().is_empty() {
            return Err(ActivationEnvelopeError::MissingIdentity("request_id"));
        }
        if registration_id.trim().is_empty() {
            return Err(ActivationEnvelopeError::MissingIdentity("registration_id"));
        }
        let masked_email = masked_email.map(str::to_string).map(|value| {
            if looks_masked(&value) {
                Ok(value)
            } else {
                Err(ActivationEnvelopeError::UnmaskedEmail)
            }
        });
        let masked_email = match masked_email {
            Some(Ok(value)) => Some(value),
            Some(Err(error)) => return Err(error),
            None => None,
        };
        let presenter = presenter_state(state);
        let next_action = error
            .as_ref()
            .map(|value| value.code.safe_next_action().to_string())
            .unwrap_or_else(|| presenter_next_action(presenter).to_string());
        Ok(Self {
            schema: ENVELOPE_SCHEMA.to_string(),
            request_id: request_id.to_string(),
            registration_id: registration_id.to_string(),
            state: presenter.label().to_string(),
            terminal: state.is_terminal(),
            retry,
            next_action,
            masked_email,
            safe_url,
            verification_delivery_status,
            one_time_key_envelope,
            node_id,
            lease_envelope,
            error: error.map(|value| ActivationErrorEnvelope {
                code: value.code.label().to_string(),
                next_action: value.code.safe_next_action().to_string(),
            }),
        })
    }
}

/// Safe next action for a presenter state when no error is present. Rendering
/// guidance only; it never re-decides a transition.
pub const fn presenter_next_action(presenter: PresenterActivationState) -> &'static str {
    match presenter {
        PresenterActivationState::EmailRequired => "provide_email",
        PresenterActivationState::EmailVerificationPending => "verify_email",
        PresenterActivationState::EmailVerified => "select_offer",
        PresenterActivationState::SelectionRequired => "select_offer",
        PresenterActivationState::CheckoutRequired => "open_checkout",
        PresenterActivationState::PaymentPending => "poll_after_retry_after",
        PresenterActivationState::LicenseDeliveryReady => "deliver_license",
        PresenterActivationState::Activated => "activated",
        PresenterActivationState::Denied => "activate_or_manage_entitlement",
        PresenterActivationState::RecoveryOnly => "recovery",
    }
}

/// Fail-closed masked-email check: `^[^@]*\*[^@]*@[^@]+$`.
fn looks_masked(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.is_empty() || domain.contains('@') || domain.contains('*') {
        return false;
    }
    let (head, tail) = local.split_once('*').unwrap_or(("", ""));
    !head.is_empty() && (!tail.is_empty() || local.ends_with('*'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activation_facade::ActivationErrorCode;

    fn all_states() -> [ActivationState; 19] {
        [
            ActivationState::AttemptCreated,
            ActivationState::EmailChallengeSent,
            ActivationState::EmailVerified,
            ActivationState::AccountPromoted,
            ActivationState::OfferSelected,
            ActivationState::CheckoutPending,
            ActivationState::LimitedAccessReview,
            ActivationState::ExistingKeyReview,
            ActivationState::EntitlementIssued,
            ActivationState::TerminalDeliveryReady,
            ActivationState::DeviceRegistered,
            ActivationState::LeaseIssued,
            ActivationState::Delivered,
            ActivationState::Expired,
            ActivationState::Denied,
            ActivationState::Refunded,
            ActivationState::Revoked,
            ActivationState::Superseded,
            ActivationState::RecoveryOnly,
        ]
    }

    #[test]
    fn reducer_exactly_matches_frozen_contract_transitions() {
        let legal: &[(&str, &str, &str)] = &[
            ("attempt_created", "challenge_delivered", "email_challenge_sent"),
            ("email_challenge_sent", "email_verified", "email_verified"),
            ("email_verified", "account_promoted", "account_promoted"),
            ("account_promoted", "offer_selected", "offer_selected"),
            ("account_promoted", "limited_access_chosen", "limited_access_review"),
            ("account_promoted", "existing_key_chosen", "existing_key_review"),
            ("offer_selected", "checkout_started", "checkout_pending"),
            ("offer_selected", "limited_access_chosen", "limited_access_review"),
            ("offer_selected", "existing_key_chosen", "existing_key_review"),
            ("checkout_pending", "entitlement_issued", "entitlement_issued"),
            ("limited_access_review", "device_registered", "device_registered"),
            ("existing_key_review", "entitlement_issued", "entitlement_issued"),
            ("entitlement_issued", "terminal_delivery_ready", "terminal_delivery_ready"),
            ("entitlement_issued", "device_registered", "device_registered"),
            ("terminal_delivery_ready", "device_registered", "device_registered"),
            ("device_registered", "lease_issued", "lease_issued"),
            ("lease_issued", "delivered", "delivered"),
            ("lease_issued", "superseded", "superseded"),
            ("delivered", "superseded", "superseded"),
            ("attempt_created", "expired", "expired"),
            ("email_challenge_sent", "expired", "expired"),
            ("checkout_pending", "expired", "expired"),
            ("attempt_created", "denied", "denied"),
            ("email_challenge_sent", "denied", "denied"),
            ("email_verified", "denied", "denied"),
            ("account_promoted", "denied", "denied"),
            ("offer_selected", "denied", "denied"),
            ("checkout_pending", "denied", "denied"),
            ("limited_access_review", "denied", "denied"),
            ("existing_key_review", "denied", "denied"),
            ("device_registered", "denied", "denied"),
            ("entitlement_issued", "refunded", "refunded"),
            ("terminal_delivery_ready", "refunded", "refunded"),
            ("device_registered", "refunded", "refunded"),
            ("lease_issued", "refunded", "refunded"),
            ("delivered", "refunded", "refunded"),
            ("entitlement_issued", "revoked", "revoked"),
            ("terminal_delivery_ready", "revoked", "revoked"),
            ("device_registered", "revoked", "revoked"),
            ("lease_issued", "revoked", "revoked"),
            ("delivered", "revoked", "revoked"),
            ("lease_issued", "recovery_only", "recovery_only"),
            ("delivered", "recovery_only", "recovery_only"),
            ("expired", "recovery_only", "recovery_only"),
            ("denied", "recovery_only", "recovery_only"),
            ("refunded", "recovery_only", "recovery_only"),
            ("revoked", "recovery_only", "recovery_only"),
            ("superseded", "recovery_only", "recovery_only"),
        ];
        assert_eq!(legal.len(), 48);
        let state_of = |label: &str| -> ActivationState {
            all_states()
                .into_iter()
                .find(|state| state.label() == label)
                .unwrap_or_else(|| panic!("unknown state {label}"))
        };
        let transition_of = |label: &str| -> ActivationTransition {
            [
                "challenge_delivered",
                "email_verified",
                "account_promoted",
                "offer_selected",
                "checkout_started",
                "limited_access_chosen",
                "existing_key_chosen",
                "entitlement_issued",
                "terminal_delivery_ready",
                "device_registered",
                "lease_issued",
                "delivered",
                "expired",
                "denied",
                "refunded",
                "revoked",
                "superseded",
                "recovery_only",
            ]
            .into_iter()
            .map(|event| {
                (
                    event,
                    match event {
                        "challenge_delivered" => ActivationTransition::ChallengeDelivered,
                        "email_verified" => ActivationTransition::EmailVerified,
                        "account_promoted" => ActivationTransition::AccountPromoted,
                        "offer_selected" => ActivationTransition::OfferSelected,
                        "checkout_started" => ActivationTransition::CheckoutStarted,
                        "limited_access_chosen" => ActivationTransition::LimitedAccessChosen,
                        "existing_key_chosen" => ActivationTransition::ExistingKeyChosen,
                        "entitlement_issued" => ActivationTransition::EntitlementIssued,
                        "terminal_delivery_ready" => ActivationTransition::TerminalDeliveryReady,
                        "device_registered" => ActivationTransition::DeviceRegistered,
                        "lease_issued" => ActivationTransition::LeaseIssued,
                        "delivered" => ActivationTransition::Delivered,
                        "expired" => ActivationTransition::Expired,
                        "denied" => ActivationTransition::Denied,
                        "refunded" => ActivationTransition::Refunded,
                        "revoked" => ActivationTransition::Revoked,
                        "superseded" => ActivationTransition::Superseded,
                        "recovery_only" => ActivationTransition::RecoveryOnly,
                        _ => unreachable!(),
                    },
                )
            })
            .collect::<std::collections::HashMap<_, _>>()
            .remove(label)
            .unwrap_or_else(|| panic!("unknown transition {label}"))
        };

        let transitions = [
            ActivationTransition::ChallengeDelivered,
            ActivationTransition::EmailVerified,
            ActivationTransition::AccountPromoted,
            ActivationTransition::OfferSelected,
            ActivationTransition::CheckoutStarted,
            ActivationTransition::LimitedAccessChosen,
            ActivationTransition::ExistingKeyChosen,
            ActivationTransition::EntitlementIssued,
            ActivationTransition::TerminalDeliveryReady,
            ActivationTransition::DeviceRegistered,
            ActivationTransition::LeaseIssued,
            ActivationTransition::Delivered,
            ActivationTransition::Expired,
            ActivationTransition::Denied,
            ActivationTransition::Refunded,
            ActivationTransition::Revoked,
            ActivationTransition::Superseded,
            ActivationTransition::RecoveryOnly,
        ];
        // Exactly the 48 frozen pairs succeed; every other pair fails closed.
        let mut accepted = 0;
        for state in all_states() {
            for transition in transitions {
                match reduce_activation(state, transition) {
                    Ok(next) => {
                        accepted += 1;
                        let (expected_from, expected_event, expected_to) = legal
                            .iter()
                            .find(|(from, event, _)| {
                                *from == state.label() && *event == transition.label()
                            })
                            .unwrap_or_else(|| {
                                panic!(
                                    "unexpected legal pair {} --{}--> {}",
                                    state.label(),
                                    transition.label(),
                                    next.label()
                                )
                            });
                        assert_eq!(state, state_of(expected_from));
                        assert_eq!(next, state_of(expected_to));
                        assert_eq!(transition, transition_of(expected_event));
                    }
                    Err(ActivationTransitionError::IllegalTransition) => {
                        assert!(
                            !legal
                                .iter()
                                .any(|(from, event, _)| *from == state.label()
                                    && *event == transition.label()),
                            "{} --{}--> must be accepted",
                            state.label(),
                            transition.label()
                        );
                    }

                }
            }
        }
        assert_eq!(accepted, 48);
    }

    #[test]
    fn recovery_only_never_accepts_any_transition() {
        for transition in [
            ActivationTransition::Delivered,
            ActivationTransition::EntitlementIssued,
            ActivationTransition::AccountPromoted,
            ActivationTransition::DeviceRegistered,
            ActivationTransition::LeaseIssued,
            ActivationTransition::RecoveryOnly,
        ] {
            assert_eq!(
                reduce_activation(ActivationState::RecoveryOnly, transition),
                Err(ActivationTransitionError::IllegalTransition)
            );
        }
    }

    #[test]
    fn no_unverified_promotion_or_local_issuance() {
        for state in [ActivationState::AttemptCreated, ActivationState::EmailChallengeSent] {
            assert_eq!(
                reduce_activation(state, ActivationTransition::AccountPromoted),
                Err(ActivationTransitionError::IllegalTransition)
            );
            assert_eq!(
                reduce_activation(state, ActivationTransition::EntitlementIssued),
                Err(ActivationTransitionError::IllegalTransition)
            );
            assert_eq!(
                reduce_activation(state, ActivationTransition::LeaseIssued),
                Err(ActivationTransitionError::IllegalTransition)
            );
        }
        assert_eq!(
            reduce_activation(
                ActivationState::AttemptCreated,
                ActivationTransition::DeviceRegistered
            ),
            Err(ActivationTransitionError::IllegalTransition)
        );
    }

    #[test]
    fn paid_registrations_are_never_downgraded_to_limited_access() {
        // Once a paid journey has started or entitlement exists, switching to
        // the limited-access journey fails closed. Journey choice is still
        // open only at offer_selected (frozen machine), never afterwards.
        for state in [
            ActivationState::CheckoutPending,
            ActivationState::EntitlementIssued,
            ActivationState::TerminalDeliveryReady,
            ActivationState::DeviceRegistered,
            ActivationState::LeaseIssued,
            ActivationState::Delivered,
        ] {
            assert_eq!(
                reduce_activation(state, ActivationTransition::LimitedAccessChosen),
                Err(ActivationTransitionError::IllegalTransition)
            );
        }
        // offer_selected keeps the frozen journey-choice transition.
        assert_eq!(
            reduce_activation(
                ActivationState::OfferSelected,
                ActivationTransition::LimitedAccessChosen
            ),
            Ok(ActivationState::LimitedAccessReview)
        );
    }

    #[test]
    fn presenter_mapping_covers_all_states_and_all_presenter_states_are_reachable() {
        let expected: &[(&str, &str)] = &[
            ("attempt_created", "email_required"),
            ("email_challenge_sent", "email_verification_pending"),
            ("email_verified", "email_verified"),
            ("account_promoted", "selection_required"),
            ("offer_selected", "checkout_required"),
            ("checkout_pending", "payment_pending"),
            ("limited_access_review", "selection_required"),
            ("existing_key_review", "selection_required"),
            ("entitlement_issued", "license_delivery_ready"),
            ("terminal_delivery_ready", "license_delivery_ready"),
            ("device_registered", "license_delivery_ready"),
            ("lease_issued", "activated"),
            ("delivered", "activated"),
            ("expired", "denied"),
            ("denied", "denied"),
            ("refunded", "recovery_only"),
            ("revoked", "recovery_only"),
            ("superseded", "recovery_only"),
            ("recovery_only", "recovery_only"),
        ];
        assert_eq!(expected.len(), 19);
        for (state, presenter) in expected {
            let state = all_states()
                .into_iter()
                .find(|candidate| candidate.label() == *state)
                .unwrap();
            assert_eq!(presenter_state(state).label(), *presenter);
        }
        let reachable = all_states()
            .into_iter()
            .map(|state| presenter_state(state))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(reachable.len(), 10);
    }

    #[test]
    fn envelope_never_carries_raw_email_and_error_codes_are_typed() {
        let masked = mask_email("customer@example.com").expect("maskable");
        assert_eq!(masked, "c***@example.com");
        let envelope = ActivationOutputEnvelope::build(
            "request-0001",
            "registration-0001",
            ActivationState::EmailChallengeSent,
            Some(&masked),
            None,
            None,
            None,
            None,
            None,
            None,
            PollRetryPolicy::safe_retry(),
        )
        .expect("envelope");
        let body = serde_json::to_string(&envelope).unwrap();
        assert!(!body.contains("customer@example.com"));
        assert!(body.contains("c***@example.com"));
        assert!(body.contains("\"state\":\"email_verification_pending\""));
        assert!(!envelope.terminal);

        // Raw email fails closed at the envelope boundary.
        assert_eq!(
            ActivationOutputEnvelope::build(
                "request-0001",
                "registration-0001",
                ActivationState::EmailChallengeSent,
                Some("customer@example.com"),
                None,
                None,
                None,
                None,
                None,
                None,
                PollRetryPolicy::none(),
            ),
            Err(ActivationEnvelopeError::UnmaskedEmail)
        );

        // Error envelopes expose only the typed code and safe next action.
        let error = ActivationError::new(
            ActivationErrorCode::Refunded,
            "registration-0001".to_string(),
        );
        let envelope = ActivationOutputEnvelope::build(
            "request-0001",
            "registration-0001",
            ActivationState::RecoveryOnly,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(error),
            PollRetryPolicy::new(RetryPosture::RecoveryOnly, None),
        )
        .expect("envelope");
        assert!(envelope.terminal);
        assert_eq!(envelope.state, "recovery_only");
        assert_eq!(envelope.next_action, "recovery_only");
        assert_eq!(envelope.error.as_ref().unwrap().code, "REFUNDED");
        let body = serde_json::to_string(&envelope).unwrap();
        assert!(!body.contains("full_license_key"));
        assert!(!body.contains("poll_credential"));
        assert!(!body.contains("verification_hash"));
    }

    #[test]
    fn retry_after_is_bounded_to_the_frozen_window() {
        assert_eq!(
            PollRetryPolicy::new(RetryPosture::SafeRetry, Some(9_999)),
            PollRetryPolicy {
                posture: RetryPosture::SafeRetry,
                retry_after_seconds: Some(30),
            }
        );
        assert_eq!(
            PollRetryPolicy::new(RetryPosture::SafeRetry, Some(0)),
            PollRetryPolicy {
                posture: RetryPosture::SafeRetry,
                retry_after_seconds: Some(1),
            }
        );
    }
}
