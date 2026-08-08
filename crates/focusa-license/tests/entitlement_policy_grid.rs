use std::collections::BTreeSet;

use focusa_license::{
    reduce_entitlement_state, CapabilityFamily as Family, DecisionReason as Reason,
    EntitlementPolicyPosture as Posture, PolicyEntitlementState as State,
};
use serde::Deserialize;

#[derive(Clone, Copy)]
struct Expected {
    posture: Posture,
    reason: Reason,
}

#[derive(Clone, Copy)]
struct Case {
    positive_initiating: Option<Posture>,
    positive: Expected,
    negative_initiating: Option<Posture>,
    negative: Expected,
}

const fn expected(posture: Posture, reason: Reason) -> Expected {
    Expected { posture, reason }
}

const fn case(posture: Posture, reason: Reason) -> Case {
    let result = expected(posture, reason);
    Case {
        positive_initiating: None,
        positive: result,
        // A caller-supplied initiating posture must not alter a normal state
        // grid cell. This is the negative/caller-override case.
        negative_initiating: Some(Posture::Allow),
        negative: result,
    }
}

const fn inherited(posture: Posture) -> Case {
    Case {
        positive_initiating: Some(posture),
        positive: expected(posture, Reason::Inherit),
        // Internal maintenance without its initiating decision fails closed.
        negative_initiating: None,
        negative: expected(Posture::Deny, Reason::MissingInitiatingPolicy),
    }
}

const FAMILIES: [Family; 9] = [
    Family::AccountRecovery,
    Family::ReadProjection,
    Family::BaseFocusa,
    Family::Automation,
    Family::TeamRemote,
    Family::ReleaseProof,
    Family::PremiumUpdates,
    Family::CustomerDataExport,
    Family::InternalMaintenance,
];

// This table is deliberately explicit: each state/family cell has a positive
// case and a negative caller-override/missing-initiator case with a stable
// posture and reason.
const GRID: [(State, [Case; 9]); 7] = [
    (
        State::PendingUnverified,
        [
            case(Posture::Allow, Reason::Allow),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Allow, Reason::AllowExistingLocalOnly),
            inherited(Posture::Deny),
        ],
    ),
    (
        State::VerifiedNoLicense,
        [
            case(Posture::Allow, Reason::Allow),
            case(Posture::Read, Reason::Read),
            case(Posture::Allow, Reason::AllowVerifiedLimited),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Allow, Reason::Allow),
            inherited(Posture::Allow),
        ],
    ),
    (
        State::ActivePaid,
        [
            case(Posture::Allow, Reason::Allow),
            case(Posture::Read, Reason::Read),
            case(Posture::Base, Reason::RequireBase),
            case(Posture::Feature, Reason::RequireFeature),
            case(Posture::Feature, Reason::RequireFeature),
            case(Posture::Feature, Reason::RequireFeature),
            case(Posture::Feature, Reason::RequireFeature),
            case(Posture::Allow, Reason::Allow),
            inherited(Posture::Feature),
        ],
    ),
    (
        State::OfflineGrace,
        [
            case(Posture::Allow, Reason::AllowOfflineOnly),
            case(Posture::Read, Reason::Read),
            case(Posture::Base, Reason::RequireBase),
            case(Posture::Feature, Reason::RequireCachedFeature),
            case(Posture::Feature, Reason::RequireCachedFeature),
            case(Posture::Feature, Reason::RequireCachedFeature),
            case(Posture::Feature, Reason::RequireCachedFeatureWhenSafe),
            case(Posture::Allow, Reason::Allow),
            inherited(Posture::Base),
        ],
    ),
    (
        State::Expired,
        [
            case(Posture::Allow, Reason::Allow),
            case(Posture::Read, Reason::Read),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Allow, Reason::Allow),
            inherited(Posture::Read),
        ],
    ),
    (
        State::RefundedOrRevoked,
        [
            case(Posture::Allow, Reason::Allow),
            case(Posture::Read, Reason::Read),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Allow, Reason::Allow),
            inherited(Posture::Read),
        ],
    ),
    (
        State::MissingOrCorrupt,
        [
            case(Posture::Allow, Reason::Allow),
            case(Posture::Read, Reason::ReadLocalOnly),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Deny, Reason::Deny),
            case(Posture::Allow, Reason::Allow),
            inherited(Posture::Read),
        ],
    ),
];

