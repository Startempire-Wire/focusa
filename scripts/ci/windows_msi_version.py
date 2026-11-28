"""Project the exact stable/dev release identity into Windows Installer's version field."""

import argparse
import json
from pathlib import Path
import re

TAURI_CONFIG = Path('apps/menubar/src-tauri/tauri.conf.json')
RELEASE_VERSION = re.compile(r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-dev)?')


def project_msi_version(version):
    match = RELEASE_VERSION.fullmatch(version)
    if not match:
        raise ValueError(f'unsupported Windows release version: {version!r}')
    components = match.groups()[:3]
    if any(int(component) > maximum for component, maximum in zip(components, (255, 255, 65535))):
        raise ValueError(f'Windows MSI ProductVersion out of range: {version!r}')
    numeric = '.'.join(components)
    return numeric, {'version': numeric} if match[4] else None


def stamp_tauri_version(config, version):
    """Called only by the canonical version writer; preserve unrelated Tauri settings."""
    previous = config['version']
    # Nightly and non-release SemVer surfaces have no MSI producer. Stable/dev
    # patches are unique across release channels, so the numeric tuple orders them.
    next_override = project_msi_version(version)[1] if RELEASE_VERSION.fullmatch(version) else None
    windows = config.get('bundle', {}).get('windows', {})
    existing = windows.get('wix')
    if existing is not None:
        previous_override = project_msi_version(previous)[1] if RELEASE_VERSION.fullmatch(previous) else None
        if existing != previous_override:
            raise ValueError('refusing to replace a non-generated WiX configuration')
    if next_override is not None:
        config.setdefault('bundle', {}).setdefault('windows', {})['wix'] = next_override
    elif existing is not None:
        del windows['wix']
        if not windows:
            del config['bundle']['windows']
    config['version'] = version


def checked_msi_version(config, release_tag):
    version = release_tag.removeprefix('v')
    numeric, expected = project_msi_version(version)
    if config.get('version') != version:
        raise ValueError('Tauri version does not match the exact release tag')
    if config.get('bundle', {}).get('windows', {}).get('wix') != expected:
        raise ValueError(f'missing or unexpected WiX ProductVersion projection for {release_tag}')
    return numeric


def checked_source_msi_version(source, release_tag):
    config = json.loads((source / TAURI_CONFIG).read_text(encoding='utf-8'))
    return checked_msi_version(config, release_tag)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('tag')
    parser.add_argument('--source', type=Path, default=Path.cwd())
    args = parser.parse_args()
    print(checked_source_msi_version(args.source, args.tag))
