#!/usr/bin/env python3
"""Static gate for fail-closed Spec 158 WorkstreamContext extraction."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "crates/focusa-core/src/workstream_context.rs").read_text()
LIB = (ROOT / "crates/focusa-core/src/lib.rs").read_text()

assert "pub mod workstream_context;" in LIB
context = re.search(r"pub struct WorkstreamContext<A, U> \{(?P<body>.*?)\n\}", SOURCE, re.S).group("body")
for field in ("workstream", "continuity_id", "attachment", "workspace_binding_id", "actor", "authority"):
    assert field in context
assert "pub fn extract" in SOURCE
assert "MissingWorkstream" in SOURCE
assert "WorkstreamMismatch" in SOURCE
assert "ContinuityMismatch" in SOURCE
assert "WorkspaceBindingMismatch" in SOURCE
for forbidden in ("current_project", "last_active", "current_ui", "nearest", "process_cwd"):
    assert forbidden not in SOURCE.lower()
assert "ambiguous_workstream_ownership_fails_closed" in SOURCE
assert "continuity_or_session_without_workstream_cannot_resolve_context" in SOURCE

print("Spec 158 Workstream context source contract: PASS")
