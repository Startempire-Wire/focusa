#!/usr/bin/env python3
"""Focused contract regressions for the OVH Windows cross-compile adapter."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

import yaml

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('windows_xwin', ROOT / 'scripts/ci/build-windows-xwin-release.py')
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class WindowsXwinContractTests(unittest.TestCase):
    def test_real_contract_derives_all_packages_surfaces_and_targets(self):
        packages, surfaces, targets, environment = MODULE.recipe(ROOT)
        self.assertEqual(set(packages), {'focusa-cli', 'focusa-api', 'focusa-session-runner', 'focusa-tui'})
        self.assertEqual(set(surfaces), {'focusa', 'focusa-daemon', 'focusa-session-runner', 'focusa-tui'})
        self.assertEqual(targets, ['x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc'])
        self.assertIn('authority-root-20260907-0055c689', environment['FOCUSA_AUTHORITY_ROOT_KEYS_JSON'])
        self.assertEqual(environment['CARGO_PROFILE_RELEASE_LTO'], 'false')

    def test_release_automatically_calls_existing_windows_producer(self):
        # BaseLoader preserves YAML's `on` key (rather than YAML 1.1 boolean).
        release = yaml.load((ROOT / '.github/workflows/release.yml').read_text(), Loader=yaml.BaseLoader)
        producer = yaml.load((ROOT / '.github/workflows/windows-ovh-build.yml').read_text(), Loader=yaml.BaseLoader)
        job = release['jobs']['windows-ovh-executables']
        self.assertEqual(job['needs'], 'create-release')
        self.assertEqual(job['uses'], './.github/workflows/windows-ovh-build.yml')
        self.assertEqual(set(job['with']), {'release_tag', 'release_sha'})
        self.assertIn('workflow_call', producer['on'])
        self.assertIn('windows-ovh-executables', release['jobs']['external-rust-binaries']['needs'])
        ci = yaml.load((ROOT / '.github/workflows/ci.yml').read_text(), Loader=yaml.BaseLoader)
        self.assertTrue(any('windows_xwin_release_adapter_test.py' in step.get('run', '')
                            for step in ci['jobs']['release-automation-static']['steps']))
        queue = release['jobs']['queue-appveyor-windows']['steps']
        self.assertEqual(queue[0]['uses'], 'actions/checkout@v6')
        self.assertIn('export APPVEYOR_ACCOUNT APPVEYOR_SLUG', queue[1]['run'])

    def test_full_release_still_requires_desktop_and_signed_trust(self):
        release = yaml.load((ROOT / '.github/workflows/release.yml').read_text(), Loader=yaml.BaseLoader)
        self.assertIn('external-menubar-receipts', release['jobs']['checksums']['needs'])
        self.assertEqual(release['jobs']['dispatch-deploy-live-daemon']['needs'], 'checksums')
        steps = release['jobs']['checksums']['steps']
        self.assertTrue(any('release-trust-metadata.py' in step.get('run', '') for step in steps))

    def altered_contract(self, transform):
        contract = yaml.safe_load((ROOT / '.appveyor.yml').read_text())
        transform(contract)
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        folder = Path(temporary.name)
        (folder / '.appveyor.yml').write_text(yaml.safe_dump(contract))
        return folder

    def test_missing_production_roots_rejected(self):
        folder = self.altered_contract(lambda c: c['environment'].pop('FOCUSA_AUTHORITY_ROOT_KEYS_JSON'))
        with self.assertRaisesRegex(ValueError, 'production authority roots missing'):
            MODULE.recipe(folder)

    def test_duplicate_target_rejected(self):
        def duplicate(contract):
            contract['environment']['matrix'].append(contract['environment']['matrix'][0].copy())
        with self.assertRaisesRegex(ValueError, 'ambiguous canonical Windows matrix'):
            MODULE.recipe(self.altered_contract(duplicate))

    def test_staging_adapter_cannot_upload_publish_or_claim_native_acceptance(self):
        adapter = (ROOT / 'scripts/ci/build-windows-xwin-release.py').read_text()
        self.assertNotIn('gh release upload', adapter)
        self.assertNotIn('gh release edit', adapter)
        self.assertIn("'native_windows_proof': False", adapter)
        self.assertIn("'installer_proof': False", adapter)
        self.assertIn("'published': False", adapter)
        self.assertIn('verify-embedded-authority-root.py', adapter)
        self.assertIn("'status', '--porcelain'", adapter)


if __name__ == '__main__':
    unittest.main()
