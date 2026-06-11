#!/usr/bin/env python3
"""Stamp the Focusa menubar app version from a release tag.

Usage:
  scripts/stamp-menubar-version.py v0.9.16-dev
  scripts/stamp-menubar-version.py 0.9.16-dev

This keeps Tauri DMG/app asset names diffable per dev release.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION_RE = re.compile(r"^v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$")
OLD_VERSION_RE = re.compile(r"\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?")


def parse_version(raw: str) -> str:
    value = raw.strip()
    match = VERSION_RE.match(value)
    if not match:
        raise SystemExit(f"Invalid menubar release tag/version: {raw!r}")
    return match.group(1)


def replace_json_version(path: str, version: str) -> None:
    file_path = ROOT / path
    data = json.loads(file_path.read_text())
    data["version"] = version
    if path.endswith("package-lock.json"):
        packages = data.get("packages")
        if isinstance(packages, dict):
            root_pkg = packages.get("")
            if isinstance(root_pkg, dict):
                root_pkg["version"] = version
    file_path.write_text(json.dumps(data, indent=2) + "\n")


def replace_key_value_version(path: str, version: str) -> None:
    file_path = ROOT / path
    text = file_path.read_text()
    text = re.sub(r'(?m)^version\s*=\s*"[^"]+"', f'version = "{version}"', text, count=1)
    file_path.write_text(text)


def replace_display_version(path: str, version: str) -> None:
    file_path = ROOT / path
    text = file_path.read_text()
    text = re.sub(r"Focusa v" + OLD_VERSION_RE.pattern, f"Focusa v{version}", text)
    file_path.write_text(text)


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("Usage: scripts/stamp-menubar-version.py <tag-or-version>")
    version = parse_version(sys.argv[1])
    replace_json_version("apps/menubar/package.json", version)
    replace_json_version("apps/menubar/package-lock.json", version)
    replace_json_version("apps/menubar/src-tauri/tauri.conf.json", version)
    replace_key_value_version("apps/menubar/src-tauri/Cargo.toml", version)
    replace_key_value_version("apps/menubar/src-tauri/Cargo.lock", version)
    replace_display_version("apps/menubar/src/lib/components/Settings.svelte", version)
    print(f"Stamped menubar version {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
