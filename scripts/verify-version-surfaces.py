#!/usr/bin/env python3
"""Verify Focusa version surfaces are aligned to one expected release version.

Usage:
  scripts/verify-version-surfaces.py v0.9.41-dev
  scripts/verify-version-surfaces.py 0.9.41-dev
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION_RE = re.compile(r"^v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$")
SETTINGS_RE = re.compile(r"(v?)(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)")
PACKAGE_RE = re.compile(r'^name\s*=\s*"([^"]+)"\s*$')
LOCK_VERSION_RE = re.compile(r'^version\s*=\s*"([^"]+)"\s*$')

ROOT_RUST_PACKAGES = {
    "focusa-api",
    "focusa-bench",
    "focusa-cli",
    "focusa-core",
    "focusa-harness-adapters",
    "focusa-license",
    "focusa-session-runner",
    "focusa-terminal-ui",
    "focusa-tui",
}
MENUBAR_RUST_PACKAGES = {"focusa-menubar"}


def parse_version(raw: str) -> str:
    match = VERSION_RE.match(raw.strip())
    if not match:
        raise SystemExit(f"Invalid version/tag: {raw!r}")
    return match.group(1)


def read_toml_version(path: str) -> str:
    for line in (ROOT / path).read_text(encoding="utf-8").splitlines():
        if line.startswith("version = "):
            return line.split('"')[1]
    raise SystemExit(f"version key not found: {path}")


def read_json_version(path: str) -> str:
    return json.loads((ROOT / path).read_text(encoding="utf-8"))["version"]


def read_settings_version(path: str) -> str:
    text = (ROOT / path).read_text(encoding="utf-8")
    match = re.search(r"v\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?", text)
    if not match:
        raise SystemExit(f"display version not found: {path}")
    return match.group(0).lstrip("v")


def read_extension_build_version(path: str, package_name: str) -> str:
    text = (ROOT / path).read_text(encoding="utf-8")
    match = re.search(
        rf'const EXTENSION_BUILD = "{re.escape(package_name)}@'
        r'(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)"',
        text,
    )
    if not match:
        raise SystemExit(f"EXTENSION_BUILD identity not found: {path}")
    return match.group(1)


def read_installer_version(path: str) -> str:
    text = (ROOT / path).read_text(encoding="utf-8")
    match = re.search(r'(?m)^FOCUSA_INSTALLER_VERSION="([^"]+)"$', text)
    if not match:
        raise SystemExit(f"installer version not found: {path}")
    return match.group(1)


def read_lock_versions(path: str, package_names: set[str]) -> dict[str, str]:
    current_name: str | None = None
    versions: dict[str, str] = {}
    for line in (ROOT / path).read_text(encoding="utf-8").splitlines():
        name_match = PACKAGE_RE.match(line)
        if name_match:
            current_name = name_match.group(1)
            continue
        version_match = LOCK_VERSION_RE.match(line)
        if (
            version_match
            and current_name in package_names
            and current_name not in versions
        ):
            versions[current_name] = version_match.group(1)
    missing = package_names - versions.keys()
    if missing:
        raise SystemExit(f"Missing package(s) in {path}: {', '.join(sorted(missing))}")
    return versions


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("Usage: scripts/verify-version-surfaces.py <tag-or-version>")
    expected = parse_version(sys.argv[1])

    checks: list[tuple[str, str]] = [
        ("Cargo.toml", read_toml_version("Cargo.toml")),
        (
            "apps/pi-extension/package.json",
            read_json_version("apps/pi-extension/package.json"),
        ),
        (
            "apps/pi-extension/package-lock.json",
            read_json_version("apps/pi-extension/package-lock.json"),
        ),
        (
            "apps/pi-extension/src/auto-compaction.ts::EXTENSION_BUILD",
            read_extension_build_version(
                "apps/pi-extension/src/auto-compaction.ts", "focusa-pi-bridge"
            ),
        ),
        (
            "docs/contracts/spec141/generated-capability-v2/agent-card.json",
            read_json_version(
                "docs/contracts/spec141/generated-capability-v2/agent-card.json"
            ),
        ),
        (
            "scripts/install-focusa.sh::FOCUSA_INSTALLER_VERSION",
            read_installer_version("scripts/install-focusa.sh"),
        ),
        ("apps/menubar/package.json", read_json_version("apps/menubar/package.json")),
        (
            "apps/menubar/src-tauri/tauri.conf.json",
            read_json_version("apps/menubar/src-tauri/tauri.conf.json"),
        ),
        (
            "apps/menubar/src-tauri/Cargo.toml",
            read_toml_version("apps/menubar/src-tauri/Cargo.toml"),
        ),
        (
            "apps/menubar/src/lib/components/Settings.svelte",
            read_settings_version("apps/menubar/src/lib/components/Settings.svelte"),
        ),
    ]

    checks.extend(
        (f"Cargo.lock::{name}", version)
        for name, version in sorted(
            read_lock_versions("Cargo.lock", ROOT_RUST_PACKAGES).items()
        )
    )
    checks.extend(
        (f"apps/menubar/src-tauri/Cargo.lock::{name}", version)
        for name, version in sorted(
            read_lock_versions(
                "apps/menubar/src-tauri/Cargo.lock", MENUBAR_RUST_PACKAGES
            ).items()
        )
    )

    mismatches = [(label, actual) for label, actual in checks if actual != expected]
    if mismatches:
        print(f"Version surface mismatch; expected {expected}:", file=sys.stderr)
        for label, actual in mismatches:
            print(f"  - {label}: {actual}", file=sys.stderr)
        return 1

    print(f"All checked version surfaces match {expected}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
