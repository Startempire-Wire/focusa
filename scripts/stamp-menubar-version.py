#!/usr/bin/env python3
"""Stamp all Focusa release-version surfaces from one release tag.

Usage:
  scripts/stamp-menubar-version.py v0.9.22-dev
  scripts/stamp-menubar-version.py 0.9.22-dev

This is intentionally the single version-stamping template used by
scripts/create-dev-release-tag.sh. It updates Rust workspace CLI/API/TUI/session-runner/core,
root lockfile package entries, the menubar package/Tauri metadata, and the
operator-visible Settings version.
"""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path

from distribution_manifest import build_manifest

ROOT = Path(__file__).resolve().parents[1]
VERSION_RE = re.compile(r"^v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$")
OLD_VERSION_RE = re.compile(r"\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?")

ROOT_RUST_PACKAGES = {
    "agent-stateful-cognitive-runtime",
    "cognitive-state-projection",
    "focusa-api",
    "focusa-bench",
    "focusa-cli",
    "focusa-core",
    "focusa-harness-adapters",
    "focusa-license",
    "focusa-session-runner",
    "focusa-terminal-ui",
    "focusa-tui",
    "letta-adapter",
    "pi-client-tool-gateway",
}
MENUBAR_RUST_PACKAGES = {"focusa-menubar"}


def parse_version(raw: str) -> str:
    value = raw.strip()
    match = VERSION_RE.match(value)
    if not match:
        raise SystemExit(f"Invalid Focusa release tag/version: {raw!r}")
    return match.group(1)


def replace_json_version(path: str, version: str) -> None:
    file_path = ROOT / path
    data = json.loads(file_path.read_text(encoding="utf-8"))
    data["version"] = version
    if path.endswith("package-lock.json"):
        packages = data.get("packages")
        if isinstance(packages, dict):
            root_pkg = packages.get("")
            if isinstance(root_pkg, dict):
                root_pkg["version"] = version
    file_path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def replace_key_value_version(path: str, version: str) -> None:
    file_path = ROOT / path
    text = file_path.read_text(encoding="utf-8")
    next_text, count = re.subn(
        r'(?m)^version\s*=\s*"[^"]+"',
        f'version = "{version}"',
        text,
        count=1,
    )
    if count != 1:
        raise SystemExit(f"Expected one top-level version in {path}")
    file_path.write_text(next_text, encoding="utf-8")


def replace_installer_version(path: str, version: str) -> None:
    file_path = ROOT / path
    text = file_path.read_text(encoding="utf-8")
    next_text, count = re.subn(
        r'(?m)^FOCUSA_INSTALLER_VERSION="[^"]+"$',
        f'FOCUSA_INSTALLER_VERSION="{version}"',
        text,
        count=1,
    )
    if count != 1:
        raise SystemExit(f"Expected one installer version in {path}")
    file_path.write_text(next_text, encoding="utf-8")


def replace_display_version(path: str, version: str) -> None:
    file_path = ROOT / path
    text = file_path.read_text(encoding="utf-8")
    next_text, count = re.subn(
        r"Focusa v" + OLD_VERSION_RE.pattern, f"Focusa v{version}", text
    )
    if count < 1:
        raise SystemExit(f"Expected Focusa display version in {path}")
    file_path.write_text(next_text, encoding="utf-8")


def replace_extension_build(path: str, package_name: str, version: str) -> None:
    file_path = ROOT / path
    text = file_path.read_text(encoding="utf-8")
    next_text, count = re.subn(
        rf'const EXTENSION_BUILD = "{re.escape(package_name)}@{OLD_VERSION_RE.pattern}"',
        f'const EXTENSION_BUILD = "{package_name}@{version}"',
        text,
        count=1,
    )
    if count != 1:
        raise SystemExit(f"Expected one EXTENSION_BUILD identity in {path}")
    file_path.write_text(next_text, encoding="utf-8")


