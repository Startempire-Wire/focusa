#!/usr/bin/env python3
"""Prove test-only file matches are excluded without hiding runtime modules."""

import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GENERATOR = ROOT / "scripts/generate-spec152-entitlement-coverage.py"
OUTPUT = ROOT / "docs/contracts/spec152-entitlement-coverage.v1.json"
BASELINE_SHARD = ROOT / "docs/contracts/spec152f-surface-reconciliation/runtime_files.v1.json"

spec = importlib.util.spec_from_file_location("spec152_coverage_generator", GENERATOR)
module = importlib.util.module_from_spec(spec)
assert spec.loader
spec.loader.exec_module(module)

runtime = module.build()
baseline = module.build(include_test_files=True)
actual = json.loads(OUTPUT.read_text(encoding="utf-8"))
frozen = json.loads(BASELINE_SHARD.read_text(encoding="utf-8"))
assert actual == runtime, "generated runtime coverage is stale or nondeterministic"

expected_exclusions = {
    "crates/focusa-cli/tests/silent_proof_export_parity_e2e.rs": "tests_directory",
    "crates/focusa-cli/tests/spec128_update_runtime_e2e.rs": "tests_directory",
    "crates/focusa-core/src/release_adapters_test.rs": "recognized_test_module",
    "crates/focusa-core/src/release_calibration_test.rs": "recognized_test_module",
    "crates/focusa-core/src/release_cycle_test.rs": "recognized_test_module",
    "crates/focusa-core/src/release_ledger_test.rs": "recognized_test_module",
    "crates/focusa-core/src/release_orchestrator_test.rs": "recognized_test_module",
}
receipt = runtime["scanner_exclusions"]
assert receipt["schema"] == "focusa.spec152f.surface_scanner_exclusions.v1"
assert receipt["count"] == len(receipt["entries"]) == 7
assert {row["path"]: row["rule"] for row in receipt["entries"]} == expected_exclusions
assert receipt["entries"] == sorted(receipt["entries"], key=lambda row: (row["surface"], row["path"]))

for path, rule in expected_exclusions.items():
    candidate = ROOT / path
    assert candidate.is_file(), path
    assert module._is_test_path(path)
    if rule == "tests_directory":
        assert "tests" in Path(path).parts
    else:
        assert path.endswith("_test.rs")

runtime_rows = [
    row
    for row in runtime["unmatched_surfaces"]
    if row["surface"] in {"worker", "scheduler", "export", "update", "release"}
]
baseline_rows = [
    row
    for row in baseline["unmatched_surfaces"]
    if row["surface"] in {"worker", "scheduler", "export", "update", "release"}
]
assert len(baseline_rows) == 27
assert len(runtime_rows) == 20
assert {row["symbol_or_route"] for row in baseline_rows} - {
    row["symbol_or_route"] for row in runtime_rows
} == set(expected_exclusions)
assert not ({row["symbol_or_route"] for row in runtime_rows} & set(expected_exclusions))
assert baseline["counts"] == {"covered": 569, "unmatched": 395, "total": 964}
assert runtime["counts"] == {"covered": 569, "unmatched": 388, "total": 957}

frozen_exclusions = {
    row["symbol_or_route"]
    for row in frozen["rows"]
    if row["resolution"] == "scanner_exclusion_test_only"
}
assert frozen_exclusions == set(expected_exclusions)
frozen_runtime = {
    row["symbol_or_route"]
    for row in frozen["rows"]
    if row["resolution"] != "scanner_exclusion_test_only"
}
assert frozen_runtime == {row["symbol_or_route"] for row in runtime_rows}

# Embedded test helpers do not make a production module a test-only path.
production_with_test_helpers = {
    row["symbol_or_route"]
    for row in runtime_rows
    if "#[cfg(test)]" in (ROOT / row["symbol_or_route"]).read_text(encoding="utf-8")
}
assert production_with_test_helpers == {
    "crates/focusa-api/src/routes/update.rs",
    "crates/focusa-cli/src/commands/export.rs",
    "crates/focusa-cli/src/commands/update.rs",
    "crates/focusa-cli/src/commands/update_trust.rs",
    "crates/focusa-core/src/release_adapters.rs",
    "crates/focusa-core/src/release_calibration.rs",
    "crates/focusa-core/src/release_cycle.rs",
    "crates/focusa-core/src/release_intelligence.rs",
    "crates/focusa-core/src/release_ledger.rs",
    "crates/focusa-core/src/release_orchestrator.rs",
    "crates/focusa-core/src/silent_session_scheduler.rs",
    "crates/focusa-core/src/temporal_release_gate.rs",
    "crates/focusa-core/src/update.rs",
    "crates/focusa-core/src/work_item/scheduler.rs",
}
assert all(not module._is_test_path(path) for path in production_with_test_helpers)

print(json.dumps({
    "schema": "focusa.spec152f.surface_scanner_validation.v1",
    "baseline_file_matches": len(baseline_rows),
    "excluded_test_only": len(expected_exclusions),
    "runtime_entrypoints": len(runtime_rows),
    "retained_production_modules_with_test_helpers": len(production_with_test_helpers),
    "result": "passed",
}, sort_keys=True))
