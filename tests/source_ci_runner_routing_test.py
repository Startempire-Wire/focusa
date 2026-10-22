#!/usr/bin/env python3
"""Source validation cannot spill onto the production/deploy runner."""
from pathlib import Path
import importlib.util
import tempfile
import unittest

import yaml

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    "workflow_validation", ROOT / "scripts/validate-github-workflows.py"
)
owner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(owner)


class SourceRunnerRouting(unittest.TestCase):
    def test_actual_source_jobs_use_the_existing_rust_build_lane(self):
        ci = yaml.safe_load((ROOT / ".github/workflows/ci.yml").read_text())
        lane = ci["jobs"]["rust"]["runs-on"]
        self.assertIn("ovh-build-2", lane)
        for name, job in [
            ("ci.yml", "commit-messages"),
            ("spec152-documentation-consistency.yml", "consistency"),
            ("structural-guards.yml", "guards"),
        ]:
            workflow = yaml.safe_load((ROOT / ".github/workflows" / name).read_text())
            with self.subTest(workflow=name):
                self.assertEqual(workflow["jobs"][job]["runs-on"], lane)
                self.assertEqual(owner.validate(ROOT / ".github/workflows" / name), [])

    def test_canonical_validator_rejects_unlabelled_source_validation(self):
        with tempfile.TemporaryDirectory() as tmp:
            for name in ["ci.yml", "structural-guards.yml", "spec152-documentation-consistency.yml"]:
                path = Path(tmp) / name
                path.write_text("name: fixture\non: push\njobs:\n  check:\n    runs-on: self-hosted\n    steps: []\n")
                with self.subTest(workflow=name):
                    self.assertTrue(any("OVH" in error for error in owner.validate(path)))

    def test_explicit_production_delivery_route_remains_separate(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "deploy-live-daemon.yml"
            path.write_text("name: fixture\non: workflow_dispatch\njobs:\n  deploy:\n    runs-on: [self-hosted, linux, x64, focusa-deploy, production]\n    steps: []\n")
            self.assertEqual(owner.validate(path), [])


if __name__ == "__main__":
    unittest.main()
