#!/usr/bin/env python3
"""Verify Spec137A tranche state, settlement truth, and publication blocking."""
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[1]
STATE = json.loads((ROOT / "docs/contracts/spec137a-tranche-settlement.v1.yaml").read_text())
CORE = (ROOT / "crates/focusa-core/src/temporal_release_gate.rs").read_text()
assert len(STATE["tranches"]) == 9
assert STATE["status"] in {"release_blocked", "eligible_not_requested"}
assert not STATE["publication_requested"]
by_id = {row["tranche_id"]: row for row in STATE["tranches"]}
assert len(by_id) == 9
assert by_id["spec137-parent"]["state"] == "settled"
assert not by_id["spec137-parent"]["open_requirement_refs"]
assert by_id["spec137-parent"]["settlement"]["factual_completion_proven"]
open_refs = []
for row in STATE["tranches"]:
    assert all(dep in by_id for dep in row["depends_on"]), row["tranche_id"]
    if row["state"] == "proof_pending":
        assert row["open_requirement_refs"], row["tranche_id"]
        assert row["settlement"] is None, row["tranche_id"]
        open_refs.extend(row["open_requirement_refs"])
    else:
        assert row["state"] == "settled", row["tranche_id"]
        assert not row["open_requirement_refs"] and not row["unsupported_requirement_refs"]
        assert row["settlement"]["factual_completion_proven"]
if STATE["status"] == "release_blocked":
    assert not STATE["parent_complete_claimed"] and not STATE["publication_allowed"]
    assert STATE["full_conformance_receipt_ref"] is None
    assert len(open_refs) == 172 == len(set(open_refs))
else:
    assert STATE["parent_complete_claimed"] and STATE["publication_allowed"]
    assert STATE["full_conformance_receipt_ref"] and not open_refs
for symbol in ("TemporalSettlement", "TrancheSettlement", "ReleaseConformanceRequest", "validate_release_conformance", "PublicationBlocked", "DependencyUnsettled", "MissingMissedTargetReceipt"):
    assert symbol in CORE, symbol
print("Spec137A tranche settlement release gate: PASS (open rows block; settled proof permits eligibility without publication)")