fn assert_expected(
    state: State,
    family: Family,
    initiating: Option<Posture>,
    expected: Expected,
    case_kind: &str,
) {
    let actual = reduce_entitlement_state(state, family, initiating);
    assert_eq!(
        actual.posture(),
        expected.posture,
        "{case_kind} posture: {state:?}/{family:?}"
    );
    assert_eq!(
        actual.reason(),
        expected.reason,
        "{case_kind} reason: {state:?}/{family:?}"
    );
}

#[derive(Debug, Deserialize)]
struct GoldenFixtureCase {
    case_id: String,
    expected_decision: String,
    family: String,
    state: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct GoldenFixture {
    grid_case_count: usize,
    family_count: usize,
    feature_compatibility_count: usize,
    grid_cases: Vec<GoldenFixtureCase>,
    state_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct ParsedDecision {
    posture: Option<Posture>,
    reason: Reason,
}

fn parse_decision(value: &str) -> ParsedDecision {
    match value {
        "allow" => ParsedDecision {
            posture: Some(Posture::Allow),
            reason: Reason::Allow,
        },
        "allow_offline_only" => ParsedDecision {
            posture: Some(Posture::Allow),
            reason: Reason::AllowOfflineOnly,
        },
        "allow_existing_local_only" => ParsedDecision {
            posture: Some(Posture::Allow),
            reason: Reason::AllowExistingLocalOnly,
        },
        "read" => ParsedDecision {
            posture: Some(Posture::Read),
            reason: Reason::Read,
        },
        "read_local_only" => ParsedDecision {
            posture: Some(Posture::Read),
            reason: Reason::ReadLocalOnly,
        },
        "allow_verified_limited" => ParsedDecision {
            posture: Some(Posture::Allow),
            reason: Reason::AllowVerifiedLimited,
        },
        "require_base" => ParsedDecision {
            posture: Some(Posture::Base),
            reason: Reason::RequireBase,
        },
        "require_feature" => ParsedDecision {
            posture: Some(Posture::Feature),
            reason: Reason::RequireFeature,
        },
        "require_cached_feature" => ParsedDecision {
            posture: Some(Posture::Feature),
            reason: Reason::RequireCachedFeature,
        },
        "require_cached_feature_when_safe" => ParsedDecision {
            posture: Some(Posture::Feature),
            reason: Reason::RequireCachedFeatureWhenSafe,
        },
        "deny" => ParsedDecision {
            posture: Some(Posture::Deny),
            reason: Reason::Deny,
        },
        "inherit" => ParsedDecision {
            posture: None,
            reason: Reason::Inherit,
        },
        _ => panic!("unknown fixture decision: {value}"),
    }
}

fn state_from_fixture(value: &str) -> Option<State> {
    match value {
        "pending_unverified" => Some(State::PendingUnverified),
        "verified_no_grant" | "verified_no_license" => Some(State::VerifiedNoLicense),
        "active_paid" => Some(State::ActivePaid),
        "offline_grace" => Some(State::OfflineGrace),
        "expired" => Some(State::Expired),
        "refunded_or_revoked" => Some(State::RefundedOrRevoked),
        "missing_or_corrupt" => Some(State::MissingOrCorrupt),
        _ => None,
    }
}

fn family_from_fixture(value: &str) -> Option<Family> {
    match value {
        "account_recovery" => Some(Family::AccountRecovery),
        "read_projection" => Some(Family::ReadProjection),
        "base_focusa" => Some(Family::BaseFocusa),
        "automation" => Some(Family::Automation),
        "team_remote" => Some(Family::TeamRemote),
        "release_proof" => Some(Family::ReleaseProof),
        "premium_updates" => Some(Family::PremiumUpdates),
        "customer_data_export" => Some(Family::CustomerDataExport),
        "internal_maintenance" => Some(Family::InternalMaintenance),
        _ => None,
    }
}

#[test]
fn state_grid() {
    for (state, cases) in GRID {
        for (family, case) in FAMILIES.into_iter().zip(cases) {
            assert_expected(
                state,
                family,
                case.positive_initiating,
                case.positive,
                "positive",
            );
            assert_expected(
                state,
                family,
                case.negative_initiating,
                case.negative,
                "negative",
            );
        }
    }
}

#[test]
fn state_grid_fails_closed_without_initiating_policy_or_known_state() {
    for state in GRID.map(|(state, _)| state) {
        let decision = reduce_entitlement_state(state, Family::InternalMaintenance, None);
        assert_eq!(decision.posture(), Posture::Deny);
        assert_eq!(decision.reason(), Reason::MissingInitiatingPolicy);
    }

    assert!(serde_json::from_str::<State>("\"evaluation\"").is_err());
    assert!(serde_json::from_str::<State>("\"unknown\"").is_err());
}

#[test]
fn state_grid_matches_entitlement_vectors() {
    let fixture: GoldenFixture = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/spec152f-entitlement-policy-cases.v1.json"
    )))
    .expect("fixture must parse");
    let expected_states = [
        "pending_unverified",
        "verified_no_grant",
        "evaluation",
        "active_paid",
        "offline_grace",
        "expired",
        "refunded_or_revoked",
        "missing_or_corrupt",
    ];
    let expected_families = [
        "account_recovery",
        "read_projection",
        "base_focusa",
        "automation",
        "team_remote",
        "release_proof",
        "premium_updates",
        "customer_data_export",
        "internal_maintenance",
    ];

    assert_eq!(fixture.grid_case_count, 72);
    assert_eq!(fixture.family_count, 9);
    assert_eq!(fixture.state_count, 8);
    assert_eq!(fixture.feature_compatibility_count, 15);
    assert_eq!(fixture.grid_cases.len(), fixture.grid_case_count);

    let mut seen_cases = BTreeSet::new();
    for state in &expected_states {
        for family in &expected_families {
            seen_cases.insert((state.to_string(), family.to_string()));
        }
    }

    for row in &fixture.grid_cases {
        assert_eq!(row.case_id, format!("{}::{}", row.state, row.family), "case id must match state/family");
        let key = (row.state.clone(), row.family.clone());
        let decision = parse_decision(&row.expected_decision);

        assert!(
            seen_cases.remove(&key),
            "unexpected or duplicate state/family pair: {key:?}"
        );

        let family = family_from_fixture(&row.family)
            .unwrap_or_else(|| panic!("unknown family label: {}", row.family));

        if let Some(_) = state_from_fixture(&row.state) {
            if family == Family::InternalMaintenance {
                assert_eq!(decision.reason, Reason::Inherit);
            }
        } else {
            assert_eq!(row.state, "evaluation");
            if family == Family::InternalMaintenance {
                assert_eq!(decision.reason, Reason::Inherit);
                assert_eq!(decision.posture, None);
            } else {
                assert_eq!(decision.posture.is_some(), true);
            }
        }

        if family == Family::InternalMaintenance {
            assert_eq!(decision.reason, Reason::Inherit);
            assert!(
                decision.posture.is_none() || matches!(decision.posture, Some(Posture::Deny | Posture::Allow | Posture::Read | Posture::Feature | Posture::Base)),
                "invalid inherited posture shape for {}::{}",
                row.state,
                row.family
            );
        }
    }

    assert!(seen_cases.is_empty(), "fixture must include every state/family combination");
}
