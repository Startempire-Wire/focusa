#!/usr/bin/env python3
"""Spec147 operationalization and no-spam conformance tests."""

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GOVERNOR = ROOT / "scripts" / "self_heal_governor.py"
ADAPTER = ROOT / "scripts" / "master-release-github-adapter.py"


def load_governor():
    spec = importlib.util.spec_from_file_location("self_heal_governor", GOVERNOR)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


G = load_governor()


def failure(**overrides):
    value = {
        "schema": G.FAILURE_SCHEMA,
        "repository": "Startempire-Wire/focusa",
        "workflow": "CI",
        "failure_class": "github_transient",
        "exact_sha": "0123456789abcdef",
        "action_scope": "run:42",
        "deterministic": False,
    }
    value.update(overrides)
    return value


def envelope(*, mode="plan", mutates=False, approvals=None):
    return {
        "schema": "focusa.release_plugin_envelope.v1",
        "operation": {
            "operation_id": "ci-preflight",
            "stage": "preflighted",
            "executor_id": "focusa-github",
            "kind": "github_workflow",
            "action": "CI",
            "surface_ids": [],
            "mutates": mutates,
            "timeout_seconds": 60,
            "parallel_group": None,
            "inputs": {
                "provider_mode": mode,
                "repository": "Startempire-Wire/focusa",
                "workflow": "ci.yml",
            },
        },
        "request": {
            "candidate_id": "release:spec147",
            "idempotency_key": "release:spec147:preflighted:1",
            "exact_sha": "0123456789abcdef",
            "version": "proof",
            "project_root": "/tmp/spec147",
            "topology": {
                "schema": "focusa.release_topology.v1",
                "project_id": "proof",
                "profile": "cli_library",
                "surfaces": [],
                "dependencies": [],
                "release_groups": [],
                "promotion_order": [],
                "evidence_policy": {
                    "exact_sha_required": True,
                    "required_checks": [],
                    "required_artifact_kinds": [],
                },
            },
            "stage": "preflighted",
            "surface_waves": [],
            "tuning": {
                "max_parallel_operations": 1,
                "prefer_reusable_evidence": True,
            },
            "immutable_artifact_set_id": None,
            "approval_refs": approvals or [],
        },
    }


