#!/usr/bin/env python3
from pathlib import Path
import hashlib
import yaml

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md"
LEDGER = ROOT / "docs/contracts/spec137-complete-feature-ledger.v1.yaml"
ledger = yaml.safe_load(LEDGER.read_text())

assert ledger["source_spec_sha256"] == hashlib.sha256(SPEC.read_bytes()).hexdigest()
assert ledger["status"] == "verified_complete"
assert ledger["combined_normative_source_v2"]["full_conformance_status"] == "verified_complete"
assert not ledger["excluded_applicable_mandatory_requirement_refs"]
assert not ledger["optional_unimplemented_refs"]
assert not ledger["not_applicable_refs"]
assert not ledger["variance_refs"]

required_ref_fields = (
    "core_types",
    "persistence",
    "reducer_events",
    "api_operations",
    "cli_commands",
    "pi_tools",
    "ui_surfaces",
    "migrations",
    "positive_tests",
    "negative_tests",
    "security_tests",
    "restart_recovery_tests",
    "accessibility_tests",
    "evidence_refs",
    "receipt_refs",
)
allowed_closed = {"verified_complete", "verified_not_applicable", "verified_optional_unimplemented"}
rows = ledger["requirements"]
assert len(rows) == 86
assert len({row["requirement_id"] for row in rows}) == 86

for row in rows:
    rid = row["requirement_id"]
    assert row["status"] in allowed_closed, f"{rid}: open status {row['status']}"
    assert row["implementation_owner"], f"{rid}: missing implementation owner"
    assert row["primitive_owner"], f"{rid}: missing primitive owner"
    assert row["applicability_evidence_refs"], f"{rid}: missing applicability evidence"
    if row["applicability"] in {"mandatory", "activated_conditional"}:
        for field in required_ref_fields:
            assert row[field], f"{rid}: missing {field}"
        for ref in sum((row[field] for field in required_ref_fields), []):
            if ref.startswith(("crates/", "apps/", "tests/", "docs/", "release-proof/")):
                assert (ROOT / ref.split("#", 1)[0]).exists(), f"{rid}: missing ref {ref}"

source_rows = ledger["spec137a_requirement_rows"]
assert len(source_rows) == 172
assert (ROOT / "tests/spec137a_source_ledger_integrity_gate.py").exists()
assert (ROOT / "tests/spec137a_zero_omission_runtime_gate.py").exists()
assert "validate_spec137a_closure" in (ROOT / "crates/focusa-core/src/temporal_foundation.rs").read_text()
for row in source_rows:
    rid = row["requirement_id"]
    assert row["runtime_status"] == "verified_complete", f"{rid}: runtime open"
    assert row["evidence_status"] == "verified_complete", f"{rid}: evidence open"
    assert row["receipt_status"] == "verified_complete", f"{rid}: receipt open"
    assert row["documentation_status"] == "verified_complete", f"{rid}: docs open"

print("Spec137/137a full conformance gate: PASS (86 parent + 172 addendum rows, zero omissions)")
