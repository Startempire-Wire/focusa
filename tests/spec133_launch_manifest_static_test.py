#!/usr/bin/env python3
"""Non-compiling Spec133 Phase 3.2 launch-manifest defect-prevention lint."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = (ROOT / "crates/focusa-core/src/silent_sessions/launch_manifest.rs").read_text()
RUNNER = (ROOT / "crates/focusa-session-runner/src/main.rs").read_text()
SECURITY = (ROOT / "crates/focusa-session-runner/src/security.rs").read_text()
TESTS = (ROOT / "crates/focusa-core/src/silent_sessions/launch_manifest_test.rs").read_text()

for field in [
    "executable",
    "argv",
    "cwd",
    "safe_env",
    "secret_env_refs",
    "mission_delivery",
    "stdin_mode",
    "stdout_mode",
    "stderr_mode",
    "process_backend",
    "os_user",
    "resource_limits",
    "trust_policy",
    "adapter_config",
]:
    assert f"pub {field}:" in MANIFEST, field

for mode in ["Rpc", "Stdin", "SecureArtifact", "TypedArgument"]:
    assert mode in MANIFEST
for marker in [
    "required ResourceMode was not resolved before launch",
    "required noninteractive trust flag is absent",
    "sensitive environment values must use secret_env_refs",
    "focusa.launch_manifest.v1",
    "manifest_digest",
    "redact_values",
    "TypedResourceModeController",
    "resolve_resource_mode",
]: 
    assert marker in MANIFEST, marker

assert "prepare_launch_manifest" in RUNNER
assert ".args(&manifest.argv)" in SECURITY
assert "Command::new(executable)" in SECURITY
assert "env_clear" in SECURITY
assert "secret://" in SECURITY and "env://" in SECURITY
assert "open_secure_artifact" in SECURITY
assert "mission artifact hash mismatch" in SECURITY
assert "typed mission argument hash mismatch" in SECURITY
assert "process_group(0)" in SECURITY
assert "sh -c" not in SECURITY
assert "bash -c" not in SECURITY
assert "curl" not in SECURITY
assert "curl" not in MANIFEST

for marker in ["$(not-a-shell)", "required_lowmem", "trust_fail_closed", "raw_sensitive_environment"]:
    assert marker in TESTS, marker

print("Spec133 typed launch manifest static contract: PASS")
