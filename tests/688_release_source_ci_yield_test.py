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


def phase(directory, runs, wait='0', provider_error=False, remote_main=A,
          dispatch_error=False, after_dispatch=None, channel='stable',
          remote_query_error=False, raw_projection=None):
    directory = Path(directory)
    fixture = directory / 'runs.json'
    fixture.write_text(raw_projection if raw_projection is not None else json.dumps(runs))
    after = directory / 'after-dispatch.json'
    after.write_text(json.dumps(after_dispatch))
    effects, calls = directory / 'effects', directory / 'calls'
    first = SOURCE.index('read_source_workflow_runs() {') if 'read_source_workflow_runs() {' in SOURCE else SOURCE.index('ensure_source_workflow() {')
    last = SOURCE.index('\nreport_workflow_failure()', first)
    start = SOURCE.index('if [[ "$PUSH" -eq 1 ]]; then\n  push_candidate_main_with_auto_rebase')
    end = SOURCE.index('echo "Created tag ', start)
    trap_start = SOURCE.index('journal_problem_on_error() {')
    trap_end = SOURCE.index('\n\nif [[ -n "$(git status --porcelain)" ]]', trap_start)
    shell = '''set -euo pipefail
WAIT_CI="$MODE"; CI_TIMEOUT_SECS=60; PUSH=1; RELEASE_CHANNEL="$CHANNEL"; RELEASE_JOURNAL_ACTIVE=1; FORCE_RELEASE=0; TAG=v99.0.0; PREVIOUS_TAG=v98.0.0
gh() {
  printf '%s\\n' "$*" >> "$CALLS"
  case "$1 $2" in
    'run list')
      if [ "$PROVIDER_ERROR" = 1 ]; then echo 'owned provider outage' >&2; return 4; fi
      /usr/bin/python3 -c 'import os;print(open(os.environ["FIXTURE"]).read())';;
    'workflow run')
      if [ "$DISPATCH_ERROR" = 1 ]; then echo 'owned dispatch rejected' >&2; return 5; fi
      echo workflow_dispatched >> "$EFFECTS"
      if [ "$REPLACE_AFTER_DISPATCH" = 1 ]; then cp "$AFTER_DISPATCH" "$FIXTURE"; fi;;
    *) echo unexpected_fixture_gh >&2; return 90;;
  esac
}
sleep() { echo runner_wait >> "$EFFECTS"; return 99; }
push_candidate_main_with_auto_rebase() { echo candidate_pushed >> "$EFFECTS"; }
journal_client() { echo "journal:$1" >> "$EFFECTS"; }
git() {
  printf 'git %s\\n' "$*" >> "$CALLS"
  case "$1" in
    rev-parse) echo "$CANDIDATE_SHA";;
    ls-remote)
      if [ "$REMOTE_QUERY_ERROR" = 1 ]; then echo 'owned remote lookup outage' >&2; return 4; fi
      printf '%s\\trefs/heads/main\\n' "$REMOTE_MAIN";;
    diff) echo docs/contracts/spec141/generated-capability-v2/distribution-manifest.json;;
    tag) echo "tag:$CANDIDATE_SHA" >> "$EFFECTS";;
    *) echo unexpected_fixture_git >&2; return 90;;
  esac
}
''' + SOURCE[first:last] + SOURCE[trap_start:trap_end] + '\n' + SOURCE[start:end]
    env = dict(os.environ, MODE=wait, FIXTURE=str(fixture), EFFECTS=str(effects),
               CALLS=str(calls), CANDIDATE_SHA=A, PROVIDER_ERROR=str(int(provider_error)),
               REMOTE_MAIN=remote_main, DISPATCH_ERROR=str(int(dispatch_error)),
               AFTER_DISPATCH=str(after), REPLACE_AFTER_DISPATCH=str(int(after_dispatch is not None)),
               CHANNEL=channel, REMOTE_QUERY_ERROR=str(int(remote_query_error)))
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
                self.assertEqual(result.returncode, 78 if not data else 1)
                self.assertNotIn('tag:' + A, effects)
                if data:
                    self.assertNotIn('workflow run', (Path(directory) / 'calls').read_text())

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
            self.assertNotIn('workflow run', (Path(directory) / 'calls').read_text())
            self.assertNotIn('tag:' + A, effects)

    def test_invalid_query_projection_is_refused(self):
        for data in [{}, [{'status': 'completed', 'conclusion': 'success'}]]:
            with self.subTest(data=data), tempfile.TemporaryDirectory() as directory:
                result, effects, _ = phase(directory, data)
                self.assertEqual(result.returncode, 1)
                self.assertIn('source_gate_unavailable', result.stderr)
                self.assertNotIn('tag:' + A, effects)

    def test_malformed_json_cannot_dispatch_or_qualify_candidate(self):
        with tempfile.TemporaryDirectory() as directory:
            result, effects, calls = phase(directory, [], raw_projection='{')
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertIn('source_gate_unavailable', result.stderr)
            self.assertNotIn('workflow run', calls)
            self.assertNotIn('tag:' + A, effects)

    def test_explicit_wait_mode_retains_its_wait(self):
        with tempfile.TemporaryDirectory() as directory:
            result, effects, _ = phase(directory, [run(A, 'queued')], wait='1')
            self.assertEqual(result.returncode, 99)
            self.assertIn('runner_wait', effects)
            self.assertNotIn('tag:' + A, effects)

    def test_missing_docs_only_candidate_dispatches_once_then_yields_without_tag(self):
        with tempfile.TemporaryDirectory() as directory:
            result, effects, calls = phase(directory, [])
            self.assertEqual(result.returncode, 78, result.stderr)
            self.assertEqual(effects, ['candidate_pushed', 'workflow_dispatched'])
            self.assertEqual(calls.count('workflow run CI --ref main'), 1)
            self.assertIn('git ls-remote origin refs/heads/main', calls)
            self.assertNotIn('tag:' + A, effects)
            self.assertNotIn('journal:progress', effects)

    def test_dispatched_docs_only_candidate_needs_exact_success_before_tag(self):
        with tempfile.TemporaryDirectory() as directory:
            result, effects, calls = phase(directory, [], after_dispatch=[run(A, 'completed', 'success')])
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(effects, ['candidate_pushed', 'workflow_dispatched', 'journal:progress', 'tag:' + A])
            self.assertEqual(calls.count('workflow run CI --ref main'), 1)
            self.assertLess(calls.index('--commit ' + A), calls.index('git tag'))

    def test_existing_pending_or_success_never_dispatches_duplicate(self):
        for status, conclusion, code in [('queued', None, 78), ('in_progress', None, 78), ('completed', 'success', 0)]:
            with self.subTest(status=status), tempfile.TemporaryDirectory() as directory:
                result, effects, calls = phase(directory, [run(A, status, conclusion)])
                self.assertEqual(result.returncode, code, result.stderr)
                self.assertNotIn('workflow run', calls)
                self.assertNotIn('workflow_dispatched', effects)

    def test_foreign_remote_main_or_failed_lookup_cannot_dispatch_or_tag(self):
        for options in [{'remote_main': B}, {'remote_query_error': True}]:
            with self.subTest(options=options), tempfile.TemporaryDirectory() as directory:
                result, effects, calls = phase(directory, [], **options)
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertNotIn('workflow run', calls)
                self.assertNotIn('tag:' + A, effects)
                self.assertIn('journal:problem', effects)

    def test_rejected_dispatch_does_not_become_pending_success_or_tag(self):
        with tempfile.TemporaryDirectory() as directory:
            result, effects, calls = phase(directory, [], dispatch_error=True)
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertIn('owned dispatch rejected', result.stderr)
            self.assertIn('source_gate_dispatch_failed', result.stderr)
            self.assertEqual(calls.count('workflow run CI --ref main'), 1)
            self.assertNotIn('workflow_dispatched', effects)
            self.assertNotIn('tag:' + A, effects)

    def test_preview_keeps_advisory_boundary_without_source_ci_dispatch(self):
        with tempfile.TemporaryDirectory() as directory:
            result, effects, calls = phase(directory, [], channel='preview')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertNotIn('workflow run', calls)
            self.assertNotIn('run list', calls)
            self.assertIn('tag:' + A, effects)

    def test_single_builder_yields_then_finalizes_same_green_candidate(self):
        with tempfile.TemporaryDirectory() as directory:
            pending, effects, _ = phase(directory, [run(A, 'queued')])
            self.assertEqual(pending.returncode, 78, pending.stderr)
            self.assertNotIn('tag:' + A, effects)
            first = controller_fixture.ControllerEditorialOverrideTest()
            code, pending_args, pending_version, dispatched, duplicate = first.exercise(helper_exit=pending.returncode)
            self.assertEqual(code, 78, first.last_output)
            self.assertFalse(dispatched)
            self.assertFalse(duplicate)
            green, effects, _ = phase(directory, [run(A, 'completed', 'success')])
            self.assertEqual(green.returncode, 0, green.stderr)
            self.assertEqual(effects.count('tag:' + A), 1)
            second = controller_fixture.ControllerEditorialOverrideTest()
            code, ready_args, ready_version, dispatched, duplicate = second.exercise(helper_exit=green.returncode)
            self.assertEqual(code, 0, second.last_output)
            self.assertTrue(dispatched)
            self.assertFalse(duplicate)
            self.assertEqual(pending_args, ready_args)
            self.assertEqual(pending_version, ready_version)
            self.assertIn('--no-wait-ci', ready_args)

    def test_actual_controller_cannot_finalize_failed_or_foreign_candidate(self):
        for runs in [[run(A, 'completed', 'failure')], [run(B, 'completed', 'success')]]:
            with self.subTest(runs=runs), tempfile.TemporaryDirectory() as directory:
                refused, effects, _ = phase(directory, runs)
                self.assertEqual(refused.returncode, 1, refused.stderr)
                self.assertNotIn('tag:' + A, effects)
                owner = controller_fixture.ControllerEditorialOverrideTest()
                code, _, _, dispatched, duplicate = owner.exercise(helper_exit=refused.returncode)
                self.assertEqual(code, 1, owner.last_output)
                self.assertFalse(dispatched)
                self.assertFalse(duplicate)

    def test_actual_controller_propagates_pending_without_release_dispatch(self):
        owner = controller_fixture.ControllerEditorialOverrideTest()
        code, args, _, dispatched, duplicate = owner.exercise(helper_exit=78)
        self.assertEqual(code, 78, owner.last_output)
        self.assertIn('--no-wait-ci', args)
        self.assertFalse(dispatched)
        self.assertFalse(duplicate)


if __name__ == '__main__':
    unittest.main()
