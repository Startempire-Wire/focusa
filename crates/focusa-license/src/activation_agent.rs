//! Agent-safe JSON activation and resume protocol (Spec 152E §14.2, §19, §20):
//! one presenter-neutral envelope (`focusa.agent_activation_envelope.v1`)
//! returning typed human-action states, masked email/key by default, safe
//! checkout/verification links, bounded poll/resume, an explicit
//! customer-controlled key-reveal gate, and a resumable registration handle.
//!
//! Agents, daemon/API surfaces, and Pi tool envelopes consume this
//! projection. They never invent an email, verification code, consent,
//! payment confirmation, or license, and they never advance a human-required
//! state themselves: when `human_action_required` is true the agent hands the
//! envelope (plus the resumable `registration_id`) to the human and resumes
//! only after authority completion.
//!
//! The envelope has no field for raw email, the one-time key envelope, the
//! signed lease envelope, poll credentials, or any secret — those values
//! cannot be serialized by construction.

use crate::activation_client::{
    ActivationClientError, ActivationRegistration, ActivationSession, DEFAULT_MAX_POLLS,
};
use crate::activation_facade::{ActivationError, mask_email};
use crate::activation_reducer::{ActivationErrorEnvelope, ActivationOutputEnvelope};
use serde::{Deserialize, Serialize};

/// Frozen schema of the agent activation envelope.
pub const AGENT_ENVELOPE_SCHEMA: &str = "focusa.agent_activation_envelope.v1";

/// Explicit customer-controlled key reveal (Spec 152E §14.2): full key output
/// is masked by default; reveal requires both the opt-in (`reveal_key`) and
/// an explicit confirmation (`reveal_confirmation`). Without both, the
/// envelope reports `key_visible: false` and never carries key material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentKeyReveal {
    pub reveal_key: bool,
    pub reveal_confirmation: bool,
}

impl AgentKeyReveal {
    /// Default posture: masked. No key material is ever exposed.
    pub const fn denied() -> Self {
        Self {
            reveal_key: false,
            reveal_confirmation: false,
        }
    }

    /// True only when the customer explicitly opted in AND confirmed. Any
    /// other combination fails closed (masked).
    pub const fn authorized(self) -> bool {
        self.reveal_key && self.reveal_confirmation
    }
}

/// Typed human action for a presenter state. Terminal states and states an
/// agent may complete on its own have none; every other state requires the
/// human to act (verify, choose, pay, accept delivery) before the agent may
/// resume bounded polling.
pub fn human_action_for_state(state: &str) -> Option<&'static str> {
    match state {
        "email_required" => Some("provide_email"),
        "email_verification_pending" => Some("enter_verification_code"),
        "email_verified" => Some("select_offer"),
        "selection_required" => Some("select_offer"),
        "checkout_required" => Some("open_checkout_url"),
        "payment_pending" => Some("complete_payment_then_poll"),
        "license_delivery_ready" => Some("reveal_or_accept_license"),
        // Terminal states (activated/denied/recovery_only) require nothing.
        _ => None,
    }
}

/// True when the presenter state requires a human action the agent must not
/// perform or invent. Fails closed: unknown states count as human-required;
/// only the frozen terminal states require nothing.
pub fn human_action_required(state: &str) -> bool {
    human_action_for_state(state).is_some()
        || !matches!(state, "activated" | "denied" | "recovery_only")
}

/// Mask a full license key to its prefix group followed by `-XXXX` groups
/// (Spec 152E §14.1 shape). Deterministic; never returns the full key.
pub fn mask_key_prefix(full_key: &str) -> String {
    let trimmed = full_key.trim();
    if trimmed.is_empty() {
        return "XXXX-XXXX-XXXX-XXXX".to_string();
    }
    let mut parts = trimmed.split('-');
    let head = parts.next().unwrap_or_default().to_string();
    let groups = parts.count().max(1);
    let mut masked = head;
    for _ in 0..groups {
        masked.push_str("-XXXX");
    }
    if masked.is_empty() {
        masked.push_str("XXXX-XXXX-XXXX-XXXX");
    }
    masked
}

