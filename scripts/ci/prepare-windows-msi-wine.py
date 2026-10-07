#!/usr/bin/env python3
"""Prepare an isolated Wine/WiX build-tool prefix; never claim native Windows proof."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import urllib.request
import zipfile

WIX_URL = 'https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip'
# Authoritative tauri-cli-v2.11.2 tauri-bundler/windows/msi constant.
WIX_SHA256 = '6ac824e1642d6f7277d0ed7ea09411a508f6116ba6fae0aa5f2c7daa2ff43d31'
MONO_URL = 'https://github.com/wine-mono/wine-mono/releases/download/wine-mono-9.0.0/wine-mono-9.0.0-x86.msi'
PACKAGES = ['wine64=9.0~repack-4build3', 'libwine=9.0~repack-4build3']


def download(url, destination, expected=None):
    with urllib.request.urlopen(url, timeout=60) as response, destination.open('wb') as out:
        while chunk := response.read(1024 * 1024):
            out.write(chunk)
    digest = hashlib.sha256(destination.read_bytes()).hexdigest()
    if expected and digest != expected:
        raise ValueError('upstream tool checksum mismatch: ' + destination.name)
    return {'url': url, 'name': destination.name, 'sha256': digest,
            'external_checksum_verified': expected is not None}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tools-directory', required=True, type=Path)
    parser.add_argument('--plan', action='store_true')
    args = parser.parse_args()
    if args.plan:
        print(json.dumps({'packages': PACKAGES, 'wix_url': WIX_URL,
                          'wix_sha256': WIX_SHA256, 'mono_url': MONO_URL,
                          'native_windows_proof': False, 'installer_proof': False}))
        return
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        raise ValueError('approved self-hosted CI build-tool context required')
    tools = args.tools_directory.resolve()
    tools.relative_to(Path(os.environ['RUNNER_TEMP']).resolve())
    tools.mkdir(parents=True, exist_ok=False)
    packages = tools / 'packages'
    packages.mkdir()
    root = tools / 'root'
    root.mkdir()
    subprocess.run(['apt-get', 'download', *PACKAGES], cwd=packages, check=True)
    for package in packages.glob('*.deb'):
        subprocess.run(['dpkg-deb', '-x', str(package), str(root)], check=True)
    wine = root / 'usr/lib/wine/wine64'
    server = root / 'usr/lib/wine/wineserver64'
    library = root / 'usr/lib/x86_64-linux-gnu/wine'
    env = os.environ.copy()
    env['WINEPREFIX'] = str(tools / 'prefix')
    env['WINEARCH'] = 'win64'
    env['WINESERVER'] = str(server)
    env['WINELOADER'] = str(wine)
    env['PATH'] = str(wine.parent) + os.pathsep + env['PATH']
    env['WINEDLLPATH'] = ':'.join(str(library / arch) for arch in ['x86_64-windows', 'x86_64-unix'])
    env['LD_LIBRARY_PATH'] = str(library / 'x86_64-unix')
    env['WINEDLLOVERRIDES'] = 'mscoree,mshtml='
    subprocess.run([str(server), '--version'], env=env, check=True, timeout=30)
    subprocess.run([str(wine), 'wineboot', '-u'], env=env, check=True, timeout=120)
    artifacts = [download(WIX_URL, tools / 'wix.zip', WIX_SHA256),
                 download(MONO_URL, tools / 'wine-mono-9.0.0-x86.msi')]
    wix = tools / 'wix'
    wix.mkdir()
    with zipfile.ZipFile(tools / 'wix.zip') as archive:
        for member in archive.infolist():
            (wix / member.filename).resolve().relative_to(wix.resolve())
        archive.extractall(wix)
    env.pop('WINEDLLOVERRIDES')
    subprocess.run([str(wine), 'msiexec', '/i', str(tools / 'wine-mono-9.0.0-x86.msi'),
                    '/quiet', '/norestart'], env=env, check=True, timeout=180)
    subprocess.run([str(wine), str(wix / 'candle.exe'), '-?'], env=env, check=True, timeout=90)
    subprocess.run([str(wine), str(wix / 'light.exe'), '-?'], env=env, check=True, timeout=90)
    receipt = {'kind': 'ovh_wine_wix_toolchain', 'artifacts': artifacts,
               'wine_api_execution': 'passed', 'native_windows_proof': False,
               'installer_proof': False, 'prefix': env['WINEPREFIX']}
    (tools / 'toolchain-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()
