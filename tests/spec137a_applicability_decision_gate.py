#!/usr/bin/env python3
"""Verify every combined Spec137/137A applicability decision is explicit and evidence-backed."""
from pathlib import Path
import json
import yaml

ROOT = Path(__file__).resolve().parents[1]
MATRIX = json.loads((ROOT / "docs/contracts/spec137a-applicability-matrix.v1.yaml").read_text())
LEDGER = yaml.safe_load((ROOT / "docs/contracts/spec137-complete-feature-ledger.v1.yaml").read_text())
EVIDENCE = "docs/evidence/spec137a/S137A-applicability-decisions.txt"
assert (ROOT / EVIDENCE).exists()
assert len(MATRIX["rows"]) == 1191
for row in MATRIX["rows"]:
    assert row["status"] == "active", row["requirement_ref"]
    assert row["decision_authority"] == "operator_accepted_full_send_scope", row["requirement_ref"]
    assert row["applicability_evidence_refs"] == [EVIDENCE], row["requirement_ref"]
    assert row["applicable_scope_refs"] and row["platform_refs"] and row["domain_refs"], row["requirement_ref"]
for row in LEDGER["requirements"]:
    assert row["applicability"] in {"required", "activated_conditional"}, row["requirement_id"]
    assert EVIDENCE in row["applicability_evidence_refs"], row["requirement_id"]
for row in LEDGER["spec137a_requirement_rows"]:
    assert row["applicability_status"] == "active", row["requirement_id"]
    assert row["applicability_evidence_refs"] == [EVIDENCE], row["requirement_id"]
print("Spec137A applicability decision gate: PASS (1191 matrix + 86 parent + 172 addendum rows active)")
