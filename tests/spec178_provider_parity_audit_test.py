#!/usr/bin/env python3
"""Regression coverage for Spec 178 provider parity and restoration safety."""

from __future__ import annotations

import json
import os
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "scripts/audit-spec178-provider-parity.py"
CONFIG = ROOT / "config/spec178-provider-authority.json"


class Spec178ProviderParityAuditTest(unittest.TestCase):
    def run_audit(self, *, billing_locked: bool) -> dict[str, object]:
        env = os.environ.copy()
        if billing_locked:
            env["FOCUSA_GITHUB_HOSTED_BILLING_LOCKED"] = "1"
        else:
            env.pop("FOCUSA_GITHUB_HOSTED_BILLING_LOCKED", None)
        result = subprocess.run(
            ["python3", str(AUDIT)],
            cwd=ROOT,
            env=env,
            check=True,
            capture_output=True,
            text=True,
        )
        return json.loads(result.stdout)

    def test_machine_provider_map_matches_temporary_spec178_routes(self) -> None:
        config = json.loads(CONFIG.read_text(encoding="utf-8"))
        self.assertEqual(
            {
                surface: details["provider"]
                for surface, details in config["active_providers"].items()
            },
            {
                "linux": "github_actions_self_hosted",
                "windows": "appveyor",
                "macos": "codemagic",
            },
        )
        self.assertEqual(
            config["hosted_restoration"]["mode"],
            "operator_triggered_all_at_once",
        )
        self.assertFalse(config["hosted_restoration"]["automatic_mutation"])

    def test_hosted_billing_lock_emits_actionable_nonmutating_reminder(self) -> None:
        result = self.run_audit(billing_locked=True)
        self.assertEqual(result["status"], "passed")
        self.assertTrue(result["hosted_billing_locked"])
        self.assertFalse(result["mutation_performed"])
        reminder = str(result["restoration_reminder"])
        self.assertIn("operator-triggered", reminder)
        self.assertIn("all-at-once", reminder)
        self.assertIn("parity proof", reminder)

    def test_unlocked_probe_still_forbids_partial_automatic_restoration(self) -> None:
        result = self.run_audit(billing_locked=False)
        self.assertFalse(result["hosted_billing_locked"])
        self.assertFalse(result["mutation_performed"])
        self.assertIn("do not restore partially or automatically", result["restoration_reminder"])


if __name__ == "__main__":
    unittest.main()
