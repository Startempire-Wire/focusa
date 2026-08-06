#!/usr/bin/env python3
"""Static gate for append-only Spec 158 legacy ownership quarantine."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "crates/focusa-core/src/workstream_quarantine.rs").read_text()
LIB = (ROOT / "crates/focusa-core/src/lib.rs").read_text()

assert "pub mod workstream_quarantine;" in LIB
assert "pub enum QuarantineReason" in SOURCE
for reason in (
    "MissingScope", "MissingWorkstreamIdentity", "MultipleCandidateWorkstreams",
    "ConflictingProjectRoots", "ConflictingThreadLineage", "ContinuityCollision",
    "SessionOnlyIdentity", "ForeignHostOrWorktree", "InvalidCausalHistory",
    "CorruptSnapshot", "UnsupportedProjectionVersion"
):
    assert reason in SOURCE
assert "pub struct LegacyQuarantineRow" in SOURCE
for field in ("source_ref", "source_hash", "payload_ref", "reason", "candidate_workstreams", "evidence_refs", "quarantined_at"):
    assert f"pub {field}:" in SOURCE
assert "pub fn classify" in SOURCE
assert "pub fn append" in SOURCE
assert "pub fn rows(&self) -> &[LegacyQuarantineRow]" in SOURCE
assert "DuplicateSource" in SOURCE
assert "ambiguous_record_is_retained_without_canonical_assignment" in SOURCE
assert "session_only_identity_is_quarantined_not_promoted" in SOURCE

print("Spec 158 Workstream quarantine source contract: PASS")
