#!/usr/bin/env python3
from pathlib import Path

R = Path(__file__).resolve().parents[1]
F = (R / "tests/spec133_fault_fixture.py").read_text()
T = (
    (R / "crates/focusa-core/src/silent_sessions/types.rs").read_text()
    + (R / "crates/focusa-core/src/silent_sessions/runner_protocol.rs").read_text()
    + (R / "crates/focusa-core/src/silent_sessions/capability_catalog.rs").read_text()
)
X = (R / "crates/focusa-core/src/silent_sessions/retention.rs").read_text()
P8 = (R / "tests/spec133_phase8_backend_gate.sh").read_text()
P9 = (R / "tests/spec133_phase9_final_gate.sh").read_text()

for marker in [
    "harness",
    "subprocess",
    "child-leak",
    "prompt-wait",
    "output-flood",
    "model-mismatch",
    "retry-failure",
    "isolated-git",
    "entitlement",
    "runner-disconnect",
]:
    assert marker in F, marker
for marker in [
    "DAEMON_RUNNER_PROTOCOL_VERSION",
    "HARNESS_ADAPTER_PROTOCOL_VERSION",
    "PROCESS_BACKEND_PROTOCOL_VERSION",
    "ProtocolVersions",
    "capabilities",
]:
    assert marker in T, marker
for marker in [
    "SilentSessionPurgePlan",
    "set_evidence_hold",
    "purge_session",
    "export_session_bundle",
    "ordinary_delete_session",
]:
    assert marker.lower() in X.lower(), marker
for marker in [
    "focusa-harness-adapters",
    "silent_sessions::platform_backends",
    "Windows_NT",
    "unsupported Phase 8 platform",
]:
    assert marker in P8, marker
for marker in [
    "spec133_phase4_runtime_gate.sh",
    "spec133_phase5_isolation_gate.sh",
    "spec133_phase6_evidence_gate.sh",
    "spec133_phase7_operator_gate.sh",
    "spec133_phase8_backend_gate.sh",
    "work_loop_checkpoint_recovery_test.sh",
    "work_loop_process_tree_supervision_test.sh",
    "work_loop_writer_lease_fencing_test.sh",
    "cargo test --workspace --all-targets --no-fail-fast",
    "cargo clippy --workspace --all-targets -- -D warnings",
]:
    assert marker in P9, marker

print("Spec133 final fixture/protocol/retention/platform/runtime-gate contract: PASS")
