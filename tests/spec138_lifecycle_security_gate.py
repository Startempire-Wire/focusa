#!/usr/bin/env python3
"""Spec138 consolidation, forgetting, reactivation, privacy, and security gate."""
from pathlib import Path
import json
import yaml

ROOT = Path(__file__).resolve().parents[1]
SECURITY = json.loads((ROOT / "docs/contracts/spec138-security-privacy-retention-matrix.v1.yaml").read_text())
TRANSFER = json.loads((ROOT / "docs/contracts/spec138-transfer-self-model-and-consolidation-matrix.v1.yaml").read_text())
PROOF = yaml.safe_load((ROOT / "docs/contracts/spec138-runtime-proof-map.v1.yaml").read_text())
RECEIPT = yaml.safe_load((ROOT / "docs/contracts/138-focusa-lifecycle-security-receipt.v1.yaml").read_text())
MEMORY = (ROOT / "crates/focusa-core/src/epistemic_memory_lifecycle.rs").read_text()
CORE = (ROOT / "crates/focusa-core/src/epistemic_security.rs").read_text()
assert SECURITY["runtime_status"] == "verified_complete"
assert len(SECURITY["controls"]) == 13
assert all(value == "verified_complete" for value in SECURITY["control_statuses"].values())
assert SECURITY["high_consequence_fail_mode"] == "closed"
assert TRANSFER["consolidation_status"] == "verified_complete" and TRANSFER["runtime_status"] == "verified_complete"
for symbol in ("LearningMemoryRecord", "RetentionPolicy", "consolidate_memories", "MemoryLifecycleAction", "apply_memory_lifecycle", "LegalHoldBlocksDeletion", "ReactivationProofRequired"):
    assert symbol in MEMORY, symbol
for symbol in ("SourceSecurityPolicy", "SourceIngestionRequest", "evaluate_source_ingestion", "SourceSecurityAuditExport", "build_security_audit_export", "PublicSummaryOnly", "Quarantine", "EncryptionRequired"):
    assert symbol in CORE, symbol
assert len(MEMORY.splitlines()) < 500 and len(CORE.splitlines()) < 500
parent = "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md"
rows = [row for row in PROOF["rows"] if row["source_path"] == parent and 1917 <= row["source_line"] <= 2018]
assert len(rows) == 13 and all(row["status"] == "verified_complete" for row in rows)
assert RECEIPT["status"] == "verified_slice" and RECEIPT["full_conformance_status"] == "open"
print("Spec138 lifecycle/security gate: PASS (13 source rows; consolidation, retention, adversarial/privacy controls)")