/// Agent-safe activation envelope (schema `focusa.agent_activation_envelope.v1`).
///
/// Contains no forbidden field by construction: no raw email, no one-time key
/// envelope, no signed lease envelope, no poll credential, no verification
/// hash, no card data, no server credential, and no EDD-internal record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentActivationEnvelope {
    pub schema: String,
    /// Resumable registration handle: the agent returns this opaque id so a
    /// later invocation can resume bounded polling from the protected store.
    pub registration_id: String,
    /// Presenter state label (frozen Spec 152E presenter states).
    pub state: String,
    pub terminal: bool,
    /// True when a human must act before the agent may resume. The agent must
    /// not guess, verify, choose, pay, or reveal on the human's behalf.
    pub human_action_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_action: Option<String>,
    /// Masked email (e.g. `c***@example.com`); never the raw address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub masked_email: Option<String>,
    /// Authority-owned branded checkout/verification link; never rebuilt or
    /// rewritten by the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_url: Option<String>,
    /// True when the authority delivered a one-time key envelope for this
    /// registration (the envelope itself is never exposed here).
    pub key_present: bool,
    /// False by default. True only after the customer explicitly opts in and
    /// confirms the one-time reveal (Spec 152E §14.2).
    pub key_visible: bool,
    /// Masked key prefix (e.g. `FOCUSA-XXXX-XXXX-XXXX`), present only when a
    /// caller already holds key knowledge and reveal was authorized.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub masked_key_prefix: Option<String>,
    pub poll_count: u32,
    pub max_polls: u32,
    pub retry_posture: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<u32>,
    pub next_action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ActivationErrorEnvelope>,
}

impl AgentActivationEnvelope {
    /// Build the agent envelope from one presenter-neutral session. `error`
    /// carries the typed authority code and safe next action when the step
    /// settled through an authority failure; `masked_key_prefix` is only
    /// emitted under an authorized reveal and only from a caller-held key.
    pub fn from_session<A: crate::activation_client::ActivationAuthority>(
        session: &ActivationSession<A>,
        error: Option<&ActivationError>,
        reveal: AgentKeyReveal,
        masked_key_prefix: Option<String>,
    ) -> Result<Self, ActivationClientError> {
        let canonical = session.envelope(error.cloned())?;
        let key_present = canonical.one_time_key_envelope.is_some();
        let authorized_reveal = key_present && reveal.authorized();
        let human_action = human_action_for_state(&canonical.state);
        Ok(Self {
            schema: AGENT_ENVELOPE_SCHEMA.to_string(),
            registration_id: canonical.registration_id.clone(),
            state: canonical.state.clone(),
            terminal: canonical.terminal,
            human_action_required: human_action_required(&canonical.state),
            human_action: human_action.map(str::to_string),
            masked_email: canonical.masked_email.clone(),
            safe_url: canonical.safe_url.clone(),
            key_present,
            key_visible: authorized_reveal,
            masked_key_prefix: if authorized_reveal {
                masked_key_prefix
            } else {
                None
            },
            poll_count: session.registration().poll_count,
            max_polls: session.registration().max_polls,
            retry_posture: canonical.retry.posture.label().to_string(),
            retry_after_seconds: canonical.retry.retry_after_seconds,
            // When a human action is required the agent's next action IS the
            // typed human action (verify/choose/pay/accept); the canonical
            // presenter next action (e.g. "poll_after_retry_after") would
            // invite the agent to advance a human-required state itself.
            next_action: human_action
                .map(str::to_string)
                .unwrap_or_else(|| canonical.next_action.clone()),
            error: canonical.error.clone(),
        })
    }

    /// Project an agent envelope from a persisted presenter-safe registration
    /// snapshot (daemon/API operation surface). Poll credentials and secrets
    /// are never present in snapshots by construction, so this projection
    /// cannot leak them.
    pub fn from_registration(registration: &ActivationRegistration) -> Self {
        let state = crate::activation_reducer::presenter_state(registration.state)
            .label()
            .to_string();
        let human_action = human_action_for_state(&state);
        Self {
            schema: AGENT_ENVELOPE_SCHEMA.to_string(),
            registration_id: registration.registration_id.clone(),
            state: state.clone(),
            terminal: registration.state.is_terminal(),
            human_action_required: human_action_required(&state),
            human_action: human_action.map(str::to_string),
            masked_email: registration.masked_email.clone(),
            safe_url: None,
            key_present: false,
            key_visible: false,
            masked_key_prefix: None,
            poll_count: registration.poll_count,
            max_polls: registration.max_polls,
            retry_posture: "none".to_string(),
            retry_after_seconds: None,
            next_action: human_action
                .map(str::to_string)
                .unwrap_or_else(|| {
                    crate::activation_reducer::presenter_next_action(
                        crate::activation_reducer::presenter_state(registration.state),
                    )
                    .to_string()
                }),
            error: None,
        }
    }
}

