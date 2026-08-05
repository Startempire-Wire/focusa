#!/usr/bin/env python3
"""Ensure public license CLI cannot mint or trust plaintext entitlement."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "crates/focusa-cli/src/commands/license.rs").read_text()
INSTALL = (ROOT / "crates/focusa-cli/src/commands/install.rs").read_text()
API_CLIENT = (ROOT / "crates/focusa-cli/src/api_client.rs").read_text()

dispatch = SOURCE[SOURCE.index("pub async fn run("):SOURCE.index("async fn run_activate")]
assert "run_activate" not in dispatch
assert "run_deactivate" not in dispatch
assert "run_devmode_full" not in dispatch
assert "run_refresh" not in dispatch
assert "run_watch" not in dispatch
assert "E_AUTHORITY_COMMAND_RETIRED" in dispatch

status = SOURCE[SOURCE.index("async fn run_status"):SOURCE.index("async fn run_deactivate")]
assert "focusa_license::resolve_license_guard()" in status
assert "core_status" not in status
assert "customer_email" not in status
assert "key_hash" not in status
assert "marketing_preference" in status

check = SOURCE[SOURCE.index("async fn run_check_feature"):SOURCE.index("// Avoid unused import")]
assert "resolve_license_guard" in check
assert "snapshot.features.get(feature)" in check
assert "unknown_or_not_granted" in check
assert "core_check_feature" not in check

assert 'return Ok("eval".to_string())' not in INSTALL
assert "E_AUTHORITY_RAW_KEY_FORBIDDEN" in INSTALL
assert "E_AUTHORITY_LEASE_UNUSABLE" in INSTALL
assert "body_idempotency_key" in API_CLIENT
assert 'req.header("Idempotency-Key", key.trim())' in API_CLIENT
assert 'format!("Idempotency-Key: {}", key.trim())' in API_CLIENT

print("Spec152 CLI authority gate: PASS")
