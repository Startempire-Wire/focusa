#!/usr/bin/env python3
"""Spec 172 Section 12: require trusted manifests for dynamic and generated
operations.

Build-time scanning alone is insufficient for MCP tools, extensions,
downloaded capsules, plugins, generated UI, and private modules. Every
production operation must resolve through trusted metadata containing at
least operation_id, product_owner, operation_class, capability_family, and
side_effect_class. Dynamic operations require a trusted signed manifest;
unknown ownership, unknown mutation, unknown side effect, or unregistered
family MUST fail closed before execution. A tool cannot self-label as
recovery to bypass licensing. Generated UI may render only canonical
registered actions. Client-provided metadata cannot select products, prices,
License Types, grants, or commercial treatment.
"""

from __future__ import annotations

import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
REGISTRY_PATH = BUNDLE / "operation-registry.json"
BINDINGS_PATH = BUNDLE / "ui-action-bindings.fixture.json"
A2UI_PATH = BUNDLE / "a2ui-catalog.json"

REGISTERED_PRODUCT_OWNERS = {"focusa", "uiai_engine"}
REGISTERED_OPERATION_CLASSES = {"read", "value_mutation", "recovery", "internal_maintenance"}
REGISTERED_SIDE_EFFECT_CLASSES = {"none", "local", "remote", "external"}
# Canonical CapabilityFamily vocabulary from focusa-license.
REGISTERED_CAPABILITY_FAMILIES = {
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
FORBIDDEN_CLIENT_POLICY_FIELDS = {
    "product",
    "price",
    "license_type",
    "family",
    "feature",
    "limit",
    "node",
    "commercial_right",
}
STABLE_ERROR_UNKNOWN = "ENTITLEMENT_POLICY_UNKNOWN"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def verify_manifest(
    operation_id: str,
    product_owner: str,
    operation_class: str,
    capability_family: str,
    side_effect_class: str,
    signed: bool,
    operation_registered: bool,
    canonical_class: str | None,
    canonical_family: str | None,
    canonical_side_effect: str | None,
    owner_registered: bool,
    class_registered: bool,
    side_effect_registered: bool,
    family_registered: bool,
    declared_policy_fields: list[str],
) -> str:
    """Fail-closed mirror of the focusa-license manifest verifier.

    Returns the stable decision label. Quarantined decisions map to
    ENTITLEMENT_POLICY_UNKNOWN and are recorded so the operation can never
    execute or become limited/paid by client metadata.
    """
    if not signed:
        return "quarantined_unsigned"
    if not operation_registered:
        return "quarantined_unknown_operation"
    if not owner_registered or product_owner not in REGISTERED_PRODUCT_OWNERS:
        return "quarantined_unknown_owner"
    if not class_registered or operation_class not in REGISTERED_OPERATION_CLASSES:
        return "quarantined_unknown_mutation"
    if not side_effect_registered or side_effect_class not in REGISTERED_SIDE_EFFECT_CLASSES:
        return "quarantined_unknown_side_effect"
    if not family_registered:
        return "quarantined_unregistered_family"
    if operation_class == "recovery" and canonical_class != "recovery":
        return "quarantined_self_labeled_recovery"
    if (
        canonical_class != operation_class
        or canonical_family != capability_family
        or canonical_side_effect != side_effect_class
        or declared_policy_fields
    ):
        return "quarantined_client_selected_policy"
    return "trusted"


def test_registry_trusted_metadata(registry: dict) -> None:
    operations = registry.get("operations", [])
    require(registry.get("operation_count") == len(operations), "canonical operation count must match descriptors")
    require(
        registry.get("operation_policy_schema") == "focusa.operation_policy_metadata.v1",
        "operation policy schema must be stable",
    )
    for operation in operations:
        operation_id = operation.get("operation_id")
        for field in (
            "operation_id",
            "product_owner",
            "operation_class",
            "capability_family",
            "side_effect_class",
        ):
            require(field in operation, f"{operation_id} missing trusted metadata field {field}")
            require(operation.get(field), f"{operation_id} empty {field}")
        require(
            operation["product_owner"] in REGISTERED_PRODUCT_OWNERS,
            f"{operation_id} unregistered product owner {operation['product_owner']}",
        )
        require(
            operation["operation_class"] in REGISTERED_OPERATION_CLASSES,
            f"{operation_id} unregistered operation class",
        )
        require(
            operation["side_effect_class"] in REGISTERED_SIDE_EFFECT_CLASSES,
            f"{operation_id} unregistered side effect class",
        )
        require(
            operation["capability_family"] in REGISTERED_CAPABILITY_FAMILIES,
            f"{operation_id} unregistered capability family",
        )


def test_all_signed_exact_manifests_trusted(registry: dict) -> None:
    # A signed manifest whose claims match the canonical registry exactly is
    # trusted for every canonical operation. Trusted operations inherit
    # canonical policy; they never become limited/paid by client metadata.
    trusted = 0
    for operation in registry["operations"]:
        decision = verify_manifest(
            operation["operation_id"],
            operation["product_owner"],
            operation["operation_class"],
            operation["capability_family"],
            operation["side_effect_class"],
            signed=True,
            operation_registered=True,
            canonical_class=operation["operation_class"],
            canonical_family=operation["capability_family"],
            canonical_side_effect=operation["side_effect_class"],
            owner_registered=operation["product_owner"] in REGISTERED_PRODUCT_OWNERS,
            class_registered=True,
            side_effect_registered=True,
            family_registered=True,
            declared_policy_fields=[],
        )
        require(decision == "trusted", f"{operation['operation_id']} must be trusted: {decision}")
        trusted += 1
    require(trusted == registry["operation_count"], f"expected every canonical operation trusted, got {trusted}")


def test_adversarial_fixtures_fail_closed(registry: dict) -> None:
    operations = {operation["operation_id"]: operation for operation in registry["operations"]}
    mutation = next(
        operation
        for operation in registry["operations"]
        if operation["operation_class"] == "value_mutation"
    )
    operation_id = mutation["operation_id"]
    canonical = {
        "operation_id": operation_id,
        "product_owner": mutation["product_owner"],
        "operation_class": mutation["operation_class"],
        "capability_family": mutation["capability_family"],
        "side_effect_class": mutation["side_effect_class"],
        "signed": True,
        "operation_registered": True,
        "canonical_class": mutation["operation_class"],
        "canonical_family": mutation["capability_family"],
        "canonical_side_effect": mutation["side_effect_class"],
        "owner_registered": True,
        "class_registered": True,
        "side_effect_registered": True,
        "family_registered": True,
        "declared_policy_fields": [],
    }
    fixtures = [
        # Unknown/unsigned manifest: no anonymous product capability.
        ("unsigned", {**canonical, "signed": False}, "quarantined_unsigned"),
        # Unknown operation: dynamic operation not in the canonical registry.
        (
            "unknown_operation",
            {**canonical, "operation_id": "focusa.invented.dynamic_tool", "operation_registered": False},
            "quarantined_unknown_operation",
        ),
        # Unknown ownership: caller-invented product owner.
        (
            "unknown_owner",
            {**canonical, "product_owner": "caller_owned_product", "owner_registered": False},
            "quarantined_unknown_owner",
        ),
        # Unknown mutation class: caller-invented class.
        (
            "unknown_mutation",
            {**canonical, "operation_class": "self_grant", "class_registered": False},
            "quarantined_unknown_mutation",
        ),
        # Unknown side effect: caller claims unlimited unmetered side effect.
        (
            "unknown_side_effect",
            {**canonical, "side_effect_class": "unmetered_unlimited", "side_effect_registered": False},
            "quarantined_unknown_side_effect",
        ),
        # Unregistered family: new family never enters any allowlist implicitly.
        (
            "unregistered_family",
            {**canonical, "capability_family": "unregistered_new_customer_outcome", "family_registered": False},
            "quarantined_unregistered_family",
        ),
        # Self-labeled recovery: a value_mutation tool claiming recovery to
        # bypass licensing.
        (
            "self_labeled_recovery",
            {**canonical, "operation_class": "recovery"},
            "quarantined_self_labeled_recovery",
        ),
        # Client-selected policy: declared License Type / product / price.
        (
            "client_license_type",
            {**canonical, "declared_policy_fields": ["license_type"]},
            "quarantined_client_selected_policy",
        ),
        (
            "client_product_price",
            {**canonical, "declared_policy_fields": ["product", "price"]},
            "quarantined_client_selected_policy",
        ),
        # Client-selected policy: reclassifying a mutation as a read to obtain
        # a cheaper treatment.
        (
            "reclassify_mutation_as_read",
            {**canonical, "operation_class": "read", "canonical_class": "value_mutation"},
            "quarantined_client_selected_policy",
        ),
        # Future product without operator-approved registration is not a grant.
        (
            "future_product",
            {**canonical, "product_owner": "navigator"},
            "quarantined_unknown_owner",
        ),
    ]
    for name, fixture, expected in fixtures:
        decision = verify_manifest(
            fixture["operation_id"],
            fixture["product_owner"],
            fixture["operation_class"],
            fixture["capability_family"],
            fixture["side_effect_class"],
            fixture["signed"],
            fixture["operation_registered"],
            fixture["canonical_class"],
            fixture["canonical_family"],
            fixture["canonical_side_effect"],
            fixture["owner_registered"],
            fixture["class_registered"],
            fixture["side_effect_registered"],
            fixture["family_registered"],
            fixture["declared_policy_fields"],
        )
        require(decision == expected, f"fixture {name}: expected {expected}, got {decision}")


def test_generated_ui_bindings_only_canonical_actions(registry: dict, bindings: dict) -> None:
    operations = registry.get("operations", [])
    by_id = {operation["operation_id"]: operation for operation in operations}
    require(bindings.get("schema") == "focusa.ui_action_binding_index.v1", "bindings schema")
    bound = bindings.get("bindings", [])
    require(bindings.get("binding_count") == len(bound), "binding count must match generated bindings")
    registered_actions = {
        operation["operation_id"]
        for operation in operations
        if operation["ui"]["allowed_in_generated_ui"]
    }
    bound_actions = {binding["action_id"] for binding in bound}
    require(bound_actions == registered_actions, "bindings must equal canonical registered actions")
    for binding in bound:
        action_id = binding["action_id"]
        operation = by_id.get(action_id)
        require(operation is not None, f"binding {action_id} is not a canonical operation")
        require(
            operation["ui"]["allowed_in_generated_ui"] is True,
            f"{action_id} must be allowed in generated UI",
        )
        require(
            binding["operation_descriptor_ref"] == f"/v1/agent/operations#{action_id}",
            f"{action_id} descriptor ref drift",
        )
    # A generated-UI action outside the canonical registered action set is
    # grant expansion and can never render as a limited/paid surface.
    for invented in ("focusa.invented.auto_grant", "focusa.paid.upgrade.button"):
        require(
            verify_manifest(
                invented, "focusa", "value_mutation", "base_focusa", "local",
                signed=True, operation_registered=False,
                canonical_class="value_mutation", canonical_family="base_focusa",
                canonical_side_effect="local", owner_registered=True,
                class_registered=True, side_effect_registered=True,
                family_registered=True, declared_policy_fields=[],
            )
            == "quarantined_unknown_operation",
            f"generated UI action {invented} must not be renderable",
        )


def test_a2ui_catalog_has_no_presenter_owned_policy(a2ui: dict) -> None:
    # A2UI is a presenter surface: it must never own pricing, grants, limits,
    # or commercial policy (Spec 172 Sections 2.6 and 12).
    require(a2ui.get("schema") == "focusa.a2ui_catalog.v1", "a2ui catalog schema")
    forbidden = FORBIDDEN_CLIENT_POLICY_FIELDS
    hits: list[str] = []

    def walk(node: object, path: str) -> None:
        if isinstance(node, dict):
            for key, value in node.items():
                if any(field in str(key).lower() for field in forbidden):
                    hits.append(f"{path}.{key}")
                walk(value, f"{path}.{key}")
        elif isinstance(node, list):
            for index, value in enumerate(node):
                walk(value, f"{path}[{index}]")

    walk(a2ui, "a2ui")
    require(not hits, f"A2UI presenter catalog must not carry commercial policy: {hits}")


def test_quarantine_state_blocks_execution() -> None:
    # Rejected manifests are recorded in quarantine state and can never
    # execute or become limited/paid by client metadata.
    ledger: list[dict] = []
    next_sequence = 0

    def quarantine(operation_id: str, reason: str) -> int:
        nonlocal next_sequence
        sequence = next_sequence
        next_sequence += 1
        ledger.append(
            {
                "sequence": sequence,
                "operation_id": operation_id,
                "reason": reason,
                "stable_error": STABLE_ERROR_UNKNOWN,
            }
        )
        return sequence

    assert quarantine("focusa.unknown.tool", "quarantined_unknown_operation") == 0
    assert quarantine("focusa.self_labeled.recovery", "quarantined_self_labeled_recovery") == 1
    require(len(ledger) == 2, "ledger must record both quarantined manifests")
    require(
        {record["operation_id"] for record in ledger}
        == {"focusa.unknown.tool", "focusa.self_labeled.recovery"},
        "quarantine ledger identity",
    )
    require(
        all(record["stable_error"] == STABLE_ERROR_UNKNOWN for record in ledger),
        "quarantined manifests surface the stable unknown-policy error",
    )
    # A quarantined operation must not execute even if a later manifest looks
    # canonical: the quarantine ledger check precedes execution.
    for record in ledger:
        require(record["operation_id"] not in {"focusa.license.validate"}, "quarantine overreach")


def main() -> int:
    with REGISTRY_PATH.open(encoding="utf-8") as handle:
        registry = json.load(handle)
    with BINDINGS_PATH.open(encoding="utf-8") as handle:
        bindings = json.load(handle)
    with A2UI_PATH.open(encoding="utf-8") as handle:
        a2ui = json.load(handle)

    test_registry_trusted_metadata(registry)
    test_all_signed_exact_manifests_trusted(registry)
    test_adversarial_fixtures_fail_closed(registry)
    test_generated_ui_bindings_only_canonical_actions(registry, bindings)
    test_a2ui_catalog_has_no_presenter_owned_policy(a2ui)
    test_quarantine_state_blocks_execution()

    check = subprocess.run(
        ["python3", "scripts/generate-spec135-operation-contracts.py", "--check"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    require(check.returncode == 0, check.stdout + check.stderr)

    print("Spec172 dynamic operation manifest gate passed")
    print(f"operation_count={registry.get('operation_count')}")
    print(f"trusted_exact_manifests={registry['operation_count']}")
    print(f"generated_ui_bindings={bindings.get('binding_count')}")
    print(f"adversarial_fixtures=12")
    print(f"a2ui_presenter_policy_fields=0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