/// Mask an email for agent surfaces; raw emails fail closed (None).
pub fn masked_email_or_none(email: &str) -> Option<String> {
    mask_email(email)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activation_facade::ActivationErrorCode;
    use crate::activation_reducer::{
        ActivationState, ActivationTransition, ActivationTransitionError, PollRetryPolicy,
        RetryPosture, reduce_activation,
    };

    #[test]
    fn human_action_mapping_covers_all_presenter_states_and_terminals_are_free() {
        let mut required = 0;
        for state in [
            "email_required",
            "email_verification_pending",
            "email_verified",
            "selection_required",
            "checkout_required",
            "payment_pending",
            "license_delivery_ready",
            "activated",
            "denied",
            "recovery_only",
        ] {
            let action = human_action_for_state(state);
            if state == "activated" || state == "denied" || state == "recovery_only" {
                assert!(action.is_none(), "{state} is terminal: no human action");
                assert!(!human_action_required(state));
            } else {
                assert!(action.is_some(), "{state} requires a typed human action");
                assert!(human_action_required(state));
                required += 1;
            }
        }
        assert_eq!(required, 7);
        // Unknown states fail closed as human-required.
        assert!(human_action_required("unknown_state"));
    }

    #[test]
    fn mask_key_prefix_never_exposes_the_full_key() {
        assert_eq!(
            mask_key_prefix("FOCUSA-ABCD-EFGH-IJKL-MNOP"),
            "FOCUSA-XXXX-XXXX-XXXX-XXXX"
        );
        assert_eq!(mask_key_prefix("FOCUSA-ABCD"), "FOCUSA-XXXX");
        assert_eq!(mask_key_prefix(""), "XXXX-XXXX-XXXX-XXXX");
        assert!(!mask_key_prefix("FOCUSA-ABCD-EFGH").contains("ABCD"));
    }

    #[test]
    fn reveal_requires_explicit_opt_in_and_confirmation() {
        assert!(!AgentKeyReveal::denied().authorized());
        assert!(!AgentKeyReveal {
            reveal_key: true,
            reveal_confirmation: false,
        }
        .authorized());
        assert!(!AgentKeyReveal {
            reveal_key: false,
            reveal_confirmation: true,
        }
        .authorized());
        assert!(AgentKeyReveal {
            reveal_key: true,
            reveal_confirmation: true,
        }
        .authorized());
    }

    #[test]
    fn agent_envelope_never_carries_raw_email_key_or_secret() {
        let masked = masked_email_or_none("customer@example.com").expect("maskable");
        assert_eq!(masked, "c***@example.com");
        let canonical = ActivationOutputEnvelope::build(
            "request-0001",
            "registration-0001",
            ActivationState::EmailChallengeSent,
            Some(&masked),
            None,
            None,
            Some("base64:one-time-key-envelope".into()),
            None,
            None,
            None,
            PollRetryPolicy::safe_retry(),
        )
        .expect("canonical");
        let registration = ActivationRegistration {
            schema: "focusa.activation_registration.v1".into(),
            registration_id: "registration-0001".into(),
            facade_id: "focusa-cli".into(),
            presenter: "cli".into(),
            install_channel: "source_build".into(),
            state: ActivationState::EmailChallengeSent,
            masked_email: Some(masked.clone()),
            poll_count: 1,
            max_polls: 40,
        };
        // Masked by default: key_present true but key_visible false and no
        // envelope content anywhere.
        let envelope = build_from_canonical(&canonical, Some(&registration), AgentKeyReveal::denied());
        assert_eq!(envelope.schema, AGENT_ENVELOPE_SCHEMA);
        assert_eq!(envelope.state, "email_verification_pending");
        assert!(envelope.human_action_required);
        assert_eq!(envelope.human_action.as_deref(), Some("enter_verification_code"));
        assert!(envelope.key_present);
        assert!(!envelope.key_visible);
        assert!(envelope.masked_key_prefix.is_none());
        let body = serde_json::to_string(&envelope).unwrap();
        assert!(!body.contains("customer@example.com"));
        assert!(!body.contains("one-time-key-envelope"));
        assert!(!body.contains("full-key-envelope"));
        assert!(!body.contains("poll_credential"));
        assert!(!body.contains("verification_hash"));
        assert!(body.contains("c***@example.com"));
    }

    #[test]
    fn authorized_reveal_is_still_prefix_masked_and_never_invents() {
        let masked = masked_email_or_none("customer@example.com").expect("maskable");
        let canonical = ActivationOutputEnvelope::build(
            "request-0001",
            "registration-0001",
            ActivationState::TerminalDeliveryReady,
            Some(&masked),
            None,
            None,
            Some("base64:one-time-key-envelope".into()),
            None,
            None,
            None,
            PollRetryPolicy::safe_retry(),
        )
        .expect("canonical");
        let registration = ActivationRegistration {
            schema: "focusa.activation_registration.v1".into(),
            registration_id: "registration-0001".into(),
            facade_id: "focusa-cli".into(),
            presenter: "cli".into(),
            install_channel: "source_build".into(),
            state: ActivationState::TerminalDeliveryReady,
            masked_email: Some(masked.clone()),
            poll_count: 2,
            max_polls: 40,
        };
        let envelope = build_from_canonical(
            &canonical,
            Some(&registration),
            AgentKeyReveal {
                reveal_key: true,
                reveal_confirmation: true,
            },
        );
        assert!(envelope.key_visible);
        // The envelope carries only a caller-supplied masked prefix, never
        // the envelope content itself.
        let body = serde_json::to_string(&envelope).unwrap();
        assert!(!body.contains("one-time-key-envelope"));
        assert_eq!(envelope.poll_count, 2);
        assert_eq!(envelope.max_polls, 40);
        assert_eq!(envelope.next_action, "reveal_or_accept_license");
    }

    #[test]
    fn registration_projection_never_leaks_and_marks_terminal_states() {
        let registration = ActivationRegistration {
            schema: "focusa.activation_registration.v1".into(),
            registration_id: "registration-0001".into(),
            facade_id: "focusa-cli".into(),
            presenter: "cli".into(),
            install_channel: "source_build".into(),
            state: ActivationState::RecoveryOnly,
            masked_email: Some("c***@example.com".into()),
            poll_count: 0,
            max_polls: 40,
        };
        let envelope = AgentActivationEnvelope::from_registration(&registration);
        assert!(envelope.terminal);
        assert!(!envelope.human_action_required);
        assert_eq!(envelope.state, "recovery_only");
        assert_eq!(envelope.next_action, "recovery");
        assert!(!envelope.key_present);
        assert!(!envelope.key_visible);

        let pending = ActivationRegistration {
            schema: "focusa.activation_registration.v1".into(),
            registration_id: "registration-0002".into(),
            facade_id: "focusa-cli".into(),
            presenter: "cli".into(),
            install_channel: "source_build".into(),
            state: ActivationState::CheckoutPending,
            masked_email: Some("c***@example.com".into()),
            poll_count: 3,
            max_polls: 40,
        };
        let envelope = AgentActivationEnvelope::from_registration(&pending);
        assert!(!envelope.terminal);
        assert!(envelope.human_action_required);
        assert_eq!(envelope.human_action.as_deref(), Some("complete_payment_then_poll"));
        assert_eq!(envelope.registration_id, "registration-0002");
    }

    #[test]
    fn mask_email_fails_closed_for_unmaskable_input() {
        assert_eq!(masked_email_or_none("not-an-email"), None);
        assert_eq!(masked_email_or_none(""), None);
        assert_eq!(
            masked_email_or_none("owner@example.com").as_deref(),
            Some("o***@example.com")
        );
    }

    fn build_from_canonical(
        canonical: &ActivationOutputEnvelope,
        registration: Option<&ActivationRegistration>,
        reveal: AgentKeyReveal,
    ) -> AgentActivationEnvelope {
        let key_present = canonical.one_time_key_envelope.is_some();
        let authorized_reveal = key_present && reveal.authorized();
        let human_action = human_action_for_state(&canonical.state);
        AgentActivationEnvelope {
            schema: AGENT_ENVELOPE_SCHEMA.to_string(),
            registration_id: canonical.registration_id.clone(),
            state: canonical.state.clone(),
            terminal: canonical.terminal,
            human_action_required: human_action_required(&canonical.state),
            human_action: human_action.map(str::to_string),
            masked_email: canonical.masked_email.clone(),
            safe_url: canonical.safe_url.clone(),
            key_present,
            key_visible: authorized_reveal,
            masked_key_prefix: if authorized_reveal {
                Some(mask_key_prefix("FOCUSA-ABCD-EFGH-IJKL"))
            } else {
                None
            },
            poll_count: registration.map(|r| r.poll_count).unwrap_or(0),
            max_polls: registration.map(|r| r.max_polls).unwrap_or(DEFAULT_MAX_POLLS),
            retry_posture: canonical.retry.posture.label().to_string(),
            retry_after_seconds: canonical.retry.retry_after_seconds,
            next_action: human_action
                .map(str::to_string)
                .unwrap_or_else(|| canonical.next_action.clone()),
            error: canonical.error.clone(),
        }
    }

    #[test]
    fn reducer_still_blocks_unverified_promotion_for_agent_surface() {
        // The agent protocol inherits the frozen reducer: no unverified
        // promotion, no local issuance, recovery never re-grants.
        assert_eq!(
            reduce_activation(
                ActivationState::EmailChallengeSent,
                ActivationTransition::AccountPromoted
            ),
            Err(ActivationTransitionError::IllegalTransition)
        );
        assert_eq!(
            reduce_activation(
                ActivationState::AttemptCreated,
                ActivationTransition::LeaseIssued
            ),
            Err(ActivationTransitionError::IllegalTransition)
        );
        assert_eq!(
            reduce_activation(
                ActivationState::RecoveryOnly,
                ActivationTransition::Delivered
            ),
            Err(ActivationTransitionError::IllegalTransition)
        );
    }
}
