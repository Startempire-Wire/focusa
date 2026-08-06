#!/usr/bin/env python3
"""Static gate for explicit Spec 158 Workstream migration mappings."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "crates/focusa-core/src/workstream_migration.rs").read_text()
LIB = (ROOT / "crates/focusa-core/src/lib.rs").read_text()

assert "pub mod workstream_migration;" in LIB
assert '"focusa.workstream_migration_mapping.v1"' in SOURCE
assert "pub struct MigrationInventory" in SOURCE
assert "pub struct WorkstreamMigrationMapping" in SOURCE
for field in (
    "source_refs", "scope_ref", "workstream_id", "confidence", "evidence_refs",
    "rationale", "approved_by", "approval_ref", "created_at"
):
    assert f"pub {field}:" in SOURCE
assert "MigrationRule" in SOURCE and "Operator" in SOURCE
assert "MigrationConfidence::Proven" in SOURCE
assert "AmbiguousCandidates" in SOURCE
assert "MissingCandidate" in SOURCE
assert "multiple_candidate_workstreams_fail_closed" in SOURCE
assert "continuity_source_without_evidence_or_approval_is_rejected" in SOURCE
for forbidden in ("similarity", "nearest", "default_workstream", "last_active"):
    assert forbidden not in SOURCE.lower()

print("Spec 158 Workstream migration mapping source contract: PASS")
