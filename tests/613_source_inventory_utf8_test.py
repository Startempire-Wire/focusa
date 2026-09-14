#!/usr/bin/env python3
"""Strict UTF-8 boundary tests; full CLI locale proof runs separately."""
import importlib.util
import json
import os
import subprocess
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]


def load(name, script):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / script)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


generator = load('route_encoding', 'generate-agent-route-classification.py')
audit = load('audit_encoding', 'audit-agent-first-tool-surfaces.py')


class Utf8Boundary(unittest.TestCase):
    def test_audit_existing_reader_preserves_unicode(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            value = 'Unicode: Ελληνικά → café'
            (root / 'source.rs').write_bytes(value.encode('utf-8'))
            with patch.object(audit, 'ROOT', root):
                self.assertEqual(audit.text('source.rs'), value)
                self.assertEqual(audit.text(root / 'source.rs'), value)

    def test_audit_existing_reader_rejects_malformed_utf8(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'source.rs').write_bytes(b'// invalid UTF-8: \xff')
            with patch.object(audit, 'ROOT', root), self.assertRaises(UnicodeDecodeError):
                audit.text('source.rs')

    def test_real_commands_in_non_utf8_locale(self):
        env = {**os.environ, 'LC_ALL': 'C', 'PYTHONUTF8': '0', 'PYTHONCOERCECLOCALE': '0'}
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / 'audit.json'
            markdown = Path(directory) / 'audit.md'
            commands = [
                ['scripts/generate-agent-route-classification.py', '--check'],
                ['scripts/audit-agent-first-tool-surfaces.py', '--strict',
                 '--json', str(report), '--markdown', str(markdown)],
            ]
            for command in commands:
                with self.subTest(command=command[0]):
                    result = subprocess.run(
                        [sys.executable, *command], cwd=ROOT, env=env,
                        capture_output=True, encoding='utf-8', timeout=240,
                    )
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(json.loads(report.read_text(encoding='utf-8'))['release_gate'], 'pass')
            self.assertIn('## Findings', markdown.read_text(encoding='utf-8'))

    def test_generator_rejects_malformed_source_before_artifact_write(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'crates/focusa-api/src'
            source.mkdir(parents=True)
            (source / 'invalid.rs').write_bytes(b'// invalid UTF-8: \xff')
            output, reference = root / 'routes.json', root / 'api.md'
            with patch.object(generator, 'ROOT', root), \
                 patch.object(generator, 'OUTPUT', output), \
                 patch.object(generator, 'API_REFERENCE', reference), \
                 patch.object(sys, 'argv', ['generate-agent-route-classification.py']), \
                 self.assertRaises(UnicodeDecodeError):
                generator.main()
            self.assertFalse(output.exists())
            self.assertFalse(reference.exists())


if __name__ == '__main__':
    unittest.main()
