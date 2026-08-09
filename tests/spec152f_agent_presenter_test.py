#!/usr/bin/env python3
"""Spec 152F §7 presenter parity: Pi and agent tools project canonical
entitlement decisions; licensing never grants operator/cognitive authority;
tool discovery/visibility never grants entitlement."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CAP = ROOT / "docs/contracts/spec141/generated-capability-v2"
PI_TOOLS_PATH = CAP / "pi-tools.json"
DESCRIPTORS_PATH = CAP / "agent-capability-descriptors.json"
CARD_PATH = CAP / "agent-card.json"
OPENAI_TOOLS_PATH = CAP / "openai-tools.json"
MCP_TOOLS_PATH = CAP / "mcp-tools.json"
ADAPTER_PATH = ROOT / "apps/pi-extension/src/entitlement-policy-adapter.ts"

pi_tools = json.loads(PI_TOOLS_PATH.read_text(encoding="utf-8"))
descriptors = json.loads(DESCRIPTORS_PATH.read_text(encoding="utf-8"))
card = json.loads(CARD_PATH.read_text(encoding="utf-8"))
openai_tools = json.loads(OPENAI_TOOLS_PATH.read_text(encoding="utf-8"))
mcp_tools = json.loads(MCP_TOOLS_PATH.read_text(encoding="utf-8"))
adapter = ADAPTER_PATH.read_text(encoding="utf-8")

POLICY_FIELDS = {
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

# ── 1. Every Pi tool inherits its canonical operation policy ────────────────
assert pi_tools["schema"] == "focusa.pi_tool_projection.v2"
assert len(pi_tools["tools"]) == descriptors["capability_count"] == len(descriptors["descriptors"]) == 136

by_pi_name = {descriptor["capability_id"].replace(".", "_"): descriptor for descriptor in descriptors["descriptors"]}
for tool in pi_tools["tools"]:
    name = tool["name"]
    descriptor = by_pi_name.get(name)
    assert descriptor is not None, f"Pi tool {name} has no Agent Capability Descriptor V2"
    policy = descriptor.get("operation_policy")
    assert policy is not None, f"Pi tool {name} descriptor carries no operation policy"
    assert POLICY_FIELDS <= policy.keys(), f"Pi tool {name} operation policy incomplete"
    assert policy["policy_owner"] == "entitlement_policy_resolver", name
    assert policy["policy_activation"] == "active", name

# ── 2. Licensing never grants operator/cognitive authority ──────────────────
AUTHORITY_GRANT_RE = re.compile(
    r"(operator|cognitive)_(authority|power|approval)_granted|"
    r"approval_inferred|role_permission_granted|mutation_confirmation_granted|"
    r"workstream_authority_granted|trajectory_authority_granted|workpoint_authority_granted"
)
GRANT_TRUE = re.compile(r'"([^"]*(?:authority|approval|permission|confirmation)[^"]*)"\s*:\s*true')

for descriptor in descriptors["descriptors"]:
    text = json.dumps(descriptor)
    assert not AUTHORITY_GRANT_RE.search(text), f"descriptor grants authority: {descriptor['capability_id']}"
    # `confirmation` may declare an operator-confirmation requirement (preserved
    # independently) but must never be derived from or granted by licensing.
    confirmation = descriptor.get("confirmation") or {}
    assert set(confirmation.keys()) <= {"required", "preview_supported", "confirmation_type"}, descriptor["capability_id"]
    for hit in GRANT_TRUE.finditer(text):
        key = hit.group(1)
        assert key == "required" or key == "preview_supported" or key == "confirmation_type", (
            f"descriptor field {key} is true: {descriptor['capability_id']}"
        )

# ── 3. Tool discovery/visibility never grants entitlement ───────────────────
DISCOVERY_GRANT_FIELDS = {
    "operation_policy",
    "commercial_treatment",
    "required_feature",
    "limit_bucket",
    "policy_activation",
    "policy_owner",
    "licensing_grants_capability_only",
}
for projection in (pi_tools, openai_tools, mcp_tools):
    for tool in projection["tools"]:
        keys = set(tool.keys())
        assert not keys & DISCOVERY_GRANT_FIELDS, (
            f"{projection['schema']} discovery payload carries a grant field: {tool.get('name') or tool}"
        )
assert card["schema"] == "focusa.agent_card.v1"
card_text = json.dumps(card)
assert not AUTHORITY_GRANT_RE.search(card_text), "agent card grants authority"
assert not GRANT_TRUE.search(card_text), "agent card contains a true grant field"
# Agent Card capabilities are functional transport claims, not commercial grants.
assert set(card["capabilities"]) <= {
    "streaming",
    "durable_tasks",
    "list_changed",
    "progressive_discovery",
    "structured_output",
}, card["capabilities"]

# ── 4. Pi policy adapter: fail-closed preflight, stable JSON, recovery ──────
assert "focusa.entitlement_decision.v1" in adapter
assert "focusa.license_authority_boundary.v1" in adapter
assert "focusa.tool_discovery_policy.v1" in adapter
assert "unknown_tool_has_no_operation_policy" in adapter, "unknown tools must fail closed"
assert "authority_posture_" in adapter, "value mutations require usable authority"
assert "account_recovery_is_always_available" in adapter
assert "update_for_recovery" in adapter
assert '"uninstall"' in adapter
assert "safe_read" in adapter
assert 'status_path' in adapter and "/v1/license/status" in adapter
assert "preflightAuthority" in adapter, "adapter exposes preflight before side effects"
assert "resolveOperationPolicyForTool" in adapter, "adapter resolves canonical policy per tool"
assert "projectEntitlementDecision" in adapter, "adapter projects stable machine JSON"
assert "recoveryActionsFor" in adapter, "adapter exposes recovery actions"
# Licensing grants capability only; the adapter never emits authority grants.
for forbidden in (
    "operator_authority_granted: true",
    "cognitive_authority_granted: true",
    "approval_inferred: true",
    "discovery_visibility_granted: true",
    "role_permission_granted: true",
    "mutation_confirmation_granted: true",
):
    assert forbidden not in adapter, f"adapter may not emit {forbidden}"
assert "licensing_grants_capability_only: true" in adapter
assert "operator_authority_granted: false" in adapter
assert "cognitive_authority_granted: false" in adapter
assert "approval_inferred: false" in adapter
assert "discovery_visibility_granted: false" in adapter

# ── 5. Presenter parity: the same canonical decision drives Pi and agents ───
# Value-mutating families resolve to base entitlement or optional premium; the
# descriptor must agree with the Pi tool's contract projection on the family
# boundary (base vs optional premium) so Pi and agent surfaces cannot diverge.
premium_families = {"automation", "team_remote", "release_proof", "premium_updates"}
base_families = {"base_focusa"}
for descriptor in descriptors["descriptors"]:
    policy = descriptor["operation_policy"]
    family = policy["capability_family"]
    treatment = policy["commercial_treatment"]
    assert (family in premium_families) == (treatment == "optional_premium"), descriptor["capability_id"]
    assert (family in base_families) == (treatment == "base_entitlement"), descriptor["capability_id"]
    assert (family in {"account_recovery", "read_projection", "customer_data_export", "internal_maintenance"}) == (
        treatment in {"always_available", "read_allowance", "always_available_basic_with_optional_premium_packaging", "inherit_initiating_operation"}
    ), descriptor["capability_id"]

print(
    "Spec 152F agent presenter parity: PASS "
    f"({len(pi_tools['tools'])} Pi tools inherit descriptor policy, "
    f"{len(descriptors['descriptors'])} descriptors, discovery advisory only)"
)
