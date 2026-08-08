use chrono::{Duration, Utc};
use focusa_license::{
    authority::{EntitlementSnapshot, EntitlementState},
    base_product_compatibility_projection, resolve_base_focusa_product, BaseProductDecision,
    feature_decision::{
        FeatureDecision, FeatureDecisionDenial, FeatureDiscoverability, FeatureOperationClass,
        FeatureRecoveryPosture, ProductFeatureDefinition, ProductFeatureRegistry,
    },
    uiai_child_token::{
        AuthorityChildTokenEnvelope, UiaiChildTokenBroker, UiaiChildTokenError, UiaiChildTokenRequest,
    },
    CapabilityFamily as Family, DecisionReason as Reason, EntitlementPolicyPosture as Posture,
    PolicyEntitlementState as State,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use uuid::Uuid;

use focusa_license::{reduce_entitlement_state};

const FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/spec152f-entitlement-policy-cases.v1.json"
);

const BASE_COMPAT_IDS: [&str; 3] = [
    "focusa.core.mission",
    "focusa.core.workpoint",
    "focusa.core.evidence",
];

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

#[derive(Debug, Deserialize)]
struct GoldenFixture {
    grid_case_count: usize,
    family_count: usize,
    feature_compatibility_count: usize,
    grid_cases: Vec<FixtureGridCase>,
    negative_mutations: Vec<NegativeMutationCase>,
    #[serde(default)]
    base_product_compatibility_cases: Vec<BaseProductVectorCase>,
    #[serde(default)]
    feature_vector_cases: Vec<FeatureVectorCase>,
    #[serde(default)]
    uiai_child_token_cases: Vec<UiaiChildTokenVectorCase>,
    #[serde(default)]
    unknown_family_cases: Vec<UnknownFamilyCase>,
    #[serde(default)]
    dormant_dimension_cases: Vec<DormantDimensionCase>,
}

#[derive(Debug, Deserialize)]
struct FixtureGridCase {
    case_id: String,
    expected_decision: String,
    family: String,
    state: String,
}

#[derive(Debug, Deserialize)]
struct NegativeMutationCase {
    id: String,
}

#[derive(Debug, Deserialize)]
struct BaseProductVectorCase {
    case_id: String,
    state: String,
    product: String,
    stored_features: BTreeMap<String, bool>,
    expected_decision: String,
    expected_projection: BTreeMap<String, bool>,
}

