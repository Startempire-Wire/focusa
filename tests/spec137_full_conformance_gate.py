#!/usr/bin/env python3
"""Fail closed if audited-open Spec137/137A truth is falsely re-closed."""

from pathlib import Path
import hashlib
import yaml

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md"
CONTRACTS = ROOT / "docs/contracts"
REASON = "audited_runtime_conformance_missing_github_436"
OPEN = "implementation_open"
ledger = yaml.safe_load((CONTRACTS / "spec137-complete-feature-ledger.v1.yaml").read_text())
parent_receipt = yaml.safe_load((CONTRACTS / "137-focusa-parent-zero-omission-receipt.v1.yaml").read_text())
addendum_receipt = yaml.safe_load((CONTRACTS / "137a-focusa-zero-omission-receipt.v1.yaml").read_text())
settlement = yaml.safe_load((CONTRACTS / "spec137a-tranche-settlement.v1.yaml").read_text())

assert ledger["source_spec_sha256"] == hashlib.sha256(SPEC.read_bytes()).hexdigest(), "Spec137 source hash drift"
assert ledger["status"] == OPEN, f"ledger false closure/status drift: {ledger['status']}"
assert ledger["invalidation_reason"] == REASON, f"ledger invalidation drift: {ledger.get('invalidation_reason')}"
combined = ledger["combined_normative_source_v2"]
assert combined["full_conformance_status"] == "open", f"combined false closure: {combined['full_conformance_status']}"
assert not ledger["excluded_applicable_mandatory_requirement_refs"]
assert not ledger["optional_unimplemented_refs"]
assert not ledger["not_applicable_refs"]
assert not ledger["variance_refs"]

ref_fields = (
    "core_types", "persistence", "reducer_events", "api_operations", "cli_commands", "pi_tools",
    "ui_surfaces", "migrations", "positive_tests", "negative_tests", "security_tests",
    "restart_recovery_tests", "accessibility_tests", "evidence_refs", "receipt_refs",
)
rows = ledger["requirements"]
assert len(rows) == 86, f"parent row count drift: {len(rows)}"
assert len({row["requirement_id"] for row in rows}) == 86, "duplicate parent requirement IDs"
for row in rows:
    rid = row["requirement_id"]
    assert row["status"] == OPEN, f"{rid}: false closure/status drift {row['status']}"
    assert row["implementation_owner"], f"{rid}: missing implementation owner"
    assert row["primitive_owner"], f"{rid}: missing primitive owner"
    assert row["applicability_evidence_refs"], f"{rid}: missing applicability evidence"
    for field in ref_fields:
        for ref in row.get(field, []):
            if ref.startswith(("crates/", "apps/", "tests/", "docs/", "release-proof/")):
                assert (ROOT / ref.split("#", 1)[0]).exists(), f"{rid}: missing retained ref {ref}"

source_rows = ledger["spec137a_requirement_rows"]
assert len(source_rows) == 172, f"addendum row count drift: {len(source_rows)}"
assert (ROOT / "tests/spec137a_source_ledger_integrity_gate.py").exists()
assert (ROOT / "tests/spec137a_zero_omission_runtime_gate.py").exists()
assert "validate_spec137a_closure" in (ROOT / "crates/focusa-core/src/temporal_foundation.rs").read_text()
for row in source_rows:
    rid = row["requirement_id"]
    for field in ("runtime_status", "evidence_status", "receipt_status", "documentation_status"):
        assert row[field] == OPEN, f"{rid}: {field} false closure/status drift {row[field]}"

for name, receipt in (("parent", parent_receipt), ("addendum", addendum_receipt)):
    assert receipt["status"] == "verification_pending", f"{name} receipt false closure: {receipt['status']}"
    assert receipt["invalidation_reason"] == REASON, f"{name} receipt invalidation drift"
    assert set(receipt["proof"].values()) == {"invalidated_pending_reproof"}, f"{name} receipt proof re-enabled"

assert settlement["status"] == "blocked_open_requirements", f"settlement status drift: {settlement['status']}"
assert settlement["invalidation_reason"] == REASON
assert settlement["parent_complete_claimed"] is False
assert settlement["publication_requested"] is False
assert settlement["publication_allowed"] is False
for tranche in settlement["tranches"]:
    assert tranche["state"] == "open", f"{tranche['tranche_id']}: false tranche closure"
    assert tranche["settlement"]["factual_completion_proven"] is False, f"{tranche['tranche_id']}: false factual completion"

print("Spec137/137A truthful open-state guard: PASS (86 parent + 172 addendum rows remain open; false closure blocked)")
