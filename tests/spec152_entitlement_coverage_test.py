#!/usr/bin/env python3
"""Verify deterministic, omission-intolerant Spec 152 entitlement inventory."""

import importlib.util
import json
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
GENERATOR = ROOT / "scripts/generate-spec152-entitlement-coverage.py"
OUTPUT = ROOT / "docs/contracts/spec152-entitlement-coverage.v1.json"

spec = importlib.util.spec_from_file_location("coverage_generator", GENERATOR)
module = importlib.util.module_from_spec(spec); assert spec.loader; spec.loader.exec_module(module)
expected = module.build()
actual = json.loads(OUTPUT.read_text())
assert actual == expected, "generated coverage is stale or nondeterministic"
assert actual["counts"]["total"] == len(actual["coverage"]) + len(actual["unmatched_surfaces"])
assert actual["counts"]["unmatched"] == 0, f"unmatched surfaces remain: {actual['counts']['unmatched']}"

operation_count = json.loads(module.OPERATIONS.read_text())["operation_count"]
route_count = json.loads(module.ROUTES.read_text())["route_count"]
capability_count = json.loads(module.CAPABILITIES.read_text())["capability_count"]
all_rows = actual["coverage"] + actual["unmatched_surfaces"]
assert sum(row["surface"] == "operation" for row in all_rows) == operation_count
assert sum(row["surface"] == "rest" for row in all_rows) == route_count
assert sum(row["surface"] == "pi_tool" for row in all_rows) == capability_count
assert sum(row["surface"] == "cli" for row in all_rows) == len(module.discover_cli())
assert sum(row["surface"] == "menubar" for row in all_rows) == len(module.discover_ui_actions(ROOT / "apps/menubar/src"))

required_fields = {
    "surface", "symbol_or_route", "mutation_class", "product", "feature",
    "limit_bucket", "gate_location", "pre_side_effect_test", "recovery_allowance", "source",
}
registry = yaml.safe_load((ROOT / "docs/contracts/spec152-feature-registry.v1.yaml").read_text())
feature_keys = {feature["key"] for feature in registry["features"]}
for row in all_rows:
    assert set(row) == required_fields, f"row fields drifted: {row}"
    assert row["product"] == "focusa"
    if row["feature"] is not None:
        assert row["feature"] in feature_keys, f"unregistered feature: {row['feature']}"
for row in actual["coverage"]:
    if row["mutation_class"] == "mutation" and not row["recovery_allowance"]:
        assert row["feature"] and row["gate_location"] and row["pre_side_effect_test"] == "required"
assert actual["counts"]["unmatched"] == 0, "every surface must resolve through base, premium, allowance, or explicit denial"

source = GENERATOR.read_text()
assert "FAMILY_FEATURE" in source and '.get("family")' in source
assert "in item[\"operation_id\"]" not in source, "substring classification detected"

print(json.dumps({
    "schema": "focusa.entitlement_coverage_validation.v1",
    "counts": actual["counts"],
    "registry_features": len(feature_keys),
    "result": "passed_with_explicit_unmatched_frontier",
}, sort_keys=True))
