#!/usr/bin/env python3
"""Validate 31 baseline unknown-method REST routes are source-backed by method.

This adds an exact-method lock for those routes so mutation/read decisions can no
longer be inferred from path shape or fall back to `ROUTE` placeholders.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ROUTE_CLASSIFICATION = ROOT / "docs/contracts/spec141/generated-capability-v2/route-classification.json"

TARGET_ROUTES = {
    "/connect",
    "/connect/firstrun",
    "/connect/room/{room_id}/scan",
    "/connect/{room_id}",
    "/pair/{device_id}",
    "/pair/{device_id}/manifest.json",
    "/pair/{device_id}/sw.js",
    "/v1/agent/prompt",
    "/v1/call-stack/list",
    "/v1/call-stack/show",
    "/v1/connect/approve",
    "/v1/connect/room/create",
    "/v1/connect/room/firstrun",
    "/v1/connect/room/start",
    "/v1/connect/room/{room_id}/approve",
    "/v1/connect/room/{room_id}/join",
    "/v1/connect/room/{room_id}/mac-offer",
    "/v1/connect/room/{room_id}/status",
    "/v1/connect/rooms",
    "/v1/connect/start",
    "/v1/connect/status",
    "/v1/context-cognition",
    "/v1/context-cognition/curate/eval",
    "/v1/context-cognition/curate/eval/runs",
    "/v1/context-cognition/curate/optimize",
    "/v1/context-cognition/optimizer/artifacts",
    "/v1/context-cognition/proof",
    "/v1/context-cognition/render",
    "/v1/device/pair/complete",
    "/v1/device/pair/list",
    "/v1/device/pair/revoke",
}

ROUTE_DECLARATION_FILES = [
    ROOT / "crates/focusa-api/src/routes/context_cognition.rs",
    ROOT / "crates/focusa-api/src/routes/call_stack.rs",
    ROOT / "crates/focusa-api/src/routes/device_pairing.rs",
    ROOT / "crates/focusa-api/src/routes/agent_reminder.rs",
]

# Keep one canonical route for all recovery-allowed paths while preserving read/mutation intent.
RECOVERY_ALLOWED = {"/v1/device/pair/revoke"}

METHOD_REGEX = re.compile(
    r'\.route\(\s*"([^"]+)"\s*,\s*(?:axum::routing::)?(get|post|patch|delete|put|head|options)\(',
    re.S,
)

READ_METHODS = {"GET", "HEAD", "OPTIONS"}
MUTATION_METHODS = {"POST", "PUT", "PATCH", "DELETE"}
SUPPORTED_METHODS = READ_METHODS | MUTATION_METHODS


def extract_declared_methods() -> dict[str, set[str]]:
    methods_by_path: dict[str, set[str]] = {}
    for source in ROUTE_DECLARATION_FILES:
        text = source.read_text(encoding="utf-8", errors="replace")
        for found_path, method in METHOD_REGEX.findall(text):
            if found_path in TARGET_ROUTES:
                methods_by_path.setdefault(found_path, set()).add(method.upper())
    return methods_by_path


classification_rows = {
    row["path"]: row
    for row in json.loads(ROUTE_CLASSIFICATION.read_text(encoding="utf-8"))["routes"]
}

declared_methods = extract_declared_methods()

assert len(TARGET_ROUTES) == 31
assert TARGET_ROUTES.issubset(declared_methods.keys()), "route parser missed declaration-bound targets"
assert TARGET_ROUTES.issubset(classification_rows.keys()), "route-classification output missed targets"

for path in sorted(TARGET_ROUTES):
    source_methods = declared_methods[path]
    row = classification_rows[path]
    route_methods = set(row["methods"])

    assert route_methods == source_methods, f"{path}: method list not source-backed"
    assert row.get("operation_refs") == [], f"{path}: unexpectedly linked to operation"
    assert row["classification"] in {"operator_only", "public_pairing"}, f"{path}: unexpected classification {row['classification']}"
    assert route_methods.issubset(SUPPORTED_METHODS), f"{path}: unsupported method in classification"
    assert all(route_methods), f"{path}: empty method set"

    side_effect = "read" if route_methods <= READ_METHODS else "mutation"
    policy_bucket = "recovery" if path in RECOVERY_ALLOWED else ("read" if side_effect == "read" else "base")
    assert policy_bucket in {"read", "recovery", "base", "premium"}

    negative_methods = SUPPORTED_METHODS - route_methods
    assert negative_methods, f"{path}: no negative-method assertions available"
    for method in negative_methods:
        assert method not in route_methods, f"{path}: negative method {method} unexpectedly present"

classification_summary = {
    path: {
        "methods": sorted(classification_rows[path]["methods"]),
        "policy_bucket": "recovery" if path in RECOVERY_ALLOWED else ("read" if set(classification_rows[path]["methods"]) <= READ_METHODS else "base"),
    }
    for path in sorted(TARGET_ROUTES)
}

print(json.dumps({
    "schema": "focusa.spec152f.unknown_route_method_validation.v1",
    "target_route_count": len(TARGET_ROUTES),
    "classified_read": sum(row["policy_bucket"] == "read" for row in classification_summary.values()),
    "classified_recovery": sum(row["policy_bucket"] == "recovery" for row in classification_summary.values()),
    "classified_base": sum(row["policy_bucket"] == "base" for row in classification_summary.values()),
    "result": "passed",
}, sort_keys=True))
