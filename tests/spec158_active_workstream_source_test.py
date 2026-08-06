#!/usr/bin/env python3
"""Non-Cargo source contract for Spec 158 active Workstream selection."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
STATE = (ROOT / "crates/focusa-core/src/workstream_state.rs").read_text()
CONTEXT = (ROOT / "crates/focusa-core/src/workstream_context.rs").read_text()
REDUCER = (ROOT / "crates/focusa-core/src/reducer.rs").read_text()

# The active-selection production slice is bounded before the pre-existing
# Workstream cognitive owner and before Rust unit fixtures.
start = STATE.index("pub const ACTIVE_WORKSTREAM_COMMAND_ID")
end = STATE.index("/// Canonical cognitive state for exactly one Workstream.")
ACTIVE = STATE[start:end]

for symbol in (
    "pub struct ActiveWorkstreamState",
    "pub struct ActiveWorkstreamCommand",
    "pub struct ActiveWorkstreamEvent",
    "pub struct ActiveWorkstreamReceipt",
    "pub struct ActiveWorkstreamReductionResult",
    "pub fn validate(&self, state: &CanonicalProjectState)",
    "pub fn reduce_active_workstream(",
    "pub fn reduce(",
    "pub struct Receipt",
    "pub fn emit(",
):
    assert symbol in ACTIVE, f"missing canonical active Workstream owner: {symbol}"

state_body = re.search(
    r"pub struct ActiveWorkstreamState \{(?P<body>.*?)\n\}", ACTIVE, re.S
).group("body")
assert "active_workstream: Option<WorkstreamKey>" in state_body
assert "revision: u64" in state_body
assert "fencing_token: u64" in state_body
assert "pub revision: u64" not in state_body
assert "pub fencing_token: u64" not in state_body
assert "pub active_workstream:" not in state_body
assert "idempotency_records" in state_body
assert "pub fn active_workstream_id(&self) -> Option<&WorkstreamId>" in ACTIVE

command_body = re.search(
    r"pub struct ActiveWorkstreamCommand \{(?P<body>.*?)\n\}", ACTIVE, re.S
).group("body")
for field in (
    "pub workstream: WorkstreamKey",
    "pub context: WorkstreamContext",
    "pub idempotency_key: String",
    "pub expected_revision: u64",
    "pub expected_fencing_token: u64",
):
    assert field in command_body, field

# Positive exact-owner/reducer/receipt path.
for marker in (
    "validate_for_workstream(&self.workstream)",
    "workstreams\n            .get(&self.workstream.workstream_id)",
    "target.key != self.workstream",
    "CrossProjectWorkstream",
    "record.command_fingerprint != self.operation_fingerprint()",
    "self.expected_revision != cursor.revision",
    "self.expected_fencing_token != cursor.fencing_token",
    "let event = ActiveWorkstreamEvent::from_command",
    "let receipt = Receipt::emit(&event)?",
    "active_workstream_state_mut()",
    "canonical: true",
    "receipt.replayed = true",
):
    assert marker in ACTIVE, f"missing active Workstream reducer guard/output: {marker}"

assert "WorkstreamState::reduce(state, command)" in REDUCER
assert "pub fn reduce_active_workstream(" in REDUCER
assert "ActiveWorkstreamReductionResult" in REDUCER

# Request transport carries the fencing cursor, but the request path never
# mutates canonical state by itself.
assert "pub expected_fencing_token: Option<u64>" in CONTEXT
assert "with_expected_fencing_token" in CONTEXT
assert "pub fn from_request(" in ACTIVE
assert "WorkstreamContext::extract(request)" in ACTIVE

# Hostile owner-resolution audit: no active-selection production path may read
# presentation, process, recency, or subordinate identity as a Workstream owner.
for forbidden in (
    "current_dir",
    "process::current_dir",
    "std::env",
    "current_tab",
    "latest_record",
    "default_workstream",
    "last_active_workstream",
    "recent_session",
    "similarity",
    "nearest_candidate",
):
    assert forbidden not in ACTIVE.lower(), f"forbidden selection fallback leaked: {forbidden}"

# The target is explicit and registered; no target is manufactured from a
# continuity/session/CWD-shaped value.
assert "pub workstream: WorkstreamKey" in command_body
assert "continuity_id" not in command_body
assert "session_id" not in command_body
assert "cwd" not in command_body.lower()

# Required positive and hostile Rust fixtures remain visible to the permitted
# non-Cargo gate.
for test_name in (
    "active_workstream_state_emits_canonical_event_and_receipt",
    "desktop_request_is_only_a_request_until_reducer_accepts_it",
    "unknown_workstream_selection_fails_closed_without_mutation",
    "foreign_workstream_key_selection_fails_closed",
    "cross_project_partition_selection_fails_closed",
    "stale_revision_and_fencing_cursors_fail_closed",
    "idempotent_replay_returns_receipt_without_second_mutation",
    "continuity_only_request_cannot_select_a_workstream",
):
    assert test_name in STATE, f"missing hostile/positive fixture: {test_name}"

print("Spec 158 active Workstream reducer selection source contract: PASS")
