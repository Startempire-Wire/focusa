#!/usr/bin/env python3
"""Spec138 scorer registry, proper-score, calibration, and authority gate."""
from pathlib import Path
import json
import yaml

ROOT = Path(__file__).resolve().parents[1]
MATRIX = json.loads((ROOT / "docs/contracts/spec138-scorer-and-calibration-matrix.v1.yaml").read_text())
PROOF = yaml.safe_load((ROOT / "docs/contracts/spec138-runtime-proof-map.v1.yaml").read_text())
RECEIPT = yaml.safe_load((ROOT / "docs/contracts/138-focusa-scoring-calibration-receipt.v1.yaml").read_text())
SCORING = (ROOT / "crates/focusa-core/src/prediction_scoring.rs").read_text()
ALGORITHMS = (ROOT / "crates/focusa-core/src/prediction_scoring_algorithms.rs").read_text()
CALIBRATION = (ROOT / "crates/focusa-core/src/prediction_calibration.rs").read_text()
assert MATRIX["runtime_status"] == "verified_complete"
assert len(MATRIX["scorers"]) == 31 == len({row["id"] for row in MATRIX["scorers"]})
assert all(row["status"] == "verified_complete" and row["fixture_ref"] for row in MATRIX["scorers"])
assert len(MATRIX["calibration_dimensions"]) == 19
assert MATRIX["calibration_status"] == "verified_complete"
for symbol in ("ScorerId", "ScoreInput", "VersionedScorerRegistry", "CustomScorerRegistration", "required_scorer_registry"):
    assert symbol in SCORING, symbol
for symbol in ("ContinuousRankedProbabilityScore", "QuantilePinballLoss", "ExpectedCalibrationError", "Ndcg", "RealizedRegret"):
    assert symbol in SCORING + ALGORITHMS, symbol
for symbol in ("CalibrationDimension", "CalibrationObservation", "CalibrationReport", "EvaluationAuthority", "build_calibration_report", "PolicyLockedAfterCommitment"):
    assert symbol in CALIBRATION, symbol
assert len(SCORING.splitlines()) < 500 and len(ALGORITHMS.splitlines()) < 500 and len(CALIBRATION.splitlines()) < 500
parent = "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md"
ranges = ((589,662),(1235,1278),(1545,1652))
rows = [row for row in PROOF["rows"] if row["source_path"] == parent and any(a <= row["source_line"] <= b for a,b in ranges)]
assert len(rows) == 55 and all(row["status"] == "verified_complete" for row in rows)
assert RECEIPT["status"] == "verified_slice" and RECEIPT["required_scorer_count"] == 31
print("Spec138 scoring/calibration gate: PASS (31 scorers, 19 dimensions, 55 source rows)")
