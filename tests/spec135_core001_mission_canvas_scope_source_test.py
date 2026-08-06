#!/usr/bin/env python3
"""Static source contract for CORE-001's canonical Mission Canvas scope."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODEL = (ROOT / "crates/focusa-core/src/mission_canvas/model.rs").read_text()
IDENTITY = (ROOT / "crates/focusa-core/src/workstream_identity.rs").read_text()
RESOLVER = (ROOT / "crates/focusa-core/src/mission_canvas/resolver.rs").read_text()
MEMORY = (ROOT / "crates/focusa-core/src/mission_canvas/memory.rs").read_text()
PERSISTENCE = (ROOT / "crates/focusa-core/src/mission_canvas/persistence.rs").read_text()
STATE = (ROOT / "crates/focusa-core/src/workstream_state.rs").read_text()
API = (ROOT / "crates/focusa-api/src/routes/mission_canvas.rs").read_text()
SCHEMA = (ROOT / "schemas/spec135/mission-canvas/composition-bundle.v1.schema.json").read_text()

# The canonical Rust identity owner validates the complete ScopeRef → WorkstreamId
# key.  Mission Canvas must not validate a path/continuity pair through the old
# legacy scope adapter.
assert "pub fn validate(&self) -> Result<(), ScopeKeyError>" in IDENTITY
assert "self.scope.validate()?" in IDENTITY
assert 'ScopeKeyError::Missing("workstream_id")' in IDENTITY
assert "self.workstream" in MODEL
assert ".validate()" in MODEL
assert ".legacy_scope()" not in MODEL

# The generated authority context remains the transport owner, while its
# constructor accepts only the canonical WorkstreamKey and exact AttachmentKey.
assert "pub struct MissionCanvasAuthorityContext" in MODEL
assert "pub type WorkstreamAuthorityContext = MissionCanvasAuthorityContext" in MODEL
assert "pub fn new(" in MODEL
assert "workstream: WorkstreamKey" in MODEL
assert "attachment: Option<AttachmentKey>" in MODEL
assert "pub fn from_parts(" in MODEL
assert "MissionCanvasScope::new(" in RESOLVER
assert "MissionCanvasScope::new(" in MEMORY
assert "MissionCanvasScope::from_parts(" in PERSISTENCE
assert "MissionCanvasScope::from_parts(" in API
assert "if self.workstream.validate().is_err()" in STATE
assert "self.workstream.legacy_scope()" not in STATE

# The former flat authority fields are not present in the canonical model or
# its bounded consumers.  Legacy records remain a separately named generated
# compatibility input and never become a scope constructor input.
for forbidden in (
    "pub project_root",
    "pub instance_id: Option<String>",
    "pub session_id: String",
    "pub attachment_id: String",
    "pub working_subpath_id",
):
    assert forbidden not in MODEL
assert '"LegacyExactScopeCompatibilityInput"' in SCHEMA
assert '"x-focusa-compatibility-only": true' in SCHEMA

# Authority-bearing API construction is centralized through the validated
# generated context adapter rather than a route-local identity resolver.
assert "parse_query_json::<WorkstreamKey>" in API
assert "MissionCanvasScope {" not in API
for forbidden in ("current_tab", "latest_record", "nearest_candidate", "process_cwd"):
    assert forbidden not in MODEL + API

print("CORE-001 MissionCanvasScope Workstream identity source contract: PASS")
