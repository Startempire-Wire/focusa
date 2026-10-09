#!/usr/bin/env python3
"""Targeted isolated-input tests; these fixtures do not establish MSI execution."""
import hashlib
import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import unittest
import json
import os
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
def load(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts/ci' / file)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

NATIVE = load('native_msi', 'prepare-native-windows-tauri.py')
WINE = load('wine_msi', 'prepare-windows-msi-wine.py')
CONTROLLER = load('msi_controller', 'run-windows-ovh-release.py')

class NativeMsiInputTests(unittest.TestCase):
    def test_packaging_budget_does_not_relax_full_build_budget(self):
        measured = {'used_percent': 90.66, 'free_gib': 17.97}
        with self.assertRaises(ValueError):
            CONTROLLER.require_headroom(measured)
        CONTROLLER.require_headroom(measured, packaging_only=True)
        for insufficient in [{'used_percent': 95, 'free_gib': 12}, {'used_percent': 90, 'free_gib': 9.99}]:
            with self.assertRaises(ValueError):
                CONTROLLER.require_headroom(insufficient, packaging_only=True)
    def test_windows_paths_are_explicit_and_absolute(self):
        self.assertEqual(NATIVE.windows_path('/tmp/example'), 'Z:\\tmp\\example')
    def test_archive_escape_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            with tarfile.open(base / 'bad.tar', 'w') as archive:
                member = tarfile.TarInfo('../outside')
                member.size = 1
                archive.addfile(member, io.BytesIO(b'x'))
            with self.assertRaises(ValueError):
                NATIVE.unpack_tar(base / 'bad.tar', base / 'destination')
            self.assertFalse((base / 'outside').exists())
    def test_locked_sha512_and_sha256_evidence_agree(self):
        data = b'fixture bytes, not a packaging tool'
        with tempfile.TemporaryDirectory() as folder, patch.object(WINE.urllib.request, 'urlopen', return_value=io.BytesIO(data)):
            receipt = WINE.download('https://example.invalid', Path(folder) / 'fixture', hashlib.sha512(data).hexdigest(), algorithm='sha512')
            self.assertTrue(receipt['external_checksum_verified'])
            self.assertEqual(receipt['sha256'], hashlib.sha256(data).hexdigest())
    def test_completed_tool_cleanup_retains_receipt_and_preserves_running_jobs(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            tools = base / 'focusa-windows-pipeline-123-1/msi-tools'
            tools.mkdir(parents=True)
            receipt = {'kind': 'ovh_wine_wix_toolchain', 'prefix': str(tools / 'prefix')}
            (tools / 'toolchain-receipt.json').write_text(json.dumps(receipt))
            with patch.dict(os.environ, {'GITHUB_REPOSITORY': 'example/focusa'}), patch.object(CONTROLLER.subprocess, 'check_output', return_value='in_progress\n'):
                CONTROLLER.reclaim_completed_msi_tools(base, base / 'retained', '456')
                self.assertTrue(tools.exists())
            with patch.dict(os.environ, {'GITHUB_REPOSITORY': 'example/focusa'}), patch.object(CONTROLLER.subprocess, 'check_output', return_value='completed\n'):
                CONTROLLER.reclaim_completed_msi_tools(base, base / 'retained', '456')
                self.assertFalse(tools.exists())
                self.assertEqual(json.loads((base / 'retained/toolchain-evidence/focusa-windows-pipeline-123-1/toolchain-receipt.json').read_text()), receipt)
    def test_native_tool_tampering_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder, patch.object(WINE.urllib.request, 'urlopen', return_value=io.BytesIO(b'changed')):
            with self.assertRaises(ValueError):
                WINE.download('https://example.invalid', Path(folder) / 'fixture', '0' * 128, algorithm='sha512')

if __name__ == '__main__':
    unittest.main()
