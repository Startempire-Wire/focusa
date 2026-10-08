#!/usr/bin/env python3
"""OVH cross-compile adapter for the immutable AppVeyor Rust build contract.

Produces executables or verified signed installer staging evidence: no upload,
publication, native-runtime claim, credential change, or release promotion.
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


def reset_bundle_marker(binary):
    """Reset only the PE64 mutable fat-string referenced by pinned tauri-utils."""
    import struct
    data = bytearray(binary.read_bytes())
    unknown = b'__TAURI_BUNDLE_TYPE_VAR_UNK'
    if unknown in data or data[:2] != b'MZ':
        return
    pe = struct.unpack_from('<I', data, 0x3c)[0]
    count = struct.unpack_from('<H', data, pe + 6)[0]
    optional_size = struct.unpack_from('<H', data, pe + 20)[0]
    if data[pe:pe+4] != b'PE\0\0' or struct.unpack_from('<H', data, pe+24)[0] != 0x20b:
        raise ValueError('supported PE64 cached desktop required')
    base = struct.unpack_from('<Q', data, pe+48)[0]
    sections = []
    for i in range(count):
        at = pe + 24 + optional_size + 40*i
        name = bytes(data[at:at+8]).rstrip(b'\0')
        size, address, raw_size, raw = struct.unpack_from('<IIII', data, at+8)
        sections.append((name, address, raw_size, raw))
    found = []
    for name, _, raw_size, raw in sections:
        if name != b'.data':
            continue
        for at in range(raw, raw + raw_size - 15, 8):
            pointer, length = struct.unpack_from('<QQ', data, at)
            if length != len(unknown):
                continue
            for _, address, size, offset in sections:
                rva = pointer - base
                if address <= rva < address + size:
                    location = offset + rva - address
                    marker = bytes(data[location:location+len(unknown)])
                    if marker in {b'__TAURI_BUNDLE_TYPE_VAR_NSS', b'__TAURI_BUNDLE_TYPE_VAR_MSI'}:
                        found.append(location)
    if len(set(found)) != 1:
        raise ValueError('unambiguous cached Tauri bundle marker required')
    location = found[0]
    data[location:location+len(unknown)] = unknown
    binary.write_bytes(data)


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
    package_format = 'msi' if getattr(args, 'desktop_msi', False) else 'nsis'
    launcher = ['node']
    if package_format == 'msi':
        import importlib.util
        spec = importlib.util.spec_from_file_location('native_tauri', Path(__file__).with_name('prepare-native-windows-tauri.py'))
        native_tauri = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(native_tauri)
        launcher, native_env = native_tauri.prepare(app, args.msi_tools.resolve(), env.copy())
    for target in targets:
        env['XWIN_ARCH'] = target.split('-', 1)[0]
        cached = None
        cache = getattr(args, 'desktop_cache', None)
        if cache and cache.is_dir():
            for handle in sorted(cache.glob('*/compilation-receipt.json'), reverse=True):
                receipt = json.loads(handle.read_text())
                if receipt.get('source_sha') != args.sha or receipt.get('tag') != args.tag:
                    continue
                for record in receipt.get('compiled_desktop_artifacts', []):
                    if record.get('target') != target:
                        continue
                    binary = Path(record['path']).resolve()
                    binary.relative_to(handle.parent.resolve())
                    if binary.is_file() and hashlib.sha256(binary.read_bytes()).hexdigest() == record['sha256']:
                        cached = binary
                        break
                if cached:
                    break
        if cached:
            destination = args.target_dir.resolve() / target / 'release/focusa-menubar.exe'
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(cached, destination)
            reset_bundle_marker(destination)
            subprocess.run(['npm', 'run', 'build'], cwd=app, env=env, check=True)
            cli_path = native_tauri.windows_path(cli) if package_format == 'msi' else str(cli)
            command = [*launcher, cli_path, 'bundle', '--target', target,
                       '--bundles', package_format, *(['--verbose'] if package_format == 'msi' else [])]
            if package_format == 'msi':
                attempt = subprocess.run(command, cwd=app, env=native_env, text=True,
                                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=180)
                print(attempt.stdout, flush=True)
                if attempt.returncode:
                    if 'experimental wow64 mode' not in attempt.stdout or 'stack overflow' not in attempt.stdout:
                        raise subprocess.CalledProcessError(attempt.returncode, command)
                    native_tauri.finish_generated_msi(app, args.msi_tools.resolve(), args.target_dir,
                                                      target, env)
            else:
                subprocess.run(command, cwd=app, env=env, check=True)
        else:
            if package_format == 'msi':
                raise ValueError('MSI packaging requires the verified immutable desktop cache')
            subprocess.run(['node', str(cli), 'build', '--runner', 'cargo-xwin',
                            '--target', target, '--bundles', 'nsis'], cwd=app, env=env, check=True)
        bundle = args.target_dir.resolve() / target / ('release/bundle/' + package_format)
        installers = list(bundle.glob('*.msi' if package_format == 'msi' else '*setup.exe'))
        if len(installers) != 1:
            raise ValueError(f'exactly one canonical {package_format.upper()} installer required for {target}')
        installer = installers[0]
        if package_format == 'msi':
            import xml.etree.ElementTree as ET
            inspection = args.target_dir.resolve() / target / 'release/msi-inspection'
            inspection.mkdir()
            xml = inspection / 'decompiled.wxs'
            dark = args.msi_tools.resolve() / 'wix/dark.exe'
            inspection_env = {key: value for key, value in native_env.items()
                              if key in {'PATH', 'HOME', 'TMP', 'TEMP', 'SYSTEMROOT'}}
            inspection_env.update(json.loads((args.msi_tools / 'toolchain-receipt.json').read_text())['launcher_env'])
            wine32 = str(args.msi_tools.resolve() / 'root/usr/lib/wine/wine')
            subprocess.run([wine32, native_tauri.windows_path(dark), native_tauri.windows_path(installer),
                            '-x', native_tauri.windows_path(inspection), '-o', native_tauri.windows_path(xml)],
                           env=inspection_env, check=True, timeout=120)
            tree = ET.parse(xml)
            product = next(element for element in tree.iter() if element.tag.endswith('}Product'))
            if product.get('Version') != args.tag.removeprefix('v'):
                raise ValueError('MSI database product version mismatch')
            expected_binary = args.target_dir.resolve() / target / 'release/focusa-menubar.exe'
            expected_hash = hashlib.sha256(expected_binary.read_bytes()).hexdigest()
            payloads = [file for file in inspection.rglob('*') if file.is_file() and file.stat().st_size == expected_binary.stat().st_size]
            if not any(hashlib.sha256(file.read_bytes()).hexdigest() == expected_hash for file in payloads):
                raise ValueError('MSI extracted application does not match the verified bundle input')
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
        # Pipeline owner preserves compiled inputs before its final Cargo cleanup.
    receipt = {'kind': 'ovh_windows_' + package_format, 'tag': args.tag, 'source_sha': args.sha,
               'artifacts': records, 'updater_signature_verification': 'passed',
               'native_windows_proof': False, 'msi_proof': package_format == 'msi',
               'full_release_acceptance': False,
               'msi_database_and_payload_verification': 'passed' if package_format == 'msi' else 'not_applicable',
               'windows_ice_validation': 'not_run_wine_unsupported' if package_format == 'msi' else 'not_applicable'}
    (output / ('windows-' + package_format + '-receipt.json')).write_text(json.dumps(receipt, indent=2) + '\n')
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
    parser.add_argument('--desktop-msi', action='store_true')
    parser.add_argument('--msi-tools', type=Path)
    parser.add_argument('--desktop-cache', type=Path)
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
        if args.desktop_nsis or args.desktop_msi:
            commands = [['node', 'node_modules/@tauri-apps/cli/tauri.js', 'build',
                         '--runner', 'cargo-xwin', '--target', target, '--bundles', 'nsis']
                        for target in targets]
            if args.desktop_msi:
                commands = [['wine', 'node.exe', 'node_modules/@tauri-apps/cli/tauri.js',
                             'bundle', '--target', target, '--bundles', 'msi'] for target in targets]
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
    if args.desktop_nsis or args.desktop_msi:
        if args.desktop_nsis and args.desktop_msi:
            raise ValueError('one packaging mode per invocation required')
        if args.desktop_msi and not args.msi_tools:
            raise ValueError('verified MSI tool directory required')
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
