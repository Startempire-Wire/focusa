#!/usr/bin/env python3
"""Fail-closed drift audit for the temporary Spec 178 CI provider route."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONFIG_PATH = ROOT / "config/spec178-provider-authority.json"
STALE_PATHS = (
    ROOT / "config/billing-bypass.json",
    ROOT / ".github/workflows/billing-bypass-expiry.yml",
)


def fail(message: str) -> None:
    print(f"SPEC178_PROVIDER_DRIFT: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    if not path.is_file():
        fail(f"missing required surface: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def main() -> None:
    try:
        config = json.loads(read(CONFIG_PATH))
    except json.JSONDecodeError as exc:
        fail(f"invalid provider authority JSON: {exc}")

    if config.get("schema") != "focusa.spec178_provider_authority.v1":
        fail("unexpected provider authority schema")
    if config.get("temporary_mode") != "active":
        fail("temporary provider mode must remain explicit until operator restoration")

    active = config.get("active_providers")
    expected = {
        "linux": "github_actions_self_hosted",
        "windows": "appveyor",
        "macos": "codemagic",
    }
    if not isinstance(active, dict):
        fail("active_providers must be an object")
    actual = {
        surface: details.get("provider") if isinstance(details, dict) else None
        for surface, details in active.items()
    }
    if actual != expected:
        fail(f"active provider map {actual!r} does not match {expected!r}")

    inactive = set(config.get("inactive_providers") or [])
    if not {"cirrus", "azure_pipelines"}.issubset(inactive):
        fail("Cirrus and Azure Pipelines must remain explicitly inactive")

    restoration = config.get("hosted_restoration") or {}
    if restoration.get("mode") != "operator_triggered_all_at_once":
        fail("hosted restoration must be operator-triggered and all-at-once")
    if restoration.get("automatic_mutation") is not False:
        fail("automatic hosted-provider mutation is forbidden")
    if restoration.get("spending_change_requires_operator_approval") is not True:
        fail("provider spending changes require explicit operator approval")
    required_steps = set(restoration.get("required_steps") or [])
    expected_steps = {
        "restore_all_github_hosted_consumers_in_one_reviewed_change",
        "run_manual_provider_parity_proof",
        "remove_temporary_provider_routes_only_after_green_proof",
        "record_durable_release_and_provider_receipts",
    }
    if required_steps != expected_steps:
        fail("hosted restoration steps are incomplete or divergent")

    for stale in STALE_PATHS:
        if stale.exists():
            fail(f"retired auto-bypass surface returned: {stale.relative_to(ROOT)}")

    spec = read(ROOT / config["canonical_spec"])
    for marker in (
        "GitHub Actions self-hosted `host-focusa-deploy`",
        "AppVeyor public-project lane",
        "Codemagic cloud `mac_mini_m2`",
        "Cirrus is",
        "operator-triggered",
        "all-at-once",
    ):
        if marker not in spec:
            fail(f"canonical Spec 178 marker missing: {marker}")

    ci = read(ROOT / ".github/workflows/ci.yml")
    release = read(ROOT / ".github/workflows/release.yml")
    appveyor = read(ROOT / ".appveyor.yml")
    codemagic = read(ROOT / "codemagic.yaml")
    if "ovh-build-2" not in ci:
        fail("CI lacks the configured self-hosted OVH runner")
    if "runs-on: self-hosted" not in release:
        fail("release workflow lacks the configured self-hosted lane")
    if "APPVEYOR_REPO_TAG" not in appveyor:
        fail("AppVeyor tag admission is missing")
    if "menubar-macos-package-proof" not in codemagic:
        fail("Codemagic macOS package-proof workflow is missing")

    reminder_path = ROOT / config["reminder"]["workflow"]
    reminder = read(reminder_path)
    if "runs-on: [self-hosted, Linux, X64, ovh-build-2]" not in reminder:
        fail("provider reminder must run on the available self-hosted lane")
    for forbidden in ("git push", "git commit", "ubuntu-latest", "macos-latest", "windows-latest"):
        if forbidden in reminder:
            fail(f"provider reminder contains forbidden mutation/hosted route: {forbidden}")

    billing_locked = os.environ.get("FOCUSA_GITHUB_HOSTED_BILLING_LOCKED") == "1"
    result = {
        "schema": "focusa.spec178_provider_parity_audit.v1",
        "status": "passed",
        "temporary_mode": "active",
        "active_providers": expected,
        "hosted_billing_locked": billing_locked,
        "restoration_reminder": (
            "GitHub-hosted capacity remains billing-locked; restoration requires an explicit "
            "operator-triggered all-at-once reviewed change and parity proof."
            if billing_locked
            else "Temporary provider routing remains active; do not restore partially or automatically."
        ),
        "mutation_performed": False,
    }
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
