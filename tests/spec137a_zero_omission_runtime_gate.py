#!/usr/bin/env python3
"""Fail-closed Spec137A row gate; schema-only rows never satisfy closure."""
from pathlib import Path
import hashlib
import yaml

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / "docs/contracts/spec137-complete-feature-ledger.v1.yaml"
ADDENDUM = ROOT / "docs/137a-focusa-temporal-zero-deferral-applicability-and-omission-firewall-addendum.md"

data = yaml.safe_load(LEDGER.read_text())
rows = data["spec137a_requirement_rows"]
assert len(rows) == 172
assert hashlib.sha256(ADDENDUM.read_bytes()).hexdigest() == "2747f7f1ff7417c4541d7223199b3a480128ca049c9fe4a45859028af99a8419"

closed = {"verified_complete", "verified_not_applicable", "verified_optional_unimplemented"}
for row in rows:
    rid = row["requirement_id"]
    assert row["applicability_status"] in {"active", "not_applicable_verified", "conditional_inactive_verified", "variance_verified"}, rid
    assert row["documentation_status"] in closed, f"{rid}: documentation not closed"
    assert row["runtime_status"] in closed, f"{rid}: runtime not closed"
    assert row["evidence_status"] in closed, f"{rid}: evidence not closed"
    assert row["receipt_status"] in closed, f"{rid}: receipt not closed"
    if row["applicability_status"] != "active":
        assert row.get("applicability_evidence_refs"), f"{rid}: missing affirmative applicability evidence"
    assert row.get("closure_impact") == "blocking_for_claimed_conformance", rid

print("Spec137A zero-omission runtime gate: PASS (172/172)")
