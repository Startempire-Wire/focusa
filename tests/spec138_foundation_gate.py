#!/usr/bin/env python3
"""Spec138/138A source integrity and canonical epistemic foundation gate."""
from pathlib import Path
import hashlib
import yaml

ROOT = Path(__file__).resolve().parents[1]
LEDGER = yaml.safe_load((ROOT / "docs/contracts/spec138-complete-feature-ledger.v1.yaml").read_text())
rows = LEDGER["requirements"]
assert len(rows) == 542 == len({row["requirement_id"] for row in rows})
paths = {row["source_path"] for row in rows}
assert paths == {
    "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md",
    "docs/138a-focusa-epistemic-zero-deferral-profile-completeness-and-omission-firewall-addendum.md",
}
seen_atoms = set()
for row in rows:
    lines = (ROOT / row["source_path"]).read_text().splitlines()
    line = lines[row["source_line"] - 1].strip()
    atom_source = f'{row["source_path"]}:{row["source_line"]}:{line}'
    suffix = hashlib.sha256(atom_source.encode()).hexdigest()[:12]
    assert row["source_atom_ref"].endswith(suffix), row["requirement_id"]
    assert row["source_atom_ref"] not in seen_atoms, row["source_atom_ref"]
    seen_atoms.add(row["source_atom_ref"])
CORE = (ROOT / "crates/focusa-core/src/prediction_authority.rs").read_text()
PRIMITIVES = (ROOT / "crates/focusa-core/src/epistemic_primitives.rs").read_text()
REGISTRY = (ROOT / "crates/focusa-core/src/epistemic_primitives.txt").read_text().splitlines()
STORAGE = (ROOT / "crates/focusa-core/src/prediction_authority_storage.rs").read_text()
for symbol in ("EpistemicScope", "PredictionQuestion", "PredictionCommitment", "OutcomeClaim", "OutcomeResolution", "PredictionEvaluation", "LearningRecord", "TransferPrediction", "PredictionAuthorityEvent", "ScopedAuthorityEvent"):
    assert symbol in CORE, symbol
assert len(REGISTRY) == 629 == len(set(REGISTRY))
for symbol in ("EpistemicPrimitiveDescriptor", "EpistemicPrimitiveRecord", "EpistemicProvenance", "canonical_primitive_registry", "validate_epistemic_primitive", "SPEC138_PRIMITIVE_REGISTRY_SHA256"):
    assert symbol in PRIMITIVES, symbol
for symbol in ("DurablePredictionEvent", "PersistentPredictionAuthorityLedger", "append_batch", "sync_all", "backup_to", "restore_from_backup", "schema_version", "predecessor_digest", "InvalidChain", "MissingEvidence", "MissingReceipt"):
    assert symbol in STORAGE, symbol
PROOF = yaml.safe_load((ROOT / "docs/contracts/spec138-runtime-proof-map.v1.yaml").read_text())
RECEIPT = yaml.safe_load((ROOT / "docs/contracts/138-focusa-foundation-receipt.v1.yaml").read_text())
assert PROOF["row_count"] == 542
parent = "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md"
foundation = [row for row in PROOF["rows"] if row["source_path"] == parent and (163 <= row["source_line"] <= 1367 or 2051 <= row["source_line"] <= 2105)]
assert len(foundation) == 42 and all(row["status"] == "verified_complete" for row in foundation)
assert PROOF["verified_row_count"] >= 42
assert RECEIPT["status"] == "verified_slice" and RECEIPT["full_conformance_status"] == "open"
assert RECEIPT["primitive_registry_count"] == 629
print("Spec138 foundation gate: PASS (542 source rows; 42 foundation rows; 629 primitives; durable causal storage)")