#[derive(Debug, Deserialize)]
struct FeatureVectorCase {
    case_id: String,
    feature: String,
    feature_product: String,
    requested_product: String,
    #[serde(default)]
    family: Option<String>,
    registered: bool,
    operation_class: String,
    recovery_posture: String,
    limit_bucket: Option<String>,
    limit_unit: Option<String>,
    discoverability: String,
    owner: String,
    requested_units: u64,
    granted_features: BTreeSet<String>,
    limits: BTreeMap<String, u64>,
    expected_outcome: String,
    #[serde(default)]
    expected_reserved_units: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct UiaiChildTokenVectorCase {
    case_id: String,
    expected_outcome: String,
    #[serde(default)]
    focusa_product: Option<String>,
    #[serde(default)]
    focusa_node: Option<String>,
    #[serde(default)]
    focusa_lease_id: Option<String>,
    #[serde(default)]
    focusa_lease_digest: Option<String>,
    #[serde(default)]
    focusa_snapshot_sequence: Option<u64>,
    #[serde(default)]
    request_parent_lease_sequence: Option<u64>,
    #[serde(default)]
    uiai_product: Option<String>,
    #[serde(default)]
    uiai_node: Option<String>,
    #[serde(default)]
    uiai_snapshot_sequence: Option<u64>,
    #[serde(default)]
    request_uiai_grant_sequence: Option<u64>,
    #[serde(default)]
    request_node: Option<String>,
    #[serde(default)]
    request_parent_lease_id: Option<String>,
    #[serde(default)]
    request_uiai_lease_id: Option<String>,
    #[serde(default)]
    request_audience: Option<String>,
    #[serde(default)]
    request_client_id: Option<String>,
    #[serde(default)]
    request_nonce: Option<String>,
    #[serde(default)]
    envelope_audience: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UnknownFamilyCase {
    state: String,
    family: String,
    expected_result: String,
}

#[derive(Debug, Deserialize)]
struct DormantDimensionCase {
    case_id: String,
    dimension: String,
    activation: String,
    missing_claim_effect: String,
    expected_result: String,
}

#[derive(Clone, Copy)]
struct ParsedDecision {
    posture: Option<Posture>,
    reason: Reason,
}

fn fixture() -> GoldenFixture {
    let raw = fs::read_to_string(FIXTURE_PATH).expect("fixture must parse");
    serde_json::from_str(&raw).expect("fixture must decode")
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
        "verified_no_license" => Some(State::VerifiedNoLicense),
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

fn parse_feature_operation_class(value: &str) -> FeatureOperationClass {
    match value {
        "read" => FeatureOperationClass::Read,
        "write" => FeatureOperationClass::Write,
        "execute" => FeatureOperationClass::Execute,
        "recovery" => FeatureOperationClass::Recovery,
        "admin" => FeatureOperationClass::Admin,
        "update" => FeatureOperationClass::Update,
        "install" => FeatureOperationClass::Install,
        _ => panic!("unknown feature operation class: {value}"),
    }
}

fn parse_feature_recovery_posture(value: &str) -> FeatureRecoveryPosture {
    match value {
        "entitlement_required" => FeatureRecoveryPosture::EntitlementRequired,
        "always_available" => FeatureRecoveryPosture::AlwaysAvailable,
        _ => panic!("unknown feature recovery posture: {value}"),
    }
}

fn parse_feature_discoverability(value: &str) -> FeatureDiscoverability {
    match value {
        "visible" => FeatureDiscoverability::Visible,
        "advanced" => FeatureDiscoverability::Advanced,
        "internal" => FeatureDiscoverability::Internal,
        _ => panic!("unknown feature discoverability: {value}"),
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
    let fixture = fixture();
    let expected_states = [
        "pending_unverified",
        "verified_no_license",
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

    assert_eq!(fixture.grid_case_count, 63);
    assert_eq!(fixture.family_count, expected_families.len());
    assert_eq!(fixture.feature_compatibility_count, 15);
    assert_eq!(fixture.grid_cases.len(), fixture.grid_case_count);

    let mut expected_pairs = BTreeSet::new();
    for state in expected_states {
        for family in expected_families {
            expected_pairs.insert((state, family));
        }
    }

    let mut expected_pairs: BTreeSet<(String, String)> = BTreeSet::new();
    for state in expected_states {
        for family in expected_families {
            expected_pairs.insert((state.to_string(), family.to_string()));
        }
    }

    for row in &fixture.grid_cases {
        let key = (row.state.clone(), row.family.clone());
        assert_eq!(row.case_id, format!("{}::{}", key.0, key.1));
        assert!(expected_pairs.remove(&key), "unexpected or duplicate state/family pair: {key:?}");

        let family = family_from_fixture(&row.family)
            .unwrap_or_else(|| panic!("unknown family label: {}", row.family));
        let decision = parse_decision(&row.expected_decision);

        if let Some(state) = state_from_fixture(&row.state) {
            if family == Family::InternalMaintenance {
                let reduction = reduce_entitlement_state(state, family, Some(Posture::Deny));
                assert_eq!(reduction.reason(), Reason::Inherit);
                continue;
            }
            let reduction = reduce_entitlement_state(state, family, None);
            assert_eq!(
                decision.posture,
                Some(reduction.posture()),
                "case {key:?}",
            );
            assert_eq!(decision.reason, reduction.reason(), "case {key:?}");
        } else {
            panic!("unknown state label: {}", row.state);
        }
    }

    assert!(expected_pairs.is_empty(), "fixture must include every state/family combination");
}

#[test]
fn base_product_compatibility_vectors_match_projection_logic() {
    let fixture = fixture();
    for case in fixture.base_product_compatibility_cases {
        let state = state_from_fixture(&case.state)
            .unwrap_or_else(|| panic!("unknown base state in fixture: {}", case.state));
        let decision = resolve_base_focusa_product(&case.product, state);
        assert_eq!(decision.label(), case.expected_decision);
        assert_eq!(decision, parse_base_product_decision(&case.expected_decision));

        let projected = base_product_compatibility_projection(decision, &case.stored_features);
        for id in BASE_COMPAT_IDS {
            assert_eq!(
                projected.get(id),
                case.expected_projection.get(id),
                "compatibility projection mismatch for {} / {}",
                case.case_id,
                id,
            );
        }
    }
}

#[test]
fn feature_vector_cases_match_decision_semantics() {
    let fixture = fixture();
    for case in fixture.feature_vector_cases {
        let registry = if case.registered {
            ProductFeatureRegistry {
                schema: "focusa.feature_registry.v1".to_string(),
                product: case.feature_product.clone(),
                features: vec![ProductFeatureDefinition {
                    key: case.feature.clone(),
                    product: case.feature_product.clone(),
                    operation_class: parse_feature_operation_class(&case.operation_class),
                    recovery_posture: parse_feature_recovery_posture(&case.recovery_posture),
                    limit_bucket: case.limit_bucket.clone(),
                    limit_unit: case.limit_unit.clone(),
                    discoverability: parse_feature_discoverability(&case.discoverability),
                    owner: case.owner.clone(),
                }],
            }
        } else {
            ProductFeatureRegistry {
                schema: "focusa.feature_registry.v1".to_string(),
                product: case.feature_product.clone(),
                features: Vec::new(),
            }
        };
        let decision = registry.decide(
            &case.requested_product,
            &case.feature,
            case.requested_units,
            &case.granted_features,
            &case.limits,
        );

        match (case.expected_outcome.as_str(), decision) {
            ("granted", FeatureDecision::Granted { reserved_units }) => {
                assert_eq!(reserved_units, case.expected_reserved_units.unwrap_or(reserved_units));
            }
            ("recovery_allowed", FeatureDecision::RecoveryAllowed) => {}
            ("denied_unknown_feature", FeatureDecision::Denied(FeatureDecisionDenial::UnknownFeature)) => {}
            ("denied_wrong_product", FeatureDecision::Denied(FeatureDecisionDenial::WrongProduct)) => {}
            ("denied_feature_not_granted", FeatureDecision::Denied(FeatureDecisionDenial::FeatureNotGranted)) => {}
            ("denied_limit_not_granted", FeatureDecision::Denied(FeatureDecisionDenial::LimitNotGranted)) => {}
            ("denied_limit_exhausted", FeatureDecision::Denied(FeatureDecisionDenial::LimitExhausted)) => {}
            ("denied_invalid_registry", FeatureDecision::Denied(FeatureDecisionDenial::InvalidRegistry)) => {}
            (expected, actual) => {
                panic!(
                    "feature vector {}/{} produced {actual:?} but expected {expected}",
                    case.case_id,
                    case.feature
                )
            }
        }
    }
}

#[test]
fn uiai_child_token_vectors_fail_closed_for_wrong_inputs() {
    for case in fixture().uiai_child_token_cases {
        let now = Utc::now();
        let focusa = build_focusa_snapshot(&case, now);
        let uiai = build_uiai_snapshot(&case, now);
        let request = build_uiai_request(&case);
        let broker = UiaiChildTokenBroker::default();
        let validation = broker.validate_request(&request, &focusa, &uiai, now);

        match case.expected_outcome.as_str() {
            "parent_entitlement_invalid" => {
                assert_eq!(validation, Err(UiaiChildTokenError::ParentEntitlementInvalid));
            }
            "uiai_grant_invalid" => {
                assert_eq!(validation, Err(UiaiChildTokenError::UiaiGrantInvalid));
            }
            "authority_response_mismatch" => {
                assert!(validation.is_ok());
                let mut accepting_broker = UiaiChildTokenBroker::default();
                let envelope = build_uiai_envelope(&case, &request, now);
                assert_eq!(
                    accepting_broker.accept_authority_token(&request, &focusa, &uiai, envelope, now),
                    Err(UiaiChildTokenError::AuthorityResponseMismatch),
                );
            }
            "request_accepted" => {
                assert!(validation.is_ok());
                let mut accepting_broker = UiaiChildTokenBroker::default();
                let envelope = build_uiai_envelope(&case, &request, now);
                assert!(accepting_broker
                    .accept_authority_token(&request, &focusa, &uiai, envelope, now)
                    .is_ok());
            }
            other => panic!("unexpected uiai child token outcome: {other}"),
        }
    }
}

#[test]
fn unknown_family_rows_are_rejected() {
    for case in fixture().unknown_family_cases {
        assert!(family_from_fixture(&case.family).is_none());
        assert_eq!(case.expected_result, "unknown_family");
        assert!(state_from_fixture(&case.state).is_some());
    }
}

#[test]
fn dormant_field_vectors_record_no_authority_effect() {
    for case in fixture().dormant_dimension_cases {
        assert!(case.activation == "dormant" || case.activation == "dormant_for_commerce");
        assert_eq!(case.expected_result, "no_authority_effect");
        assert!(
            case.case_id.contains("has_no") && case.case_id.ends_with("_effect"),
            "dormant vector case naming should remain explicit: {}",
            case.case_id
        );
    }
}

fn build_focusa_snapshot(case: &UiaiChildTokenVectorCase, now: chrono::DateTime<Utc>) -> EntitlementSnapshot {
    let mut snapshot = EntitlementSnapshot::unactivated(
        case.focusa_product.as_deref().unwrap_or("focusa"),
        case.focusa_node.as_deref().unwrap_or("node-001"),
    );
    snapshot.state = EntitlementState::Active;
    snapshot.sequence = Some(case.focusa_snapshot_sequence.unwrap_or(7));
    snapshot.lease_id = Some(case.focusa_lease_id.clone().unwrap_or_else(|| "lease-focusa".to_string()));
    snapshot.lease_digest = Some(case.focusa_lease_digest.clone().unwrap_or_else(|| "sha256:focusa".to_string()));
    snapshot.expires_at = Some(now + Duration::minutes(60));
    snapshot.features.insert("focusa.agent.parallelism".to_string(), true);
    snapshot
}

fn build_uiai_snapshot(case: &UiaiChildTokenVectorCase, now: chrono::DateTime<Utc>) -> EntitlementSnapshot {
    let mut snapshot = EntitlementSnapshot::unactivated(
        case.uiai_product.as_deref().unwrap_or("uiai-engine"),
        case.uiai_node.as_deref().unwrap_or_else(|| case.focusa_node.as_deref().unwrap_or("node-001")),
    );
    snapshot.state = EntitlementState::Active;
    snapshot.sequence = Some(case.uiai_snapshot_sequence.unwrap_or(11));
    snapshot.lease_id = Some("lease-uiai".to_string());
    snapshot.lease_digest = Some("sha256:uiai".to_string());
    snapshot.expires_at = Some(now + Duration::minutes(60));
    snapshot.features.insert("focusa.agent.parallelism".to_string(), true);
    snapshot
}

fn build_uiai_request(case: &UiaiChildTokenVectorCase) -> UiaiChildTokenRequest {
    let request_id = Uuid::nil();
    UiaiChildTokenRequest {
        request_id,
        audience: case.request_audience.clone().unwrap_or_else(|| "aud-focusa".to_string()),
        node_id: case
            .request_node
            .clone()
            .unwrap_or_else(|| "node-001".to_string()),
        client_id: case.request_client_id.clone().unwrap_or_else(|| "client-focusa".to_string()),
        parent_lease_id: case.request_parent_lease_id.clone().unwrap_or_else(|| {
            case.focusa_lease_id
                .as_ref()
                .cloned()
                .unwrap_or_else(|| "lease-focusa".to_string())
        }),
        parent_lease_sequence: case.request_parent_lease_sequence.unwrap_or(case.focusa_snapshot_sequence.unwrap_or(7)),
        parent_lease_digest: case
            .focusa_lease_digest
            .clone()
            .unwrap_or_else(|| "sha256:focusa".to_string()),
        uiai_grant_lease_id: case.request_uiai_lease_id.clone().unwrap_or_else(|| "lease-uiai".to_string()),
        uiai_grant_sequence: case.request_uiai_grant_sequence.unwrap_or(case.uiai_snapshot_sequence.unwrap_or(11)),
        requested_features: BTreeSet::from(["focusa.agent.parallelism".to_string()]),
        requested_limits: BTreeMap::new(),
        nonce: case
            .request_nonce
            .clone()
            .unwrap_or_else(|| format!("{}-nonce", case.case_id)),
    }
}

fn build_uiai_envelope(
    case: &UiaiChildTokenVectorCase,
    request: &UiaiChildTokenRequest,
    now: chrono::DateTime<Utc>,
) -> AuthorityChildTokenEnvelope {
    let issued_at = now - Duration::minutes(1);
    let expires_at = now + Duration::minutes(5);
    AuthorityChildTokenEnvelope {
        schema: "focusa.uiai_child_token.v1".to_string(),
        token: format!("token-{}", case.case_id),
        token_id: format!("token-id-{}", case.case_id),
        audience: case
            .envelope_audience
            .clone()
            .unwrap_or_else(|| request.audience.clone()),
        node_id: request.node_id.clone(),
        client_id: request.client_id.clone(),
        parent_lease_id: request.parent_lease_id.clone(),
        parent_lease_sequence: request.parent_lease_sequence,
        parent_lease_digest: request.parent_lease_digest.clone(),
        uiai_grant_lease_id: request.uiai_grant_lease_id.clone(),
        uiai_grant_sequence: request.uiai_grant_sequence,
        features: request.requested_features.clone(),
        limits: request.requested_limits.clone(),
        nonce: request.nonce.clone(),
        issued_at,
        expires_at,
    }
}

fn parse_base_product_decision(value: &str) -> BaseProductDecision {
    match value {
        "entitled" => BaseProductDecision::Entitled,
        "limited" => BaseProductDecision::Limited,
        "denied" => BaseProductDecision::Denied,
        _ => panic!("bad fixture base decision: {value}"),
    }
}
