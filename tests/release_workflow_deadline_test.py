#!/usr/bin/env python3
"""Run canonical workflow-wait functions with simulated time and GitHub replies."""

import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "scripts/create-dev-release-tag.sh").read_text()
PREFIX = SOURCE.split("push_candidate_main_with_auto_rebase() {", 1)[0]


def function(name):
    match = re.search(r"(?ms)^" + name + r"\(\) \{\n.*?^\}", SOURCE)
    assert match, name
    return match.group(0)


class ReleaseWorkflowDeadlineTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="focusa-release-clock-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.events = self.root / "events.jsonl"
        gh = self.root / "gh"
        gh.write_text('''#!/usr/bin/python3
import json, os, pathlib, sys
args = sys.argv[1:]
p = pathlib.Path(os.environ["EVENTS"])
events = [json.loads(l) for l in p.read_text().splitlines()] if p.exists() else []
with p.open("a") as out:
    out.write(json.dumps(args) + "\\n")
if args[:2] == ["run", "list"]:
    assert args[args.index("--commit") + 1] == os.environ["TEST_SHA"]
    assert args[args.index("--workflow") + 1] == os.environ["TEST_WORKFLOW"]
    n = sum(e[:2] == ["run", "list"] for e in events)
    found = n >= int(os.environ.get("DISCOVERY_MISSES", "0"))
    if "--jq" in args:
        print("42" if found else "")
    else:
        print(json.dumps([{"databaseId": 99, "headBranch": "wrong-tag"},
                          {"databaseId": 42, "headBranch": os.environ["TEST_TAG"]}]
                         if found else []))
elif args[:3] == ["run", "view", "42"]:
    if "--jq" in args:
        print("bounded timeout diagnostics")
    else:
        n = sum(e[:3] == ["run", "view", "42"] and "--jq" not in e for e in events)
        mode = os.environ["SCENARIO"]
        done = mode != "forever" and n >= 2
        print(json.dumps({"status": "completed" if done else "in_progress",
                          "conclusion": mode if done else "",
                          "url": "https://example.invalid/run/42", "jobs": []}))
else:
    raise SystemExit("unexpected API call: " + repr(args))
''')
        gh.chmod(0o755)
        self.probe = self.root / "probe.sh"
        self.probe.write_text(
            PREFIX + "\n" + function("watch_workflow_run_bounded") + "\n" + function("wait_for_workflow") + '''
report_workflow_failure() { echo "REAL_TERMINAL_FAILURE $1 $2" >&2; }
sleep() { SECONDS=$((SECONDS + 10)); }
SECONDS=0
wait_for_workflow "$TEST_WORKFLOW" "$TEST_SHA" "$TEST_TAG"
''')

    def run_probe(self, scenario="success", args=(), workflow="Release", misses=0):
        env = dict(os.environ, PATH=str(self.root) + ":" + os.environ["PATH"],
                   EVENTS=str(self.events), SCENARIO=scenario, TEST_SHA="a" * 40,
                   TEST_TAG="v0.9.198" if workflow == "Release" else "",
                   TEST_WORKFLOW=workflow, DISCOVERY_MISSES=str(misses))
        result = subprocess.run(["bash", str(self.probe), "--ci-timeout", "1", *args],
                                capture_output=True, text=True, env=env, timeout=10)
        return result, result.stdout + result.stderr

    def test_release_succeeds_after_source_ci_deadline(self):
        result, output = self.run_probe()
        self.assertEqual(result.returncode, 0, output)
        self.assertIn("workflow_completed name=Release", output)
        self.assertIn("conclusion=success", output)
        self.assertNotIn("workflow_timeout", output)

    def test_release_terminal_failure_is_not_misreported_as_timeout(self):
        result, output = self.run_probe("failure")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("REAL_TERMINAL_FAILURE Release 42", output)
        self.assertNotIn("workflow_timeout", output)

    def test_release_deadline_is_finite_and_reports_its_own_budget(self):
        result, output = self.run_probe("forever", ("--release-timeout", "25"))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("workflow_timeout name=Release run_id=42 timeout_s=25", output)
        self.assertIn("bounded timeout diagnostics", output)
        self.assertNotIn("REAL_TERMINAL_FAILURE", output)

    def test_discovery_does_not_renew_release_budget(self):
        result, output = self.run_probe(args=("--release-timeout", "25"), misses=2)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("workflow_timeout name=Release run_id=42 timeout_s=25", output)
        events = [json.loads(line) for line in self.events.read_text().splitlines()]
        views = [e for e in events if e[:3] == ["run", "view", "42"] and "--jq" not in e]
        self.assertEqual(len(views), 1, output)

    def test_missing_release_run_expires_discovery(self):
        result, output = self.run_probe(args=("--release-timeout", "25"), misses=100)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("workflow_discovery_timeout name=Release", output)
        self.assertIn("timeout_s=25", output)

    def test_source_ci_budget_remains_short(self):
        result, output = self.run_probe(workflow="CI")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("workflow_timeout name=CI run_id=42 timeout_s=1", output)

    def test_invalid_release_budgets_fail_before_api_calls(self):
        for value in ("0", "-1", "abc", "01", "999999999999999999999999"):
            with self.subTest(value=value):
                result, output = self.run_probe(args=("--release-timeout", value))
                self.assertEqual(result.returncode, 2, output)
                self.assertIn("Invalid --release-timeout", output)
                self.assertFalse(self.events.exists())

    def test_default_covers_external_jobs_and_bounded_margin(self):
        match = re.search(r"^RELEASE_TIMEOUT_SECS=(\d+)$", SOURCE, re.M)
        self.assertIsNotNone(match)
        budget = int(match.group(1))
        workflow = (ROOT / ".github/workflows/release.yml").read_text()
        longest_job = max(map(int, re.findall(r"timeout-minutes:\s*(\d+)", workflow)))
        self.assertGreaterEqual(budget, longest_job * 60 + 1800)
        self.assertLessEqual(budget, 86400)


if __name__ == "__main__":
    unittest.main(verbosity=2)
