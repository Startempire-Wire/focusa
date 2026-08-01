#!/usr/bin/env python3
"""Validate P01 authority closure and the dependency boundary into P02."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CLOSURE = json.loads((ROOT / "docs/contracts/spec135-p01-authority-closure.v1.json").read_text())
ISSUES = {
    record["id"]: record
    for record in (
        json.loads(line)
        for line in (ROOT / ".beads/issues.jsonl").read_text().splitlines()
        if line.strip()
    )
}

assert CLOSURE["schema"] == "focusa.spec135.p01_authority_closure.v1"
assert CLOSURE["status"] == "closed"
assert CLOSURE["work_items"]["closed_count"] == 24
assert CLOSURE["canonical_rules"]["resolver_type"] == "ResolvedWorkspaceProjection"
assert CLOSURE["canonical_rules"]["canonical_owner"] == "Focusa Core"
assert CLOSURE["canonical_rules"]["eligibility_before_geometry"] is True
assert CLOSURE["canonical_rules"]["omission_reason_count"] == 8
assert CLOSURE["canonical_rules"]["proof_requirement_count"] == 13
assert CLOSURE["canonical_rules"]["semantic_counterfeiting_forbidden"] is True
assert CLOSURE["canonical_rules"]["client_local_ad_hoc_reflow_forbidden"] is True
assert CLOSURE["proof"]["runtime_proofs_status"] == "all 13 remain pending until P10"
assert CLOSURE["proof"]["false_closure_blocked"] is True

for sequence in range(18, 42):
    assert ISSUES[f"focusa-mc2.2.{sequence:03d}"]["status"] == "closed"
for issue_id in CLOSURE["next_phase"]["ready_work_items"]:
    assert ISSUES[issue_id]["status"] == "closed"
    assert all(
        ISSUES[dependency["depends_on_id"]]["status"] == "closed"
        for dependency in ISSUES[issue_id].get("dependencies") or []
        if dependency["type"] == "blocks"
    )

print("Spec 135 P01 authority closure: PASS (P02 contract wave subsequently completed)")
