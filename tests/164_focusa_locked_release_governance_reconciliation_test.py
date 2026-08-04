#!/usr/bin/env python3
"""Structural gate for the GH#106.2 governance reconciliation snapshot."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "release-proof" / "audit"
LEDGER = json.loads(
    (AUDIT / "next-locked-release-governance-reconciliation.json").read_text()
)
MEMBERS = [
    json.loads(line)
    for line in (AUDIT / "next-locked-release-workset-members.jsonl")
    .read_text()
    .splitlines()
    if line
]
INVENTORY = json.loads(
    (AUDIT / "next-locked-release-governance-inventory.json").read_text()
)

assert LEDGER["schema"] == "focusa.locked_release_governance_reconciliation.v1"
assert LEDGER["workset_id"] == "workset:focusa-next-locked-release:r7"
assert LEDGER["inventory_digest"] == INVENTORY["inventory_digest"]
assert LEDGER["provider_snapshot"]["sha256"].startswith("sha256:")
assert len(LEDGER["provider_snapshot"]["sha256"]) == 71

mappings = LEDGER["mappings"]
ids = [row["bead_id"] for row in mappings]
assert len(ids) == len(set(ids)) == LEDGER["admitted_mapping_count"]
assert LEDGER["immutable_mapping_count"] == len(MEMBERS) == 275
assert {row["member_id"] for row in MEMBERS}.issubset(ids)
assert LEDGER["repair_overlay_mapping_count"] == 14

state_total = sum(LEDGER["provider_state_counts"].values())
evidence_total = sum(LEDGER["evidence_state_counts"].values())
assert state_total == evidence_total == len(mappings)

non_drift_gaps = {
    key: value for key, value in LEDGER["gaps"].items() if key != "projection_drift_ids"
}
assert LEDGER["unresolved_gap_count"] == sum(map(len, non_drift_gaps.values()))
assert LEDGER["status"] == (
    "reconciled" if LEDGER["unresolved_gap_count"] == 0 else "blocked"
)

# Authority identity and release-label coverage are already reconciled. Remaining
# blockers must stay explicit rather than being erased by administrative closure.
assert LEDGER["gaps"]["orphan_bead_ids"] == []
assert LEDGER["gaps"]["duplicate_provider_ids"] == []
assert LEDGER["gaps"]["untracked_locked_release_ids"] == []
assert LEDGER["status"] == "blocked"
assert LEDGER["gaps"]["pending_technical_acceptance_ids"]
assert LEDGER["gaps"]["closed_without_proof_ids"]

allowed_evidence_states = {
    "orphan",
    "pending_technical_acceptance",
    "ambiguous_duplicate_closure",
    "duplicate_target_without_proof",
    "exact_duplicate_receipt",
    "evidence_linked",
    "closed_without_proof",
}
for row in mappings:
    assert row["authority"] in {
        "immutable_workset_r7",
        "authorized_release_repair_overlay",
    }
    assert row["evidence_state"] in allowed_evidence_states
    if row["provider_state"] == "closed":
        assert row["closure_receipt"] is not None
    else:
        assert row["closure_receipt"] is None
    if row["evidence_state"] == "evidence_linked":
        assert (
            row["implementation_commit_refs"]
            or row["runtime_or_acceptance_evidence_refs"]
        )

print("GH#106.2 locked-release governance reconciliation: PASS (truthfully blocked)")
