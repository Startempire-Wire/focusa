#!/usr/bin/env python3
"""OVH cross-compile adapter for the immutable AppVeyor Rust build contract.

Produces executables or verified NSIS staging evidence: no upload, publication,
native-runtime claim, MSI claim, credential change, or release promotion.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

import yaml


def recipe(source):
    contract = yaml.safe_load((source / '.appveyor.yml').read_text())
    environment = contract['environment']
    blocks = '\n'.join(step.get('ps', '') for step in contract['build_script'])
    packages = list(dict.fromkeys(re.findall(r'-p\s+(focusa-[a-z-]+)', blocks)))
    packaging = '\n'.join(step.get('ps', '') for step in contract['after_build']
                          if isinstance(step, dict))
    match = re.search(r'foreach \(\$bin in @\(([^)]+)\)\)', packaging)
    if not packages or not match:
        raise ValueError('canonical Windows package/surface contract missing')
    surfaces = re.findall(r'"([a-z-]+)"', match.group(1))
    targets = [row['RUST_TARGET'] for row in environment['matrix']
               if row['SURFACE'] == 'binaries']
    roots = environment.get('FOCUSA_AUTHORITY_ROOT_KEYS_JSON')
    if not roots or not json.loads(roots):
        raise ValueError('canonical production authority roots missing')
    if len(targets) != len(set(targets)) or len(surfaces) != len(packages):
        raise ValueError('ambiguous canonical Windows matrix')
    return packages, surfaces, targets, {
        'FOCUSA_AUTHORITY_ROOT_KEYS_JSON': roots,
        'CARGO_PROFILE_RELEASE_LTO': str(environment['CARGO_PROFILE_RELEASE_LTO']),
    }


def git(source, *args):
    return subprocess.check_output(['git', '-C', str(source), *args], text=True).strip()


def build_nsis(args, source, targets, env):
    """Use the package-owned Tauri bundler; verify updater signatures, not native runtime."""
    import base64
    app = source / 'apps/menubar'
    config = json.loads((app / 'src-tauri/tauri.conf.json').read_text())
    public_box = base64.b64decode(config['plugins']['updater']['pubkey']).decode()
    public_key = next(line for line in public_box.splitlines()
                      if line and not line.startswith('untrusted comment:'))
    converter = source / 'scripts/ci/convert-legacy-tauri-signing-key.py'
    normalized = subprocess.check_output(['python3', str(converter)], env=env, text=True).strip()
    # GitHub masks transformed secret material too; never print it elsewhere.
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        raise ValueError('signed packaging requires the approved GitHub secret-injection context')
    print('::add-mask::' + normalized, flush=True)
    env['TAURI_SIGNING_PRIVATE_KEY'] = normalized
    subprocess.run(['npm', 'ci'], cwd=app, env=env, check=True)
    cli = app / 'node_modules/@tauri-apps/cli/tauri.js'
    env['CARGO_TARGET_DIR'] = str(args.target_dir.resolve())
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    records = []
    for target in targets:
        env['XWIN_ARCH'] = target.split('-', 1)[0]
        subprocess.run(['node', str(cli), 'build', '--runner', 'cargo-xwin',
                        '--target', target, '--bundles', 'nsis'], cwd=app, env=env, check=True)
        bundle = args.target_dir.resolve() / target / 'release/bundle/nsis'
        installers = list(bundle.glob('*setup.exe'))
        if len(installers) != 1:
            raise ValueError(f'exactly one canonical NSIS installer required for {target}')
        installer = installers[0]
        signature = Path(str(installer) + '.sig')
        if not signature.is_file():
            raise ValueError(f'updater signature missing for {target}')
        decoded = output / (signature.name + '.minisig')
        decoded.write_bytes(base64.b64decode(signature.read_text().strip(), validate=True))
        subprocess.run(['minisign', '-V', '-m', str(installer), '-x', str(decoded),
                        '-P', public_key], cwd=app, env=env, check=True)
        for artifact in [installer, signature]:
            destination = output / artifact.name
            shutil.copy2(artifact, destination)
            records.append({'name': destination.name, 'target': target,
                            'sha256': hashlib.sha256(destination.read_bytes()).hexdigest()})
        # Canonical Cargo cleanup, restricted to this job-owned architecture target.
        subprocess.run(['cargo', 'clean', '--target-dir', str(args.target_dir.resolve()),
                        '--target', target], cwd=app / 'src-tauri', env=env, check=True)
    receipt = {'kind': 'ovh_windows_nsis', 'tag': args.tag, 'source_sha': args.sha,
               'artifacts': records, 'updater_signature_verification': 'passed',
               'native_windows_proof': False, 'msi_proof': False,
               'full_release_acceptance': False}
    (output / 'windows-nsis-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', required=True, type=Path)
    parser.add_argument('--tag', required=True)
    parser.add_argument('--sha', required=True)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--target-dir', required=True, type=Path)
    parser.add_argument('--sdk-cache', required=True, type=Path)
    parser.add_argument('--clang-shim', default='/opt/xwin-shim', type=Path)
    parser.add_argument('--plan', action='store_true')
    parser.add_argument('--desktop-nsis', action='store_true')
    args = parser.parse_args()
    source = args.source.resolve()
    if not re.fullmatch(r'v\d+\.\d+\.\d+(?:-dev)?', args.tag):
        raise ValueError('exact existing release tag required')
    if not re.fullmatch(r'[a-f0-9]{40}', args.sha):
        raise ValueError('exact source SHA required')
    if git(source, 'rev-parse', 'HEAD') != args.sha:
        raise ValueError('candidate HEAD does not match requested SHA')
    if git(source, 'rev-parse', args.tag + '^{commit}') != args.sha:
        raise ValueError('immutable tag does not match candidate SHA')
    if git(source, 'status', '--porcelain'):
        raise ValueError('candidate source must be clean')
    packages, surfaces, targets, contract_env = recipe(source)
    commands = [['cargo', 'xwin', 'build', '--locked', '--release', '--target', target,
                 '--target-dir', str(args.target_dir.resolve()),
                 *[value for package in packages for value in ('-p', package)]]
                for target in targets]
    if args.plan:
        if args.desktop_nsis:
            commands = [['node', 'node_modules/@tauri-apps/cli/tauri.js', 'build',
                         '--runner', 'cargo-xwin', '--target', target, '--bundles', 'nsis']
                        for target in targets]
        print(json.dumps({'kind': 'cross_compile_plan', 'tag': args.tag,
                          'source_sha': args.sha, 'commands': commands,
                          'surfaces': surfaces, 'native_windows_proof': False}))
        return
    if not args.sdk_cache.is_dir() or not args.clang_shim.is_dir():
        raise ValueError('existing SDK cache and compiler shim required; no auto-install')
    env = os.environ.copy()
    env.update(contract_env)
    env['XWIN_CACHE_DIR'] = str(args.sdk_cache.resolve())
    env['CARGO_BUILD_JOBS'] = '2'
    env['PATH'] = str(args.clang_shim.resolve()) + os.pathsep + env['PATH']
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    if args.desktop_nsis:
        build_nsis(args, source, targets, env)
        return
    records = []
    for target, command in zip(targets, commands):
        env['XWIN_ARCH'] = target.split('-', 1)[0]
        subprocess.run(command, cwd=source, env=env, check=True)
        binaries = args.target_dir.resolve() / target / 'release'
        subprocess.run(['python3', str(source / 'scripts/verify-embedded-authority-root.py'),
                        str(binaries / 'focusa.exe'), str(binaries / 'focusa-daemon.exe')],
                       cwd=source, env=env, check=True)
        for surface in surfaces:
            original = binaries / (surface + '.exe')
            destination = output / f'{surface}-{args.tag}-{target}.exe'
            if not original.is_file() or original.stat().st_size == 0:
                raise ValueError(f'missing canonical Windows surface: {surface}/{target}')
            shutil.copy2(original, destination)
            records.append({'name': destination.name,
                            'sha256': hashlib.sha256(destination.read_bytes()).hexdigest()})
    receipt = {'kind': 'ovh_windows_cross_compile', 'source_sha': args.sha,
               'tag': args.tag, 'production_root_embedding': 'passed',
               'artifacts': records, 'native_windows_proof': False,
               'installer_proof': False, 'published': False}
    (output / 'windows-cross-compile-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()
