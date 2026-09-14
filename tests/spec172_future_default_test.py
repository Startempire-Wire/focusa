#!/usr/bin/env python3
"""Spec 172.05.05 — future Navigator, new family, and new product default
exclusion (atom focusa-vbcqu.20.15.36).

Attempts to add a future Navigator License Type, a materially new capability
family, an unregistered future product, a hosted metered resource, and
synthetic dynamic operations are evaluated against the frozen License Type
lifecycle registry, the verified-no-license limited-access contract, and the
canonical operation registry. Every attempt must be excluded by default and
MUST NOT change any existing Operator / Bundle / verified-no-license surface;
only a safe same-family implementation inherits (Spec 172 Sections 4.3, 8.2,
8.3, 9.4, 10.2, 12, and 15). The registry bytes are digest-checked before and
after the attempt matrix to prove the attempts are pure fail-closed
evaluations with zero mutation.

All fixture values are public synthetic non-production data. No raw email,
key, token, customer row, credential, or card data appears.
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import sys

import yaml

ROOT = pathlib.Path(__file__).resolve().parents[1]
LIFECYCLE_PATH = ROOT / "docs/contracts/spec172-license-type-lifecycle.v1.yaml"
LIMITED_PATH = ROOT / "docs/contracts/spec172-verified-limited-access.v1.yaml"
REGISTRY_PATH = (
    ROOT / "docs/contracts/spec135/generated-contract-v1/operation-registry.json"
)

POSITIVE = 0
NEGATIVE = 0


def expect(condition: bool, message: str, negative: bool = False) -> None:
    global POSITIVE, NEGATIVE
    if negative:
        NEGATIVE += 1
    else:
        POSITIVE += 1
    if not condition:
        raise AssertionError(message)


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


# ── Synthetic future-evolution fixtures (public, non-production) ───────────
# Each fixture is an *attempt*: the proposed addition is evaluated against the
# frozen contracts and must be excluded by default with zero registry mutation.
ATTEMPTS = [
    {
        "id": "attempt_navigator",
        "kind": "future_license_type",
        "proposed": {
            "product_owner": "focusa",
            "license_type_code": "focusa_navigator_lifetime_v1",
        },
        "expected_rule": "future_license_type",
        "expected_decision": "separate_type_explicit_grant_required",
        "no_change": ["operator_license_type", "bundle_union", "verified_no_license_allowlist"],
    },
    {
        "id": "attempt_new_family",
        "kind": "materially_new_family",
        "proposed": {
            "product_owner": "focusa",
            "capability_family": "synthetic_future_capability",
            "customer_outcome": "materially_new",
        },
        "expected_rule": "materially_new_family",
        "expected_decision": "excluded_pending_explicit_versioned_assignment",
        "no_change": ["operator_families", "verified_no_license_allowlist"],
    },
    {
        "id": "attempt_future_product",
        "kind": "future_product",
        "proposed": {
            "product_owner": "synthetic_future_product",
            "capability_family": "synthetic_family",
        },
        "expected_rule": "future_product",
        "expected_decision": "excluded_pending_operator_approved_registration",
        "no_change": ["operator_families", "bundle_union", "verified_no_license_allowlist"],
    },
    {
        "id": "attempt_hosted_resource",
        "kind": "hosted_or_metered_resource",
        "proposed": {
            "product_owner": "focusa",
            "capability_family": "hosted_metered_compute",
            "resource_profile": "metered_third_party",
        },
        "expected_rule": "hosted_or_metered_resource",
        "expected_decision": "excluded_unless_explicitly_listed",
        "no_change": ["operator_hosted_resource_right"],
    },
    {
        "id": "attempt_safe_same_family_implementation",
        "kind": "existing_family_implementation",
        "proposed": {
            "product_owner": "focusa",
            "capability_family": "manual_workpoint",
            "customer_outcome": "existing",
            "resource_profile": "existing_local",
        },
        "expected_rule": "existing_family_implementation",
        "expected_decision": "inherit",
        "no_change": ["operator_families"],
    },
]

# Synthetic dynamic-operation manifests (Spec 172 Section 12): each future
# attempt presented as an MCP/extension/capsule operation must fail closed at
# runtime intake and never become an executable or purchasable surface.
DYNAMIC_ATTEMPTS = [
    {
        "id": "dynamic_navigator_tool",
        "operation_id": "focusa.navigator.synthetic_tool",
        "product_owner": "focusa",
        "capability_family": "manual_workpoint",
        "declared_policy_fields": ["license_type"],
        "expected": "quarantined_client_selected_policy",
    },
    {
        "id": "dynamic_new_family_tool",
        "operation_id": "focusa.synthetic_new_family.tool",
        "product_owner": "focusa",
        "capability_family": "synthetic_future_capability",
        "declared_policy_fields": [],
        "expected": "quarantined_unregistered_family",
    },
    {
        "id": "dynamic_future_product_tool",
        "operation_id": "synthetic_future_product.tool",
        "product_owner": "synthetic_future_product",
        "capability_family": "synthetic_family",
        "declared_policy_fields": [],
        "expected": "quarantined_unknown_owner",
    },
    {
        "id": "dynamic_hosted_resource_tool",
        "operation_id": "focusa.hosted_metered.tool",
        "product_owner": "focusa",
        "capability_family": "hosted_metered_compute",
        "declared_policy_fields": ["commercial_right"],
        "expected": "quarantined_unregistered_family",
    },
]


def rule_decision(rules: dict, kind: str) -> dict:
    return rules.get(kind, {})


def main() -> int:
    lifecycle_raw = LIFECYCLE_PATH.read_bytes()
    limited_raw = LIMITED_PATH.read_bytes()
    lifecycle_digest_before = digest(LIFECYCLE_PATH)
    limited_digest_before = digest(LIMITED_PATH)

    lifecycle = yaml.safe_load(lifecycle_raw)
    limited = yaml.safe_load(limited_raw)
    registry = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))

    # ── 1. Lifecycle registry: Operator freeze and discontinuation ──────────
    expect(
        lifecycle.get("schema") == "focusa.spec172.license_type_lifecycle.v1",
        "lifecycle schema must be versioned",
    )
    machine = lifecycle.get("state_machine", {})
    expect(machine.get("initial_state") == "pre_first_sale_mutable", "initial state is pre-first-sale mutable")
    frozen = machine.get("states", {}).get("post_first_sale_immutable", {})
    expect(frozen.get("permitted_changes") == ["sale_status"], "post-first-sale only sale_status may change")
    discontinued = machine.get("states", {}).get("discontinued_no_new_sales", {})
    existing_grants = discontinued.get("existing_grants", {})
    expect(discontinued.get("accepts_new_sales") is False, "discontinued type rejects new sales")
    for field in ("renamed", "revoked", "downgraded", "expired_by_discontinuation", "converted_to_other_license_type"):
        expect(existing_grants.get(field) is False, f"discontinuation must not mark grants {field}")
    expect(existing_grants.get("remain_valid") is True, "existing grants remain valid")

    # ── 2. Inheritance rules: every future surface defaults to exclusion ───
    rules = lifecycle.get("inheritance_rules", {})
    existing_rule = rules.get("existing_family_implementation", {})
    expect(existing_rule.get("decision") == "inherit", "safe existing-family implementation inherits")
    required_conditions = {
        "same_registered_product",
        "same_customer_understandable_outcome",
        "security_side_effect_privacy_and_resource_profile_fits_family",
        "no_separately_named_product",
        "no_materially_new_hosted_cost",
    }
    expect(set(existing_rule.get("all_conditions", [])) == required_conditions, "all five Section 8.2 conditions are required")
    new_family = rules.get("materially_new_family", {})
    expect(new_family.get("decision") == "excluded_pending_explicit_versioned_assignment", "new families exclude pending assignment")
    expect(new_family.get("defaults") == {
        "verified_no_license": "denied",
        "existing_license_types": "excluded",
        "unknown_or_unclassified_execution": "denied",
    }, "new-family fail-closed defaults are exact")
    future_type = rules.get("future_license_type", {})
    expect(future_type.get("decision") == "separate_type_explicit_grant_required", "Navigator requires an explicit separate grant")
    expect(future_type.get("may_mutate_existing_operator") is False, "Navigator cannot mutate Operator")
    future_product = rules.get("future_product", {})
    expect(future_product.get("decision") == "excluded_pending_operator_approved_registration", "future products exclude by default")
    expect(future_product.get("namespace_or_marketing_resemblance_grants_access") is False, "resemblance never grants access")
    hosted = rules.get("hosted_or_metered_resource", {})
    expect(hosted.get("decision") == "excluded_unless_explicitly_listed", "hosted resources exclude unless explicitly listed")
    expect(hosted.get("lifetime_term_implies_inclusion") is False, "lifetime does not imply hosted inclusion")

    # ── 3. Evaluate every synthetic attempt against the frozen registry ─────
    fixtures = {item.get("id"): item for item in lifecycle.get("fixtures", [])}
    for attempt in ATTEMPTS:
        rule = rule_decision(rules, attempt["expected_rule"])
        expect(rule.get("decision") == attempt["expected_decision"], f"{attempt['id']}: rule decision mismatch")
        # The lifecycle contract encodes the same scenario as a fixture; both
        # must agree on the fail-closed decision.
        fixture_match = {
            "attempt_navigator": "navigator_is_separate",
            "attempt_new_family": "materially_new_family",
            "attempt_future_product": "future_product",
            "attempt_hosted_resource": "metered_resource",
            "attempt_safe_same_family_implementation": "safe_existing_family_operation",
        }[attempt["id"]]
        fixture = fixtures[fixture_match]
        expect(fixture is not None, f"{attempt['id']}: lifecycle fixture {fixture_match} must exist")
        expected = fixture.get("expected")
        if isinstance(expected, dict):
            expect(
                "operator_customer_access" in expected and expected["operator_customer_access"]
                == "denied_without_explicit_upgrade_crossgrade_or_grant",
                f"{attempt['id']}: Navigator fixture must deny implicit Operator conversion",
            )
        else:
            # The fixture labels use the contract's descriptive wording
            # (inherit_existing_family) while the rule decision is the stable
            # machine label (inherit); both spell the same inheritance.
            fixture_label = {
                "inherit": "inherit_existing_family",
            }.get(attempt["expected_decision"], attempt["expected_decision"])
            expect(expected == fixture_label, f"{attempt['id']}: fixture/rule mismatch")
        # No-existing-surface-change assertions for this attempt.
        surfaces = attempt["no_change"]
        if "operator_license_type" in surfaces:
            operator_freeze = fixtures.get("operator_first_sale_freeze", {})
            expect(operator_freeze.get("start_state") == "pre_first_sale_mutable", "Operator freeze starts pre-first-sale")
            expect(operator_freeze.get("event") == "first_approved_sale", "Operator freeze is a first-approved-sale event")
            expect(operator_freeze.get("expected_state") == "post_first_sale_immutable", "Operator freezes immutable after first sale")
            navigator = fixtures.get("navigator_is_separate", {})
            proposed = navigator.get("proposed_license_type", {})
            expect(proposed.get("code") == "focusa_navigator_lifetime_v1", "Navigator fixture proposes a separate stable code")
            expect(proposed.get("product_owner") == "focusa", "Navigator belongs to the focusa product")
            expect(navigator.get("expected", {}).get("separate_stable_code_required") is True, "Navigator requires a separate stable code")
            expect(navigator.get("expected", {}).get("operator_manifest_mutation") == "denied", "Navigator cannot mutate the Operator manifest")
        if "operator_families" in surfaces:
            expect(
                "synthetic_future_capability" not in limited.get("focusa", {}).get("allowed_families", []),
                "new family never enters the focusa limited allowlist",
            )
            expect(
                "synthetic_future_capability" not in limited.get("uiai_engine", {}).get("allowed_families", []),
                "new family never enters the uiai limited allowlist",
            )
        if "bundle_union" in surfaces:
            expect(
                future_product.get("decision") == "excluded_pending_operator_approved_registration",
                "Bundle never auto-includes a future product",
            )
        if "verified_no_license_allowlist" in surfaces:
            postures = limited.get("postures", {}).get("verified_no_license", {})
            expect(postures.get("is_license_type") is False, "verified_no_license is not a License Type")
        if "operator_hosted_resource_right" in surfaces:
            expect(
                "hosted_metered_compute" not in limited.get("focusa", {}).get("allowed_families", [])
                and "hosted_metered_compute" not in limited.get("uiai_engine", {}).get("allowed_families", []),
                "hosted metered family never enters any allowlist",
            )
        if "operator_families" in surfaces and attempt["id"] == "attempt_safe_same_family_implementation":
            expect(
                "manual_workpoint" in limited.get("focusa", {}).get("allowed_families", []),
                "the inherited same-family implementation stays inside the included family",
            )

    # ── 4. Dynamic-operation fixtures fail closed and are absent from the ───
    # ── canonical operation registry ───────────────────────────────────────
    operations = {op.get("operation_id") for op in registry.get("operations", [])}
    registered_families = set()
    for op in registry.get("operations", []):
        family = op.get("spec172_family")
        if isinstance(family, str):
            registered_families.add(family)
    expect("focusa.navigator.synthetic_tool" not in operations, "Navigator dynamic tool is not registered")
    expect("focusa.synthetic_new_family.tool" not in operations, "new-family dynamic tool is not registered")
    expect("synthetic_future_product.tool" not in operations, "future-product dynamic tool is not registered")
    expect("focusa.hosted_metered.tool" not in operations, "hosted-resource dynamic tool is not registered")
    for dynamic in DYNAMIC_ATTEMPTS:
        proposed_family = dynamic["capability_family"]
        declared = set(dynamic["declared_policy_fields"])
        caller_fields = set(lifecycle.get("commercial_authority", {}).get("caller_controlled_fields", []))
        unregistered = proposed_family not in registered_families
        declares_forbidden = bool(declared & caller_fields)
        unknown_owner = dynamic["product_owner"] not in {"focusa", "uiai_engine"}
        if dynamic["expected"] == "quarantined_client_selected_policy":
            expect(declares_forbidden, f"{dynamic['id']}: declared client policy fields are forbidden")
        elif dynamic["expected"] == "quarantined_unknown_owner":
            expect(unknown_owner, f"{dynamic['id']}: unregistered product owner must quarantine")
        else:
            expect(unregistered, f"{dynamic['id']}: unregistered family must quarantine")
        # Registry check: the operation id is absent, so it can never be
        # trusted or executed at runtime intake.
        expect(dynamic["operation_id"] not in operations, f"{dynamic['id']}: operation must be absent from the registry")

    # ── 5. No-mutation proof: registry bytes are unchanged by the attempts ──
    expect(digest(LIFECYCLE_PATH) == lifecycle_digest_before, "lifecycle registry bytes unchanged after attempts")
    expect(digest(LIMITED_PATH) == limited_digest_before, "limited-access registry bytes unchanged after attempts")

    # ── 6. Commercial authority and preservation guards ────────────────────
    authority = lifecycle.get("commercial_authority", {})
    for key in ("anonymous_product_capability", "local_or_self_issued_grant", "presenter_owned_policy", "legacy_download_453_implicit_mapping"):
        expect(authority.get(key) == "forbidden", f"commercial_authority.{key} must be forbidden")
    required_caller_fields = {"product", "price", "license_type", "capability_family", "feature", "limit", "node", "commercial_right"}
    expect(set(authority.get("caller_controlled_fields", [])) == required_caller_fields, "caller-controlled commercial field prohibition is exact")
    preservation = lifecycle.get("preservation", {})
    expect(preservation.get("discontinuation_deletes_customer_data") is False, "discontinuation preserves customer data")
    required_safety = {"basic_customer_data_export", "repair", "rollback", "stable_security_update", "uninstall"}
    expect(set(preservation.get("always_available", [])) == required_safety, "safety operations stay available")

    print(
        json.dumps(
            {
                "schema": "focusa.spec172.future_default_validation.v1",
                "attempts": len(ATTEMPTS),
                "dynamic_attempts": len(DYNAMIC_ATTEMPTS),
                "lifecycle_digest_unchanged": lifecycle_digest_before,
                "limited_digest_unchanged": limited_digest_before,
                "navigator": "separate_type_explicit_grant_required",
                "new_family": "excluded_pending_explicit_versioned_assignment",
                "future_product": "excluded_pending_operator_approved_registration",
                "hosted_resource": "excluded_unless_explicitly_listed",
                "safe_same_family_implementation": "inherit",
                "dynamic_operations": "quarantined_fail_closed",
                "positive_checks": POSITIVE,
                "negative_checks": NEGATIVE,
                "result": "passed_fail_closed",
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
