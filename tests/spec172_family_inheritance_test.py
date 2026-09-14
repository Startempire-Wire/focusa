#!/usr/bin/env python3
"""Validate Spec 172 verified-no-license family inheritance defaults.

The runtime posture for verified-no-license must be explicit and fail-closed:
- focusa and uiai inheritables are strict allowlists
- blocked families override any implicit default
- unknown families and unknown products are denied
- default behaviors for materially new family surfaces are explicit and
  deny-by-default.
"""

from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import yaml

ROOT = pathlib.Path(__file__).resolve().parents[1]
REGISTRY_PATH = ROOT / "docs/contracts/spec135/generated-contract-v1/operation-registry.json"
LIMITED_PATH = ROOT / "docs/contracts/spec172-verified-limited-access.v1.yaml"
LIFECYCLE_PATH = ROOT / "docs/contracts/spec172-license-type-lifecycle.v1.yaml"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def test_contract_focusa_and_uiai_allowlists(contract: dict) -> None:
    focusa = contract.get("focusa", {})
    uiai = contract.get("uiai_engine", {})
    require(contract.get("schema") == "focusa.spec172.verified_limited_access.v1", "unexpected limited-access schema")
    require(contract.get("postures", {}).get("verified_no_license", {}).get("is_license_type") is False, "verified_no_license must not be a license type")
    focusa_allowed = focusa.get("allowed_families", [])
    focusa_blocked = set(focusa.get("blocked_families", []))
    uiai_allowed = uiai.get("allowed_families", [])
    uiai_blocked = set(uiai.get("blocked_families", []))
    require(set(focusa_allowed) == {
        "manual_project",
        "manual_mission",
        "manual_focus_state",
        "manual_workpoint",
        "manual_trajectory",
        "manual_basic_evidence",
    }, "focusa allowlist must be exact")
    require(focusa_blocked == {
        "automation",
        "team_remote",
        "release_proof",
        "premium_updates",
    }, "focusa blocked families must be exact")
    require(set(uiai_allowed) == {
        "public_search",
        "source_to_markdown",
        "public_page_read",
        "accessibility_snapshot",
        "screenshot",
        "basic_diagnostics",
    }, "uiai allowlist must be exact")
    require(uiai_blocked == {
        "browser_action",
        "browser_persistence",
        "authenticated_private_targets",
        "unattended_browser_automation",
        "scheduled_batch_qa",
        "premium_hosted_resources",
    }, "uiai blocked families must be exact")

    defaults = contract.get("defaults", {})
    require(defaults.get("unknown_family") == "deny", "unknown family must deny by default")
    require(defaults.get("new_family") == "deny_pending_explicit_assignment", "new family must be denied pending explicit assignment")
    require(defaults.get("unknown_product") == "deny", "unknown product must deny by default")


def test_lifecycle_inheritance_contract(contract: dict) -> None:
    rules = contract.get("inheritance_rules", {})
    existing_rule = rules.get("existing_family_implementation", {})
    required_conditions = {
        "same_registered_product",
        "same_customer_understandable_outcome",
        "security_side_effect_privacy_and_resource_profile_fits_family",
        "no_separately_named_product",
        "no_materially_new_hosted_cost",
    }
    require(existing_rule.get("decision") == "inherit", "existing-family implementation must inherit")
    require(set(existing_rule.get("all_conditions", [])) == required_conditions, "existing-family inheritance must require all five conditions")
    new_family = rules.get("materially_new_family", {})
    require(new_family.get("decision") == "excluded_pending_explicit_versioned_assignment", "new family must be explicitly assigned")
    require(new_family.get("defaults", {}).get("verified_no_license") == "denied", "verified_no-license materially-new families must deny")
    require(new_family.get("defaults", {}).get("existing_license_types") == "excluded", "existing license types materially-new families must exclude")
    require(new_family.get("defaults", {}).get("unknown_or_unclassified_execution") == "denied", "unknown execution materially-new families must deny")
    future_type = rules.get("future_license_type", {})
    require(future_type.get("decision") == "separate_type_explicit_grant_required", "future license type must be explicit")
    require(future_type.get("may_mutate_existing_operator") is False, "future types cannot mutate existing operator")
    future_product = rules.get("future_product", {})
    require(future_product.get("decision") == "excluded_pending_operator_approved_registration", "future product must be excluded by default")
    require(future_product.get("namespace_or_marketing_resemblance_grants_access") is False, "future products cannot mimic operator access")
    hosted = rules.get("hosted_or_metered_resource", {})
    require(hosted.get("decision") == "excluded_unless_explicitly_listed", "hosted resources require explicit listing")
    require(hosted.get("lifetime_term_implies_inclusion") is False, "hosted resources cannot auto-include by lifetime")

    authority = contract.get("commercial_authority", {})
    for key in (
        "anonymous_product_capability",
        "local_or_self_issued_grant",
        "presenter_owned_policy",
        "legacy_download_453_implicit_mapping",
    ):
        require(authority.get(key) == "forbidden", f"{key} must be forbidden")


def test_operation_registry_metadata(registry: dict, focusa_families: set[str]) -> None:
    require(registry.get("operation_policy_schema") == "focusa.operation_policy_metadata.v1", "operation_policy schema must be stable")
    operations = registry.get("operations", [])
    require(len(operations) == registry["operation_count"], f"unexpected operation count: {len(operations)}")

    covered = set()
    for operation in operations:
        require("spec172_family" in operation, f"missing spec172_family: {operation.get('operation_id')}")
        classification = operation.get("spec172_family")
        require(
            classification is None or classification in focusa_families,
            f"unknown spec172 family in {operation.get('operation_id')}",
        )
        if isinstance(classification, str):
            covered.add(classification)

    require(focusa_families <= covered, "every allowlisted focusa family must map to an operation")


def main() -> int:
    with LIMITED_PATH.open(encoding="utf-8") as handle:
        limited = yaml.safe_load(handle)
    with LIFECYCLE_PATH.open(encoding="utf-8") as handle:
        lifecycle = yaml.safe_load(handle)
    with REGISTRY_PATH.open(encoding="utf-8") as handle:
        registry = json.load(handle)

    test_contract_focusa_and_uiai_allowlists(limited)
    test_lifecycle_inheritance_contract(lifecycle)
    test_operation_registry_metadata(
        registry,
        set(limited.get("focusa", {}).get("allowed_families", [])),
    )

    check = subprocess.run(
        ["python3", "scripts/generate-spec135-operation-contracts.py", "--check"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    require(check.returncode == 0, check.stdout + check.stderr)

    print("Spec172 family inheritance gate passed")
    print(f"focusa_allowed={len(limited.get('focusa', {}).get('allowed_families', []))}")
    print(f"uiai_allowed={len(limited.get('uiai_engine', {}).get('allowed_families', []))}")
    print(f"operation_count={registry.get('operation_count')}")
    print(f"spec172_family_coverage={len(registry.get('operations', []))}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
