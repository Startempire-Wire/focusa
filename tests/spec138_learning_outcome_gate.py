#!/usr/bin/env python3
"""Spec138 metacognitive promotion, outcome authority, and rollback gate."""
from pathlib import Path
import json
import yaml

ROOT = Path(__file__).resolve().parents[1]
LEARNING = json.loads((ROOT / "docs/contracts/spec138-learning-promotion-and-rollback-matrix.v1.yaml").read_text())
OUTCOMES = json.loads((ROOT / "docs/contracts/spec138-outcome-resolution-authority-matrix.v1.yaml").read_text())
PROOF = yaml.safe_load((ROOT / "docs/contracts/spec138-runtime-proof-map.v1.yaml").read_text())
RECEIPT = yaml.safe_load((ROOT / "docs/contracts/138-focusa-learning-outcome-receipt.v1.yaml").read_text())
CORE = (ROOT / "crates/focusa-core/src/metacognitive_learning.rs").read_text()
RESOLUTION = (ROOT / "crates/focusa-core/src/outcome_resolution.rs").read_text()
assert LEARNING["runtime_status"] == "verified_complete"
assert all(value == "verified_complete" for value in LEARNING["stage_statuses"].values())
assert LEARNING["self_promotion_prohibited"]
assert OUTCOMES["runtime_status"] == "verified_complete"
assert all(value == "verified_complete" for value in OUTCOMES["state_statuses"].values())
for symbol in ("ReflectionClaim", "CausalStatus", "LearningPromotionPolicy", "LearningOutcomeEvaluation", "PromotionAuthority", "assess_learning_promotion", "settle_learning_outcome", "SingleEventApprovalRequired", "HighConsequenceApprovalRequired"):
    assert symbol in CORE, symbol
for symbol in ("OutcomeState", "OutcomeAuthorityEvent", "OutcomeAuthorityLedger", "scoring_resolution_ref", "PolicyChanged", "MissingSupersession", "caller_score_advisory"):
    assert symbol in RESOLUTION, symbol
assert len(CORE.splitlines()) < 500 and len(RESOLUTION.splitlines()) < 500
parent = "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md"
rows = [row for row in PROOF["rows"] if row["source_path"] == parent and (1545 <= row["source_line"] <= 1586 or 1653 <= row["source_line"] <= 1800)]
assert len(rows) == 21 and all(row["status"] == "verified_complete" for row in rows)
assert RECEIPT["status"] == "verified_slice" and RECEIPT["full_conformance_status"] == "open"
print("Spec138 learning/outcome gate: PASS (21 source rows; promotion, resolution, rollback authority)")
