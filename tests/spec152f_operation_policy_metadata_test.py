#!/usr/bin/env python3
"""Validate the generated Spec 152F operation-policy projections."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = ROOT / "docs/contracts/spec135/generated-contract-v1/operation-registry.json"
DESCRIPTORS_PATH = ROOT / "docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json"
REST_PATH = ROOT / "docs/contracts/spec141/generated-capability-v2/rest-agent-operations.json"

registry = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
descriptors = json.loads(DESCRIPTORS_PATH.read_text(encoding="utf-8"))
rest = json.loads(REST_PATH.read_text(encoding="utf-8"))

fields = {
    "operation_class",
    "capability_family",
    "commercial_treatment",
    "policy_activation",
    "required_feature",
    "limit_bucket",
    "recovery_allowance",
    "source_owner",
    "policy_owner",
}
classes = {"read", "value_mutation", "recovery", "internal_maintenance"}
family_treatment = {
    "account_recovery": "always_available",
    "read_projection": "read_allowance",
    "base_focusa": "base_entitlement",
    "automation": "optional_premium",
    "team_remote": "optional_premium",
    "release_proof": "optional_premium",
    "premium_updates": "optional_premium",
    "customer_data_export": "always_available_basic_with_optional_premium_packaging",
    "internal_maintenance": "inherit_initiating_operation",
}

assert registry["operation_policy_schema"] == "focusa.operation_policy_metadata.v1"
assert registry["operation_policy_authority"] == "docs/contracts/spec152f-entitlement-policy.v1.yaml"
operations = registry["operations"]
assert len(operations) == registry["operation_count"] == 108
assert len({operation["operation_id"] for operation in operations}) == len(operations)
for operation in operations:
    assert fields <= operation.keys(), operation["operation_id"]
    assert operation["operation_class"] in classes
    family = operation["capability_family"]
    assert family in family_treatment
    assert operation["commercial_treatment"] == family_treatment[family]
    assert operation["policy_activation"] == "active"
    assert operation["policy_owner"] == "entitlement_policy_resolver"
    assert operation["source_owner"] == operation["ownership"]["subsystem"]
    if family == "account_recovery":
        assert operation["operation_class"] == "recovery"
        assert operation["recovery_allowance"] == "account_recovery"
    elif family == "read_projection":
        assert operation["operation_class"] == "read"
        assert operation["recovery_allowance"] == "read_projection"
    else:
        assert operation["operation_class"] == "value_mutation"
        assert operation["recovery_allowance"] == "none"
    if operation["commercial_treatment"] != "optional_premium":
        assert operation["required_feature"] is None
        assert operation["limit_bucket"] is None

by_route = {(operation["method"], operation["path"]): operation for operation in operations}
assert descriptors["capability_count"] == len(descriptors["descriptors"]) == 136
for descriptor in descriptors["descriptors"]:
    assert fields <= descriptor.keys(), descriptor["capability_id"]
    assert fields <= descriptor["operation_policy"].keys()
    assert descriptor["policy_owner"] == "entitlement_policy_resolver"
    for policy in descriptor["operation_policies"]:
        assert fields <= policy.keys()
        if policy["operation_id"] is not None:
            canonical = by_route[(policy["method"], policy["path"])]
            for field in fields:
                assert policy[field] == canonical[field], (descriptor["capability_id"], field)

assert len(rest["operations"]) == sum(
    len(descriptor["operation_policies"]) for descriptor in descriptors["descriptors"]
)
assert all(fields <= operation.keys() for operation in rest["operations"])

check = subprocess.run(
    ["python3", "scripts/generate-spec135-operation-contracts.py", "--check"],
    cwd=ROOT,
    capture_output=True,
    text=True,
)
assert check.returncode == 0, check.stdout + check.stderr

print(
    f"Spec 152F operation policy metadata: PASS ({len(operations)} canonical operations, "
    f"{len(descriptors['descriptors'])} Agent Descriptor V2 capabilities)"
)
