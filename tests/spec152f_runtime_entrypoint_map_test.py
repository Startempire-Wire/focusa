#!/usr/bin/env python3
"""Validate Spec152F runtime entrypoint map — 20 production file-derived entrypoints.

Each entry must resolve by callable operation metadata, not filename.
Test-only files are excluded and must not appear in the map.
"""

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

MAP_PATH = ROOT / "docs/contracts/spec152f-runtime-entrypoint-map.v1.json"
RECON_PATH = ROOT / "docs/contracts/spec152f-surface-reconciliation/runtime_files.v1.json"

ALLOWED_RESOLUTIONS = {
    "customer_data_export",
    "release_proof",
    "inherit_initiating_operation",
}

ALLOWED_SURFACES = {"export", "release", "scheduler", "update"}

ALLOWED_FAMILIES = {
    "customer_data_export",
    "release_proof",
    "internal_maintenance",
}

ALLOWED_CLASSES = {"read", "value_mutation", "internal_maintenance"}

REQUIRED_KEYS = {
    "source_path",
    "surface",
    "resolution",
    "capability_family",
    "operation_class",
    "rationale",
}

# Exact test-only scanner exclusions (atom focusa-vbcqu.20.14.27 plus the two
# Spec 152F premium-family test files discovered by the scanner after the 152e
# presenter unification merged into this branch).
SCANNER_EXCLUSIONS = {
    "crates/focusa-cli/tests/silent_proof_export_parity_e2e.rs",
    "crates/focusa-core/src/release_adapters_test.rs",
    "crates/focusa-core/src/release_calibration_test.rs",
    "crates/focusa-core/src/release_cycle_test.rs",
    "crates/focusa-core/src/release_ledger_test.rs",
    "crates/focusa-core/src/release_orchestrator_test.rs",
    "crates/focusa-cli/tests/spec128_update_runtime_e2e.rs",
    "crates/focusa-license/tests/spec152f_export_entitlement.rs",
    "crates/focusa-license/tests/spec152f_release_proof_entitlement.rs",
}

# Forbidden: files that were excluded by the scanner but whose surface groups
# must not re-enter production classification.
FORBIDDEN_PATTERNS = {"_test.rs", "/tests/"}


def _load_json(path: Path):
    raw = path.read_text(encoding="utf-8")
    return raw, json.loads(raw)


def _check_no_filename_policy(entry):
    """No file is classified merely because its filename contains release/update/export/scheduler."""
    path = entry["source_path"]
    rationale = entry.get("rationale", "")
    assert isinstance(rationale, str) and len(rationale.strip()) > 0, (
        f"entry {path} must have a non-empty rationale — "
        "classification must be based on callable operations, not filename"
    )
    surface = entry["surface"]
    assert surface in ALLOWED_SURFACES


def test_map_exists_and_schema():
    raw, payload = _load_json(MAP_PATH)
    assert payload["schema"] == "focusa.spec152f.runtime_entrypoint_map.v1"
    assert payload["row_count"] == 20
    assert len(payload["rows"]) == 20
    assert len(raw) > 0
    digest = hashlib.sha256(raw.encode()).hexdigest()
    assert len(digest) == 64


def test_exactly_20_entries():
    _, payload = _load_json(MAP_PATH)
    rows = payload["rows"]
    assert len(rows) == 20, f"expected 20, got {len(rows)}"
    assert payload["row_count"] == 20


def test_all_keys_present():
    _, payload = _load_json(MAP_PATH)
    for row in payload["rows"]:
        missing = REQUIRED_KEYS - set(row.keys())
        assert not missing, f"entry {row.get('source_path', '?')} missing keys: {missing}"


def test_all_sources_are_unique():
    _, payload = _load_json(MAP_PATH)
    sources = [row["source_path"] for row in payload["rows"]]
    assert len(sources) == len(set(sources)), "duplicate source paths in map"


def test_no_filename_based_policy():
    _, payload = _load_json(MAP_PATH)
    for row in payload["rows"]:
        _check_no_filename_policy(row)


def test_resolutions_are_valid():
    _, payload = _load_json(MAP_PATH)
    for row in payload["rows"]:
        assert row["resolution"] in ALLOWED_RESOLUTIONS, (
            f"invalid resolution {row['resolution']} for {row['source_path']}"
        )


def test_surfaces_are_valid():
    _, payload = _load_json(MAP_PATH)
    for row in payload["rows"]:
        assert row["surface"] in ALLOWED_SURFACES, (
            f"invalid surface {row['surface']} for {row['source_path']}"
        )


def test_capability_families_are_valid():
    _, payload = _load_json(MAP_PATH)
    for row in payload["rows"]:
        assert row["capability_family"] in ALLOWED_FAMILIES, (
            f"invalid capability_family {row['capability_family']} for {row['source_path']}"
        )


def test_operation_classes_are_valid():
    _, payload = _load_json(MAP_PATH)
    for row in payload["rows"]:
        assert row["operation_class"] in ALLOWED_CLASSES, (
            f"invalid operation_class {row['operation_class']} for {row['source_path']}"
        )


def test_export_entries_always_available():
    _, payload = _load_json(MAP_PATH)
    exports = [r for r in payload["rows"] if r["surface"] == "export"]
    assert len(exports) == 2, f"expected 2 export entries, got {len(exports)}"
    for entry in exports:
        assert entry["resolution"] == "customer_data_export", (
            f"export {entry['source_path']} must be customer_data_export"
        )
        assert entry["capability_family"] == "customer_data_export"
        assert entry["operation_class"] in {"read", "recovery"}, (
            "basic export is read or recovery"
        )


