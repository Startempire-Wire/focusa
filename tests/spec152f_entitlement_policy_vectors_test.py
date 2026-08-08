#!/usr/bin/env python3
"""Cross-language parity guard for Spec152F entitlement policy vectors."""

from collections import defaultdict
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/spec152f-entitlement-policy-cases.v1.json"

EXPECTED_STATES = [
    "pending_unverified",
    "verified_no_grant",
    "evaluation",
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

NEGATIVE_SCENARIO_IDS = {
    "duplicate_family",
    "unknown_active_feature",
    "recovery_requires_feature",
    "dormant_absence_denies",
    "missing_state_family",
    "duplicate_feature_compatibility",
}

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


def assert_true(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    raw = json.loads(FIXTURE.read_text())

    assert_true(raw.get("schema") == "focusa.spec152f.entitlement_policy_cases.v1", "schema must remain Spec152F v1")
    assert_true(raw.get("policy_id") == "focusa-simple-entitlement", "policy_id mismatch")
    assert_true(raw.get("policy_version") == 1, "policy_version must be 1")
    assert_true(
        raw.get("feature_compatibility_count") == 15,
        f"feature_compatibility_count mismatch: {raw.get('feature_compatibility_count')}",
    )

    assert_true(raw.get("family_count") == len(EXPECTED_FAMILIES), "family_count mismatch")
    assert_true(raw.get("state_count") == len(EXPECTED_STATES), "state_count mismatch")

    grid_cases = raw.get("grid_cases", [])
    negative_mutations = raw.get("negative_mutations", [])

    assert_true(len(grid_cases) == raw.get("grid_case_count"), "grid_case_count must equal row count")
    assert_true(raw.get("grid_case_count") == 72, "expected 72 grid cases")

    seen_pairs = set()
    by_state = defaultdict(int)
    by_family = defaultdict(int)
    state_rows = defaultdict(dict)

    for row in grid_cases:
        case_id = row["case_id"]
        state = row["state"]
        family = row["family"]
        expected_decision = row["expected_decision"]

        assert_true(state in EXPECTED_STATES, f"unexpected state {state}")
        assert_true(family in EXPECTED_FAMILIES, f"unexpected family {family}")
        assert_true(expected_decision in KNOWN_DECISIONS, f"unexpected decision {expected_decision}")

        expected_case_id = f"{state}::{family}"
        assert_true(case_id == expected_case_id, f"case_id mismatch: {case_id} != {expected_case_id}")

        pair = (state, family)
        assert_true(pair not in seen_pairs, f"duplicate case: {case_id}")
        seen_pairs.add(pair)

        by_state[state] += 1
        by_family[family] += 1
        state_rows[state][family] = expected_decision

    expected_pairs = {(state, family) for state in EXPECTED_STATES for family in EXPECTED_FAMILIES}
    assert_true(seen_pairs == expected_pairs, "incomplete or duplicated grid coverage")

    for state in EXPECTED_STATES:
        assert_true(by_state[state] == len(EXPECTED_FAMILIES), f"state {state} missing family rows")
    for family in EXPECTED_FAMILIES:
        assert_true(by_family[family] == len(EXPECTED_STATES), f"family {family} missing state rows")

    assert_true(
        state_rows["evaluation"]["internal_maintenance"] == "inherit",
        "evaluation/internal_maintenance must inherit",
    )
    assert_true(
        state_rows["evaluation"]["account_recovery"] == "allow",
        "evaluation/account_recovery must allow",
    )
    assert_true(
        state_rows["evaluation"]["base_focusa"] == "require_base",
        "evaluation/base_focusa must require base",
    )
    assert_true(
        state_rows["evaluation"]["automation"] == "require_feature",
        "evaluation/automation must require feature",
    )
    assert_true(
        state_rows["evaluation"]["team_remote"] == "require_feature",
        "evaluation/team_remote must require feature",
    )

    for state in EXPECTED_STATES:
        if state == "evaluation":
            continue
        assert_true(
            state_rows[state]["internal_maintenance"] == "inherit",
            f"{state}/internal_maintenance must inherit",
        )

    ids = {row["id"] for row in negative_mutations}
    assert_true(ids == NEGATIVE_SCENARIO_IDS, "negative scenario set must remain canonical")

    for row in negative_mutations:
        assert_true(
            row.get("expected_error") and len(row["expected_error"]) > 1,
            "negative scenarios must have non-empty expected_error",
        )

    print(
        json.dumps(
            {
                "schema": "focusa.spec152f.entitlement_policy_vectors.v1",
                "policy_id": raw["policy_id"],
                "policy_version": raw["policy_version"],
                "state_count": raw["state_count"],
                "family_count": raw["family_count"],
                "grid_case_count": raw["grid_case_count"],
                "feature_compatibility_count": raw["feature_compatibility_count"],
                "negative_mutation_count": len(negative_mutations),
                "result": "passed_with_explicit_grid_parity",
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
