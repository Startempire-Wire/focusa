#!/usr/bin/env python3
"""Validate the deterministic 390-row Spec 152F reconciliation frontier."""

import hashlib
import importlib.util
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GENERATOR = ROOT / "scripts/generate-spec152-entitlement-coverage.py"
COVERAGE_PATH = ROOT / "docs/contracts/spec152-entitlement-coverage.v1.json"
INDEX_PATH = ROOT / "docs/contracts/spec152f-surface-reconciliation.v1.json"

spec = importlib.util.spec_from_file_location("spec152_coverage_generator", GENERATOR)
module = importlib.util.module_from_spec(spec)
assert spec.loader
spec.loader.exec_module(module)

runtime_coverage = json.loads(COVERAGE_PATH.read_text(encoding="utf-8"))
baseline_coverage = module.build(include_test_files=True)
index = json.loads(INDEX_PATH.read_text(encoding="utf-8"))
expected_index, expected_shards = module.build_reconciliation()
assert index == expected_index, "Spec 152F reconciliation index is stale or nondeterministic"
assert len(INDEX_PATH.read_text().splitlines()) < 500

assert index["schema"] == "focusa.spec152f.surface_reconciliation.v1"
assert index["authority"] == "docs/152f-simple-entitlement-gating-and-future-granularity-addendum.md"
assert index["policy"] == "docs/contracts/spec152f-entitlement-policy.v1.yaml"
assert index["baseline_coverage"] == "docs/contracts/spec152-entitlement-coverage.v1.json"
assert index["baseline_counts"] == {"covered": 600, "unmatched": 390, "total": 990}
assert index["surface_counts"] == {
    "rest": 189,
    "cli": 87,
    "menubar": 85,
    "release": 18,
    "update": 5,
    "export": 4,
    "scheduler": 2,
}
assert index["resolution_counts"] == {
    "base_entitlement_candidate": 149,
    "inherit_canonical_operation": 178,
    "premium_family_candidate": 51,
    "recovery_or_read_allowance": 3,
    "scanner_exclusion_test_only": 9,
}
assert index["unknown_method_routes"] == 0
assert index["test_only_scanner_exclusions"] == 9
assert index["runtime_file_entries"] == 29
assert index["runtime_file_entries_after_test_exclusion"] == 20
assert index["coverage_canonical_sha256"] == hashlib.sha256(
    json.dumps(baseline_coverage, sort_keys=True, separators=(",", ":")).encode()
).hexdigest()
assert index["policy_file_sha256"] == hashlib.sha256(
    (ROOT / index["policy"]).read_bytes()
).hexdigest()
assert index["source_digests"] == baseline_coverage["source_digests"]
assert runtime_coverage["counts"]["unmatched"] == 0
assert runtime_coverage["scanner_exclusions"]["count"] == 9

assert set(index["shards"]) == {"rest", "cli", "menubar", "runtime_files"}
rows = []
for group, ref in sorted(index["shards"].items()):
    path = ROOT / ref["path"]
    raw = path.read_text(encoding="utf-8")
    assert raw == expected_shards[ref["path"]], f"stale shard: {ref['path']}"
    assert hashlib.sha256(raw.encode()).hexdigest() == ref["sha256"]
    assert len(raw.splitlines()) < 500
    shard = json.loads(raw)
    assert shard["schema"] == "focusa.spec152f.surface_reconciliation_shard.v1"
    assert shard["surface_group"] == group
    assert shard["row_count"] == ref["row_count"] == len(shard["rows"])
    rows.extend(shard["rows"])

assert len(rows) == 390
assert len({row["baseline_id"] for row in rows}) == 390
required_fields = {
    "baseline_id",
    "surface",
    "symbol_or_route",
    "mutation_class",
    "source",
    "resolution",
    "candidate_family",
    "owner_task",
    "rationale",
}
allowed_resolutions = set(index["resolution_counts"])
for row in rows:
    assert set(row) == required_fields
    assert row["resolution"] in allowed_resolutions
    assert row["owner_task"].startswith("focusa-vbcqu.20.14.")
    assert row["rationale"]
    serialized = json.dumps(row).lower()
    for forbidden in ('"price"', '"sku"', '"tier"', '"caller_grant"'):
        assert forbidden not in serialized, f"surface became commercial catalog entry: {row['baseline_id']}"

