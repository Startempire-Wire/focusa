#!/usr/bin/env python3
"""Static contract gate for canonical Spec 158 request-context extraction."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
CONTEXT = (ROOT / "crates/focusa-core/src/workstream_context.rs").read_text()
PRODUCTION_CONTEXT = CONTEXT.split("#[cfg(test)]", 1)[0]
STATE = (ROOT / "crates/focusa-core/src/workstream_state.rs").read_text()
SCOPE = (ROOT / "crates/focusa-api/src/scope.rs").read_text()
PRINCIPAL = (ROOT / "crates/focusa-api/src/middleware/principal.rs").read_text()
LIB = (ROOT / "crates/focusa-core/src/lib.rs").read_text()

assert "pub mod workstream_context;" in LIB
assert re.search(r"pub struct WorkstreamContext \{(?P<body>.*?)\n\}", CONTEXT, re.S)
for field in ("workstream", "continuity_id", "attachment", "workspace_binding_id", "actor", "authority"):
    assert f"pub {field}:" in CONTEXT
assert "pub actor: ActorRef" in CONTEXT
assert "pub authority: AuthorityContext" in CONTEXT
assert "pub struct ActorRef" in CONTEXT
assert "pub struct AuthorityContext" in CONTEXT
assert "pub struct WorkstreamRequestEnvelope" in CONTEXT
assert "pub fn extract(input: WorkstreamRequestEnvelope)" in CONTEXT
assert "validate_owner(&workstream)" in CONTEXT
assert "pub type WorkstreamContextInput = WorkstreamRequestEnvelope;" in CONTEXT
assert "WorkstreamContext<A" not in CONTEXT
assert "WorkstreamContextInput<A" not in CONTEXT

# Positive exact-owner paths and hostile cases must remain source-visible.
for symbol in (
    "exact_workstream_request_resolves_with_concrete_actor_and_authority",
    "exact_attachment_resolves_its_owner_without_inference",
    "authenticated_api_principal_adapts_to_typed_actor_ref",
):
    assert symbol in CONTEXT
for symbol in (
    "ambiguous_workstream_ownership_fails_closed",
    "continuity_only_request_cannot_resolve_context",
    "session_only_request_cannot_resolve_context",
    "missing_actor_and_authority_fail_closed",
    "non_canonical_authority_fails_closed",
    "conflicting_attachment_metadata_fails_closed",
    "request_envelope_has_no_presentation_or_runtime_owner_fallback",
):
    assert symbol in CONTEXT

# The only reducer event construction path extracts the canonical envelope first.
assert "pub fn from_request(" in STATE
assert "WorkstreamContext::extract(request)" in STATE
assert "validate_for_workstream" in (ROOT / "crates/focusa-core/src/reducer.rs").read_text()
assert "WorkstreamEvent::from_request" in STATE

# API scope and authentication owners adapt into the same core envelope; neither
# is allowed to manufacture Workstream authority from legacy request metadata.
assert "canonical_request_envelope" in SCOPE
assert "resolve_workstream_context" in SCOPE
assert "WorkstreamContext::extract" in SCOPE
assert "ActorRef::from_authenticated_principal" in PRINCIPAL

for forbidden in (
    "ui_selection",
    "focused_work_surface_id",
    "current_project",
    "process_cwd",
    "latest_record",
    "similarity",
    "nearest_candidate",
    "default_workstream",
):
    assert forbidden not in PRODUCTION_CONTEXT.lower(), f"forbidden fallback leaked into context: {forbidden}"

print("Spec 158 Workstream context source contract: PASS")