def test_release_entries_are_release_proof():
    _, payload = _load_json(MAP_PATH)
    releases = [r for r in payload["rows"] if r["surface"] == "release"]
    assert len(releases) == 12, f"expected 12 release entries, got {len(releases)}"
    for entry in releases:
        assert entry["resolution"] == "release_proof", (
            f"release {entry['source_path']} must be release_proof"
        )
        assert entry["capability_family"] == "release_proof"
        assert entry["operation_class"] == "value_mutation"


def test_schedulers_inherit_initiating_operation():
    _, payload = _load_json(MAP_PATH)
    schedulers = [r for r in payload["rows"] if r["surface"] == "scheduler"]
    assert len(schedulers) == 2, f"expected 2 scheduler entries, got {len(schedulers)}"
    for entry in schedulers:
        assert entry["resolution"] == "inherit_initiating_operation", (
            f"scheduler {entry['source_path']} must inherit initiating operation"
        )
        assert entry["capability_family"] == "internal_maintenance"
        assert entry["operation_class"] == "internal_maintenance"


def test_updates_inherit_initiating_operation():
    _, payload = _load_json(MAP_PATH)
    updates = [r for r in payload["rows"] if r["surface"] == "update"]
    assert len(updates) == 4, f"expected 4 update entries, got {len(updates)}"
    for entry in updates:
        assert entry["resolution"] == "inherit_initiating_operation", (
            f"update {entry['source_path']} must inherit initiating operation"
        )
        assert entry["capability_family"] == "internal_maintenance"


def test_test_files_are_not_in_map():
    _, payload = _load_json(MAP_PATH)
    sources = {row["source_path"] for row in payload["rows"]}
    for excluded in SCANNER_EXCLUSIONS:
        assert excluded not in sources, (
            f"test-only file {excluded} must not appear in runtime entrypoint map"
        )
    for source in sources:
        assert not source.endswith("_test.rs"), (
            f"test file {source} must not appear in runtime entrypoint map"
        )
        assert "/tests/" not in source, (
            f"test directory file {source} must not appear in runtime entrypoint map"
        )


def test_map_matches_reconciliation_shard_after_exclusions():
    """Every non-test-excluded reconciliation entry must be in the map, and vice versa."""
    _, map_payload = _load_json(MAP_PATH)
    _, recon_payload = _load_json(RECON_PATH)

    map_sources = {row["source_path"] for row in map_payload["rows"]}
    recon_sources = {
        row["symbol_or_route"]
        for row in recon_payload["rows"]
        if row["symbol_or_route"] not in SCANNER_EXCLUSIONS
    }

    assert map_sources == recon_sources, (
        f"Map and reconciliation differ.\n"
        f"  In map but not recon: {map_sources - recon_sources}\n"
        f"  In recon but not map: {recon_sources - map_sources}"
    )
    assert len(map_sources) == 20
    assert len(recon_sources) == 20


def test_resolution_counts_match():
    _, payload = _load_json(MAP_PATH)
    counts = payload.get("resolution_counts", {})
    rows = payload["rows"]

    expected = {
        "customer_data_export": sum(1 for r in rows if r["resolution"] == "customer_data_export"),
        "release_proof": sum(1 for r in rows if r["resolution"] == "release_proof"),
        "inherit_initiating_operation": sum(1 for r in rows if r["resolution"] == "inherit_initiating_operation"),
    }
    assert counts == expected, f"resolution_counts mismatch: {counts} vs {expected}"


def test_surface_counts_match():
    _, payload = _load_json(MAP_PATH)
    counts = payload.get("surface_counts", {})
    rows = payload["rows"]

    expected = {
        "export": sum(1 for r in rows if r["surface"] == "export"),
        "release": sum(1 for r in rows if r["surface"] == "release"),
        "scheduler": sum(1 for r in rows if r["surface"] == "scheduler"),
        "update": sum(1 for r in rows if r["surface"] == "update"),
    }
    assert counts == expected, f"surface_counts mismatch: {counts} vs {expected}"


def test_disjoint_surfaces():
    """Each source must appear in exactly one surface group."""
    _, payload = _load_json(MAP_PATH)
    by_surface = {
        "export": {r["source_path"] for r in payload["rows"] if r["surface"] == "export"},
        "release": {r["source_path"] for r in payload["rows"] if r["surface"] == "release"},
        "scheduler": {r["source_path"] for r in payload["rows"] if r["surface"] == "scheduler"},
        "update": {r["source_path"] for r in payload["rows"] if r["surface"] == "update"},
    }
    all_sources = set()
    for surface_name, sources in by_surface.items():
        overlap = all_sources & sources
        assert not overlap, f"surface {surface_name} overlaps with another: {overlap}"
        all_sources |= sources
    assert len(all_sources) == 20


def main() -> None:
    test_map_exists_and_schema()
    test_exactly_20_entries()
    test_all_keys_present()
    test_all_sources_are_unique()
    test_no_filename_based_policy()
    test_resolutions_are_valid()
    test_surfaces_are_valid()
    test_capability_families_are_valid()
    test_operation_classes_are_valid()
    test_export_entries_always_available()
    test_release_entries_are_release_proof()
    test_schedulers_inherit_initiating_operation()
    test_updates_inherit_initiating_operation()
    test_test_files_are_not_in_map()
    test_map_matches_reconciliation_shard_after_exclusions()
    test_resolution_counts_match()
    test_surface_counts_match()
    test_disjoint_surfaces()

    result = {
        "schema": "focusa.spec152f.runtime_entrypoint_map_validation.v1",
        "result": "passed",
        "rows": 20,
        "export_count": 2,
        "release_count": 12,
        "scheduler_count": 2,
        "update_count": 4,
        "customer_data_export_count": 2,
        "release_proof_count": 12,
        "inheriting_count": 6,
    }
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
