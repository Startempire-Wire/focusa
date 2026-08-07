use focusa_license::{
    reduce_entitlement_state, CapabilityFamily as Family, DecisionReason as Reason,
    EntitlementPolicyPosture as Posture, PolicyEntitlementState as State,
};

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
