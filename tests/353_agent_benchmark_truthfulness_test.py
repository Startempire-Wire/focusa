#!/usr/bin/env python3
"""Actual evaluator CLI/journal refusal checks, not agent/model efficacy proof."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts/run-agent-intelligence-evals.sh"
CASES = ROOT / "tests/evals/agent_intelligence_cases.json"


class BenchmarkTruthfulness(unittest.TestCase):
    def run_cli(self, arm="on", definitions=None):
        env = dict(os.environ, ARM=arm, PYTHONDONTWRITEBYTECODE="1")
        args = ["bash", str(RUNNER)]
        if definitions is None:
            return subprocess.run(args, cwd=ROOT, env=env, text=True, capture_output=True, timeout=10)
        with tempfile.TemporaryDirectory(prefix="focusa-353-owned-") as directory:
            scenarios = Path(directory) / "scenarios.json"
            scenarios.write_text(json.dumps(definitions))
            return subprocess.run(args + [str(scenarios)], cwd=ROOT, env=env, text=True, capture_output=True, timeout=10)

    def definitions(self):
        return json.loads(CASES.read_text())

    def assert_unmeasured(self, result, arm):
        self.assertEqual(result.returncode, 78, result.stdout + result.stderr)
        self.assertIn("UNMEASURED:", result.stderr)
        self.assertIn(f"arm={arm};", result.stderr)
        self.assertNotIn("PASS", result.stdout + result.stderr)
        self.assertNotIn("aggregate=", result.stdout + result.stderr)

    def test_actual_requested_arms_are_unmeasured_not_falsely_passed(self):
        # Different labels are not presented as execution/configuration isolation.
        for arm in ["on", "off", "candidate"]:
            with self.subTest(arm=arm):
                self.assert_unmeasured(self.run_cli(arm), arm)

    def test_legacy_self_scored_input_is_rejected(self):
        definitions = self.definitions()
        for case in definitions["cases"]:
            case["score"] = 1.0
        result = self.run_cli(definitions=definitions)
        self.assertEqual(result.returncode, 1)
        self.assertIn("untrusted fixture score", result.stderr)

    def test_lowering_expected_thresholds_cannot_manufacture_observations(self):
        definitions = self.definitions()
        definitions["aggregate_threshold"] = 0
        for case in definitions["cases"]:
            case["threshold"] = 0
        self.assert_unmeasured(self.run_cli(definitions=definitions), "on")

    def test_missing_file_reference_is_rejected(self):
        definitions = self.definitions()
        definitions["cases"][0]["required_refs"] = ["tests/353-deliberately-nonexistent-reference"]
        result = self.run_cli(definitions=definitions)
        self.assertEqual(result.returncode, 1)
        self.assertIn("missing repository file reference", result.stderr)

    def test_missing_declared_tool_reference_is_rejected(self):
        definitions = self.definitions()
        definitions["cases"][0]["required_refs"] = ["focusa_353_deliberately_nonexistent_tool"]
        result = self.run_cli(definitions=definitions)
        self.assertEqual(result.returncode, 1)
        self.assertIn("missing declared tool reference", result.stderr)

    def test_empty_cases_cannot_produce_nan_or_pass(self):
        definitions = self.definitions()
        definitions["cases"] = []
        result = self.run_cli(definitions=definitions)
        self.assertEqual(result.returncode, 1)
        self.assertNotIn("PASS", result.stdout + result.stderr)

    def test_unknown_arm_is_rejected(self):
        result = self.run_cli(arm="not-an-arm")
        self.assertEqual(result.returncode, 1)
        self.assertIn("unsupported evaluation arm", result.stderr)

    def test_actual_journal_owner_runs_real_cli_and_refuses_missing_measurements(self):
        spec = importlib.util.spec_from_file_location("focusa_353_journal", ROOT / "scripts/canonical-release-journal.py")
        journal = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(journal)
        # Direct owner call invokes the actual runner, not a command/API mock.
        # It cannot reach publish(), a provider, later build gates or a new release.
        with self.assertRaises(RuntimeError) as caught:
            journal.run_benchmark("v0.9.209")
        message = str(caught.exception)
        self.assertIn("scripts/run-agent-intelligence-evals.sh", message)
        self.assertIn('"exit_code": 78', message)
        self.assertIn("UNMEASURED:", message)
        self.assertNotIn('"status": "passed"', message)


if __name__ == "__main__":
    unittest.main(verbosity=2)
