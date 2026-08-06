#!/usr/bin/env python3
"""Static contract gate for the additive Spec 158 identity migration seam.

Cargo execution remains separately milestone-gated; this test prevents the source
shape from regressing before the bounded Rust validation milestone.
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "crates/focusa-core/src/workstream_identity.rs").read_text()
LIB = (ROOT / "crates/focusa-core/src/lib.rs").read_text()

assert "pub mod workstream_identity;" in LIB
assert "pub enum ScopeRef" in SOURCE
assert "Project(ProjectRootKey)" in SOURCE
assert "Host(HostScopeKey)" in SOURCE
assert "pub struct HostScopeKey" in SOURCE
assert "pub fn project(scope: LegacyScopeRef)" in SOURCE
assert "pub fn host(scope: LegacyScopeRef)" in SOURCE
assert "ProjectRootKey::new(scope)" in SOURCE
assert "HostScopeKey::new(scope)" in SOURCE
assert "scope.validate()?" in SOURCE
assert "continuity_id" not in SOURCE
assert "current_session" not in SOURCE
assert "default_workstream" not in SOURCE
assert "identical_paths_with_different_host_worktree_fingerprints_are_distinct" in SOURCE

print("Spec 158 Workstream identity source contract: PASS")
