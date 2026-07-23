#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "crates/focusa-core/src/silent_sessions/harness_adapter.rs").read_text()

for method in [
    "capabilities",
    "preflight",
    "build_launch_manifest",
    "parse_event",
    "send_prompt",
    "send_input",
    "abort",
    "query_state",
    "query_model",
    "resume_native_session",
]:
    assert f"fn {method}" in SOURCE, method

for marker in [
    "HARNESS_ADAPTER_PROTOCOL_MAJOR",
    "HARNESS_ADAPTER_PROTOCOL_MINOR",
    "CapabilitySupport::Unsupported",
    "DeterministicFakeAdapter",
    "PiRpcAdapter",
    "DirectProcessBackend",
    "GenericRpcBackend",
    "GenericPtyBackend",
    "ProcessBackendCapabilities",
    "harness protocol major version mismatch",
    "hard_pause",
    "process_tree_kill",
]:
    assert marker in SOURCE, marker

assert SOURCE.count("CapabilitySupport::Unsupported") >= 4
assert len(SOURCE.splitlines()) <= 500
print("Spec133 harness/backend capability static contract: PASS")
