#!/usr/bin/env python3
"""Strict static closure lint for ready-frontier Q1, Q3, Q4, and E1 contracts."""

import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
C = R / "docs/contracts"


def load(name):
    return json.loads((C / name).read_text())


def assert_refs(contract):
    for ref in contract["evidence_refs"]:
        assert (R / ref).exists(), f"missing evidence ref: {ref}"


q1 = load("spec135-q1-security-scope-policy.v1.yaml")
assert q1["required_scope_keys"] == ["project_root", "continuity_id", "attachment_id"]
assert set(q1["protected_surfaces"]) == {
    "generated_actions",
    "work_surfaces",
    "connectors",
    "events",
    "multi_agent_worktrees",
}
for surface in q1["protected_surfaces"].values():
    assert surface["fail_closed_if"]
    for ref in surface["evidence_refs"]:
        assert (R / ref).exists(), f"missing Q1 evidence ref: {ref}"
assert q1["operator_control"]["steering_precedence"] is True

q3 = load("spec135-q3-performance-budgets.v1.yaml")
assert all("degraded_action" in budget for budget in q3["budgets"].values())
assert q3["budgets"]["graph_projection_nodes"]["maximum"] <= 500
assert q3["budgets"]["artifact_inline_bytes"]["maximum"] <= 65536
assert_refs(q3)

q4 = load("spec135-q4-recovery-matrix.v1.yaml")
assert all(row["mechanisms"] for row in q4["recovery_contracts"])
assert q4["required_failure_posture"]["scope_mismatch"] == "fail_closed"
assert q4["acceptance"]["duplicate_commits_prevented"] is True
assert_refs(q4)

e1 = load("spec135-e1-migration-inventory.v1.yaml")
assert len(e1["inventory"]) >= 7
for row in e1["inventory"]:
    assert row["old_owner"] and row["new_owner"]
    assert row["readers"] and row["writers"] and row["migration_state"]
assert_refs(e1)

print("Spec 135 ready-frontier Q1/Q3/Q4/E1 strict contract lint: PASS")
