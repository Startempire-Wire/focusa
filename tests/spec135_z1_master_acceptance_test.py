#!/usr/bin/env python3
"""Spec 135 master acceptance truth gate with optional final-release strictness."""
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
C = json.loads(
    (ROOT / "docs/contracts/spec135-master-final-acceptance.v1.json").read_text()
)

assert C["gate_count"] == 14
assert len(C["checks"]) == C["gate_count"]
assert len({row["check_id"] for row in C["checks"]}) == C["gate_count"]
assert C["passed_count"] == sum(row["status"] == "passed" for row in C["checks"])
assert C["merge_ready"] is (C["passed_count"] == C["gate_count"])
assert C["status"] == ("verified" if C["merge_ready"] else "reopened")
assert C["authority_ref"] == "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml"
assert (ROOT / C["authority_ref"]).exists()
assert C["branch_policy"] == "feature branch + PR only; never direct commit to main"
assert "provider-owned" in C["beads_closure_authority"]

required_ids = {
    "focusa_pi_native_terminal",
    "same_session_canvas_toggle",
    "generated_crist_rich_work_surfaces",
    "vertical_professional_workspaces",
    "interaction_mode_contract",
}
assert required_ids.issubset({row["check_id"] for row in C["checks"]})

for row in C["checks"]:
    ref = row["evidence_ref"]
    # Pending evidence may deliberately point at the shared invalidated proof,
    # but no gate is marked passed unless its evidence file exists.
    if row["status"] == "passed":
        assert (ROOT / ref).exists(), row

# Normal branch CI validates truthful state and allows incremental progress.
# Final-release/merge automation must explicitly opt into strict closure.
require_final = os.environ.get("FOCUSA_REQUIRE_SPEC135_FINAL") == "1"
if require_final:
    assert C["merge_ready"] is True, C
    assert all(row["status"] == "passed" for row in C["checks"]), C["checks"]

print(
    "Spec 135 master acceptance truth: PASS "
    f"({C['passed_count']}/{C['gate_count']}, merge_ready={C['merge_ready']}, "
    f"strict={require_final})"
)
