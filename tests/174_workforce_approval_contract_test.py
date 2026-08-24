#!/usr/bin/env python3
"""Static freeze gate for the Spec 174 approval issuance contract."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / "crates/focusa-api/src/routes/silent_sessions_contract.rs").read_text()

for required in [
    '"focusa.silent_session_approval_request.v1"',
    '"focusa.silent_session_approval_response.v1"',
    '"/v1/silent-sessions/{session_id}/approvals"',
    "pub enum ApprovalRequestAction",
    "Start,",
    "Input,",
    "Steer,",
    "FollowUp,",
    "Keys,",
    "Cancel,",
    "#[serde(deny_unknown_fields)]",
    "pub risk_acknowledged: bool",
    "pub idempotency_key: String",
    "pub run_id: SilentSessionRunId",
    "pub generation: RunGeneration",
]:
    assert required in source, f"approval contract missing {required}"

request_block = source.split("pub struct ApprovalCreateRequest", 1)[1].split("}", 1)[0]
for server_derived in [
    "action_digest", "permitted_side_effects", "config_hash", "model_binding",
    "workspace", "risk_class", "operator_actor", "expires_at",
]:
    assert server_derived not in request_block, f"client controls {server_derived}"

assert 'for unsupported in ["send_input", "interrupt", "adopt", "force_kill", "release"]' in source
print("PASS: Spec 174 durable approval HTTP contract is frozen")
