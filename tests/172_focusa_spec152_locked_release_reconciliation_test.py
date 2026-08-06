#!/usr/bin/env python3
"""Fail closed on drift in the expanded Spec 152 locked-release reconciliation."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "release-proof" / "audit"
SUMMARY = json.loads(
    (AUDIT / "spec152-locked-release-implementation-reconciliation-summary.json").read_text()
)
LEDGER = json.loads(
    (AUDIT / "next-locked-release-governance-reconciliation.json").read_text()
)
GATE = json.loads(
    (AUDIT / "next-locked-release-technical-closure-gate.json").read_text()
)

assert SUMMARY["schema"] == "focusa.spec152_locked_release_implementation_reconciliation_summary.v1"
assert SUMMARY["provider_snapshot"] == LEDGER["provider_snapshot"]
assert SUMMARY["governance"]["admitted_mapping_count"] == LEDGER["admitted_mapping_count"]
assert SUMMARY["governance"]["untracked_locked_release_count"] == 0
assert SUMMARY["governance"]["invalid_closed_count"] == GATE["invalid_closed_count"] == 0
assert SUMMARY["governance"]["technically_accepted_count"] == GATE["technically_accepted_count"]
assert SUMMARY["governance"]["technically_pending_count"] == GATE["technically_pending_count"]
assert GATE["technically_accepted_count"] + GATE["technically_pending_count"] == GATE["mapping_count"]
assert SUMMARY["governance"]["locked_release_detail_tombstones_excluded"] == 75

by_name = {row["name"]: row for row in SUMMARY["licensing_graph"]}
assert by_name["spec152_granular_legacy"]["node_count"] == 40
assert by_name["spec152_replay"]["node_count"] == 6
assert by_name["spec152_protected_distribution"]["node_count"] == 6
assert by_name["spec152e_correction_atoms"]["node_count"] == 63
assert by_name["final_release_milestones"]["node_count"] == 7
assert (
    by_name["spec152_granular_legacy"]["reconciliation_classification"]
    ["partial_with_commit_or_test_evidence"]
    == 31
)
assert (
    by_name["spec152_replay"]["reconciliation_classification"]
    ["partial_with_commit_or_test_evidence"]
    == 5
)
assert SUMMARY["build_independent_gates"]["spec152_python"] == {
    "evidence": "docs/evidence/release/focusa-vbcqu.20-locked-release-reconciliation.txt",
    "passed": 19,
    "total": 19,
}
assert SUMMARY["current_frontier"]["work_item"] == "focusa-vbcqu.20.13.2"
assert SUMMARY["current_frontier"]["publication"] == "forbidden"
assert any("395" in finding for finding in SUMMARY["material_findings"])
assert any("stale-excluded" in finding for finding in SUMMARY["material_findings"])

print("Spec 152 expanded locked-release reconciliation: PASS")
print(f"admitted={GATE['mapping_count']} accepted={GATE['technically_accepted_count']} pending={GATE['technically_pending_count']}")
