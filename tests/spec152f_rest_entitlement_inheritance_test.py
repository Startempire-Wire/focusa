#!/usr/bin/env python3
"""Validate all REST inventory entries resolve through inheritance to canonical operations.

Each REST route in the Spec 152F reconciliation manifest must resolve its entitlement
policy by inheriting from a canonical operation family. No route may own pricing, tier,
or caller-controlled grants. Routes without explicit operation_refs fall back to
segment-based family resolution; premium features are asserted only at approved
family boundaries (automation, team_remote, release_proof, premium_updates).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# --- Load data sources ---
routes = json.loads(
    (ROOT / "docs/contracts/spec141/generated-capability-v2/route-classification.json").read_text()
)["routes"]
rest_rows = json.loads(
    (ROOT / "docs/contracts/spec152f-surface-reconciliation/rest.v1.json").read_text()
)["rows"]

# --- Import the route entitlement table generator for canonical resolution ---
sys.path.insert(0, str(ROOT / "scripts"))
import importlib.util

spec = importlib.util.spec_from_file_location(
    "route_entitlements", ROOT / "scripts/generate-spec152-route-entitlement-table.py"
)
generator = importlib.util.module_from_spec(spec)
assert spec.loader
spec.loader.exec_module(generator)

requirements = generator.requirements()
requirement_map = {path: (feature, bucket) for path, feature, bucket in requirements}

# --- Authority: premium families from the policy ---
PREMIUM_FEATURES = {
    "focusa.agent.parallelism",
    "focusa.agent.silent_sessions",
    "focusa.remote.stream",
    "focusa.team.multi_operator",
    "focusa.release.proof",
    "focusa.update.unattended",
    "focusa.install.channel.nightly",
    "focusa.install.channel.preview",
}

BASE_FEATURES = {
    "focusa.core.workpoint",
    "focusa.core.evidence",
    "focusa.core.mission",
}

# --- Validation ---
errors = []
resolved = 0
unresolved = []

for row in rest_rows:
    path = row["symbol_or_route"]
    resolution = row["resolution"]

    # Recovery-only routes are handled by middleware recovery logic
    if resolution == "recovery_or_read_allowance":
        assert path == "/v1/update/rollback", f"Unexpected recovery route: {path}"
        assert path not in requirement_map, (
            f"{path} must not appear in the entitlement table"
        )
        resolved += 1
        continue

    # All base_entitlement_candidate and premium_family_candidate routes must resolve
    if path in requirement_map:
        feature, bucket = requirement_map[path]

        if feature in PREMIUM_FEATURES and resolution != "premium_family_candidate":
            errors.append(
                f"{path}: carries premium feature {feature} but resolution is {resolution}"
            )

        if feature in BASE_FEATURES and resolution == "premium_family_candidate":
            errors.append(
                f"{path}: resolution is premium but carries base feature {feature}"
            )

        resolved += 1
    else:
        unresolved.append(path)

# --- Verify completeness ---
rest_row_count = len(rest_rows)
assert rest_row_count >= 189, f"Expected >=189 REST entries, got {rest_row_count}"

# All reconciliation REST entries must resolve
all_resolved = resolved + len(
    [r for r in rest_rows if r["resolution"] == "metadata_repair_required"]
) == rest_row_count

result = {
    "schema": "focusa.spec152f.rest_entitlement_inheritance.v1",
    "total_rest_entries": rest_row_count,
    "resolved_in_table": resolved,
    "unresolved": len(unresolved),
    "unresolved_paths": unresolved[:20],
    "errors": errors[:20],
    "result": "passed" if resolved == rest_row_count and not errors else "failed",
}

print(json.dumps(result, sort_keys=True, indent=2))

if result["result"] != "passed":
    sys.exit(1)
