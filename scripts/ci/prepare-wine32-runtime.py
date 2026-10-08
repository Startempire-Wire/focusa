#!/usr/bin/env python3
"""Matching i386 Wine inputs in the existing owned prefix; no host installation."""
import hashlib
import json
import os
from pathlib import Path
import subprocess


def prepare(tools, root):
    apt = tools / 'apt32'
    (apt / 'lists/partial').mkdir(parents=True)
    (apt / 'archives/partial').mkdir(parents=True)
    (apt / 'status').write_text('')
    config = apt / 'isolated.conf'
    config.write_text('Dir::Etc::parts "-";\nDir::Etc::main "/dev/null";\n')
    env = os.environ.copy()
    env['APT_CONFIG'] = str(config)
    sources = apt / 'sources.list'
    sources.write_text('deb [arch=i386 signed-by=/usr/share/keyrings/ubuntu-archive-keyring.gpg] http://archive.ubuntu.com/ubuntu noble main universe\n'
                       'deb [arch=i386 signed-by=/usr/share/keyrings/ubuntu-archive-keyring.gpg] http://archive.ubuntu.com/ubuntu noble-updates main universe\n'
                       'deb [arch=i386 signed-by=/usr/share/keyrings/ubuntu-archive-keyring.gpg] http://security.ubuntu.com/ubuntu noble-security main universe\n')
    options = ['-o', 'APT::Architecture=i386', '-o', 'APT::Architectures::=i386',
               '-o', 'Dir::Etc::sourcelist=' + str(sources), '-o', 'Dir::Etc::sourceparts=-',
               '-o', 'Dir::State::lists=' + str(apt / 'lists'),
               '-o', 'Dir::State::status=' + str(apt / 'status'),
               '-o', 'Dir::Cache::archives=' + str(apt / 'archives'),
               '-o', 'Debug::NoLocking=true', '-o', 'APT::Install-Recommends=false',
               '-o', 'APT::Sandbox::User=wirebot']
    subprocess.run(['apt-get', *options, 'update'], env=env, check=True, timeout=180)
    subprocess.run(['apt-get', *options, '--download-only', '--yes', 'install',
                    'wine32:i386=9.0~repack-4build3', 'libwine:i386=9.0~repack-4build3'],
                   env=env, check=True, timeout=300)
    records = []
    for package in sorted((apt / 'archives').glob('*.deb')):
        identity = subprocess.check_output(['dpkg-deb', '-f', str(package), 'Package', 'Version', 'Architecture'], text=True).strip()
        records.append({'identity': identity, 'sha256': hashlib.sha256(package.read_bytes()).hexdigest()})
        subprocess.run(['dpkg-deb', '-x', str(package), str(root)], check=True)
    if not (root / 'usr/lib/wine/wine').is_file():
        raise ValueError('matching classic Wine32 loader missing')
    (tools / 'wine32-input-sha256.json').write_text(json.dumps(records, indent=2) + '\n')
