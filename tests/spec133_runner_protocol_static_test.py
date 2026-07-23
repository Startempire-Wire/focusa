#!/usr/bin/env python3
"""Non-compiling Spec133 Phase 3.1 runner security/conformance lint."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = (ROOT / "Cargo.toml").read_text()
LOCK = (ROOT / "Cargo.lock").read_text()
SECURITY = (ROOT / "crates/focusa-core/src/silent_sessions/runner_security.rs").read_text()
PROTOCOL = (ROOT / "crates/focusa-core/src/silent_sessions/runner_protocol.rs").read_text()
RUNNER = (ROOT / "crates/focusa-session-runner/src/main.rs").read_text()
RUNNER_SECURITY = (ROOT / "crates/focusa-session-runner/src/security.rs").read_text()
RUNNER_SURFACE = RUNNER + RUNNER_SECURITY
CLIENT = (ROOT / "crates/focusa-core/src/silent_sessions/runner_client.rs").read_text()
RUNNER_CARGO = (ROOT / "crates/focusa-session-runner/Cargo.toml").read_text()

assert '"crates/focusa-session-runner"' in WORKSPACE
assert 'name = "focusa-session-runner"' in LOCK
assert 'name = "focusa-session-runner"' in RUNNER_CARGO

assert 'focusa.session_runner_protocol.v1' in PROTOCOL
assert "EmbeddedSameUser" in PROTOCOL
assert "PerUserSocket" in PROTOCOL
assert "cross-user execution requires a protected user-scoped runner socket" in PROTOCOL
assert "RunnerHeartbeat" in PROTOCOL
assert "ProcessTreeIdentity" in PROTOCOL
assert "manifest_digest" in PROTOCOL
assert "pub manifest: LaunchManifest" in PROTOCOL
assert "action_digest" in PROTOCOL

assert "PayloadMismatch" in SECURITY
assert "authenticate_payload" in SECURITY
assert "self.payload_hash" in SECURITY
assert "constant_time_eq" in SECURITY
assert "validate_binding" in PROTOCOL
assert "consume_runner_nonce" in CLIENT
assert "AmbiguousDelivery" in CLIENT
assert "verify_runner_socket" in CLIENT
assert "expected_owner_uid" in CLIENT
assert "MAX_RESPONSE_BYTES" in CLIENT
send_fn = CLIENT.index("pub async fn send_runner_request")
assert CLIENT.index("validate_binding", send_fn) < CLIENT.index("consume_runner_nonce", send_fn)

for marker in [
    "actual_user == args.owner_os_user",
    "workspace is not owned by the verified runner user",
    "process_group(0)",
    'Permissions::from_mode(0o600)',
    'Permissions::from_mode(0o700)',
    "runner key permissions must exclude group and other users",
    "runner key owner mismatch",
    "nonce ledger owner mismatch",
    "refusing to replace an active runner socket",
    "nonce ledger must share the protected runner socket directory",
    "sync_data",
    "process_group_exists",
    "exact process tree is not owned by this runner",
    "adoption manifest digest is required",
    "adoption process group is not alive",
    "MAX_REQUEST_BYTES",
]:
    assert marker in RUNNER_SURFACE, marker

# Durable nonce append happens before in-memory consumption becomes authoritative.
append_pos = RUNNER.index("append_nonce(&self.nonce_ledger")
assign_pos = RUNNER.index("self.consumed_nonces = consumed_nonces")
assert append_pos < assign_pos

# Runner starts from a clean environment and resolves only manifest-declared values/references.
assert ".env_clear()" in RUNNER_SECURITY
assert "safe_env" in RUNNER_SECURITY
assert "secret_env_refs" in RUNNER_SECURITY
assert "environment: BTreeMap" not in PROTOCOL

print("Spec133 protected per-user runner static contract: PASS")
