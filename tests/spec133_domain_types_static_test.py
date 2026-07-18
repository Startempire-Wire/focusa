#!/usr/bin/env python3
from pathlib import Path

root = Path(__file__).resolve().parents[1]
source = (root / "crates/focusa-core/src/silent_session.rs").read_text()
required_types = [
    "SilentSession", "SilentSessionRun", "SilentSessionConfig", "SilentSessionConfigRevision",
    "SilentSessionEvent", "SilentSessionCheckpoint", "SilentSessionLease",
    "SilentSessionCompletionEvaluation", "SilentSessionVersions",
]
for name in required_types:
    assert f"pub struct {name}" in source or f"pub enum {name}" in source, name
for field in [
    "silent_session_schema_version", "config_schema_version", "event_schema_version",
    "daemon_runner_protocol_version", "harness_adapter_protocol_version",
    "process_backend_protocol_version", "stream_chunk_format_version", "receipt_mapping_version",
]:
    assert f"pub {field}: u32" in source, field
for identity in [
    "SilentSessionId", "SilentSessionRunId", "SilentSessionEventId",
    "SilentSessionConfigRevisionId", "SilentSessionCheckpointId", "SilentSessionLeaseId",
    "SilentSessionCompletionEvaluationId",
]:
    assert f"uuid_v7_id!({identity})" in source, identity
assert "self.project_root == config.identity.project_root" in source
assert "self.continuity_id == config.identity.continuity_id" in source
assert "Runtime {" in source
assert "CanonicalWorkpoint" in source
assert "tmux" not in source.lower().replace("tmux names", ""), "tmux must not become a canonical domain field"
print("Spec133 domain types, UUIDv7 identities, independent versions, and immutable scope contract: PASS")
