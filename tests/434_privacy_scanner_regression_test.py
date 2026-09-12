#!/usr/bin/env python3
"""Exercise the actual scanner with exact paths, positives and tool failures."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCANNER = Path("tests/security_persisted_state_privacy_static_test.sh")
PREREQUISITES = [SCANNER, Path("docs/current/PERSISTED_STATE_PRIVACY_CLASSES.md"),
    Path("docs/focusa-tools/tools/focusa_predict_record.md"),
    Path("docs/focusa-tools/tools/focusa_metacog_capture.md"),
    Path("apps/pi-extension/src/tools.ts")]
# Construct only a synthetic marker: no private-key payload is used or emitted.
MARKER = "-" * 5 + "BEGIN PRIVATE KEY" + "-" * 5 + "\n"


class PrivacyScannerTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="focusa-privacy-regression-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for path in PREREQUISITES:
            dest = self.root / path
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, dest)

    def run_gate(self, env=None):
        return subprocess.run(["bash", str(self.root / SCANNER)], cwd=self.root,
            env=env, capture_output=True, text=True, timeout=30)

    def marker(self, path):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(MARKER)
        return target

    def test_baseline_and_exact_exclusions(self):
        self.assertEqual(self.run_gate().returncode, 0)
        for path in ["crates/focusa-core/src/silent_sessions/runner_security_test.rs",
                     "docs/evidence/PUBLIC_DOCS_RELEASE_SYNC_2026-05-26.md"]:
            self.marker(path)
        # The gate itself contains its pattern, exercising the third exclusion.
        result = self.run_gate()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_source_docs_and_neighboring_fixture_are_not_excluded(self):
        for path in ["crates/probe.rs", "docs/probe.md",
                     "crates/focusa-core/src/silent_sessions/neighbor.rs",
                     "docs/evidence/neighbor.md", "tests/neighbor.sh"]:
            with self.subTest(path=path):
                target = self.marker(path)
                result = self.run_gate()
                self.assertEqual(result.returncode, 1)
                self.assertIn(str(target), result.stderr)
                self.assertNotIn(MARKER.strip(), result.stderr)
                target.unlink()

    def test_scanner_failure_is_not_a_clean_scan(self):
        tools = self.root / "fake-tools"
        tools.mkdir()
        fake = tools / "rg"
        fake.write_text("#!/bin/sh\necho injected-scanner-error >&2\nexit 2\n")
        fake.chmod(0o700)
        env = dict(os.environ, PATH=str(tools) + os.pathsep + os.environ["PATH"])
        result = self.run_gate(env)
        self.assertEqual(result.returncode, 2)
        self.assertIn("injected-scanner-error", result.stderr)
        self.assertIn("privacy check incomplete", result.stderr)


if __name__ == "__main__":
    unittest.main()
