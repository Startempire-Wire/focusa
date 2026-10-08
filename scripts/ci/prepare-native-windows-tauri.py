#!/usr/bin/env python3
"""Run the lock-owned Windows Tauri packager in the verified isolated Wine prefix."""
import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import urllib.request
import zipfile

NODE_VERSION = '22.22.3'
CARGO_VERSION = '1.91.0'
PACKAGE = 'node_modules/@tauri-apps/cli-win32-x64-msvc'


def windows_path(path):
    return 'Z:' + str(Path(path).resolve()).replace('/', '\\')


def unpack_tar(archive, destination):
    destination.mkdir(parents=True, exist_ok=True)
    with tarfile.open(archive) as source:
        for member in source.getmembers():
            (destination / member.name).resolve().relative_to(destination.resolve())
        source.extractall(destination, filter='data')


def prepare(app, tools, env):
    receipt = json.loads((tools / 'toolchain-receipt.json').read_text())
    if receipt.get('wine_api_execution') != 'passed':
        raise ValueError('verified Wine/WiX prefix required')
    spec = importlib.util.spec_from_file_location('wine_tools', Path(__file__).with_name('prepare-windows-msi-wine.py'))
    owner = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(owner)
    native = tools / 'native-cli'
    native.mkdir()
    node_name = f'node-v{NODE_VERSION}-win-x64.zip'
    node_base = f'https://nodejs.org/dist/v{NODE_VERSION}/'
    with urllib.request.urlopen(node_base + 'SHASUMS256.txt', timeout=60) as response:
        checksums = dict((line.split()[1], line.split()[0]) for line in response.read().decode().splitlines() if len(line.split()) == 2)
    inputs = [owner.download(node_base + node_name, native / node_name, checksums[node_name])]
    with zipfile.ZipFile(native / node_name) as archive:
        for member in archive.infolist():
            (native / member.filename).resolve().relative_to(native.resolve())
        archive.extractall(native)
    node = native / f'node-v{NODE_VERSION}-win-x64/node.exe'
    lock = json.loads((app / 'package-lock.json').read_text())
    dependency = lock['packages'][PACKAGE]
    cli_version = lock['packages']['node_modules/@tauri-apps/cli']['version']
    if dependency['version'] != cli_version or not dependency['integrity'].startswith('sha512-'):
        raise ValueError('native CLI must match the package-owned locked Tauri version')
    if not dependency['resolved'].startswith('https://registry.npmjs.org/@tauri-apps/'):
        raise ValueError('unexpected native CLI source')
    digest = base64.b64decode(dependency['integrity'].split('-', 1)[1], validate=True).hex()
    inputs.append(owner.download(dependency['resolved'], native / 'tauri-native.tgz', digest, algorithm='sha512'))
    unpack_tar(native / 'tauri-native.tgz', native / 'tauri')
    shutil.copytree(native / 'tauri/package', app / PACKAGE)
    cargo_name = f'cargo-{CARGO_VERSION}-x86_64-pc-windows-msvc.tar.xz'
    cargo_url = 'https://static.rust-lang.org/dist/' + cargo_name
    with urllib.request.urlopen(cargo_url + '.sha256', timeout=60) as response:
        cargo_digest = response.read().decode().split()[0]
    if not re.fullmatch('[a-f0-9]{64}', cargo_digest):
        raise ValueError('invalid upstream Cargo checksum')
    inputs.append(owner.download(cargo_url, native / cargo_name, cargo_digest))
    unpack_tar(native / cargo_name, native / 'cargo')
    cargo = native / 'cargo' / cargo_name.removesuffix('.tar.xz') / 'cargo/bin/cargo.exe'
    if not node.is_file() or not cargo.is_file():
        raise ValueError('pinned native packaging tools missing')
    env.update(receipt['launcher_env'])
    # Tauri intentionally clears non-TAURI environment for WiX. Relocated
    # Unix Wine loaders still need their owned prefix and ELF search paths.
    # Bind them at the executable boundary, not by weakening that secret filter.
    import shlex
    for name in ['wine', 'wine64']:
        loader = tools / 'root/usr/lib/wine' / name
        real = loader.with_name(name + '.real')
        loader.rename(real)
        exports = '\n'.join('export ' + key + '=' + shlex.quote(env[key]) for key in receipt['launcher_env'])
        loader.write_text('#!/bin/sh\n' + exports + '\nexport WINELOADER=' + shlex.quote(str(loader))
                          + '\nexec ' + shlex.quote(str(real)) + ' "$@"\n')
        loader.chmod(0o755)
    env['WINEPATH'] = windows_path(cargo.parent) + ';' + windows_path(node.parent)
    # MSI structured storage is authored on the prefix's local Windows drive,
    # not Wine's Unix-root Z: mapping; the link still targets the owned job tree.
    target_directory = Path(env['CARGO_TARGET_DIR']).resolve()
    local_target = Path(env['WINEPREFIX']) / 'drive_c/focusa-build-target'
    local_target.symlink_to(target_directory, target_is_directory=True)
    env['CARGO_TARGET_DIR'] = 'C:\\focusa-build-target'
    users = Path(env['WINEPREFIX']) / 'drive_c/users'
    profiles = [p for p in users.iterdir() if p.name not in {'Public', 'All Users'} and (p / 'AppData/Local').is_dir()]
    if len(profiles) != 1:
        raise ValueError('unambiguous isolated Wine user profile required')
    wix = profiles[0] / 'AppData/Local/tauri/WixTools314'
    shutil.copytree(tools / 'wix', wix, dirs_exist_ok=True)
    # Upstream WiX/Wine cannot execute Windows ICE validation. The explicit
    # compatibility adapter retains vendor bytes; publication additionally
    # requires database decompilation, exact payload identity and signatures.
    compilers = sorted((Path(env['WINEPREFIX']) / 'drive_c/windows').glob('**/csc.exe'))
    if not compilers:
        compilers = sorted((Path(env['WINEPREFIX']) / 'drive_c/windows').glob('**/mcs.exe'))
    if not compilers:
        raise ValueError('existing Mono compiler required for the Wine compatibility adapter')
    adapter = native / 'wix-wine-light.cs'
    adapter.write_text('''using System; using System.IO; using System.Reflection;
class WineLight { static int Main(string[] args) {
 try { string path=Path.Combine(Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location), "light.vendor.exe");
 string[] forwarded=new string[args.Length+1]; Array.Copy(args,forwarded,args.Length); forwarded[args.Length]="-sval";
 object result=Assembly.LoadFrom(path).EntryPoint.Invoke(null,new object[]{forwarded}); return result is int ? (int)result : 0;
 } catch(Exception e) {Console.Error.WriteLine(e.GetBaseException()); return 1;}
} }''')
    (wix / 'light.exe').rename(wix / 'light.vendor.exe')
    subprocess.run([receipt['launcher'], str(compilers[0]), '/nologo', '/target:exe', '/platform:x86',
                    '/out:' + windows_path(wix / 'light.exe'), windows_path(adapter)], env=env, check=True, timeout=90)
    inputs.append({'kind': 'wix_wine_compatibility_adapter', 'source_sha256': hashlib.sha256(adapter.read_bytes()).hexdigest(),
                   'windows_ice_validation': 'not_run_wine_unsupported'})
    (native / 'native-tool-inputs.json').write_text(json.dumps({'inputs': inputs, 'tauri_cli_version': cli_version, 'native_windows_proof': False}, indent=2) + '\n')
    return [receipt['launcher'], str(node)], env
