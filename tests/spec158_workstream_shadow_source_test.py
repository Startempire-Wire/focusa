#!/usr/bin/env python3
"""Non-Cargo source gate for Spec 158 shadow Workstream persistence."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "crates/focusa-core/src/workstream_shadow.rs").read_text()
PRODUCTION = SOURCE.split("#[cfg(test)]", 1)[0]
LIB = (ROOT / "crates/focusa-core/src/lib.rs").read_text()
QUARANTINE = (ROOT / "crates/focusa-core/src/workstream_quarantine.rs").read_text()

assert "pub mod workstream_shadow;" in LIB
for symbol in (
    "pub struct LegacyState",
    "pub fn read(",
    "pub struct LegacyStateRecord",
    "pub struct ShadowWorkstreamRow",
    "pub struct ShadowWorkstreamStore",
    "pub fn write(",
    "pub struct ParityComparator",
    "pub fn compare(",
    "pub struct ParityReport",
):
    assert symbol in SOURCE, symbol

# Typed rows retain exact Workstream ownership and provenance; the state is not
# replaced by an untyped JSON bag.
assert "WorkstreamState::from_focusa_state" in PRODUCTION
assert "mapping_key(mapping)" in PRODUCTION
assert "source_ref" in PRODUCTION and "source_hash" in PRODUCTION
assert "pub fn key(&self) -> &WorkstreamKey" in SOURCE
assert "pub fn mapping(&self) -> &WorkstreamMigrationMapping" in SOURCE
assert "pub fn state(&self) -> &WorkstreamState" in SOURCE
assert "pub fn rows(&self) -> &[ShadowWorkstreamRow]" in SOURCE
assert "pub fn quarantine(&self) -> &LegacyQuarantine" in SOURCE
assert "pub fn omissions(&self) -> &[ShadowOmission]" in SOURCE
assert "pub fn is_advisory(&self) -> bool" in SOURCE

# All six Spec 158 parity outcomes must remain distinguishable.
for disposition in (
    "EqualMappedState",
    "ExpectedRemovalOfUnsafeFallback",
    "MigrationMismatch",
    "QuarantinedAmbiguity",
    "DeprecatedData",
    "SerializationOnlyDifference",
):
    assert disposition in SOURCE, disposition
assert "canonical_json" in PRODUCTION
assert "bounded: true" in SOURCE
assert "truncated" in PRODUCTION

# Positive source proof: equal mapped state is materialized and accepted.
for symbol in (
    "equal_mapped_state_passes_bounded_parity",
    "assert!(report.passes())",
    "ParityDisposition::EqualMappedState",
    "serialization_only_difference_is_not_a_migration_mismatch",
    "two_unique_mappings_materialize_distinct_exact_workstream_keys",
    "mod workstream_shadow_materialization",
):
    assert symbol in SOURCE, symbol

# Hostile source proof: ambiguous mappings produce no row and remain quarantined.
for symbol in (
    "ambiguous_mapping_is_quarantined_without_default_assignment",
    "assert_eq!(write.materialized_rows, 0)",
    "assert_eq!(shadow.rows().len(), 0)",
    "QuarantineReason::MultipleCandidateWorkstreams",
    "ParityDisposition::QuarantinedAmbiguity",
    "unmapped_record_is_quarantined_without_owner",
    "foreign_declared_scope_is_quarantined_without_repair",
):
    assert symbol in SOURCE, symbol

# Shared canonical state is read through &FocusaState and the shadow module has
# no reducer or mutable row escape hatch.
assert "state: &FocusaState" in PRODUCTION
assert "&mut FocusaState" not in PRODUCTION
assert "reduce_workstream" not in PRODUCTION
assert "rows_mut" not in PRODUCTION
assert "state_mut" not in PRODUCTION
assert "canonical write path" in PRODUCTION

# Fail closed: no fallback owner, recency, similarity, or session-only inference
# may appear in the production implementation.
for forbidden in (
    "default_workstream",
    "last_active",
    "nearest",
    "similarity",
    "current_project",
    "session_only_mapping",
):
    assert forbidden not in PRODUCTION.lower(), forbidden

# Unmapped and conflicting inputs have explicit quarantine classes rather than
# being silently dropped or assigned.
for reason in (
    "UnmappedLegacyRecord",
    "ConflictingWorkstreamMappings",
    "InvalidMigrationMapping",
):
    assert reason in QUARANTINE, reason
assert "self.quarantine.append" in PRODUCTION
assert "self.rows.push" in PRODUCTION

print("Spec 158 shadow Workstream persistence source contract: PASS")
