#!/usr/bin/env python3
"""Spec 104 machine-enforced mutable-global and unscoped-state inventory.

Default mode fails on unknown/stale findings or malformed classifications.
--closure additionally fails while any remediation entry remains open.
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "config/spec104-scoped-state-inventory.json"
SPEC = ROOT / "docs/104-typed-scoped-runtime-and-singleton-elimination-spec.md"
SOURCE_ROOTS = [ROOT / "crates", ROOT / "apps", ROOT / "packages"]
SOURCE_SUFFIXES = {".rs", ".ts", ".swift"}
STATIC_RE = re.compile(r"^\s*static\s+(?:mut\s+)?([A-Z][A-Z0-9_]+)\s*:\s*(.+)$")
MUTABLE_MARKERS = (
    "Mutex",
    "RwLock",
    "OnceLock",
    "LazyLock",
    "HashMap",
    "HashSet",
    "Vec<",
    "DashMap",
)
VALID_CLASSIFICATIONS = {
    "eliminate",
    "scope_key",
    "infra_allowlist",
    "consumer_migrate",
}
VALID_STATUSES = {"open", "eliminated", "scope_keyed", "infra_allowed", "migrated"}


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def source_files():
    for base in SOURCE_ROOTS:
        if not base.exists():
            continue
        for path in sorted(base.rglob("*")):
            if path.suffix not in SOURCE_SUFFIXES:
                continue
            if "target" in path.parts or "node_modules" in path.parts:
                continue
            yield path


def discover() -> dict[tuple[str, str], dict[str, object]]:
    findings: dict[tuple[str, str], dict[str, object]] = {}
    for path in source_files():
        text = path.read_text(errors="ignore")
        lines = text.splitlines()
        for line_no, line in enumerate(lines, 1):
            match = STATIC_RE.match(line)
            if match and any(marker in match.group(2) for marker in MUTABLE_MARKERS):
                key = (rel(path), match.group(1))
                findings[key] = {
                    "path": key[0],
                    "symbol": key[1],
                    "line": line_no,
                    "kind": "mutable_static",
                }
        if rel(path) == "apps/pi-extension/src/state.ts":
            for symbol in ("S", "runtimeState"):
                if re.search(rf"export\s+const\s+{symbol}\s*=", text):
                    findings[(rel(path), symbol)] = {
                        "path": rel(path),
                        "symbol": symbol,
                        "kind": "adapter_singleton",
                    }

    marker_rules = [
        (
            "crates/focusa-api/src/routes/predictions.rs",
            "spec92_predictions.json",
            "GLOBAL_FILE:spec92_predictions.json",
        ),
        (
            "crates/focusa-api/src/routes/ontology.rs",
            "spec92_predictions.json",
            "CONSUMER_FILE:spec92_predictions.json",
        ),
        (
            "crates/focusa-api/src/routes/metacognition.rs",
            "fn metacog_base_dir",
            "GLOBAL_DIR:runtime/metacognition",
        ),
        (
            "apps/pi-extension/src/state.ts",
            "focusa-project-root.json",
            "GLOBAL_FILE:focusa-project-root.json",
        ),
        (
            "apps/focusa-awareness/index.ts",
            "/data/wirebot/users/verious",
            "HARDCODED_SCOPE_DEFAULT",
        ),
    ]
    for path_text, needle, symbol in marker_rules:
        path = ROOT / path_text
        if path.exists() and needle in path.read_text(errors="ignore"):
            findings[(path_text, symbol)] = {
                "path": path_text,
                "symbol": symbol,
                "kind": "unscoped_state_marker",
            }
    return findings


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--closure", action="store_true", help="fail while any remediation remains open"
    )
    args = parser.parse_args()

    payload = json.loads(INVENTORY.read_text())
    entries = payload.get("entries", [])
    by_key: dict[tuple[str, str], dict[str, object]] = {}
    errors: list[str] = []
    for entry in entries:
        key = (str(entry.get("path", "")), str(entry.get("symbol", "")))
        if key in by_key:
            errors.append(f"duplicate inventory key: {key}")
        by_key[key] = entry
        if entry.get("classification") not in VALID_CLASSIFICATIONS:
            errors.append(
                f"invalid classification for {key}: {entry.get('classification')}"
            )
        if entry.get("status") not in VALID_STATUSES:
            errors.append(f"invalid status for {key}: {entry.get('status')}")
        for required in ("annex_id", "target", "acceptance_test"):
            if not str(entry.get(required, "")).strip():
                errors.append(f"missing {required} for {key}")
        if (
            entry.get("classification") == "infra_allowlist"
            and entry.get("authority_bearing") is not False
        ):
            errors.append(f"infra allowlist cannot be authority-bearing: {key}")

    findings = discover()
    unknown = sorted(set(findings) - set(by_key))
    if unknown:
        errors.extend(
            f"unclassified singleton/non-scoped state: {key}" for key in unknown
        )

    active_statuses = {"open", "scope_keyed", "infra_allowed", "migrated"}
    stale = sorted(
        key
        for key, entry in by_key.items()
        if entry.get("status") in active_statuses and key not in findings
    )
    if stale:
        errors.extend(
            f"inventory says active but source marker is absent; mark eliminated or update detector: {key}"
            for key in stale
        )

    spec_text = SPEC.read_text()
    for key, entry in by_key.items():
        if str(entry.get("annex_id")) not in spec_text:
            errors.append(
                f"inventory annex id absent from Spec 104: {key} -> {entry.get('annex_id')}"
            )

    open_entries = [entry for entry in entries if entry.get("status") == "open"]
    if args.closure and open_entries:
        errors.append(
            f"Spec 104 closure blocked: {len(open_entries)} inventory entries remain open"
        )

    print(
        f"Spec104 inventory: findings={len(findings)} classified={len(by_key)} open={len(open_entries)} closure={args.closure}"
    )
    for entry in open_entries:
        print(
            f"OPEN {entry['annex_id']} {entry['path']}::{entry['symbol']} -> {entry['classification']}"
        )
    if errors:
        for error in errors:
            print(f"FAIL {error}")
        return 1
    print(
        "PASS: every detected singleton/non-scoped state marker is classified; no unknown or stale inventory entries"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
