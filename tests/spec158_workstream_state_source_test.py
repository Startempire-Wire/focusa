#!/usr/bin/env python3
"""Static gate for the pre-Cargo Spec 158 ProjectState partition slice."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "crates/focusa-core/src/workstream_state.rs").read_text()
LIB = (ROOT / "crates/focusa-core/src/lib.rs").read_text()

assert "pub mod workstream_state;" in LIB
assert "pub struct ProjectState<W>" in SOURCE
project_state = re.search(r"pub struct ProjectState<W> \{(?P<body>.*?)\n\}", SOURCE, re.S).group("body")
assert "BTreeMap<WorkstreamId, W>" in project_state
for forbidden in ("ContinuityId", "continuity_id", "SessionId", "session_id", "AttachmentId", "attachment_id"):
    assert forbidden not in project_state
assert "pub fn register_workstream" in SOURCE
assert "pub fn workstream(" in SOURCE
assert "pub fn workstream_mut(" in SOURCE
assert "AlreadyRegistered" in SOURCE
assert "one_project_routes_two_workstreams_to_distinct_state" in SOURCE
assert "registration_cannot_silently_replace_existing_state" in SOURCE

print("Spec 158 Workstream state partition source contract: PASS")
