#!/usr/bin/env python3
"""Fail closed when a canonical Focusa release omits any supported surface/system."""

from __future__ import annotations

import argparse
import fnmatch
import os
from pathlib import Path

RUST_TARGETS = (
    ("aarch64-apple-darwin", ""),
    ("x86_64-apple-darwin", ""),
    ("x86_64-unknown-linux-gnu", ""),
    ("x86_64-unknown-linux-musl", ""),
    ("x86_64-pc-windows-msvc", ".exe"),
    ("aarch64-pc-windows-msvc", ".exe"),
)
RUST_SURFACES = ("focusa", "focusa-daemon", "focusa-tui")


def temp_ovh_lane() -> bool:
    # TEMPORARY-OVH-LANE (until GitHub-hosted minutes restore ~2026-09-01):
    # only the Linux surfaces producible on the OVH builder lane are required;
    # macOS/Windows legs (and their desktop bundles) are deferred until GH
    # returns. The flag is set explicitly by the release workflow, never default.
    return os.environ.get("FOCUSA_TEMP_OVH_LANE") == "1"


def supported_rust_targets() -> tuple[tuple[str, str], ...]:
    if temp_ovh_lane():
        return (
            ("x86_64-unknown-linux-gnu", ""),
            ("x86_64-unknown-linux-musl", ""),
        )
    return RUST_TARGETS


def required_exact(tag: str) -> list[str]:
    required = [
        f"{surface}-{tag}-{target}{suffix}"
        for target, suffix in supported_rust_targets()
        for surface in RUST_SURFACES
    ]
    required.extend(
        [
            f"focusa-pi-extension-{tag}.tar.gz",
            f"focusa-agent-context-{tag}.tar.gz",
            f"focusa-generated-clients-{tag}.tar.gz",
            f"focusa-installer-{tag}.sh",
            f"focusa-installer-{tag}.ps1",
        ]
    )
    if temp_ovh_lane():
        # TEMPORARY-OVH-LANE: desktop bundles are deferred with tauri-build.
        return required
    required.extend(
        [
            f"Focusa-{tag}-aarch64-apple-darwin.app.zip",
            f"Focusa-{tag}-x86_64-apple-darwin.app.zip",
        ]
    )
    return required


def required_patterns() -> list[str]:
    if temp_ovh_lane():
        return []
    return [
        "Focusa_*aarch64*.dmg",
        "Focusa_*x64*.dmg",
        "Focusa_*x64*setup.exe",
        "Focusa_*arm64*setup.exe",
        "Focusa_*x64*.msi",
        "Focusa_*arm64*.msi",
    ]


def verify(directory: Path, tag: str) -> list[str]:
    names = {path.name for path in directory.iterdir() if path.is_file()}
    missing = [name for name in required_exact(tag) if name not in names]
    missing.extend(
        f"pattern:{pattern}"
        for pattern in required_patterns()
        if not any(fnmatch.fnmatchcase(name, pattern) for name in names)
    )
    return missing


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dist", type=Path, required=True)
    parser.add_argument("--tag", required=True)
    args = parser.parse_args()
    missing = verify(args.dist, args.tag)
    if missing:
        print("canonical_release_assets=FAIL")
        for item in missing:
            print(f"missing={item}")
        return 1
    if temp_ovh_lane():
        # TEMPORARY-OVH-LANE: report the honest deferred scope.
        print(
            "canonical_release_assets=PASS (TEMPORARY_OVH_LANE=1) "
            "systems=linux surfaces=cli,daemon,tui,pi,agent-context,generated-clients,installers "
            "deferred=desktop-macos,desktop-windows (restore ~2026-09-01)"
        )
        return 0
    print(
        "canonical_release_assets=PASS "
        "systems=macos,linux,windows surfaces=cli,daemon,tui,desktop,pi,agent-context,generated-clients,installers"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
