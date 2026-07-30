#!/usr/bin/env python3
"""Verify Spec137A omission/variance artifacts stay fail-closed until row proof closes."""
from pathlib import Path
import json
import yaml

ROOT = Path(__file__).resolve().parents[1]
LEDGER = yaml.safe_load((ROOT / "docs/contracts/spec137-complete-feature-ledger.v1.yaml").read_text())
PROOF = yaml.safe_load((ROOT / "docs/contracts/spec137a-runtime-proof-map.v1.yaml").read_text())
DAG = yaml.safe_load((ROOT / "docs/contracts/spec137a-root-delivery-dag.v1.yaml").read_text())
RECEIPT = yaml.safe_load((ROOT / "docs/contracts/137a-focusa-zero-omission-receipt.v1.yaml").read_text())
AUDIT = json.loads((ROOT / "docs/contracts/spec137a-forbidden-placeholder-audit.v1.yaml").read_text())
CORE = (ROOT / "crates/focusa-core/src/temporal_conformance.rs").read_text()
for symbol in ("OmissionFirewallInput", "ShouldVarianceRecord", "validate_omission_firewall", "BroadCompletionQualifier", "ActiveRowNotComplete", "ForbiddenDisposition"):
    assert symbol in CORE, symbol
assert len(PROOF["rows"]) == 172
if PROOF["status"] == "proof_pending":
    assert all(row["status"] == "proof_pending" for row in PROOF["rows"])
    assert DAG["status"] == "proof_pending"
    assert RECEIPT["status"] == "verification_pending"
    assert LEDGER["combined_normative_source_v2"]["full_conformance_status"] == "open"
else:
    assert PROOF["status"] == "verified_complete"
    assert all(row["status"] == "verified_complete" for row in PROOF["rows"])
    assert DAG["status"] == "verified_complete"
    assert RECEIPT["status"] == "verified_complete"
    assert LEDGER["combined_normative_source_v2"]["full_conformance_status"] == "verified_complete"
assert not LEDGER["variance_refs"]
assert not LEDGER["optional_unimplemented_refs"]
assert not LEDGER["not_applicable_refs"]
for row in LEDGER["requirements"]:
    assert row.get("variance_ref") is None, row["requirement_id"]
for row in LEDGER["spec137a_requirement_rows"]:
    assert row["applicability_status"] == "active", row["requirement_id"]
forbidden = {item.lower() for item in AUDIT["forbidden_dispositions"]}
for row in PROOF["rows"]:
    disposition = str(row.get("disposition", "")).lower()
    assert not any(term in disposition for term in forbidden), row["requirement_id"]
print("Spec137A omission/variance firewall gate: PASS (pending rows block; verified rows require proof; zero hidden dispositions/variances)")