class AdapterTests(unittest.TestCase):
    def invoke(self, body):
        return subprocess.run(
            [str(ADAPTER)],
            input=json.dumps(body),
            text=True,
            capture_output=True,
            env={},
            cwd=ROOT,
            check=False,
        )

    def test_plan_is_mutation_and_network_free(self):
        proc = self.invoke(envelope())
        self.assertEqual(proc.returncode, 0, proc.stderr)
        result = json.loads(proc.stdout)
        self.assertEqual(result["outcome"], "passed")
        self.assertTrue(result["evidence_refs"][0].startswith("github-plan:"))
        self.assertIsNone(result["artifact_set_id"])

    def test_execute_fails_closed_without_approval(self):
        proc = self.invoke(envelope(mode="execute", mutates=True))
        self.assertEqual(proc.returncode, 2)
        self.assertIn("requires explicit operator", proc.stderr)

    def test_approved_execute_does_not_mistake_dispatch_for_completion(self):
        proc = self.invoke(
            envelope(mode="execute", mutates=True, approvals=["operator:spec147-proof"])
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        result = json.loads(proc.stdout)
        self.assertEqual(result["outcome"], "blocked")
        self.assertEqual(result["reason_codes"], ["provider_execution_binding_required"])


class GovernorTests(unittest.TestCase):
    def test_fingerprint_is_stable_and_scope_sensitive(self):
        first = G.failure_fingerprint(failure())
        second = G.failure_fingerprint(failure())
        changed = G.failure_fingerprint(failure(exact_sha="fedcba9876543210"))
        self.assertEqual(first, second)
        self.assertNotEqual(first, changed)

    def test_deterministic_failure_never_retries(self):
        decision = G.decide(
            failure(
                failure_class="ci_clippy_failure",
                deterministic=True,
                retry_policy="hard_failure_no_rerun",
            ),
            [],
            G.parse_time("2026-07-27T20:00:00Z"),
        )
        self.assertFalse(decision["allowed"])
        self.assertEqual(decision["action"], "operator_review")

    def test_classifier_rerun_policy_maps_to_bounded_retry(self):
        decision = G.decide(
            failure(
                failure_class="runner_resource_failure",
                retry_policy="rerun_once",
            ),
            [],
            G.parse_time("2026-07-27T20:00:00Z"),
        )
        self.assertTrue(decision["allowed"])
        self.assertEqual(decision["action"], "rerun_failed_jobs")

    def test_one_claim_and_one_settlement(self):
        with tempfile.TemporaryDirectory() as tmp:
            ledger = Path(tmp) / "ledger.jsonl"
            now = G.parse_time("2026-07-27T20:00:00Z")
            decision, claim = G.claim(failure(), ledger, now, "policy:test")
            self.assertTrue(decision["allowed"])
            self.assertIsNotNone(claim)
            duplicate, duplicate_claim = G.claim(failure(), ledger, now, "policy:test")
            self.assertFalse(duplicate["allowed"])
            self.assertEqual(duplicate["reason"], "active_fingerprint_claim")
            self.assertIsNone(duplicate_claim)
            row = G.settle(
                ledger,
                decision["fingerprint"],
                claim["claim_id"],
                "healed",
                "test:healed",
                now,
                None,
            )
            self.assertEqual(row["status"], "healed")
            after, after_claim = G.claim(failure(), ledger, now, "policy:test")
            self.assertFalse(after["allowed"])
            self.assertEqual(after["reason"], "fingerprint_healed")
            self.assertIsNone(after_claim)

    def test_concurrent_processes_create_one_claim(self):
        with tempfile.TemporaryDirectory() as tmp:
            failure_path = Path(tmp) / "failure.json"
            ledger = Path(tmp) / "ledger.jsonl"
            failure_path.write_text(json.dumps(failure()))
            command = [
                str(GOVERNOR),
                "claim",
                "--failure",
                str(failure_path),
                "--ledger",
                str(ledger),
                "--approval-ref",
                "policy:concurrency-test",
                "--now",
                "2026-07-27T20:00:00Z",
            ]
            first = subprocess.Popen(command, text=True, stdout=subprocess.PIPE)
            second = subprocess.Popen(command, text=True, stdout=subprocess.PIPE)
            outputs = [json.loads(first.communicate()[0]), json.loads(second.communicate()[0])]
            self.assertEqual(first.returncode, 0)
            self.assertEqual(second.returncode, 0)
            self.assertEqual(sum(item["claim"] is not None for item in outputs), 1)
            claims = [row for row in G.load_rows(ledger) if row["schema"] == G.CLAIM_SCHEMA]
            self.assertEqual(len(claims), 1)


class StaticMigrationTests(unittest.TestCase):
    def test_only_governed_watchdog_has_automatic_retry_authority(self):
        quarantined = (ROOT / ".github/workflows/auto-retry-deploy.yml").read_text()
        watchdog = (ROOT / ".github/workflows/release-pipeline-watchdog.yml").read_text()
        self.assertNotIn("workflow_run:", quarantined)
        self.assertIn("self_heal_governor.py claim", watchdog)
        self.assertIn("--mutation-budget \"$MAX_HEALS\"", watchdog)
        self.assertIn("attempt >= 2", watchdog)

    def test_proposal_identity_is_fingerprint_not_run_id(self):
        workflow = (ROOT / ".github/workflows/audit-recorder.yml").read_text()
        self.assertIn("self-heal/fp-", workflow)
        self.assertNotIn('branch="self-heal/${{ github.run_id }}"', workflow)
        self.assertNotIn("--force", workflow)
        self.assertIn("open_count >= 3", workflow)
        self.assertIn('git ls-remote --exit-code origin "refs/heads/$branch"', workflow)
        self.assertIn("recovering fingerprint branch without a PR", workflow)

    def test_inventory_has_explicit_dispositions(self):
        spec = (ROOT / "docs/147-focusa-master-release-cycle-operationalization-spec.md").read_text()
        for disposition in ("KEEP", "WRAP", "QUARANTINE", "RETIRE"):
            self.assertIn(f"| {disposition} |", spec)
        for component in (
            "audit-recorder.yml",
            "auto-retry-deploy.yml",
            "release-pipeline-watchdog.yml",
            "propose-system-fix.py",
            "update runtime/CLI/scheduler",
        ):
            self.assertIn(component, spec)


if __name__ == "__main__":
    unittest.main()
