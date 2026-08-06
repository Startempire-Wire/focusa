#!/usr/bin/env python3
"""Static contract gate for the additive Spec 158 identity migration seam.

Cargo execution remains separately milestone-gated; this test prevents the source
shape from regressing before the bounded Rust validation milestone.
"""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "crates/focusa-core/src/workstream_identity.rs").read_text()
LIB = (ROOT / "crates/focusa-core/src/lib.rs").read_text()

assert "pub mod workstream_identity;" in LIB
assert "pub enum ScopeRef" in SOURCE
assert "Project(ProjectRootKey)" in SOURCE
assert "Host(HostScopeKey)" in SOURCE
assert "pub struct HostScopeKey" in SOURCE
assert "pub struct WorkstreamId(String)" in SOURCE
assert "pub struct WorkstreamKey" in SOURCE
assert "pub scope: ScopeRef" in SOURCE
assert "pub workstream_id: WorkstreamId" in SOURCE
assert "pub struct AttachmentKey" in SOURCE
for identity_type in ("ContinuityId", "InstanceId", "SessionId", "AttachmentId", "WorkspaceBindingId"):
    assert f"subordinate_id!({identity_type}" in SOURCE
assert "pub fn project(scope: LegacyScopeRef)" in SOURCE
assert "pub fn host(scope: LegacyScopeRef)" in SOURCE
assert "ProjectRootKey::new(scope)" in SOURCE
assert "HostScopeKey::new(scope)" in SOURCE
assert "scope.validate()?" in SOURCE
scope_enum = re.search(r"pub enum ScopeRef \{(?P<body>.*?)\n\}", SOURCE, re.S).group("body")
workstream_key = re.search(r"pub struct WorkstreamKey \{(?P<body>.*?)\n\}", SOURCE, re.S).group("body")
attachment_key = re.search(r"pub struct AttachmentKey \{(?P<body>.*?)\n\}", SOURCE, re.S).group("body")
assert "continuity_id" not in scope_enum
assert "continuity_id" not in workstream_key
assert "session_id" not in workstream_key
for field in ("workstream", "continuity_id", "instance_id", "session_id", "attachment_id", "workspace_binding_id"):
    assert field in attachment_key
assert "current_session" not in scope_enum
assert "default_workstream" not in scope_enum
assert "identical_paths_with_different_host_worktree_fingerprints_are_distinct" in SOURCE
assert "two_workstreams_under_one_project_remain_distinct_keys" in SOURCE
assert "continuity_is_not_part_of_serialized_workstream_identity" in SOURCE
assert "attachment_accepts_only_its_owning_workstream" in SOURCE
assert "attachment_serializes_the_complete_owner_chain" in SOURCE

print("Spec 158 Workstream identity source contract: PASS")
