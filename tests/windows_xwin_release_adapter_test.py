#!/usr/bin/env python3
"""Focused contract regressions for the OVH Windows cross-compile adapter."""
import base64
import contextlib
import importlib.util
import io
import json
import subprocess
from types import SimpleNamespace
from unittest.mock import patch
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
        self.assertNotIn('queue-appveyor-windows', release['jobs']['create-release']['needs'])
        self.assertIn("FOCUSA_WINDOWS_RELEASE_PROVIDER != 'ovh'", release['jobs']['queue-appveyor-windows']['if'])
        self.assertEqual(release['jobs']['dispatch-deploy-live-daemon']['needs'], 'checksums')
        steps = release['jobs']['checksums']['steps']
        self.assertTrue(any('release-trust-metadata.py' in step.get('run', '') for step in steps))

    def test_local_nsis_tooling_is_job_owned_and_full_acceptance_is_not_claimed(self):
        producer = yaml.load((ROOT / '.github/workflows/windows-ovh-build.yml').read_text(), Loader=yaml.BaseLoader)
        self.assertEqual(producer['on']['workflow_dispatch']['inputs']['desktop_nsis']['default'], 'false')
        steps = producer['jobs']['cross-compile']['steps']
        pipeline = (ROOT / 'scripts/ci/run-windows-ovh-release.py').read_text()
        self.assertIn('nsis=3.09-4ubuntu1', pipeline)
        self.assertNotIn('sudo', pipeline)
        self.assertIn("'dpkg-deb', '-x'", pipeline)
        self.assertTrue(any('run-windows-ovh-release.py' in step.get('run', '') for step in steps))
        # Mechanical build logic has exactly one owner, not duplicate YAML helpers.
        for step in steps:
            self.assertNotIn('apt-get download', step.get('run', ''))
            self.assertNotIn('cargo clean', step.get('run', ''))
        self.assertTrue(any('TAURI_SIGNING_PRIVATE_KEY' in step.get('env', {}) for step in steps))

    def run_nsis_fixture(self, reject_signature=False):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        source = Path(temporary.name)
        app = source / 'apps/menubar'
        (app / 'src-tauri').mkdir(parents=True)
        public = base64.b64encode(b'untrusted comment: test public key\nRWfixture').decode()
        (app / 'src-tauri/tauri.conf.json').write_text(json.dumps({'plugins': {'updater': {'pubkey': public}}}))
        args = SimpleNamespace(target_dir=source / 'target', output=source / 'artifacts',
                               tag='v0.9.202', sha='a' * 40)
        target = 'x86_64-pc-windows-msvc'
        def execute(command, **kwargs):
            if command[0] == 'node':
                bundle = args.target_dir / target / 'release/bundle/nsis'
                bundle.mkdir(parents=True)
                installer = bundle / 'Focusa_0.9.202_x64-setup.exe'
                installer.write_bytes(b'test fixture, not a release executable')
                Path(str(installer) + '.sig').write_text(base64.b64encode(b'test signature').decode())
            if command[0] == 'minisign' and reject_signature:
                raise subprocess.CalledProcessError(1, command)
        with patch.dict(MODULE.os.environ, {'GITHUB_ACTIONS': 'true'}), \
                patch.object(MODULE.subprocess, 'check_output', return_value='fixture-key'), \
                patch.object(MODULE.subprocess, 'run', side_effect=execute), \
                contextlib.redirect_stdout(io.StringIO()):
            if reject_signature:
                with self.assertRaises(subprocess.CalledProcessError):
                    MODULE.build_nsis(args, source, [target], {})
                self.assertFalse(list(args.output.glob('*setup.exe')))
                self.assertFalse((args.output / 'windows-nsis-receipt.json').exists())
            else:
                MODULE.build_nsis(args, source, [target], {})
                receipt = json.loads((args.output / 'windows-nsis-receipt.json').read_text())
                self.assertEqual(len(receipt['artifacts']), 2)
                self.assertEqual(receipt['updater_signature_verification'], 'passed')
                self.assertFalse(receipt['native_windows_proof'])
                self.assertFalse(receipt['msi_proof'])
                self.assertFalse(receipt['full_release_acceptance'])

    def test_nsis_receipt_distinguishes_generation_from_native_and_full_acceptance(self):
        self.run_nsis_fixture()

    def test_nsis_signature_failure_prevents_artifact_staging_and_receipt(self):
        self.run_nsis_fixture(reject_signature=True)

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


class WindowsPipelineTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location('windows_pipeline', ROOT / 'scripts/ci/run-windows-ovh-release.py')
        cls.pipeline = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.pipeline)

    def test_fractional_disk_usage_not_rounded_into_false_blocker(self):
        with patch.object(self.pipeline.shutil, 'disk_usage', return_value=SimpleNamespace(total=1000, used=899, free=20 * 1024**3)):
            status = self.pipeline.disk_status()
        self.assertEqual(status['used_percent'], 89.9)
        self.pipeline.require_headroom(status)

    def test_actual_disk_threshold_and_minimum_space_remain_enforced(self):
        for status in [{'used_percent': 90.0, 'free_gib': 20}, {'used_percent': 50, 'free_gib': 14.9}]:
            with self.assertRaises(ValueError):
                self.pipeline.require_headroom(status)

    def test_failed_producer_never_uploads_or_promotes(self):
        source = (ROOT / 'scripts/ci/run-windows-ovh-release.py').read_text()
        self.assertLess(source.index('run(command, env=env)'), source.index('publish(output, args.tag)'))
        self.assertIn('finally:', source)
        self.assertNotIn('release edit', source)
        self.assertNotIn('systemctl', source)

    def test_no_receipt_means_no_upload(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            (output / 'something.exe').write_bytes(b'not a verified artifact')
            with patch.object(self.pipeline, 'run') as command:
                with self.assertRaises(ValueError):
                    self.pipeline.publish(output, 'v0.9.202')
                command.assert_not_called()

    def test_compiled_inputs_and_hashes_survive_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / 'target'
            binary = target / 'x86_64-pc-windows-msvc/release/focusa-menubar.exe'
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b'independent compilation fixture')
            saved = Path(directory) / 'saved'
            self.pipeline.preserve_binaries(target, saved, 'v0.9.202', 'a' * 40)
            receipt = json.loads((saved / 'compilation-receipt.json').read_text())
            self.assertEqual(receipt['source_sha'], 'a' * 40)
            self.assertTrue(Path(receipt['compiled_desktop_artifacts'][0]['path']).is_file())
            self.assertFalse(receipt['installer_proof'])


if __name__ == '__main__':
    unittest.main()