# Every unmatched source row is represented exactly once without mutating the frozen source inventory.
def source_key(row):
    return (
        row["surface"],
        row["symbol_or_route"],
        row["mutation_class"],
        json.dumps(row["source"], sort_keys=True),
    )

assert Counter(source_key(row) for row in baseline_coverage["unmatched_surfaces"]) == Counter(
    source_key(row) for row in rows
)
assert Counter(row["surface"] for row in rows) == Counter(index["surface_counts"])
assert Counter(row["resolution"] for row in rows) == Counter(index["resolution_counts"])

unknown_rest = [row for row in rows if row["surface"] == "rest" and row["mutation_class"] == "unknown"]
assert len(unknown_rest) == 0

known_rest = [row for row in rows if row["surface"] == "rest" and row["mutation_class"] == "mutation"]
assert len(known_rest) == 189
assert not any(row["resolution"] == "metadata_repair_required" for row in known_rest)
known_rest_owners = {row["owner_task"] for row in known_rest}
assert known_rest_owners == {"focusa-vbcqu.20.14.24"} or known_rest_owners == {"focusa-vbcqu.20.14.23", "focusa-vbcqu.20.14.24"}

for surface, owner in {
    "cli": "focusa-vbcqu.20.14.25",
    "menubar": "focusa-vbcqu.20.14.26",
}.items():
    selected = [row for row in rows if row["surface"] == surface]
    assert selected
    assert {row["resolution"] for row in selected} == {"inherit_canonical_operation"}
    assert {row["owner_task"] for row in selected} == {owner}

excluded = [row for row in rows if row["resolution"] == "scanner_exclusion_test_only"]
assert len(excluded) == 9
assert {row["owner_task"] for row in excluded} == {"focusa-vbcqu.20.14.27"}
assert all("/tests/" in "/" + row["symbol_or_route"] or row["symbol_or_route"].endswith("_test.rs") for row in excluded)

runtime = [row for row in rows if row["surface"] not in {"rest", "cli", "menubar"}]
assert len(runtime) == 29
assert sum(row["resolution"] != "scanner_exclusion_test_only" for row in runtime) == 20
assert {row["owner_task"] for row in runtime if row["resolution"] != "scanner_exclusion_test_only"} == {
    "focusa-vbcqu.20.14.28"
}

recovery = [row for row in rows if row["resolution"] == "recovery_or_read_allowance"]
assert {row["symbol_or_route"] for row in recovery} == {
    "/v1/update/rollback",
    "crates/focusa-api/src/routes/silent_sessions_retention_export.rs",
    "crates/focusa-cli/src/commands/export.rs",
}
assert not any(row["resolution"] in {"free", "unrestricted", "separate_sku"} for row in rows)

rules = set(index["rules"])
assert "inventory rows are not prices SKUs or independent paywalls" in rules
assert "unknown methods require source-backed metadata repair" in rules
assert "presenters inherit canonical operation policy" in rules
assert "test-only files are excluded without hiding production entrypoints" in rules
assert "recovery read export repair rollback and stable security paths remain available subject to security" in rules

print(json.dumps({
    "schema": "focusa.spec152f.surface_reconciliation_validation.v1",
    "rows": len(rows),
    "rest": index["surface_counts"]["rest"],
    "cli": index["surface_counts"]["cli"],
    "menubar": index["surface_counts"]["menubar"],
    "runtime_files": index["runtime_file_entries"],
    "unknown_methods": len(unknown_rest),
    "test_exclusions": len(excluded),
    "runtime_after_exclusion": index["runtime_file_entries_after_test_exclusion"],
    "result": "passed",
}, sort_keys=True))
