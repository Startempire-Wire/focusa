#!/usr/bin/env python3
"""Single Windows build entry point: isolated tools -> build -> verify -> upload -> cleanup.

Reuses the canonical cross-build/Wine producers. No host package installation,
service changes, credentials recovery, tag rewriting or invented native proof.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import shlex
import subprocess
import sys

NSIS_PACKAGE_LOCK = Path(__file__).with_name('windows-nsis-packages.lock')
LINUX_CACHE_TARGETS = ['x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu', 'x86_64-unknown-linux-musl']


def run(command, **kwargs):
    return subprocess.run([str(item) for item in command], check=True, **kwargs)


def disk_status(path=Path('/')):
    usage = shutil.disk_usage(path)
    return {'used_percent': usage.used * 100.0 / usage.total,
            'free_gib': usage.free / 1024**3}


def require_headroom(status, *, packaging_only=False):
    # Cached MSI/tool-only stages cannot invoke a Rust build. Retain a
    # 10 GiB tool-only reserve and the 95% emergency ceiling.
    # Binary/combined builds keep their original 15 GiB / 90% reserve.
    used_limit, free_minimum = (95.0, 10) if packaging_only else (90.0, 15)
    if status['used_percent'] >= used_limit or status['free_gib'] < free_minimum:
        raise ValueError('build headroom insufficient: ' + json.dumps(status))


def reclaim_completed_msi_tools(temporary, archive, current_run):
    """Reclaim only terminal, receipt-identified job-owned tools; retain evidence."""
    repository = os.environ['GITHUB_REPOSITORY']
    for folder in temporary.glob('focusa-windows-pipeline-*'):
        match = re.fullmatch(r'focusa-windows-pipeline-(\d+)-(\d+)', folder.name)
        if not match or match[1] == current_run or folder.is_symlink():
            continue
        tools = folder / 'msi-tools'
        handle = tools / 'toolchain-receipt.json'
        if tools.is_symlink() or not handle.is_file():
            continue
        tools.resolve().relative_to(temporary.resolve())
        receipt = json.loads(handle.read_text())
        if receipt.get('kind') != 'ovh_wine_wix_toolchain' or receipt.get('prefix') != str(tools / 'prefix'):
            continue
        status = subprocess.check_output(['gh', 'api', f'repos/{repository}/actions/runs/{match[1]}', '--jq', '.status'], text=True).strip()
        if status != 'completed':
            continue
        retained = archive / 'toolchain-evidence' / folder.name
        retained.mkdir(parents=True, exist_ok=True)
        shutil.copy2(handle, retained / handle.name)
        native = tools / 'native-cli/native-tool-inputs.json'
        if native.is_file():
            shutil.copy2(native, retained / native.name)
        shutil.rmtree(tools)
        print('Reclaimed completed isolated tools; receipt retained:', folder.name)


def preserve_binaries(target, saved, tag, sha):
    records = []
    for triple in ['x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc']:
        binary = target / triple / 'release/focusa-menubar.exe'
        if binary.is_file():
            destination = saved / triple / binary.name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(binary, destination)
            records.append({'target': triple, 'path': str(destination),
                            'sha256': hashlib.sha256(destination.read_bytes()).hexdigest()})
    if records:
        (saved / 'compilation-receipt.json').write_text(json.dumps({
            'source_sha': sha, 'tag': tag, 'compiled_desktop_artifacts': records,
            'installer_proof': False, 'native_windows_proof': False}, indent=2) + '\n')


def reclaim_idle_cache(cache, saved):
    if disk_status()['used_percent'] < 90.0:
        return
    for comm in Path('/proc').glob('[0-9]*/comm'):
        try:
            name = comm.read_text().strip()
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
        if name in {'cargo', 'cargo-xwin', 'rustc', 'clang-cl', 'lld-link'}:
            raise ValueError('active compiler present; shared caches preserved')
    records = []
    for triple in LINUX_CACHE_TARGETS:
        target = cache / triple
        if not target.is_dir() or target.is_symlink():
            continue
        directory = saved / triple
        directory.mkdir(parents=True, exist_ok=True)
        for name in ['focusa', 'focusa-daemon', 'focusa-tui', 'focusa-session-runner']:
            binary = target / triple / 'release' / name
            if binary.is_file():
                destination = directory / name
                shutil.copy2(binary, destination)
                records.append({'path': str(destination),
                                'sha256': hashlib.sha256(destination.read_bytes()).hexdigest()})
        (saved / 'preserved-binaries.json').write_text(json.dumps(records, indent=2) + '\n')
        run(['cargo', 'clean', '--target-dir', target])


def prepare_nsis(tools, env):
    # Retain exact, checksum-verified inputs rather than re-resolving retired
    # repository versions on every new runner attempt; extraction stays isolated.
    lock_digest = hashlib.sha256(NSIS_PACKAGE_LOCK.read_bytes()).hexdigest()
    cache = Path(env.get('FOCUSA_WINDOWS_PACKAGE_CACHE',
                         '/home/wirebot/build/focusa/windows-packaging-cache'))
    packages, root = cache / lock_digest, tools / 'root'
    packages.mkdir(parents=True, exist_ok=True)
    root.mkdir(parents=True, exist_ok=True)
    dependencies = [line for line in NSIS_PACKAGE_LOCK.read_text().splitlines()
                    if line and not line.startswith('#')]
    if not dependencies or any(not re.fullmatch(r'[a-z0-9+.-]+=[^\s]+', line) for line in dependencies):
        raise ValueError('invalid pinned Windows packaging tool lock')
    manifest = packages / 'input-sha256.json'
    if manifest.exists():
        expected = json.loads(manifest.read_text())
    else:
        run(['apt-get', 'download', *dependencies], cwd=packages)
        expected = {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                    for path in sorted(packages.glob('*.deb'))}
        if len(expected) != len(dependencies):
            raise ValueError('pinned packaging cache incomplete; retain original download diagnostics')
        pending = manifest.with_suffix('.pending')
        pending.write_text(json.dumps(expected, sort_keys=True) + '\n')
        pending.replace(manifest)
    if len(expected) != len(dependencies):
        raise ValueError('pinned packaging cache manifest count mismatch')
    for name, digest in sorted(expected.items()):
        if Path(name).name != name or not name.endswith('.deb'):
            raise ValueError('invalid pinned packaging cache path')
        package = packages / name
        if not package.is_file() or hashlib.sha256(package.read_bytes()).hexdigest() != digest:
            raise ValueError('pinned packaging cache checksum mismatch: ' + name)
        run(['dpkg-deb', '-x', package, root])
    # Tauri deliberately removes NSISDIR before spawning makensis. Keep the
    # portable distribution's real resource binding at the executable boundary.
    executable = root / 'usr/bin/makensis'
    real = executable.with_name('makensis.real')
    executable.rename(real)
    executable.write_text('#!/bin/sh\nexport NSISDIR=' + shlex.quote(str(root / 'usr/share/nsis')) + '\nexec ' + shlex.quote(str(real)) + ' "$@"\n')
    executable.chmod(0o755)
    env['PATH'] = str(root / 'usr/bin') + os.pathsep + env['PATH']
    env['NSISDIR'] = str(root / 'usr/share/nsis')
    env['PKG_CONFIG_PATH'] = ':'.join(str(root / item) for item in [
        'usr/lib/x86_64-linux-gnu/pkgconfig', 'usr/share/pkgconfig'])
    env['PKG_CONFIG_SYSROOT_DIR'] = str(root)
    run(['pkg-config', '--libs-only-L', 'ayatana-appindicator3-0.1'], env=env)


def publish(output, tag):
    files = sorted(path for path in output.iterdir()
                   if path.is_file() and not path.name.endswith('.minisig'))
    if not files or not list(output.glob('*receipt.json')):
        raise ValueError('verified producer receipt and artifacts required before upload')
    run(['gh', 'release', 'upload', tag, *files, '--clobber'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', required=True, type=Path)
    parser.add_argument('--tag', required=True)
    parser.add_argument('--sha', required=True)
    parser.add_argument('--mode', choices=['all', 'binaries', 'nsis', 'msi', 'msi-tools'], default='all')
    parser.add_argument('--sdk-cache', type=Path, default=Path('/home/wirebot/build/focusa/windows-sdk-cache'))
    parser.add_argument('--publish', action='store_true')
    parser.add_argument('--plan', action='store_true')
    args = parser.parse_args()
    if not re.fullmatch(r'v\d+\.\d+\.\d+(?:-dev)?', args.tag):
        raise ValueError('exact existing release tag required')
    if not re.fullmatch('[a-f0-9]{40}', args.sha):
        raise ValueError('exact immutable source SHA required')
    stages = ['validate_candidate', 'reclaim_idle_cache', 'prepare_isolated_tools',
              'check_exact_headroom', 'canonical_build_and_verify']
    if args.publish:
        stages.append('upload_verified_artifacts')
    stages += ['preserve_compiled_inputs', 'canonical_owned_target_cleanup']
    if args.plan:
        print(json.dumps({'stages': stages, 'mode': args.mode, 'tag': args.tag,
                          'source_sha': args.sha, 'native_windows_proof': False}))
        return
    if os.environ.get('GITHUB_ACTIONS') != 'true' or os.geteuid() == 0:
        raise ValueError('run on the approved non-root OVH CI build worker')
    controller = Path(__file__).resolve().parent
    temporary = Path(os.environ['RUNNER_TEMP']).resolve()
    job = os.environ['GITHUB_RUN_ID'] + '-' + os.environ['GITHUB_RUN_ATTEMPT']
    work = temporary / ('focusa-windows-pipeline-' + job)
    work.mkdir(parents=True, exist_ok=False)
    target, output = work / 'target', work / 'artifacts'
    # Declare this fresh, exact-owned job directory as rebuildable before a
    # cached binary creates it; otherwise Cargo correctly refuses to clean it.
    target.mkdir()
    (target / 'CACHEDIR.TAG').write_text('Signature: 8a477f597d28d172789f06886806bc55\n# Job-owned rebuildable Cargo target cache.\n')
    env = os.environ.copy()
    env['SOURCE_DATE_EPOCH'] = subprocess.check_output(
        ['git', '-C', str(args.source), 'show', '-s', '--format=%ct', args.sha], text=True).strip()
    env['TZ'] = 'UTC'
    env['LC_ALL'] = 'C.UTF-8'
    command = [sys.executable, controller / 'build-windows-xwin-release.py',
               '--source', args.source.resolve(), '--tag', args.tag, '--sha', args.sha,
               '--sdk-cache', args.sdk_cache, '--target-dir', target, '--output', output]
    if args.mode in {'nsis', 'msi'}:
        command.append('--desktop-' + args.mode)
        if args.mode == 'msi':
            command += ['--msi-tools', work / 'msi-tools']
        cache = Path('/home/wirebot/build/focusa') / ('windows-desktop-binaries-' + args.tag)
        command += ['--desktop-cache', cache]
    run([*command, '--plan'], env=env)
    archive = Path('/home/wirebot/build/focusa') / ('preserved-release-cache-' + job)
    reclaim_idle_cache(Path('/home/wirebot/.cache/focusa-release-target'), archive)
    reclaim_completed_msi_tools(temporary, archive, os.environ['GITHUB_RUN_ID'])
    packaging_only = args.mode in {'msi', 'msi-tools'}
    require_headroom(disk_status(), packaging_only=packaging_only)
    try:
        if args.mode in {'msi-tools', 'msi'}:
            run([sys.executable, controller / 'prepare-windows-msi-wine.py',
                 '--tools-directory', work / 'msi-tools'], env=env)
            if args.mode == 'msi-tools':
                return
        for executable in ['cargo-xwin', 'node', 'npm', 'cargo']:
            if not shutil.which(executable):
                raise ValueError('required pinned worker tool missing: ' + executable)
        if args.mode in {'all', 'nsis', 'msi'}:
            # The same pinned tool owner supplies minisign for both installer
            # formats; MSI never substitutes signing for verification.
            prepare_nsis(work / 'nsis-tools', env)
        # Tool extraction can cross the reserve after the initial idle-cache
        # pass. Reuse its compiler-safe, binary-preserving Cargo owner now.
        reclaim_idle_cache(Path('/home/wirebot/.cache/focusa-nightly-target'), archive / 'nightly')
        require_headroom(disk_status(), packaging_only=packaging_only)
        run(command, env=env)
        if args.mode == 'all':
            cache = Path('/home/wirebot/build/focusa') / ('windows-desktop-binaries-' + args.tag)
            run([*command, '--desktop-nsis', '--desktop-cache', cache], env=env)
            preserve_binaries(target, cache / job, args.tag, args.sha)
            run([sys.executable, controller / 'prepare-windows-msi-wine.py',
                 '--tools-directory', work / 'msi-tools'], env=env)
            run([*command, '--desktop-msi', '--msi-tools', work / 'msi-tools',
                 '--desktop-cache', cache], env=env)
        if args.publish:
            publish(output, args.tag)
    finally:
        if target.is_dir():
            saved = Path('/home/wirebot/build/focusa') / (
                'windows-desktop-binaries-' + args.tag) / job
            preserve_binaries(target, saved, args.tag, args.sha)
            run(['cargo', 'clean', '--target-dir', target], cwd=args.source, env=env)
        print('Post-build storage:', json.dumps(disk_status()))


if __name__ == '__main__':
    main()
