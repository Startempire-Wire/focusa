"""Actual canonical helper/controller shells; isolated provider/git boundaries only."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
import release_controller_editorial_override_test as controller_fixture

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / 'scripts/create-dev-release-tag.sh').read_text()
A = 'a' * 40
B = 'b' * 40


def phase(directory, runs, wait='0', provider_error=False):
    directory = Path(directory)
    fixture = directory / 'runs.json'
    fixture.write_text(json.dumps(runs))
    effects, calls = directory / 'effects', directory / 'calls'
    first = SOURCE.index('wait_for_source_workflow() {')
    last = SOURCE.index('\nreport_workflow_failure()', first)
    start = SOURCE.index('if [[ "$PUSH" -eq 1 ]]; then\n  push_candidate_main_with_auto_rebase')
    end = SOURCE.index('echo "Created tag ', start)
    trap_start = SOURCE.index('journal_problem_on_error() {')
    trap_end = SOURCE.index('\n\nif [[ -n "$(git status --porcelain)" ]]', trap_start)
    shell = '''set -euo pipefail
WAIT_CI="$MODE"; CI_TIMEOUT_SECS=60; PUSH=1; RELEASE_CHANNEL=stable; RELEASE_JOURNAL_ACTIVE=1; FORCE_RELEASE=0; TAG=v99.0.0; PREVIOUS_TAG=v98.0.0
gh() { printf '%s\\n' "$*" >> "$CALLS"; if [ "$PROVIDER_ERROR" = 1 ]; then echo 'owned provider outage' >&2; return 4; fi; /usr/bin/python3 -c 'import os;print(open(os.environ["FIXTURE"]).read())'; }
sleep() { echo runner_wait >> "$EFFECTS"; return 99; }
push_candidate_main_with_auto_rebase() { echo candidate_pushed >> "$EFFECTS"; }
journal_client() { echo "journal:$1" >> "$EFFECTS"; }
git() { case "$1" in rev-parse) echo "$CANDIDATE_SHA";; diff) :;; tag) echo "tag:$CANDIDATE_SHA" >> "$EFFECTS";; *) echo unexpected_fixture_git >&2; return 90;; esac; }
''' + SOURCE[first:last] + SOURCE[trap_start:trap_end] + '\n' + SOURCE[start:end]
    env = dict(os.environ, MODE=wait, FIXTURE=str(fixture), EFFECTS=str(effects),
               CALLS=str(calls), CANDIDATE_SHA=A, PROVIDER_ERROR=str(int(provider_error)))
    syntax = subprocess.run(['bash', '-n'], input=shell, env=env, text=True,
                            capture_output=True, timeout=5)
    syntax.check_returncode()  # A broken fixture cannot count as a product negative.
    result = subprocess.run(['bash', '-c', shell], env=env, text=True,
                            capture_output=True, timeout=5)
    return (result, effects.read_text().splitlines() if effects.exists() else [],
            calls.read_text() if calls.exists() else '')


def run(sha, status, conclusion=None):
    return {'headSha': sha, 'status': status, 'conclusion': conclusion,
            'url': 'https://fixture.invalid/run'}


class SourceCiYield(unittest.TestCase):
    def test_pending_nonwait_phase_yields_only_builder_without_tag(self):
        with tempfile.TemporaryDirectory() as directory:
            result, effects, calls = phase(directory, [run(A, 'queued')])
            if result.returncode != 78:
                self.assertEqual(result.returncode, 99, result.stderr)
                self.assertIn('runner_wait', effects)
            self.assertEqual(result.returncode, 78,
                             'single builder was retained by the no-wait source gate')
            self.assertEqual(effects, ['candidate_pushed'])
            self.assertIn('source_gate_pending', result.stderr)
            self.assertIn(A, result.stderr)
            self.assertIn('--commit ' + A, calls)

    def test_one_slot_resume_finalizes_only_same_completed_candidate(self):
        with tempfile.TemporaryDirectory() as directory:
            pending, _, _ = phase(directory, [run(A, 'in_progress')])
            self.assertEqual(pending.returncode, 78)
            (Path(directory) / 'effects').unlink()  # Fixture effects, never source or a live tag.
            green, effects, _ = phase(directory, [run(A, 'completed', 'success')])
            self.assertEqual(green.returncode, 0, green.stderr)
            self.assertEqual(effects, ['candidate_pushed', 'journal:progress', 'tag:' + A])

    def test_empty_or_foreign_success_never_qualifies_candidate(self):
        for data in [[], [run(B, 'completed', 'success')]]:
            with self.subTest(data=data), tempfile.TemporaryDirectory() as directory:
                result, effects, _ = phase(directory, data)
                self.assertEqual(result.returncode, 78)
                self.assertNotIn('tag:' + A, effects)

    def test_exact_failed_ci_stays_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            result, effects, _ = phase(directory, [run(A, 'completed', 'failure')])
            self.assertEqual(result.returncode, 1)
            self.assertNotIn('tag:' + A, effects)
            self.assertIn('source_gate_failed', result.stderr)
            self.assertIn('journal:problem', effects)

    def test_query_failure_is_not_pending_or_an_empty_queue(self):
        with tempfile.TemporaryDirectory() as directory:
            result, effects, _ = phase(directory, [], provider_error=True)
            self.assertEqual(result.returncode, 1)
            self.assertIn('owned provider outage', result.stderr)
            self.assertIn('source_gate_unavailable', result.stderr)
            self.assertNotIn('tag:' + A, effects)

    def test_invalid_query_projection_is_refused(self):
        for data in [{}, [{'status': 'completed', 'conclusion': 'success'}]]:
            with self.subTest(data=data), tempfile.TemporaryDirectory() as directory:
                result, effects, _ = phase(directory, data)
                self.assertEqual(result.returncode, 1)
                self.assertIn('source_gate_unavailable', result.stderr)
                self.assertNotIn('tag:' + A, effects)

    def test_explicit_wait_mode_retains_its_wait(self):
        with tempfile.TemporaryDirectory() as directory:
            result, effects, _ = phase(directory, [run(A, 'queued')], wait='1')
            self.assertEqual(result.returncode, 99)
            self.assertIn('runner_wait', effects)
            self.assertNotIn('tag:' + A, effects)

    def test_actual_controller_propagates_pending_without_release_dispatch(self):
        owner = controller_fixture.ControllerEditorialOverrideTest()
        code, args, _, dispatched, duplicate = owner.exercise(helper_exit=78)
        self.assertEqual(code, 78, owner.last_output)
        self.assertIn('--no-wait-ci', args)
        self.assertFalse(dispatched)
        self.assertFalse(duplicate)


if __name__ == '__main__':
    unittest.main()
