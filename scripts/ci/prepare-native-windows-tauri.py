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
    destination = app / PACKAGE
    shutil.copytree(native / 'tauri/package', destination)
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
    env['WINEPATH'] = windows_path(cargo.parent) + ';' + windows_path(node.parent)
    env['CARGO_TARGET_DIR'] = windows_path(env['CARGO_TARGET_DIR'])
    # Use the prefix's actual Windows cache location, not a second WiX download.
    users = Path(env['WINEPREFIX']) / 'drive_c/users'
    profiles = [p for p in users.iterdir() if p.name not in {'Public', 'All Users'} and (p / 'AppData/Local').is_dir()]
    if len(profiles) != 1:
        raise ValueError('unambiguous isolated Wine user profile required')
    shutil.copytree(tools / 'wix', profiles[0] / 'AppData/Local/tauri/WixTools314', dirs_exist_ok=True)
    (native / 'native-tool-inputs.json').write_text(json.dumps({'inputs': inputs, 'tauri_cli_version': cli_version, 'native_windows_proof': False}, indent=2) + '\n')
    return [receipt['launcher'], str(node)], env
