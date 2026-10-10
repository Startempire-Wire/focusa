#!/usr/bin/env python3
"""Protect bounded health routing, cost controls and accepted-only promotion."""
import importlib.util
import json
from pathlib import Path
import subprocess
import unittest
from unittest.mock import Mock, patch

import yaml

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("resource_gate", ROOT / "scripts/check-release-resource-gate.py")
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


def workflow(name):
    return yaml.safe_load((ROOT / ".github/workflows" / name).read_text())


class ReleaseReadinessRecoveryTest(unittest.TestCase):
    def test_existing_ssh_alias_reads_only_canonical_loopback_health(self):
        result = Mock(stdout=json.dumps({"status": "ok"}))
        with patch.object(GATE.subprocess, "run", return_value=result) as run:
            self.assertEqual(GATE.health(GATE.DEFAULT_LOCAL, ssh_host="kh"), {"status": "ok"})
        args, kwargs = run.call_args
        self.assertEqual(args[0][0:4], ["ssh", "-n", "-o", "BatchMode=yes"])
        self.assertEqual(args[0][-1], "http://127.0.0.1:8791/v1/health")
        self.assertTrue(kwargs["check"])
        self.assertEqual(kwargs["timeout"], 20)
        self.assertNotIn("shell", kwargs)

    def test_ssh_alias_and_target_cannot_inject_remote_commands(self):
        for alias in ["-R 1234", "kh; echo unsafe", "kh $(false)"]:
            with self.assertRaises(ValueError):
                GATE.health(GATE.DEFAULT_LOCAL, ssh_host=alias)
        with self.assertRaises(ValueError):
            GATE.health("http://other-host:8791", ssh_host="kh")

    def test_ssh_failure_preserves_stderr_and_blocks(self):
        error = subprocess.CalledProcessError(255, ["ssh"], stderr="connection refused")
        with patch.object(GATE.subprocess, "run", side_effect=error):
            with self.assertRaisesRegex(RuntimeError, "connection refused"):
                GATE.health(GATE.DEFAULT_LOCAL, ssh_host="kh")

    def test_launcher_sets_verified_transport_without_opening_listener(self):
        env = workflow("dev-release-tag.yml")["jobs"]["create-tag"]["env"]
        self.assertEqual(env["FOCUSA_KH_RESOURCE_HEALTH_SSH_HOST"], "kh")
        self.assertEqual(env["AGENT_KB_API_URL"], GATE.DEFAULT_LOCAL)

    def test_push_and_dispatch_share_exact_tag_concurrency(self):
        self.assertEqual(workflow("release.yml")["concurrency"]["group"],
                         "release-${{ inputs.release_tag || github.ref_name }}")

    def test_all_publication_branches_are_candidates_until_deploy(self):
        steps = workflow("release.yml")["jobs"]["checksums"]["steps"]
        publication = next(s["run"] for s in steps if s.get("name", "").startswith("Publish immutable"))
        edits = publication.split('gh release edit "$TAG"')[1:]
        self.assertEqual(len(edits), 3)
        for edit in edits:
            command = edit.split("\n", 7)[:7]
            command = "\n".join(command)
            self.assertIn("--latest=false", command)
            self.assertIn("--prerelease", command)
            self.assertNotIn("--prerelease=false", command)
        deploy = workflow("deploy-live-daemon.yml")
        promoters = [s for job in deploy["jobs"].values() for s in job.get("steps", [])
                     if s.get("name") == "Promote accepted stable release to Latest"]
        self.assertEqual(len(promoters), 1)
        self.assertIn("--prerelease=false", promoters[0]["run"])
        self.assertIn("--latest", promoters[0]["run"])

    def test_terminal_matrix_spend_is_explicit_opt_in_not_main_push_default(self):
        jobs = workflow("spec132-terminal-matrix.yml")["jobs"]
        self.assertEqual(len(jobs), 6)
        for job in jobs.values():
            self.assertEqual(job["if"], "${{ vars.FOCUSA_GITHUB_HOSTED_RELEASE_MATRIX == 'enabled' }}")
        self.assertIn("windows-conpty", jobs)
        self.assertIn("release-target-build", jobs)
        self.assertIn("aarch64-pc-windows-msvc", (ROOT / ".github/workflows/spec132-terminal-matrix.yml").read_text())


if __name__ == "__main__":
    unittest.main()
