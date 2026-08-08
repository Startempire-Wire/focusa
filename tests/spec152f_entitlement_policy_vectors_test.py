#!/usr/bin/env python3
"""Cross-language parity guard for Spec152F entitlement policy vectors."""

from collections import defaultdict
from pathlib import Path
import json
import yaml

ROOT = Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / "docs/contracts/spec152f-entitlement-policy.v1.yaml"
FIXTURE_PATH = ROOT / "tests/fixtures/spec152f-entitlement-policy-cases.v1.json"

BASE_COMPATIBILITY_IDS = {
    "focusa.core.mission",
    "focusa.core.workpoint",
    "focusa.core.evidence",
}

EXPECTED_STATES = [
    "pending_unverified",
    "verified_no_license",
    "active_paid",
    "offline_grace",
    "expired",
    "refunded_or_revoked",
    "missing_or_corrupt",
]

EXPECTED_FAMILIES = [
    "account_recovery",
    "read_projection",
    "base_focusa",
    "automation",
    "team_remote",
    "release_proof",
    "premium_updates",
    "customer_data_export",
    "internal_maintenance",
]

KNOWN_DECISIONS = {
    "allow",
    "allow_offline_only",
    "allow_existing_local_only",
    "allow_verified_limited",
    "read",
    "read_local_only",
    "require_base",
    "require_feature",
    "require_cached_feature",
    "require_cached_feature_when_safe",
    "deny",
    "inherit",
}

KNOWN_BASE_DECISIONS = {"entitled", "limited", "denied"}
KNOWN_FEATURE_OUTCOMES = {
    "granted",
    "recovery_allowed",
    "denied_unknown_feature",
    "denied_wrong_product",
    "denied_feature_not_granted",
    "denied_limit_not_granted",
    "denied_limit_exhausted",
    "denied_invalid_registry",
}
KNOWN_UIAI_OUTCOMES = {
    "request_accepted",
    "parent_entitlement_invalid",
    "uiai_grant_invalid",
    "authority_response_mismatch",
}
KNOWN_NEGATIVE_IDS = {
    "duplicate_family",
    "unknown_active_feature",
    "recovery_requires_feature",
    "dormant_absence_denies",
    "missing_state_family",
    "duplicate_feature_compatibility",
}

POLICY_DECISION_TO_FIXTURE = {
    "allow": "allow",
    "allow_offline_only": "allow_offline_only",
    "allow_basic": "allow",
    "deny": "deny",
    "read": "read",
    "read_local_only": "read_local_only",
    "require_base": "require_base",
    "require_feature": "require_feature",
    "require_cached_feature": "require_cached_feature",
    "require_cached_feature_when_safe": "require_cached_feature_when_safe",
    "registration_verification_and_safety_only": "allow",
    "deny_product_read": "deny",
    "emergency_local_recovery_only": "allow_existing_local_only",
    "allow_manual_one_mutable_project": "allow_verified_limited",
    "allow_one_foreground_ephemeral_session": "allow",
    "inherit": "inherit",
    "inherit_only_allowed_initiating_operation": "inherit",
    "deny_unless_required_for_registration_or_safety": "inherit",
}


