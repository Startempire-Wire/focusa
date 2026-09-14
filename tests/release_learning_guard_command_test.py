#!/usr/bin/env python3
"""Release checks reuse the installed Cargo route without a local toolchain."""
import importlib.util
import os
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('release_guards', ROOT / 'scripts/run-release-learning-guards.py')
guards = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guards)


class CommandResolutionTest(unittest.TestCase):
    @patch.dict(os.environ, {}, clear=True)
    def test_installed_route_needs_no_local_cargo_probe(self):
        with patch.object(guards.shutil, 'which', side_effect=['/tools/focusa-command-route', None]), patch.object(guards.subprocess, 'run') as run:
            self.assertEqual(guards.resolve_command(['cargo', 'test', '-p', 'focusa-core']),
                             ['/tools/focusa-command-route', 'cargo', '/usr/bin/cargo', 'test', '-p', 'focusa-core'])
            run.assert_not_called()

    @patch.dict(os.environ, {'FOCUSA_RELEASE_CARGO': '/tools/approved-cargo'}, clear=True)
    def test_explicit_override_precedes_unavailable_cargo(self):
        with patch.object(guards.subprocess, 'run') as run:
            self.assertEqual(guards.resolve_command(['cargo', 'test']), ['/tools/approved-cargo', 'test'])
            run.assert_not_called()

    @patch.dict(os.environ, {}, clear=True)
    def test_missing_local_cargo_can_use_existing_rustup(self):
        resolved = subprocess.CompletedProcess([], 0, stdout='/tools/cargo', stderr='')
        with patch.object(guards.shutil, 'which', return_value=None), patch.object(guards.subprocess, 'run', side_effect=[PermissionError('cargo'), resolved]):
            self.assertEqual(guards.resolve_command(['cargo', 'test']), ['rustup', 'run', 'nightly', 'cargo', 'test'])

    def test_other_commands_are_unchanged(self):
        self.assertEqual(guards.resolve_command(['bash', 'existing-check.sh']), ['bash', 'existing-check.sh'])


if __name__ == '__main__':
    unittest.main()
