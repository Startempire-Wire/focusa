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
        loader.write_text('#!/bin/sh\n' + exports + '\nexport WINELOADER=' + shlex.quote(str(tools / 'root/usr/lib/wine/wine'))
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
    inputs.append({'kind': 'wix_wine_execution_adapter',
                   'windows_ice_validation': 'not_run_wine_unsupported'})
    (native / 'native-tool-inputs.json').write_text(json.dumps({'inputs': inputs, 'tauri_cli_version': cli_version, 'native_windows_proof': False}, indent=2) + '\n')
    return [receipt['launcher'], str(node)], env


def verify_control_references(tools, installer, env):
    """Check actual MSI rows, independently of Dark's reconstructed UI tree."""
    source = tools / 'verify-msi-controls.cs'
    source.write_text('''using System; using System.Text; using System.Collections.Generic; using System.Runtime.InteropServices;
class MsiControls {
 [DllImport("msi.dll",CharSet=CharSet.Unicode)] static extern uint MsiOpenDatabaseW(string path,IntPtr mode,out uint db);
 [DllImport("msi.dll",CharSet=CharSet.Unicode)] static extern uint MsiDatabaseOpenViewW(uint db,string sql,out uint view);
 [DllImport("msi.dll")] static extern uint MsiViewExecute(uint view,uint record);
 [DllImport("msi.dll")] static extern uint MsiViewFetch(uint view,out uint record);
 [DllImport("msi.dll",CharSet=CharSet.Unicode)] static extern uint MsiRecordGetStringW(uint record,uint field,StringBuilder text,ref uint length);
 [DllImport("msi.dll")] static extern uint MsiCloseHandle(uint handle);
 static void Require(uint result){if(result!=0)throw new Exception("MSI API status "+result);}
 static HashSet<string> Rows(uint db,string query){uint view;Require(MsiDatabaseOpenViewW(db,query,out view));
  try{Require(MsiViewExecute(view,0));var rows=new HashSet<string>();uint row;uint rc;
   while((rc=MsiViewFetch(view,out row))==0){try{string key="";for(uint field=1;field<=2;field++){
    uint length=4095;var text=new StringBuilder(4096);Require(MsiRecordGetStringW(row,field,text,ref length));key+=text.ToString()+"\\u001f";
   }rows.Add(key);}finally{MsiCloseHandle(row);}}if(rc!=259)Require(rc);return rows;
  }finally{MsiCloseHandle(view);}}
 static int Main(string[] args){uint db=0;try{Require(MsiOpenDatabaseW(args[0],IntPtr.Zero,out db));
  var controls=Rows(db,"SELECT `Dialog_`, `Control` FROM `Control`");
  var events=Rows(db,"SELECT `Dialog_`, `Control_` FROM `ControlEvent`");
  if(controls.Count==0||events.Count==0)throw new Exception("Missing installer UI tables");
  foreach(string row in events)if(!controls.Contains(row))throw new Exception("Missing MSI Control foreign row: "+row);
  Console.WriteLine("MSI control references passed: controls="+controls.Count+" event controls="+events.Count);return 0;
 }catch(Exception e){Console.Error.WriteLine(e.Message);return 1;}finally{if(db!=0)MsiCloseHandle(db);}}
}''')
    receipt = json.loads((tools / 'toolchain-receipt.json').read_text())
    runtime = {key: value for key, value in env.items() if key in {'PATH', 'HOME', 'TMP', 'TEMP', 'SYSTEMROOT'}}
    runtime.update(receipt['launcher_env'])
    compilers = sorted((Path(runtime['WINEPREFIX']) / 'drive_c/windows').glob('**/csc.exe'))
    if not compilers:
        compilers = sorted((Path(runtime['WINEPREFIX']) / 'drive_c/windows').glob('**/mcs.exe'))
    if not compilers:
        raise ValueError('existing Mono compiler required for MSI table integrity proof')
    wine = str(tools / 'root/usr/lib/wine/wine')
    executable = tools / 'verify-msi-controls.exe'
    subprocess.run([wine, windows_path(compilers[0]), '/nologo', '/target:exe', '/platform:x86',
                    '/out:' + windows_path(executable), windows_path(source)], env=runtime, check=True, timeout=90)
    subprocess.run([wine, windows_path(executable), windows_path(installer)], env=runtime, check=True, timeout=90)


def finish_generated_msi(app, tools, target_dir, target, env):
    """Execute Tauri's generated WiX recipe at the owned Unix/Wine boundary."""
    config = json.loads((app / 'src-tauri/tauri.conf.json').read_text())
    if config['bundle'].get('windows', {}).get('wix'):
        raise ValueError('custom WiX recipes require their owning execution adapter')
    arch = {'x86_64-pc-windows-msvc': 'x64', 'aarch64-pc-windows-msvc': 'arm64'}[target]
    release = target_dir.resolve() / target / 'release'
    build = release / 'wix' / arch
    wxs = build / 'main.wxs'
    locale = build / 'locale.wxl'
    if not wxs.is_file() or not locale.is_file():
        raise ValueError('package-owned Tauri WiX source and locale required')
    receipt = json.loads((tools / 'toolchain-receipt.json').read_text())
    runtime = {key: value for key, value in env.items()
               if key in {'PATH', 'HOME', 'TMP', 'TEMP', 'SYSTEMROOT'}}
    runtime.update(receipt['launcher_env'])
    wine = str(tools / 'root/usr/lib/wine/wine')
    wix = tools / 'wix'
    binary = release / 'focusa-menubar.exe'
    subprocess.run([wine, windows_path(wix / 'candle.exe'), '-arch', arch,
                    windows_path(wxs), '-dSourceDir=' + windows_path(binary)],
                   cwd=build, env=runtime, check=True, timeout=120)
    msi = build / 'output.msi'
    subprocess.run([wine, windows_path(wix / 'light.exe'), '-sval',
                    '-ext', windows_path(wix / 'WixUIExtension.dll'),
                    '-ext', windows_path(wix / 'WixUtilExtension.dll'),
                    '-o', windows_path(msi), '-cultures:en-us',
                    '-loc', windows_path(locale), '*.wixobj'],
                   cwd=build, env=runtime, check=True, timeout=180)
    bundle = release / 'bundle/msi'
    bundle.mkdir(parents=True, exist_ok=True)
    installer = bundle / (config['productName'] + '_' + config['version'] + '_' + arch + '_en-US.msi')
    msi.rename(installer)
    # The package-owned signer reads approved injected key references from env;
    # no secret enters arguments, files, inspection subprocesses or receipts.
    subprocess.run(['node', str(app / 'node_modules/@tauri-apps/cli/tauri.js'),
                    'signer', 'sign', str(installer)], cwd=app, env=env, check=True, timeout=90)
