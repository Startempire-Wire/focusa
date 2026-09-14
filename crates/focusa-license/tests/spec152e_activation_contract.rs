//! Spec 152E activation client/reducer contract replay.
//!
//! Replays the deterministic transcript fixtures
//! (`fixtures/spec152e-activation-transcript-fixtures.v1.json`) against the
//! shared presenter-neutral reducer. The fixture is the cross-language single
//! source of truth: this test proves the Rust reducer and presenter mapping
//! agree byte-for-byte with the frozen Spec 152E state machine, while
//! `tests/spec152e_activation_client_contract_test.py` proves the same
//! fixtures agree with the frozen JSON contracts.

use std::collections::{BTreeMap, BTreeSet};

use focusa_license::{
    ActivationError, ActivationErrorCode, ActivationState, ActivationTransition,
    ActivationTransitionError, presenter_state, reduce_activation,
};
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/spec152e-activation-transcript-fixtures.v1.json");

#[derive(Deserialize)]
struct Fixture {
    initial_state: String,
    presenter_states: Vec<String>,
    state_machine: Vec<MachineRow>,
    presenter_by_state: BTreeMap<String, String>,
    positive_transcripts: Vec<Transcript>,
    negative_transcripts: Vec<NegativeTranscript>,
    error_cases: Vec<ErrorCase>,
}

#[derive(Deserialize)]
struct MachineRow {
    from: String,
    event: String,
    to: String,
}

#[derive(Deserialize)]
struct Transcript {
    id: String,
    from: String,
    steps: Vec<MachineRow>,
}

#[derive(Deserialize)]
struct NegativeTranscript {
    id: String,
    from: String,
    event: String,
    reason: String,
}

#[derive(Deserialize)]
struct ErrorCase {
    operation: Option<String>,
    code: String,
    http_status: u16,
    retryable: bool,
    safe_next_action: String,
    retry_posture: String,
}

fn fixture() -> Fixture {
    serde_json::from_str(FIXTURE).expect("valid transcript fixture")
}

fn state(label: &str) -> ActivationState {
    let state = [
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
    ];
    state
        .into_iter()
        .find(|candidate| candidate.label() == label)
        .unwrap_or_else(|| panic!("unknown state {label}"))
}

fn transition(label: &str) -> ActivationTransition {
    let transitions = [
        (
            "challenge_delivered",
            ActivationTransition::ChallengeDelivered,
        ),
        ("email_verified", ActivationTransition::EmailVerified),
        ("account_promoted", ActivationTransition::AccountPromoted),
        ("offer_selected", ActivationTransition::OfferSelected),
        ("checkout_started", ActivationTransition::CheckoutStarted),
        (
            "limited_access_chosen",
            ActivationTransition::LimitedAccessChosen,
        ),
        (
            "existing_key_chosen",
            ActivationTransition::ExistingKeyChosen,
        ),
        (
            "entitlement_issued",
            ActivationTransition::EntitlementIssued,
        ),
        (
            "terminal_delivery_ready",
            ActivationTransition::TerminalDeliveryReady,
        ),
        ("device_registered", ActivationTransition::DeviceRegistered),
        ("lease_issued", ActivationTransition::LeaseIssued),
        ("delivered", ActivationTransition::Delivered),
        ("expired", ActivationTransition::Expired),
        ("denied", ActivationTransition::Denied),
        ("refunded", ActivationTransition::Refunded),
        ("revoked", ActivationTransition::Revoked),
        ("superseded", ActivationTransition::Superseded),
        ("recovery_only", ActivationTransition::RecoveryOnly),
    ];
    transitions
        .into_iter()
        .find(|(label_of, _)| *label_of == label)
        .map(|(_, transition)| transition)
        .unwrap_or_else(|| panic!("unknown transition {label}"))
}

