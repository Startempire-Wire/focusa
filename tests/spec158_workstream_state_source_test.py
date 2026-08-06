#!/usr/bin/env python3
"""Static gate for the Spec 158 ProjectState Workstream partition slice."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "crates/focusa-core/src/workstream_state.rs").read_text()
REDUCER = (ROOT / "crates/focusa-core/src/reducer.rs").read_text()
LIB = (ROOT / "crates/focusa-core/src/lib.rs").read_text()

assert "pub mod workstream_state;" in LIB
assert "pub struct WorkstreamState" in SOURCE
workstream_state = re.search(
    r"pub struct WorkstreamState \{(?P<body>.*?)\n\}", SOURCE, re.S
).group("body")
assert "WorkstreamKey" in workstream_state
assert "FocusaState" in workstream_state
assert "EventHead" in workstream_state
assert "ProjectionVersion" in workstream_state
assert "serde_json::Value" not in workstream_state
assert "pub fn new(key: WorkstreamKey)" in SOURCE
assert "pub fn focus_stack(" in SOURCE
assert "pub fn focus_state(" in SOURCE
assert "pub fn workpoints(" in SOURCE
assert "pub fn trajectory(" in SOURCE
assert "pub fn work_loop(" in SOURCE
assert "pub fn context_sources(" in SOURCE
assert "pub fn context_claims(" in SOURCE
assert "pub struct WorkstreamEvent" in SOURCE
assert "pub fn workstream_id(&self)" in SOURCE
assert "pub type CanonicalProjectState = ProjectState<WorkstreamState>" in SOURCE

project_state = re.search(
    r"pub struct ProjectState<W> \{(?P<body>.*?)\n\}", SOURCE, re.S
).group("body")
assert "BTreeMap<WorkstreamId, W>" in project_state
for forbidden in (
    "ContinuityId",
    "continuity_id",
    "SessionId",
    "session_id",
    "AttachmentId",
    "attachment_id",
):
    assert forbidden not in project_state
assert "pub fn register_workstream" in SOURCE
assert "pub fn workstream(" in SOURCE
assert "pub fn workstream_mut(" in SOURCE
assert "AlreadyRegistered" in SOURCE
assert "one_project_routes_two_workstreams_to_distinct_state" in SOURCE
assert "registration_cannot_silently_replace_existing_state" in SOURCE

assert "pub struct WorkstreamReductionResult" in REDUCER
assert "pub fn reduce_workstream(" in REDUCER
assert "ProjectState<WorkstreamState>" in REDUCER
assert "let workstream_id = event.workstream_id().clone()" in REDUCER
assert "workstream_mut(&workstream_id)" in REDUCER
assert "selected.key != event.workstream" in REDUCER
assert "stale Workstream revision" in REDUCER
assert "reducer_workstream_partition_routes_one_entry_and_preserves_foreign_entry" in REDUCER

print("Spec 158 Workstream state and reducer partition source contract: PASS")
