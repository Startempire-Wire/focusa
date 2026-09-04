#!/usr/bin/env python3
"""Wait for external-provider release assets (Spec 178) to attach to a GitHub Release.

The GitHub hosted macOS/Windows runners are billing-locked, so the canonical
release matrix is carried by:
  - Linux   -> OVH self-hosted runners (built inside release.yml rust-release)
  - macOS   -> Codemagic (macos-rust-binaries + menubar-macos-package-proof)
  - Windows -> AppVeyor (full Rust binaries + Menubar NSIS/MSI)

Each provider uploads its artifacts straight back to the same GitHub Release
(tag-keyed). This script is the durable receipt gate: it polls `gh release view`
until every external surface is present, then exits 0. It fails closed when the
bounded wait expires with any surface missing.

Usage:
  wait-for-external-release-assets.py --tag vX.Y.Z --kind menubar
  wait-for-external-release-assets.py --tag vX.Y.Z --kind rust-binaries
"""

from __future__ import annotations

import argparse
import fnmatch
import subprocess
import sys
import time

# macOS Rust binaries (Codemagic macos-rust-binaries workflow).
MAC_TARGETS = ("aarch64-apple-darwin", "x86_64-apple-darwin")
RUST_SURFACES = ("focusa", "focusa-daemon", "focusa-tui", "focusa-session-runner")

# Windows Rust binaries (AppVeyor full matrix).
WIN_TARGETS = ("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc")


def rust_binaries_exact(tag: str) -> list[str]:
    names: list[str] = []
    for target in MAC_TARGETS:
        for surface in RUST_SURFACES:
            names.append(f"{surface}-{tag}-{target}")
    for target in WIN_TARGETS:
        for surface in RUST_SURFACES:
            names.append(f"{surface}-{tag}-{target}.exe")
    return names


def menubar_exact(tag: str) -> list[str]:
    return [
        f"Focusa-{tag}-aarch64-apple-darwin.app.zip",
        f"Focusa-{tag}-x86_64-apple-darwin.app.zip",
        "Focusa_aarch64.app.tar.gz",
        "Focusa_aarch64.app.tar.gz.sig",
        "Focusa_x64.app.tar.gz",
        "Focusa_x64.app.tar.gz.sig",
    ]


def menubar_patterns() -> list[str]:
    return [
        "Focusa_*aarch64*.dmg",
        "Focusa_*x64*.dmg",
        "Focusa_*x64*setup.exe",
        "Focusa_*x64*setup.exe.sig",
        "Focusa_*arm64*setup.exe",
        "Focusa_*arm64*setup.exe.sig",
        "Focusa_*x64*.msi",
        "Focusa_*x64*.msi.sig",
        "Focusa_*arm64*.msi",
        "Focusa_*arm64*.msi.sig",
    ]


def list_asset_names(tag: str) -> set[str]:
    proc = subprocess.run(
        ["gh", "release", "view", tag, "--json", "assets", "--jq", ".assets[].name"],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        return set()
    return {line for line in proc.stdout.splitlines() if line.strip()}


def missing_for_kind(kind: str, tag: str, names: set[str]) -> list[str]:
    missing: list[str] = []
    if kind in ("rust-binaries", "all"):
        missing.extend(n for n in rust_binaries_exact(tag) if n not in names)
    if kind in ("menubar", "all"):
        missing.extend(n for n in menubar_exact(tag) if n not in names)
        missing.extend(
            f"pattern:{p}"
            for p in menubar_patterns()
            if not any(fnmatch.fnmatchcase(n, p) for n in names)
        )
    return missing


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag", required=True)
    parser.add_argument(
        "--kind",
        required=True,
        choices=["menubar", "rust-binaries", "all"],
    )
    parser.add_argument("--timeout-minutes", type=int, default=45)
    parser.add_argument("--poll-seconds", type=int, default=30)
    args = parser.parse_args()

    deadline = time.monotonic() + args.timeout_minutes * 60
    while True:
        names = list_asset_names(args.tag)
        missing = missing_for_kind(args.kind, args.tag, names)
        if not missing:
            print(f"external_receipts=PASS kind={args.kind} tag={args.tag}")
            return 0
        if time.monotonic() >= deadline:
            print(f"external_receipts=FAIL kind={args.kind} tag={args.tag}", file=sys.stderr)
            for item in missing:
                print(f"missing={item}", file=sys.stderr)
            return 1
        print(
            f"external_receipts_wait kind={args.kind} tag={args.tag} "
            f"present={len(names)} missing={len(missing)}",
            file=sys.stderr,
        )
        time.sleep(args.poll_seconds)


if __name__ == "__main__":
    raise SystemExit(main())
