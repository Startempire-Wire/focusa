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

PINNED_NSIS = ['nsis=3.09-4ubuntu1', 'nsis-common=3.09-4ubuntu1', 'minisign=0.11-1']
LINUX_CACHE_TARGETS = ['x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu', 'x86_64-unknown-linux-musl']


def run(command, **kwargs):
    return subprocess.run([str(item) for item in command], check=True, **kwargs)


def disk_status(path=Path('/')):
    usage = shutil.disk_usage(path)
    return {'used_percent': usage.used * 100.0 / usage.total,
            'free_gib': usage.free / 1024**3}


def require_headroom(status):
    # df displays a rounded integer; 89.x% must not be misclassified as >=90%.
    if status['used_percent'] >= 90.0 or status['free_gib'] < 15:
        raise ValueError('build headroom insufficient: ' + json.dumps(status))


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
    packages, root = tools / 'packages', tools / 'root'
    packages.mkdir(parents=True, exist_ok=True)
    root.mkdir(parents=True, exist_ok=True)
    run(['apt-get', 'download', *PINNED_NSIS], cwd=packages)
    # Real Linux tray metadata is required by Tauri's Linux CLI even for Windows.
    # APT simulation + download + extraction never upgrade the host or its demos.
    simulation = subprocess.check_output(['apt-get', '--simulate', '--no-upgrade',
        '--no-install-recommends', 'install', 'libayatana-appindicator3-dev'], text=True)
    dependencies = [name + '=' + version for name, version in re.findall(
        r'^Inst (\S+) (?:\[[^]]+\] )?\((\S+)', simulation, re.MULTILINE)]
    if dependencies:
        run(['apt-get', 'download', *dependencies], cwd=packages)
    for package in sorted(packages.glob('*.deb')):
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
    parser.add_argument('--mode', choices=['binaries', 'nsis', 'msi-tools'], default='binaries')
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
    env = os.environ.copy()
    command = [sys.executable, controller / 'build-windows-xwin-release.py',
               '--source', args.source.resolve(), '--tag', args.tag, '--sha', args.sha,
               '--sdk-cache', args.sdk_cache, '--target-dir', target, '--output', output]
    if args.mode == 'nsis':
        command.append('--desktop-nsis')
        cache = Path('/home/wirebot/build/focusa') / ('windows-desktop-binaries-' + args.tag)
        command += ['--desktop-cache', cache]
    run([*command, '--plan'], env=env)
    archive = Path('/home/wirebot/build/focusa') / ('preserved-release-cache-' + job)
    reclaim_idle_cache(Path('/home/wirebot/.cache/focusa-release-target'), archive)
    require_headroom(disk_status())
    try:
        if args.mode == 'msi-tools':
            run([sys.executable, controller / 'prepare-windows-msi-wine.py',
                 '--tools-directory', work / 'msi-tools'], env=env)
            return
        for executable in ['cargo-xwin', 'node', 'npm', 'cargo']:
            if not shutil.which(executable):
                raise ValueError('required pinned worker tool missing: ' + executable)
        if args.mode == 'nsis':
            prepare_nsis(work / 'nsis-tools', env)
        require_headroom(disk_status())
        run(command, env=env)
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
