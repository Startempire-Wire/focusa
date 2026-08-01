#!/usr/bin/env python3
"""Fast static + executable gate for Spec144 §28 evaluation assets."""
from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/spec144/evaluation.json"
HARNESS = ROOT / "crates/focusa-bench/spec144_evaluation.py"
RUST = ROOT / "crates/focusa-bench/src/spec144.rs"
LIB = ROOT / "crates/focusa-bench/src/lib.rs"
_spec = importlib.util.spec_from_file_location("spec144_evaluation", HARNESS)
assert _spec and _spec.loader
harness = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(harness)

REQUIRED_GOLDENS = {
    "full_replay_and_migration",
    "partial_client_parity_rejected",
    "unknown_impact_blocks_promotion",
    "unsupported_not_applicable_rejected",
    "forbidden_placeholder_detection",
    "runtime_variance_rejection",
    "blocked_row_parent_closure",
}


class Spec144EvaluationGate(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.document = harness.load_fixture(FIXTURE)

    def test_static_contract_and_exact_hashes(self) -> None:
        harness.validate_hashes(self.document, FIXTURE.parent)
        rust = RUST.read_text(encoding="utf-8")
        lib = LIB.read_text(encoding="utf-8")
        for symbol in ("COMPARISON_COHORTS", "EvaluationMetrics", "PromotionThresholds", "replay_equivalence", "blocking_failures"):
            self.assertIn(symbol, rust)
        self.assertIn("pub mod spec144;", lib)
        self.assertIn("PromotionThresholds", lib)

    def test_six_cohorts_and_25_executable_named_goldens(self) -> None:
        self.assertEqual(tuple(self.document["cohorts"]), harness.SIX_COHORTS)
        scenarios = self.document["scenarios"]
        self.assertEqual(len(scenarios), 25)
        ids = {row["id"] for row in scenarios}
        self.assertTrue(REQUIRED_GOLDENS <= ids)
        self.assertEqual(len(ids), 25)
        for row in scenarios:
            self.assertTrue(row["input"].strip())
            self.assertTrue(row["expected_control"].strip())
            self.assertTrue(row["requirements"])
        self.assertEqual(len(self.document["observations"]), 6 * 25)

    def test_metrics_calibration_resources_replay_and_promotion(self) -> None:
        proof = harness.run(FIXTURE)
        candidate = proof["cohorts"]["multi_aspect_portfolio"]
        required = {"precision", "recall", "false_positive_rate", "false_negative_rate", "coverage", "calibration_ece", "p95_latency_ms", "resource_units", "replay_equivalence"}
        self.assertTrue(required <= candidate.keys())
        self.assertTrue(proof["promotion"]["eligible"])
        self.assertEqual(proof["promotion"]["failures"], [])
        encoded = json.dumps(proof, sort_keys=True, separators=(",", ":")).encode()
        self.assertEqual(hashlib.sha256(encoded).hexdigest(), self.document["promotion"]["expected_proof_sha256"])

    def test_blockers_and_runtime_variance_fail_closed(self) -> None:
        for scenario in self.document["blocking_scenarios"]:
            changed = copy.deepcopy(self.document)
            observation = next(o for o in changed["observations"] if o["cohort"] == "multi_aspect_portfolio" and o["scenario"] == scenario)
            observation["predicted"] = "clear" if observation["expected"] == "finding" else "finding"
            result = harness.evaluate(changed)["multi_aspect_portfolio"]
            eligible, failures = harness.promotion_decision(changed, result)
            self.assertFalse(eligible, scenario)
            self.assertIn("blocking_failures_max", failures, scenario)
        changed = copy.deepcopy(self.document)
        observation = next(o for o in changed["observations"] if o["cohort"] == "multi_aspect_portfolio")
        observation["replay_repeat_hash"] = "sha256:" + "f" * 64
        result = harness.evaluate(changed)["multi_aspect_portfolio"]
        self.assertFalse(result["replay_equivalence"])
        self.assertFalse(harness.promotion_decision(changed, result)[0])


if __name__ == "__main__":
    unittest.main(verbosity=2)
