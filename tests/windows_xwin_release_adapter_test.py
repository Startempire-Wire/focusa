#!/usr/bin/env python3
"""Focused contract regressions for the OVH Windows cross-compile adapter."""
import base64
import contextlib
import importlib.util
import io
import json
import subprocess
import sys
from types import SimpleNamespace
from unittest.mock import patch
from pathlib import Path
import tempfile
import unittest

import yaml

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts/ci'))
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
        self.assertEqual(environment['CARGO_BUILD_JOBS'], '1')

    def test_release_automatically_calls_existing_windows_producer(self):
        # BaseLoader preserves YAML's `on` key (rather than YAML 1.1 boolean).
        release = yaml.load((ROOT / '.github/workflows/release.yml').read_text(), Loader=yaml.BaseLoader)
        producer = yaml.load((ROOT / '.github/workflows/windows-ovh-build.yml').read_text(), Loader=yaml.BaseLoader)
        previous = 'create-release'
        release_identity = None
        for job_id, mode in (('windows-ovh-executables', 'binaries_only'),
                             ('windows-ovh-desktop-nsis', 'desktop_nsis'),
                             ('windows-ovh-desktop-msi', 'desktop_msi')):
            job = release['jobs'][job_id]
            self.assertEqual(job['needs'], previous)
            self.assertEqual(job['uses'], './.github/workflows/windows-ovh-build.yml')
            self.assertEqual(set(job['with']), {'release_tag', 'release_sha', mode})
            self.assertEqual(job['with'][mode], 'true')
            identity = (job['with']['release_tag'], job['with']['release_sha'])
            self.assertEqual(identity, release_identity or identity)
            release_identity = identity
            self.assertEqual(job['secrets'], 'inherit')
            self.assertIn(mode, producer['on']['workflow_call']['inputs'])
            previous = job_id
        self.assertIn(previous, release['jobs']['rust-release']['needs'])
        for consumer in ('external-menubar-receipts', 'external-rust-binaries'):
            self.assertTrue({previous, 'rust-release', 'pi-extension-release'} <= set(release['jobs'][consumer]['needs']))
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
        self.assertIn('nsis=3.09-4ubuntu1', (ROOT / 'scripts/ci/windows-nsis-packages.lock').read_text())
        self.assertNotIn('sudo', pipeline)
        self.assertIn("'dpkg-deb', '-x'", pipeline)
        self.assertTrue(any('run-windows-ovh-release.py' in step.get('run', '') for step in steps))
        # Mechanical build logic has exactly one owner, not duplicate YAML helpers.
        for step in steps:
            self.assertNotIn('apt-get download', step.get('run', ''))
            self.assertNotIn('cargo clean', step.get('run', ''))
        self.assertTrue(any('TAURI_SIGNING_PRIVATE_KEY' in step.get('env', {}) for step in steps))

    def run_nsis_fixture(self, reject_signature=False, cached=False):
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
        if cached:
            import hashlib
            args.desktop_cache = source / 'cache'
            folder = args.desktop_cache / 'previous-job'
            binary = folder / target / 'focusa-menubar.exe'
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b'compiled fixture, not native Windows proof')
            (folder / 'compilation-receipt.json').write_text(json.dumps({
                'source_sha': args.sha, 'tag': args.tag,
                'compiled_desktop_artifacts': [{'target': target, 'path': str(binary),
                    'sha256': hashlib.sha256(binary.read_bytes()).hexdigest()}]}))
        commands = []
        def execute(command, **kwargs):
            commands.append(command)
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
                if cached:
                    self.assertTrue(any(command[0] == 'node' and command[2] == 'bundle' for command in commands))
                    self.assertFalse(any('--runner' in command for command in commands))

    def test_nsis_receipt_distinguishes_generation_from_native_and_full_acceptance(self):
        self.run_nsis_fixture()

    def test_same_candidate_compilation_is_reused_without_rust_rebuild(self):
        self.run_nsis_fixture(cached=True)

    def test_cache_lookup_checks_all_of_tag_source_target_and_current_bytes(self):
        import hashlib
        with tempfile.TemporaryDirectory() as directory:
            cache = Path(directory)
            compiled = cache / 'completed-nsis' / 'focusa-menubar.exe'
            compiled.parent.mkdir()
            compiled.write_bytes(b'fixture-pe')
            target = 'aarch64-pc-windows-msvc'
            tag, sha = 'v0.9.214-dev', 'a' * 40
            receipt = {'tag': tag, 'source_sha': sha, 'compiled_desktop_artifacts': [
                {'target': target, 'path': str(compiled), 'sha256': hashlib.sha256(compiled.read_bytes()).hexdigest()}]}
            (compiled.parent / 'compilation-receipt.json').write_text(json.dumps(receipt))
            self.assertEqual(MODULE.find_cached_desktop(cache, tag, sha, target), compiled)
            self.assertIsNone(MODULE.find_cached_desktop(cache, 'v0.9.213-dev', sha, target))
            self.assertIsNone(MODULE.find_cached_desktop(cache, tag, 'b' * 40, target))
            self.assertIsNone(MODULE.find_cached_desktop(cache, tag, sha, 'x86_64-pc-windows-msvc'))
            compiled.write_bytes(b'corrupted')
            self.assertIsNone(MODULE.find_cached_desktop(cache, tag, sha, target))

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

    def test_portable_nsis_boundary_restores_its_real_resource_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            tools = Path(directory)
            executable = tools / 'root/usr/bin/makensis'
            executable.parent.mkdir(parents=True)
            executable.write_bytes(b'packager fixture')
            lock = tools / 'package.lock'
            lock.write_text('nsis=3.09-4ubuntu1\n')
            env = {'PATH': '/usr/bin', 'FOCUSA_WINDOWS_PACKAGE_CACHE': str(tools / 'cache')}
            def fixture_run(command, **kwargs):
                if command[0] == 'apt-get':
                    (kwargs['cwd'] / 'nsis_fixture.deb').write_bytes(b'package fixture')
            with patch.object(self.pipeline, 'NSIS_PACKAGE_LOCK', lock), patch.object(self.pipeline, 'run', side_effect=fixture_run) as runner:
                self.pipeline.prepare_nsis(tools, env)
                reused = tools / 'second'
                second_executable = reused / 'root/usr/bin/makensis'
                second_executable.parent.mkdir(parents=True)
                second_executable.write_bytes(b'packager fixture')
                self.pipeline.prepare_nsis(reused, env)
                self.assertEqual(sum(call.args[0][0] == 'apt-get' for call in runner.call_args_list), 1)
                package = next((tools / 'cache').glob('*/nsis_fixture.deb'))
                package.write_bytes(b'tampered package fixture')
                with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
                    self.pipeline.prepare_nsis(tools / 'third', env)
            self.assertEqual(executable.with_name('makensis.real').read_bytes(), b'packager fixture')
            self.assertIn('export NSISDIR=', executable.read_text())
            self.assertIn('exec ', executable.read_text())
            self.assertTrue(executable.stat().st_mode & 0o111)

    def test_build_tool_versions_and_environment_are_not_floating(self):
        source = (ROOT / 'scripts/ci/run-windows-ovh-release.py').read_text()
        self.assertNotIn('--simulate', source)
        self.assertIn('SOURCE_DATE_EPOCH', source)
        self.assertIn("env['TZ'] = 'UTC'", source)
        lock = ROOT / 'scripts/ci/windows-nsis-packages.lock'
        packages = [line for line in lock.read_text().splitlines() if line and not line.startswith('#')]
        self.assertTrue(packages)
        self.assertTrue(all('=' in line and '*' not in line for line in packages))

    def test_actual_disk_threshold_and_minimum_space_remain_enforced(self):
        for status in [{'used_percent': 90.0, 'free_gib': 20}, {'used_percent': 50, 'free_gib': 14.9}]:
            with self.assertRaises(ValueError):
                self.pipeline.require_headroom(status)

    def test_release_stage_disk_guard_fails_closed_at_exact_boundaries(self):
        project = self.pipeline.projected_headroom
        capacity = 310_894_637_056
        peak = self.pipeline.WINDOWS_STAGE_PEAK_BYTES['msi']
        allowed_use = (9 * capacity - 1) // 10 - peak
        self.assertTrue(project(allowed_use, capacity - allowed_use, peak)['passes'])
        self.assertFalse(project(allowed_use + 1, capacity - allowed_use - 1, peak)['passes'])
        self.assertTrue(project(0, 15 * 2**30 + peak, peak)['passes'])
        self.assertFalse(project(0, 15 * 2**30 + peak - 1, peak)['passes'])
        self.assertFalse(project(274_157_772_800, 36_736_864_256,
                                 self.pipeline.WINDOWS_STAGE_PEAK_BYTES['all'])['passes'])
        for bad in (None, True, -1, 2.5):
            with self.assertRaises(ValueError):
                project(0, bad, peak)

    def test_owned_cache_marker_precedes_cached_binary_creation(self):
        source = (ROOT / 'scripts/ci/run-windows-ovh-release.py').read_text()
        self.assertIn('Signature: 8a477f597d28d172789f06886806bc55', source)
        self.assertLess(source.index("(target / 'CACHEDIR.TAG')"), source.index('run(command, env=env)'))

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

    def test_producer_keeps_prior_draft_assets_and_requires_upload_custody(self):
        import hashlib
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            file = output / 'windows-msi-receipt.json'
            file.write_bytes(b'verified test receipt')
            tag = 'v0.9.214-dev'
            asset = {'name': file.name, 'state': 'uploaded', 'size': file.stat().st_size,
                     'digest': 'sha256:' + hashlib.sha256(file.read_bytes()).hexdigest()}
            draft = json.dumps({'id': 1234, 'tag_name': tag, 'draft': True}) + '\n'
            asset_line = json.dumps(asset) + '\n'
            env = {'GITHUB_REPOSITORY': 'example/focusa'}
            with patch.dict(self.pipeline.os.environ, env), patch.object(self.pipeline, 'run') as upload:
                with patch.object(self.pipeline.subprocess, 'check_output',
                                  side_effect=[draft, '', draft, asset_line]) as reads:
                    self.pipeline.publish(output, tag)
                for idx, endpoint in enumerate(['releases?per_page=100', 'releases/1234/assets?per_page=100'] * 2):
                    self.assertEqual(reads.call_args_list[idx].args[0][:4],
                                     ['gh', 'api', '--paginate', 'repos/example/focusa/' + endpoint])
                self.assertEqual(upload.call_args.args[0][-1], file)
                self.assertNotIn('--clobber', upload.call_args.args[0])
                upload.reset_mock()
                with patch.object(self.pipeline.subprocess, 'check_output',
                                  side_effect=[draft, asset_line, draft, asset_line]):
                    self.pipeline.publish(output, tag)
                for bad in ({**asset, 'digest': 'sha256:wrong'}, {**asset, 'digest': None},
                            {**asset, 'state': 'starter'}, {**asset, 'size': 0}):
                    with patch.object(self.pipeline.subprocess, 'check_output',
                                      side_effect=[draft, json.dumps(bad)]):
                        with self.assertRaisesRegex(ValueError, 'conflicts'):
                            self.pipeline.publish(output, tag)
                with patch.object(self.pipeline.subprocess, 'check_output', side_effect=[draft, asset_line * 2]):
                    with self.assertRaisesRegex(ValueError, 'ambiguous'):
                        self.pipeline.publish(output, tag)
                with patch.object(self.pipeline.subprocess, 'check_output', side_effect=[draft, asset_line, draft, '']):
                    with self.assertRaisesRegex(ValueError, 'not in draft custody'):
                        self.pipeline.publish(output, tag)
                with patch.object(self.pipeline.subprocess, 'check_output', side_effect=[draft, asset_line, draft.replace('1234', '5678')]):
                    with self.assertRaisesRegex(ValueError, 'not in draft custody'):
                        self.pipeline.publish(output, tag)
                upload.assert_not_called()

    def test_exact_draft_id_rejects_missing_duplicate_published_and_invalid_id(self):
        tag = 'v0.9.214-dev'
        draft = {'id': 1234, 'tag_name': tag, 'draft': True}
        for rows in ([], [draft, draft], [{**draft, 'draft': False}], [{**draft, 'id': True}],
                     [{**draft, 'id': -1}], [{**draft, 'tag_name': 'v0.9.213-dev'}]):
            with patch.object(self.pipeline, 'release_rows', return_value=rows):
                with self.assertRaisesRegex(ValueError, 'draft release'):
                    self.pipeline.exact_draft_id('example/focusa', tag)

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
