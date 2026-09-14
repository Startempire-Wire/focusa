#!/usr/bin/env python3
"""Validate the exhaustive Spec 152F policy registry and golden grid cases."""

import copy
import hashlib
import json
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / "docs/contracts/spec152f-entitlement-policy.v1.yaml"
FEATURE_PATH = ROOT / "docs/contracts/spec152-feature-registry.v1.yaml"
CASES_PATH = ROOT / "tests/fixtures/spec152f-entitlement-policy-cases.v1.json"

FAMILIES = {
    "account_recovery",
    "read_projection",
    "base_focusa",
    "automation",
    "team_remote",
    "release_proof",
    "premium_updates",
    "customer_data_export",
    "internal_maintenance",
}
PREMIUM = {"automation", "team_remote", "release_proof", "premium_updates"}
STATES = {
    "pending_unverified",
    "verified_no_license",
    "active_paid",
    "offline_grace",
    "expired",
    "refunded_or_revoked",
    "missing_or_corrupt",
}
CANONICAL_DECISIONS = {
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

POLICY_TO_CANONICAL_DECISION = {
    "allow": "allow",
    "allow_offline_only": "allow_offline_only",
    "allow_basic": "allow",
    "allow_manual_one_mutable_project": "allow_verified_limited",
    "allow_one_foreground_ephemeral_session": "allow",
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
    "deny_unless_required_for_registration_or_safety": "inherit",
    "inherit": "inherit",
    "inherit_only_allowed_initiating_operation": "inherit",
}
ACTIVATION_REQUIREMENTS = {
    "business_justification",
    "stable_family_boundary",
    "authority_ownership",
    "backward_compatibility",
    "recovery_analysis",
    "presenter_inheritance",
    "limit_semantics",
    "denial_ux",
    "acceptance_evidence",
    "operator_approval",
}


def canonical_sha256(value: dict) -> str:
    raw = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(raw).hexdigest()


def validate(policy: dict, feature_registry: dict) -> None:
    if policy.get("schema") != "focusa.spec152f.entitlement_policy.v1":
        raise ValueError("unsupported policy schema")
    if policy.get("policy_id") != "focusa-simple-entitlement" or policy.get("product") != "focusa":
        raise ValueError("unsupported policy identity")
    if policy.get("policy_version") != 1:
        raise ValueError("unsupported policy version")

    commercial = policy["commercial_model"]
    if commercial != {
        "base_gate_count": 1,
        "premium_family_count": 4,
        "independent_surface_paywalls_forbidden": True,
        "inventory_is_not_sku_catalog": True,
        "base_compatibility_features": [
            "focusa.core.evidence",
            "focusa.core.mission",
            "focusa.core.workpoint",
        ],
    }:
        raise ValueError("commercial model drift")

    families = policy.get("families", [])
    family_ids = [family["id"] for family in families]
    if len(family_ids) != len(set(family_ids)):
        raise ValueError("duplicate family id")
    if set(family_ids) != FAMILIES or len(family_ids) != 9:
        raise ValueError("family set is incomplete")
    if set(policy.get("premium_families", [])) != PREMIUM:
        raise ValueError("premium family set is incomplete")

    registered_features = {row["key"] for row in feature_registry.get("features", [])}
    compatibility = policy.get("feature_compatibility", [])
    compatibility_keys = [row["key"] for row in compatibility]
    if len(compatibility_keys) != len(set(compatibility_keys)):
        raise ValueError("duplicate feature compatibility key")
    if set(compatibility_keys) != registered_features:
        missing = sorted(registered_features - set(compatibility_keys))
        unknown = sorted(set(compatibility_keys) - registered_features)
        raise ValueError(f"feature compatibility mismatch missing={missing} unknown={unknown}")
    for row in compatibility:
        if row["family"] not in FAMILIES:
            raise ValueError("feature compatibility has unknown family")

    for family in families:
        active = family["active_feature_keys"]
        if len(active) != len(set(active)):
            raise ValueError("duplicate active feature")
        unknown = set(active) - registered_features
        if unknown:
            raise ValueError(f"unknown active feature: {sorted(unknown)}")
        if family["id"] == "account_recovery":
            if family["base_product_required"] or active or family["feature_resolution"] != "none":
                raise ValueError("recovery family requires commercial feature")
        if family["id"] in PREMIUM and (
            family["treatment"] != "optional_premium" or not family["base_product_required"] or not active
        ):
            raise ValueError("premium family is not base-first and feature-bound")

    state_rows = policy.get("state_grid", [])
    state_ids = [row["state"] for row in state_rows]
    if len(state_ids) != len(set(state_ids)) or set(state_ids) != STATES:
        raise ValueError("state set is incomplete or duplicated")
    for row in state_rows:
        policies = row.get("policies", {})
        if not FAMILIES.issubset(set(policies)):
            raise ValueError("state grid is not exhaustive")
        canonical_policies = {}
        for family, policy_decision in policies.items():
            canonical_decision = POLICY_TO_CANONICAL_DECISION.get(policy_decision)
            if canonical_decision is None:
                raise ValueError("state grid has unknown decision")
            canonical_policies[family] = canonical_decision
        if set(canonical_policies.values()) - CANONICAL_DECISIONS:
            raise ValueError("state grid has unknown canonical decision")
        if canonical_policies["account_recovery"] not in {"allow", "allow_offline_only"}:
            raise ValueError("recovery family denied")
        if canonical_policies["customer_data_export"] not in {"allow", "allow_existing_local_only"}:
            raise ValueError("basic customer data export denied")

    exceptions = policy["stable_update_and_export_exceptions"]
    for key in ("stable_security_update", "repair_and_rollback", "basic_customer_data_export"):
        if exceptions[key]["commercial_entitlement_required"]:
            raise ValueError(f"required allowance became commercial: {key}")
    if exceptions["premium_packaged_export"] != {
        "family": "customer_data_export",
        "commercial_entitlement_required": True,
        "required_feature": "focusa.export.packaged",
    }:
        raise ValueError("packaged export boundary drift")

    dimensions = policy.get("future_dimensions", [])
    dimension_ids = [row["id"] for row in dimensions]
    if len(dimension_ids) != len(set(dimension_ids)) or len(dimension_ids) != 10:
        raise ValueError("future dimension set is incomplete or duplicated")
    for row in dimensions:
        if row["commercial_activation"].startswith("dormant") and row["missing_claim_effect"] not in {
            "no_effect",
            "no_commercial_effect",
        }:
            raise ValueError("dormant dimension absence changes authorization")
    if set(policy.get("future_activation_requirements", [])) != ACTIVATION_REQUIREMENTS:
        raise ValueError("future activation requirements drift")

    invariants = set(policy.get("invariants", []))
    for required in {
        "recovery_family_never_requires_commercial_feature",
        "basic_customer_data_export_never_requires_commercial_feature",
        "stable_security_update_repair_rollback_never_require_commercial_feature",
        "premium_requires_base_first",
        "offline_grace_never_expands_features_or_limits",
        "dormant_dimension_absence_never_denies_capability",
        "unknown_side_effect_fails_closed_before_execution",
        "presenters_and_facades_never_own_commercial_truth",
    }:
        if required not in invariants:
            raise ValueError(f"missing invariant: {required}")


policy = yaml.safe_load(POLICY_PATH.read_text(encoding="utf-8"))
features = yaml.safe_load(FEATURE_PATH.read_text(encoding="utf-8"))
cases = json.loads(CASES_PATH.read_text(encoding="utf-8"))
validate(policy, features)

assert len(POLICY_PATH.read_text().splitlines()) < 500
assert cases["schema"] == "focusa.spec152f.entitlement_policy_cases.v1"
assert cases["policy_id"] == policy["policy_id"]
assert cases["policy_version"] == policy["policy_version"]
assert cases["policy_canonical_sha256"] == canonical_sha256(policy)
assert cases["family_count"] == 9
assert cases["state_count"] == 7
assert cases["grid_case_count"] == 63
assert cases["feature_compatibility_count"] == len(features["features"]) == 15

expected_cases = {
    (row["state"], family): POLICY_TO_CANONICAL_DECISION[decision]
    for row in policy["state_grid"]
    if row["state"] in STATES
    for family, decision in row["policies"].items()
    if family in FAMILIES
}
actual_cases = {}
for case in cases["grid_cases"]:
    key = (case["state"], case["family"])
    assert key not in actual_cases, f"duplicate grid case: {key}"
    assert case["case_id"] == f"{key[0]}::{key[1]}"
    actual_cases[key] = case["expected_decision"]
assert actual_cases == expected_cases

mutations = []
candidate = copy.deepcopy(policy)
candidate["families"].append(copy.deepcopy(candidate["families"][0]))
mutations.append(("duplicate_family", candidate))
candidate = copy.deepcopy(policy)
next(row for row in candidate["families"] if row["id"] == "automation")["active_feature_keys"].append(
    "focusa.unknown.active"
)
mutations.append(("unknown_active_feature", candidate))
candidate = copy.deepcopy(policy)
recovery = next(row for row in candidate["families"] if row["id"] == "account_recovery")
recovery["active_feature_keys"] = ["focusa.repair.execute"]
recovery["feature_resolution"] = "operation_bound"
mutations.append(("recovery_requires_feature", candidate))
candidate = copy.deepcopy(policy)
next(row for row in candidate["future_dimensions"] if row["id"] == "operation")["missing_claim_effect"] = "deny"
mutations.append(("dormant_absence_denies", candidate))
candidate = copy.deepcopy(policy)
candidate["state_grid"][0]["policies"].pop("base_focusa")
mutations.append(("missing_state_family", candidate))
candidate = copy.deepcopy(policy)
candidate["feature_compatibility"].append(copy.deepcopy(candidate["feature_compatibility"][0]))
mutations.append(("duplicate_feature_compatibility", candidate))

assert {row["id"] for row in cases["negative_mutations"]} == {name for name, _ in mutations}
for name, candidate in mutations:
    try:
        validate(candidate, features)
    except ValueError:
        pass
    else:
        raise AssertionError(f"validator accepted forbidden mutation: {name}")

feature_text = FEATURE_PATH.read_text(encoding="utf-8")
assert "spec152f-entitlement-policy.v1.yaml" in feature_text
assert "does not make core compatibility claims separate purchases" in feature_text

print(json.dumps({
    "schema": "focusa.spec152f.entitlement_policy_validation.v1",
    "policy_sha256": canonical_sha256(policy),
    "families": len(FAMILIES),
    "premium_families": len(PREMIUM),
    "states": len(STATES),
    "grid_cases": len(actual_cases),
    "feature_compatibility": len(features["features"]),
    "negative_mutations": len(mutations),
    "result": "passed",
}, sort_keys=True))
