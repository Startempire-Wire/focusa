#!/usr/bin/env python3
"""Exercise the actual gate fixture and cleanup without building or starting a daemon."""
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / 'scripts/ci/run-spec-gates.sh').read_text()
START = SOURCE.index('TEST_GIT_DIR=""')
FIXTURE = SOURCE[START:SOURCE.index('if [[ "$FOCUSA_TEST_MODE" == "1" &&', START)]
CLEANUP = SOURCE[SOURCE.index('cleanup() {'):SOURCE.index('\ntrap cleanup EXIT')]


class GitFixtureTest(unittest.TestCase):
    def exercise(self, existing):
        with tempfile.TemporaryDirectory(prefix='focusa-git-fixture-test-') as tmp:
            root = Path(tmp) / 'source'
            root.mkdir()
            env = {'PATH': '/usr/bin:/bin', 'HOME': tmp, 'ROOT_DIR': str(root),
                   'FOCUSA_TEST_MODE': '1', 'GIT_CONFIG_NOSYSTEM': '1',
                   'GIT_CONFIG_GLOBAL': '/dev/null'}
            if existing:
                subprocess.run(['git', 'init', '-q', str(root)], env=env, check=True)
                subprocess.run(['git', '-C', str(root), '-c', 'user.name=test',
                                '-c', 'user.email=test@invalid', 'commit', '--allow-empty',
                                '-qm', 'real fixture'], env=env, check=True)
                env['GIT_DIR'] = str(root / '.git')
                env['GIT_WORK_TREE'] = str(root)
            code = 'set -euo pipefail\n' + FIXTURE + '\ngit rev-list --count HEAD\n'
            code += '''
DAEMON_PID=unused
TEST_BEADS_FIXTURE=""
FOCUSA_DATA_DIR="$ROOT_DIR/data"
mkdir -p "$FOCUSA_DATA_DIR"
kill() { :; }
cleanup_ephemeral_builds() { printf 'remaining_git_dir=%s\\n' "${GIT_DIR-unset}"; }
'''
            code += CLEANUP + '\ncleanup\n'
            result = subprocess.run(['bash', '-c', code], env=env, cwd=root,
                                    text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertNotIn('fatal:', result.stderr)
            self.assertEqual(result.stdout.splitlines()[0], '1' if existing else '2')
            expected = str(root / '.git') if existing else 'unset'
            self.assertIn('remaining_git_dir=' + expected, result.stdout)
            self.assertEqual((root / '.git').exists(), existing)
            self.assertFalse(list(Path(tmp).glob('gate-git-meta.*')))

    def test_historyless_fixture_and_cleanup(self):
        self.exercise(False)

    def test_existing_repository_is_preserved(self):
        self.exercise(True)


if __name__ == '__main__':
    unittest.main()