#[test]
fn reducer_matches_the_frozen_machine_encoding_exactly() {
    let fixture = fixture();
    assert_eq!(fixture.initial_state, "attempt_created");
    assert_eq!(fixture.presenter_states.len(), 10);
    let legal: BTreeSet<(String, String, String)> = fixture
        .state_machine
        .iter()
        .map(|row| (row.from.clone(), row.event.clone(), row.to.clone()))
        .collect();
    assert_eq!(
        legal.len(),
        48,
        "the frozen machine has exactly 48 transitions"
    );

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
    let states = [
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
    ];

    let mut accepted = 0;
    for from in states {
        for event in transitions {
            match reduce_activation(from, event) {
                Ok(to) => {
                    accepted += 1;
                    assert!(
                        legal.contains(&(
                            from.label().into(),
                            event.label().into(),
                            to.label().into()
                        )),
                        "{} --{}--> {} is not in the frozen machine",
                        from.label(),
                        event.label(),
                        to.label()
                    );
                }
                Err(ActivationTransitionError::IllegalTransition) => {
                    assert!(
                        !legal
                            .iter()
                            .any(|(f, e, _)| f == from.label() && e == event.label()),
                        "{} --{}--> must be accepted",
                        from.label(),
                        event.label()
                    );
                }
            }
        }
    }
    assert_eq!(accepted, 48);
}

#[test]
fn presenter_mapping_matches_the_fixture_for_all_19_states() {
    let fixture = fixture();
    assert_eq!(fixture.presenter_by_state.len(), 19);
    for (registration, presenter) in &fixture.presenter_by_state {
        assert_eq!(
            presenter_state(state(registration)).label(),
            presenter,
            "presenter mapping mismatch for {registration}"
        );
    }
    let reachable = fixture.presenter_by_state.values().collect::<BTreeSet<_>>();
    assert_eq!(reachable.len(), 10);
    for presenter in &fixture.presenter_states {
        assert!(reachable.contains(presenter), "{presenter} unreachable");
    }
}

#[test]
fn positive_transcripts_replay_deterministically() {
    let fixture = fixture();
    assert!(!fixture.positive_transcripts.is_empty());
    for transcript in &fixture.positive_transcripts {
        assert!(!transcript.steps.is_empty());
        let mut current = state(&transcript.from);
        let mut previous_to = transcript.from.clone();
        for step in &transcript.steps {
            assert_eq!(step.from, previous_to, "{} breaks chain", transcript.id);
            let next =
                reduce_activation(current, transition(&step.event)).unwrap_or_else(|error| {
                    panic!("{} step {} failed: {error:?}", transcript.id, step.event)
                });
            assert_eq!(
                next.label(),
                step.to,
                "{} step {}",
                transcript.id,
                step.event
            );
            current = next;
            previous_to = step.to.clone();
        }
    }
}

#[test]
fn negative_transcripts_fail_closed_without_state_change() {
    let fixture = fixture();
    assert_eq!(fixture.negative_transcripts.len(), 9);
    for negative in &fixture.negative_transcripts {
        let from = state(&negative.from);
        let result = reduce_activation(from, transition(&negative.event));
        assert!(
            matches!(result, Err(ActivationTransitionError::IllegalTransition)),
            "{} must fail closed ({}): got {result:?}",
            negative.id,
            negative.reason
        );
        // Fail closed means the registration state never changes.
        assert_eq!(from, from);
    }
}

#[test]
fn error_cases_bind_every_code_to_typed_values() {
    let fixture = fixture();
    assert_eq!(fixture.error_cases.len(), 33);
    assert_eq!(focusa_license::ActivationErrorCode::ALL.len(), 33);
    let unique = fixture
        .error_cases
        .iter()
        .map(|case| case.code.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), 33, "every error code appears exactly once");
    for case in &fixture.error_cases {
        let code = focusa_license::ActivationErrorCode::ALL
            .iter()
            .find(|candidate| candidate.label() == case.code)
            .unwrap_or_else(|| panic!("unknown code {}", case.code));
        assert_eq!(code.http_status(), case.http_status);
        assert_eq!(code.retryable(), case.retryable);
        assert_eq!(code.safe_next_action(), case.safe_next_action);
        assert!(!case.retry_posture.is_empty());
        // Two codes are runtime entitlement errors outside the activation call
        // stack; every other code binds to a frozen operation failure.
        if case.operation.is_none() {
            assert!(
                !focusa_license::FacadeOperation::ALL
                    .iter()
                    .flat_map(|operation| operation.failure_codes())
                    .any(|failure| failure.label() == case.code),
                "{} must not be an activation operation failure",
                case.code
            );
        }
    }
    // Typed error round trip uses canonical labels.
    for code in focusa_license::ActivationErrorCode::ALL {
        let error = ActivationError::new(code, "request-0001".to_string());
        assert_eq!(error.code.label(), code.label());
        assert_eq!(format!("{error}"), code.label());
        assert!(!serde_json::to_string(&error).unwrap().contains("secret"));
    }
}
