#!/usr/bin/env python3
"""Programmatic gate for expensive Focusa dev release builds.

Goal: CI can run often, but full tag/build/deploy should wait until there is
significant app/runtime/release-system delta, a scheduled release window, or an
explicit operator override with a reason.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from zoneinfo import ZoneInfo

SIGNIFICANT_SCORE = 8
RELEASE_WORKFLOW_SCORE = 8
WINDOW_SCORE = 4
STALE_HOURS = 24
WINDOW_TOLERANCE_MINUTES = 30
DEFAULT_WINDOWS_PT = ("11:00", "16:00")

CRITICAL_PATTERNS = (
    "SECURITY.md",
    "scripts/install-daemon.sh",
    "scripts/install-self-hosted-runner.sh",
    "scripts/install",
    "crates/focusa-cli/src/commands/install.rs",
    "crates/focusa-cli/src/commands/codesign.rs",
    "checksum",
    "SHA256",
    "notary",
    "codesign",
)
RELEASE_SYSTEM_PREFIXES = (
    ".github/workflows/",
    "scripts/create-dev-release-tag.sh",
    "scripts/release-gate.py",
    "scripts/deploy",
    "scripts/validate-github-workflows.py",
    "tests/spec_release_pipeline_",
    "tests/release_deploy_automation_static_test.sh",
    "tests/spec_canonical_live_release_pipeline_static_test.sh",
)
APP_PREFIXES = ("crates/", "apps/", "packages/")
UI_PREFIXES = ("apps/menubar/", "apps/focusa-awareness/")
DEPENDENCY_FILES = (
    "Cargo.toml",
    "Cargo.lock",
    "package.json",
    "package-lock.json",
    "bun.lockb",
    "pnpm-lock.yaml",
)
DOC_PREFIXES = ("docs/",)
TEST_PREFIXES = ("tests/",)


@dataclass
class ScoredPath:
    path: str
    category: str
    score: int


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], text=True).strip()


def latest_tag() -> str | None:
    try:
        tag = git("describe", "--tags", "--abbrev=0")
        return tag or None
    except subprocess.CalledProcessError:
        return None


def changed_paths(since_tag: str | None) -> list[str]:
    if since_tag:
        out = git("diff", "--name-only", f"{since_tag}..HEAD")
    else:
        out = git("ls-files")
    return [line.strip() for line in out.splitlines() if line.strip()]


def tag_time(tag: str | None) -> datetime | None:
    if not tag:
        return None
    try:
        raw = git("log", "-1", "--format=%cI", tag)
        return datetime.fromisoformat(raw.replace("Z", "+00:00"))
    except Exception:
        return None


def contains_any(path: str, needles: tuple[str, ...]) -> bool:
    lower = path.lower()
    return any(needle.lower() in lower for needle in needles)


def score_path(path: str) -> ScoredPath:
    name = Path(path).name
    if contains_any(path, CRITICAL_PATTERNS):
        return ScoredPath(path, "critical_security_install_signing_checksum", 10)
    if path == ".github/workflows/release.yml":
        return ScoredPath(path, "release_deploy_system", RELEASE_WORKFLOW_SCORE)
    if path.startswith(RELEASE_SYSTEM_PREFIXES):
        return ScoredPath(path, "release_deploy_system", 6)
    if path.startswith(UI_PREFIXES):
        return ScoredPath(path, "user_visible_ui", 4)
    if path.startswith(APP_PREFIXES):
        return ScoredPath(path, "app_runtime_code", 3)
    if name in DEPENDENCY_FILES or path.endswith(DEPENDENCY_FILES):
        return ScoredPath(path, "dependencies_lockfiles", 4)
    if path.startswith(TEST_PREFIXES):
        return ScoredPath(path, "tests_static_guards", 1)
    if path.startswith(DOC_PREFIXES) or path.endswith(".md"):
        return ScoredPath(path, "docs_only", 0)
    return ScoredPath(path, "misc_low_signal", 1)


def capped_score(scored: list[ScoredPath]) -> tuple[int, dict[str, int]]:
    caps = {
        "critical_security_install_signing_checksum": 99,
        "release_deploy_system": 12,
        "user_visible_ui": 12,
        "app_runtime_code": 12,
        "dependencies_lockfiles": 8,
        "tests_static_guards": 3,
        "docs_only": 0,
        "misc_low_signal": 3,
    }
    by_cat: dict[str, int] = {}
    for item in scored:
        by_cat[item.category] = by_cat.get(item.category, 0) + item.score
    capped = {cat: min(score, caps.get(cat, score)) for cat, score in by_cat.items()}
    return sum(capped.values()), capped


def release_window_status(now: datetime) -> tuple[bool, str]:
    tz = ZoneInfo("America/Los_Angeles")
    local = now.astimezone(tz)
    windows = tuple(
        w.strip()
        for w in os.environ.get(
            "FOCUSA_RELEASE_WINDOWS_PT", ",".join(DEFAULT_WINDOWS_PT)
        ).split(",")
        if w.strip()
    )
    best = None
    for window in windows:
        hour, minute = [int(part) for part in window.split(":", 1)]
        target = local.replace(hour=hour, minute=minute, second=0, microsecond=0)
        delta = abs((local - target).total_seconds()) / 60
        if best is None or delta < best[0]:
            best = (delta, window)
    if best and best[0] <= WINDOW_TOLERANCE_MINUTES:
        return (
            True,
            f"inside scheduled release window {best[1]} PT (+/- {WINDOW_TOLERANCE_MINUTES}m)",
        )
    return False, f"outside scheduled release windows {', '.join(windows)} PT"


def evaluate(
    paths: list[str], since_tag: str | None, now: datetime
) -> dict[str, object]:
    scored = [score_path(p) for p in paths]
    total, by_category = capped_score(scored)
    last_time = tag_time(since_tag)
    age_hours = None
    if last_time:
        age_hours = max(
            0.0, (now - last_time.astimezone(timezone.utc)).total_seconds() / 3600
        )
    in_window, window_reason = release_window_status(now)
    has_critical = any(
        item.category == "critical_security_install_signing_checksum" for item in scored
    )

    allowed_reason = None
    if has_critical and total >= 10:
        allowed_reason = "critical security/install/signing/checksum delta"
    elif total >= SIGNIFICANT_SCORE:
        allowed_reason = f"significant delta score {total} >= {SIGNIFICANT_SCORE}"
    elif in_window and total >= WINDOW_SCORE:
        allowed_reason = f"scheduled release window and score {total} >= {WINDOW_SCORE}"
    elif age_hours is not None and age_hours >= STALE_HOURS and total >= WINDOW_SCORE:
        allowed_reason = f"last release age {age_hours:.1f}h >= {STALE_HOURS}h and score {total} >= {WINDOW_SCORE}"

    allowed = allowed_reason is not None
    plain_error = None
    if not allowed:
        plain_error = (
            "Blocked: not enough significant app delta since last release. "
            "Batch more app/runtime/release-system changes, wait for a release window, "
            'or use --force-release --release-reason "...".'
        )

    return {
        "schema": "focusa.release_gate.v1",
        "allowed": allowed,
        "allowed_reason": allowed_reason,
        "plain_language_error": plain_error,
        "safe_alternative": "Keep CI-only changes on main; batch release until the score/window gate passes.",
        "score": total,
        "thresholds": {
            "significant_score": SIGNIFICANT_SCORE,
            "window_score": WINDOW_SCORE,
            "stale_hours": STALE_HOURS,
            "release_windows_pt": list(DEFAULT_WINDOWS_PT),
            "window_tolerance_minutes": WINDOW_TOLERANCE_MINUTES,
        },
        "since_tag": since_tag,
        "last_release_age_hours": age_hours,
        "release_window": {"inside": in_window, "reason": window_reason},
        "changed_path_count": len(paths),
        "score_by_category": by_category,
        "scored_paths": [item.__dict__ for item in scored],
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Gate expensive Focusa release builds by significant delta."
    )
    parser.add_argument("--json", action="store_true", help="emit JSON only")
    parser.add_argument("--since-tag", help="override base tag")
    args = parser.parse_args(argv)

    tag = args.since_tag or latest_tag()
    paths = changed_paths(tag)
    result = evaluate(paths, tag, datetime.now(timezone.utc))
    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(f"release_gate_allowed={str(result['allowed']).lower()}")
        print(
            f"score={result['score']} changed_paths={result['changed_path_count']} since_tag={result['since_tag']}"
        )
        print(f"release_window={result['release_window']['reason']}")
        if result["allowed"]:
            print(f"reason={result['allowed_reason']}")
        else:
            print(result["plain_language_error"])
            print(f"safe_alternative={result['safe_alternative']}")
    return 0 if result["allowed"] else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