def replace_agent_card_version(path: str, version: str) -> None:
    file_path = ROOT / path
    card = json.loads(file_path.read_text(encoding="utf-8"))
    if not isinstance(card, dict) or "card_digest" not in card:
        raise SystemExit(f"Expected generated Agent Card with card_digest in {path}")
    card["version"] = version
    digest_base = {key: value for key, value in card.items() if key != "card_digest"}
    stable = json.dumps(
        digest_base,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    )
    card["card_digest"] = "sha256:" + hashlib.sha256(stable.encode()).hexdigest()
    file_path.write_text(
        json.dumps(card, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )



def replace_readme_version(path: str, version: str) -> None:
    file_path = ROOT / path
    text = file_path.read_text(encoding="utf-8")
    next_text, count = re.subn(
        r"Current source version: `v" + OLD_VERSION_RE.pattern + r"`",
        f"Current source version: `v{version}`",
        text,
    )
    if count != 1:
        raise SystemExit(f"Expected one Current source version badge in {path}")
    file_path.write_text(next_text, encoding="utf-8")


def regenerate_distribution_manifest(version: str) -> None:
    """Write the canonical manifest from the shared deterministic contract."""
    import datetime
    import subprocess

    manifest_path = ROOT / "docs/contracts/spec141/generated-capability-v2/distribution-manifest.json"
    current = json.loads(manifest_path.read_text(encoding="utf-8"))
    head = subprocess.check_output(
        ["git", "rev-parse", "--short", "HEAD"], cwd=str(ROOT)
    ).decode().strip()
    generated_at = (
        datetime.datetime.now(datetime.timezone.utc)
        .isoformat()
        .replace("+00:00", "Z")
    )
    data = build_manifest(ROOT, current, version, head, generated_at)
    manifest_path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")

def replace_lock_package_versions(
    path: str, package_names: set[str], version: str
) -> None:
    """Update only named [[package]] lockfile entries.

    This avoids the old unsafe behavior that replaced the first `version = ...`
    line in a lockfile, which can corrupt unrelated dependency packages.
    """
    file_path = ROOT / path
    lines = file_path.read_text(encoding="utf-8").splitlines(keepends=True)
    current_name: str | None = None
    updated: set[str] = set()
    out: list[str] = []

    for line in lines:
        if line.strip() == "[[package]]":
            current_name = None
            out.append(line)
            continue
        name_match = re.match(r'^name\s*=\s*"([^"]+)"', line)
        if name_match:
            current_name = name_match.group(1)
            out.append(line)
            continue
        if current_name in package_names and re.match(r'^version\s*=\s*"[^"]+"', line):
            out.append(f'version = "{version}"\n')
            updated.add(current_name)
            continue
        out.append(line)

    missing = package_names - updated
    if missing:
        raise SystemExit(f"Missing package(s) in {path}: {', '.join(sorted(missing))}")
    file_path.write_text("".join(out), encoding="utf-8")


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("Usage: scripts/stamp-menubar-version.py <tag-or-version>")
    version = parse_version(sys.argv[1])

    # Rust workspace surfaces: CLI, daemon/API, core, TUI, session runner.
    replace_key_value_version("Cargo.toml", version)
    replace_lock_package_versions("Cargo.lock", ROOT_RUST_PACKAGES, version)

    # Pi extension package surfaces.
    replace_json_version("apps/pi-extension/package.json", version)
    replace_json_version("apps/pi-extension/package-lock.json", version)

    # Standalone installer surface shipped by the Pi extension release job.
    replace_installer_version("scripts/install-focusa.sh", version)

    # Menubar web/Tauri surfaces.
    replace_json_version("apps/menubar/package.json", version)
    replace_json_version("apps/menubar/package-lock.json", version)
    replace_json_version("apps/menubar/src-tauri/tauri.conf.json", version)
    replace_key_value_version("apps/menubar/src-tauri/Cargo.toml", version)
    replace_lock_package_versions(
        "apps/menubar/src-tauri/Cargo.lock", MENUBAR_RUST_PACKAGES, version
    )
    replace_display_version("apps/menubar/src/lib/components/Settings.svelte", version)

    # Extension build identity + generated Agent Card version (Spec 152 surfaces).
    replace_extension_build("apps/pi-extension/src/auto-compaction.ts", "focusa-pi-bridge", version)
    replace_agent_card_version("docs/contracts/spec141/generated-capability-v2/agent-card.json", version)

    # README source-version badge (validate-docs-runtime-parity requires v< Cargo version).
    replace_readme_version("README.md", version)
    # Distribution manifest — single-source: recompute sha256 + source_commit + generated_at.
    regenerate_distribution_manifest(version)
    # Release stamp artifact (used by release invariant inputs).
    (ROOT / "docs/current/.release-version-stamp").write_text(version + "\n", encoding="utf-8")

    print(f"Stamped Focusa version {version} (including distribution-manifest)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
