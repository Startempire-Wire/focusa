#!/usr/bin/env python3
"""Spec138 source fusion, scenarios, transfer, and self-model gate."""
from pathlib import Path
import json
import yaml

ROOT = Path(__file__).resolve().parents[1]
SOURCES = json.loads((ROOT / "docs/contracts/spec138-source-independence-and-triangulation-matrix.v1.yaml").read_text())
TRANSFER = json.loads((ROOT / "docs/contracts/spec138-transfer-self-model-and-consolidation-matrix.v1.yaml").read_text())
PROOF = yaml.safe_load((ROOT / "docs/contracts/spec138-runtime-proof-map.v1.yaml").read_text())
RECEIPT = yaml.safe_load((ROOT / "docs/contracts/138-focusa-advanced-transfer-fusion-receipt.v1.yaml").read_text())
FUSION = (ROOT / "crates/focusa-core/src/epistemic_fusion.rs").read_text()
ADVANCED = (ROOT / "crates/focusa-core/src/prediction_advanced.rs").read_text()
assert SOURCES["runtime_status"] == "verified_complete" and SOURCES["status"] == "verified_complete"
assert len(SOURCES["required_dimensions"]) == 10
assert TRANSFER["transfer_status"] == "verified_complete"
assert TRANSFER["self_model_status"] == "verified_complete"
assert TRANSFER["consolidation_status"] in {"implementation_open", "verified_complete"}
for symbol in ("FusionSignal", "WeightOrigin", "MissingnessKind", "FusionPolicy", "EffectiveContribution", "fuse_signals", "IndependenceMisclassified", "SourceRiskExceeded", "ContradictionWouldBeHidden"):
    assert symbol in FUSION, symbol
for symbol in ("ScenarioDefinition", "ScenarioCausalStatus", "project_scenario", "TransferAssessment", "TransferEvaluation", "evaluate_transfer", "SelfModelEstimate", "validate_self_model", "GlobalSelfModelProhibited"):
    assert symbol in ADVANCED, symbol
assert len(FUSION.splitlines()) < 500 and len(ADVANCED.splitlines()) < 500
parent = "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md"
ranges = ((338,377),(512,559),(881,922),(947,970),(1200,1234),(1346,1508),(1801,1864))
rows = [row for row in PROOF["rows"] if row["source_path"] == parent and any(a <= row["source_line"] <= b for a,b in ranges)]
assert len(rows) == 23 and all(row["status"] == "verified_complete" for row in rows)
assert RECEIPT["status"] == "verified_slice" and RECEIPT["full_conformance_status"] == "open"
print("Spec138 advanced gate: PASS (23 source rows; fusion, scenarios, transfer, self-model)")