def assert_true(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def resolve_base_product(product: str, state: str) -> str:
    if product != "focusa":
        return "denied"
    if state in {"active_paid", "offline_grace"}:
        return "entitled"
    if state == "verified_no_license":
        return "limited"
    return "denied"


def decide_feature_case(row: dict) -> str:
    if not row["registered"]:
        return "denied_unknown_feature"

    if row["requested_product"] != row["feature_product"]:
        return "denied_wrong_product"

    if row["recovery_posture"] == "always_available":
        return "recovery_allowed"

    if row["feature"] not in row["granted_features"]:
        return "denied_feature_not_granted"

    limit_bucket = row.get("limit_bucket")
    if limit_bucket is None:
        return "granted"

    limit_value = row.get("limits", {}).get(limit_bucket)
    if limit_value is None:
        return "denied_limit_not_granted"

    requested = row.get("requested_units", 0)
    if requested == 0 or requested > limit_value:
        return "denied_limit_exhausted"

    return "granted"


def resolve_grid_from_policy(policy) -> dict[tuple[str, str], str]:
    expected = {}
    for row in policy["state_grid"]:
        state = row["state"]
        if state not in EXPECTED_STATES:
            continue
        for family, decision in row["policies"].items():
            if family not in EXPECTED_FAMILIES:
                continue
            expected[(state, family)] = POLICY_DECISION_TO_FIXTURE[decision]
    return expected


def main() -> None:
    policy = yaml.safe_load(POLICY_PATH.read_text(encoding="utf-8"))
    fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))

    assert_true(fixture.get("schema") == "focusa.spec152f.entitlement_policy_cases.v1", "schema must remain Spec152F v1")
    assert_true(fixture.get("policy_id") == "focusa-simple-entitlement", "policy_id mismatch")
    assert_true(fixture.get("policy_version") == 1, "policy_version must be 1")
    assert_true(fixture.get("family_count") == len(EXPECTED_FAMILIES), "family_count mismatch")
    assert_true(fixture.get("state_count") == len(EXPECTED_STATES), "state_count mismatch")
    assert_true(fixture.get("feature_compatibility_count") == 15, "feature_compatibility_count mismatch")

    grid_cases = fixture["grid_cases"]
    assert_true(len(grid_cases) == fixture["grid_case_count"], "grid_case_count must equal row count")
    assert_true(fixture["grid_case_count"] == 63, "expected 63 grid cases")

    policy_grid = resolve_grid_from_policy(policy)
    assert_true(len(policy_grid) == len(EXPECTED_STATES) * len(EXPECTED_FAMILIES), "policy grid coverage incomplete")

    seen_pairs = set()
    by_state = defaultdict(int)
    by_family = defaultdict(int)

    for row in grid_cases:
        state = row["state"]
        family = row["family"]
        decision = row["expected_decision"]
        case_id = row["case_id"]

        assert_true(state in EXPECTED_STATES, f"unexpected state {state}")
        assert_true(family in EXPECTED_FAMILIES, f"unexpected family {family}")
        assert_true(decision in KNOWN_DECISIONS, f"unexpected decision {decision}")
        assert_true(case_id == f"{state}::{family}", f"case_id mismatch: {case_id}")

        key = (state, family)
        assert_true(key not in seen_pairs, f"duplicate case: {case_id}")
        seen_pairs.add(key)

        by_state[state] += 1
        by_family[family] += 1
        assert_true(policy_grid.get(key) == decision, f"grid decision mismatch for {state}/{family}: {decision} != {policy_grid.get(key)}")

    expected_pairs = {(state, family) for state in EXPECTED_STATES for family in EXPECTED_FAMILIES}
    assert_true(seen_pairs == expected_pairs, "incomplete or duplicated grid coverage")
    for state in EXPECTED_STATES:
        assert_true(by_state[state] == len(EXPECTED_FAMILIES), f"state {state} missing family rows")
    for family in EXPECTED_FAMILIES:
        assert_true(by_family[family] == len(EXPECTED_STATES), f"family {family} missing state rows")

    base_cases = fixture.get("base_product_compatibility_cases", [])
    base_ids = {row["case_id"] for row in base_cases}
    assert_true("base_product_focusa_verified_no_license_limited_projection" in base_ids, "expected limited compatibility case")
    for row in base_cases:
        state = row["state"]
        decision = row["expected_decision"]
        projected = row["expected_projection"]

        assert_true(state in EXPECTED_STATES, f"unexpected base state {state}")
        assert_true(decision in KNOWN_BASE_DECISIONS, f"unexpected base decision {decision}")
        assert_true(row["product"] in {"focusa", "uiai-engine"}, "unexpected compatibility product")
        assert_true(resolve_base_product(row["product"], state) == decision, f"base product decision mismatch for {row['case_id']}")

        for compat_id in BASE_COMPATIBILITY_IDS:
            expected = decision == "entitled"
            assert_true(
                projected.get(compat_id) == expected,
                f"compatibility projection mismatch for {compat_id} in {row['case_id']}",
            )

    feature_cases = fixture["feature_vector_cases"]
    for row in feature_cases:
        outcome = row["expected_outcome"]
        assert_true(outcome in KNOWN_FEATURE_OUTCOMES, f"unknown feature outcome {outcome}")

        expected = decide_feature_case(row)
        assert_true(
            expected == outcome,
            f"feature vector mismatch for {row['case_id']}: expected {expected}, got {outcome}",
        )

        if outcome == "granted" and row.get("limit_bucket") is None:
            assert_true(row.get("expected_reserved_units", 0) == 0, f"expected reserved units must be 0 for unlimited grants: {row['case_id']}")
        if outcome == "granted" and row.get("limit_bucket") is not None:
            assert_true(
                row.get("expected_reserved_units", row["requested_units"]) == row["requested_units"],
                f"expected reserved units must match request units: {row['case_id']}",
            )

    uiai_cases = fixture["uiai_child_token_cases"]
    assert_true(len(uiai_cases) >= 1, "missing uiai child token vector cases")
    for row in uiai_cases:
        assert_true(row["expected_outcome"] in KNOWN_UIAI_OUTCOMES, f"unknown uiai outcome {row['expected_outcome']}")

    unknown_family_cases = fixture["unknown_family_cases"]
    for row in unknown_family_cases:
        assert_true(row["expected_result"] == "unknown_family", f"unknown family result mismatch: {row['case_id']}")

    dormant_cases = fixture["dormant_dimension_cases"]
    for row in dormant_cases:
        assert_true(row["expected_result"] == "no_authority_effect", f"dormant case mismatch: {row['case_id']}")

    negative_mutations = fixture["negative_mutations"]
    ids = {row["id"] for row in negative_mutations}
    assert_true(ids == KNOWN_NEGATIVE_IDS, "negative scenario set changed")
    for row in negative_mutations:
        assert_true(row.get("expected_error") and len(row["expected_error"]) > 1, "negative scenario expected_error must be non-empty")

    print(
        json.dumps(
            {
                "schema": "focusa.spec152f.entitlement_policy_vectors_validation.v1",
                "policy_sha256": fixture.get("policy_canonical_sha256"),
                "state_count": fixture["state_count"],
                "family_count": fixture["family_count"],
                "grid_case_count": fixture["grid_case_count"],
                "negative_mutation_count": len(negative_mutations),
                "feature_vector_count": len(feature_cases),
                "uiai_vector_count": len(uiai_cases),
                "result": "passed",
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
