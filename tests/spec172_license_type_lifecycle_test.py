#!/usr/bin/env python3
"""Contract gate for Spec 172 License Type lifecycle and future defaults."""

from __future__ import annotations

import pathlib
import sys

import yaml

ROOT = pathlib.Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/contracts/spec172-license-type-lifecycle.v1.yaml"


def main() -> int:
    failures: list[str] = []
    with CONTRACT.open(encoding="utf-8") as handle:
        contract = yaml.safe_load(handle)
    if not isinstance(contract, dict):
        raise AssertionError("lifecycle contract must be a mapping")

    if contract.get("schema") != "focusa.spec172.license_type_lifecycle.v1":
        failures.append("schema must be versioned as focusa.spec172.license_type_lifecycle.v1")
    if contract.get("contract_version") != 1:
        failures.append("contract_version must be 1")

    machine = contract.get("state_machine", {})
    states = machine.get("states", {})
    pre = states.get("pre_first_sale_mutable", {})
    frozen = states.get("post_first_sale_immutable", {})
    discontinued = states.get("discontinued_no_new_sales", {})
    freeze_fields = {
        "stable_license_type_code", "product_owner", "included_capability_families",
        "seat_limit", "node_limit", "local_runtime_rights", "hosted_resource_rights",
        "duration", "refund_posture", "manifest_digest",
    }
    if machine.get("initial_state") != "pre_first_sale_mutable":
        failures.append("initial state must be pre_first_sale_mutable")
    if set(pre.get("mutable_fields", [])) != freeze_fields:
        failures.append("pre-first-sale mutable fields do not equal the complete freeze boundary")
    if pre.get("transitions", {}).get("first_approved_sale") != "post_first_sale_immutable":
        failures.append("first approved sale must transition to immutable state")
    if set(frozen.get("immutable_fields", [])) != freeze_fields:
        failures.append("post-first-sale immutable fields do not equal the complete freeze boundary")
    if frozen.get("permitted_changes") != ["sale_status"]:
        failures.append("only sale_status may change after first sale")
    if frozen.get("transitions", {}).get("discontinue_new_sales") != "discontinued_no_new_sales":
        failures.append("discontinuation transition is missing")

    existing = discontinued.get("existing_grants", {})
    if discontinued.get("accepts_new_sales") is not False:
        failures.append("discontinued License Type must reject new sales")
    if existing.get("remain_valid") is not True:
        failures.append("discontinuation must preserve existing grants")
    for forbidden_mutation in (
        "renamed", "revoked", "downgraded", "expired_by_discontinuation",
        "converted_to_other_license_type",
    ):
        if existing.get(forbidden_mutation) is not False:
            failures.append(f"discontinuation must not mark existing grants {forbidden_mutation}")

    guards = contract.get("transition_guards", {})
    if guards.get("first_approved_sale", {}).get("caller_controlled_commercial_fields") != "forbidden":
        failures.append("caller-controlled freeze policy must be forbidden")
    discontinue_guard = guards.get("discontinue_new_sales", {})
    if discontinue_guard.get("only_mutates") != "sale_status" or discontinue_guard.get("revokes_existing_grants") is not False:
        failures.append("discontinuation guard must mutate only sale status without revocation")

    rules = contract.get("inheritance_rules", {})
    existing_rule = rules.get("existing_family_implementation", {})
    required_conditions = {
        "same_registered_product", "same_customer_understandable_outcome",
        "security_side_effect_privacy_and_resource_profile_fits_family",
        "no_separately_named_product", "no_materially_new_hosted_cost",
    }
    if existing_rule.get("decision") != "inherit" or set(existing_rule.get("all_conditions", [])) != required_conditions:
        failures.append("safe existing-family implementation inheritance is incomplete")
    new_family = rules.get("materially_new_family", {})
    if new_family.get("decision") != "excluded_pending_explicit_versioned_assignment":
        failures.append("materially new families must default excluded")
    if new_family.get("defaults") != {
        "verified_no_license": "denied",
        "existing_license_types": "excluded",
        "unknown_or_unclassified_execution": "denied",
    }:
        failures.append("new-family fail-closed defaults are incomplete")
    future_type = rules.get("future_license_type", {})
    if future_type.get("decision") != "separate_type_explicit_grant_required" or future_type.get("may_mutate_existing_operator") is not False:
        failures.append("future Navigator must be a separate explicit grant and cannot mutate Operator")
    future_product = rules.get("future_product", {})
    if future_product.get("decision") != "excluded_pending_operator_approved_registration" or future_product.get("namespace_or_marketing_resemblance_grants_access") is not False:
        failures.append("future products must default excluded")
    hosted = rules.get("hosted_or_metered_resource", {})
    if hosted.get("decision") != "excluded_unless_explicitly_listed" or hosted.get("lifetime_term_implies_inclusion") is not False:
        failures.append("hosted and metered resources must be explicitly listed")

    authority = contract.get("commercial_authority", {})
    for key in ("anonymous_product_capability", "local_or_self_issued_grant", "presenter_owned_policy", "legacy_download_453_implicit_mapping"):
        if authority.get(key) != "forbidden":
            failures.append(f"commercial_authority.{key} must be forbidden")
    required_caller_fields = {"product", "price", "license_type", "capability_family", "feature", "limit", "node", "commercial_right"}
    if set(authority.get("caller_controlled_fields", [])) != required_caller_fields:
        failures.append("caller-controlled commercial field prohibition is incomplete")

    preservation = contract.get("preservation", {})
    required_safety = {"basic_customer_data_export", "repair", "rollback", "stable_security_update", "uninstall"}
    if set(preservation.get("always_available", [])) != required_safety:
        failures.append("required export/recovery operations are not all preserved")
    if preservation.get("discontinuation_deletes_customer_data") is not False:
        failures.append("discontinuation must not delete customer data")

    fixtures = {item.get("id"): item for item in contract.get("fixtures", [])}
    required_fixtures = {
        "operator_first_sale_freeze", "operator_discontinued", "navigator_is_separate",
        "safe_existing_family_operation", "materially_new_family", "future_product", "metered_resource",
    }
    if set(fixtures) != required_fixtures:
        failures.append("lifecycle fixtures must cover every required scenario exactly")
    navigator = fixtures.get("navigator_is_separate", {}).get("expected", {})
    if navigator.get("operator_customer_access") != "denied_without_explicit_upgrade_crossgrade_or_grant":
        failures.append("Navigator fixture permits implicit Operator conversion")
    if fixtures.get("future_product", {}).get("expected") != "excluded_pending_operator_approved_registration":
        failures.append("future-product fixture does not prove default exclusion")
    if fixtures.get("metered_resource", {}).get("expected") != "excluded_unless_explicitly_listed":
        failures.append("metered-resource fixture does not prove default exclusion")

    if failures:
        print("Spec 172 License Type lifecycle test FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("Spec 172 License Type lifecycle test passed")
    print("states=3 fixtures=7 freeze_fields=10")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
