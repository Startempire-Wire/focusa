#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAIN = (ROOT / "crates/focusa-cli/src/main.rs").read_text()
MOD = (ROOT / "crates/focusa-cli/src/commands/mod.rs").read_text()
CLI = (ROOT / "crates/focusa-cli/src/commands/silent.rs").read_text()
CLI_RENDER = (ROOT / "crates/focusa-cli/src/commands/silent_render.rs").read_text()
CLI_SURFACE = CLI + CLI_RENDER
API_ROUTES = (ROOT / "crates/focusa-api/src/routes/silent_sessions.rs").read_text()
API_OBSERVE = (ROOT / "crates/focusa-api/src/routes/silent_sessions_observe.rs").read_text()
API_RETENTION = (ROOT / "crates/focusa-api/src/routes/silent_sessions_retention.rs").read_text()
API_RETENTION_EXPORT = (ROOT / "crates/focusa-api/src/routes/silent_sessions_retention_export.rs").read_text()
CORE_RETENTION = (ROOT / "crates/focusa-core/src/silent_sessions/retention.rs").read_text()
DB_SCHEMA = (ROOT / "crates/focusa-core/src/silent_sessions/persistence_sqlite.rs").read_text()

assert "pub mod silent;" in MOD
assert "Silent(commands::silent::SilentCmd)" in MAIN
assert "commands::silent::run(cmd, cli.json).await" in MAIN

commands = [
    "Preflight",
    "Create",
    "Start",
    "List",
    "Show",
    "Watch",
    "Output",
    "Send",
    "Steer",
    "FollowUp",
    "Key",
    "Pause",
    "Resume",
    "Interrupt",
    "Cancel",
    "Restart",
    "Adopt",
    "Config",
    "Profile",
    "Preset",
    "Model",
    "Checkpoints",
    "Evidence",
    "Receipt",
    "Export",
    "Hold",
    "Delete",
    "Purge",
    "Doctor",
]
for command in commands:
    assert f"    {command}" in CLI, command

for command in ["Resolve", "Diff", "Apply", "Rollback"]:
    assert f"    {command}" in CLI, f"config {command}"

required_routes = [
    "/v1/silent-sessions/preflight",
    "/v1/silent-sessions/config/resolve",
    "/v1/silent-sessions/profiles",
    "/v1/silent-sessions/presets",
    "/events",
    "/output",
    "/input",
    "/steer",
    "/follow-up",
    "/keys",
    "/config/preview",
    "/config/revisions",
    "/config/rollback",
    "/checkpoints",
    "/artifacts",
    "/receipts",
    "/export",
    "/evidence-hold",
    "/purge",
    "/capabilities",
    "/v1/providers/",
    "/models/preflight",
]
for route in required_routes:
    assert route in CLI, route

assert 'const CLI_SCHEMA: &str = "focusa.silent_cli.v1"' in CLI_RENDER
assert '"process_status"' in CLI_SURFACE
assert '"completion_status"' in CLI_SURFACE
assert '"side_effects"' in CLI_SURFACE
assert '"session_id"' in CLI_SURFACE
assert '"run_id"' in CLI_SURFACE
assert "fn redact(" in CLI_RENDER
for marker in ["secret", "token", "credential", "authorization", "api_key"]:
    assert f'"{marker}"' in CLI_RENDER
assert "max_polls" in CLI and ".clamp(1, 10_000)" in CLI
assert "inspect_side_effects_first" in CLI
assert "idempotency_key" in CLI
assert '"run_id"' in CLI and '"generation"' in CLI and '"approval_id"' in CLI
assert '"follow", Some("false".into())' in CLI
assert '/data/next_cursor' in CLI

for route in ["/export", "/evidence-hold", "/purge"]:
    assert route in API_RETENTION, route
assert "ordinary_delete" in API_ROUTES and ".delete(" in API_ROUTES
assert "if !query.follow" in API_OBSERVE
assert '"event_page"' in API_OBSERVE and '"output_page"' in API_OBSERVE
assert "evidence_hold_active" in API_RETENTION
assert "load_retention_operation" in API_RETENTION
assert "principal_id" in API_RETENTION
assert "ordinary_delete_session" in CORE_RETENTION
assert "export_session_bundle" in CORE_RETENTION
assert "export_output" in API_RETENTION and "SecureStreamStore" in API_RETENTION_EXPORT
assert "body.include_output" in API_RETENTION
assert "guard_exact_target" in API_RETENTION
assert "hold_expires_at" in CORE_RETENTION and "parse_from_rfc3339" in CORE_RETENTION
assert "purge_session" in CORE_RETENTION
assert "silent_session_control_retention_operations" in DB_SCHEMA
assert "SILENT_SESSION_DB_SCHEMA_VERSION: i64 = 5" in DB_SCHEMA

print("Spec133 full silent CLI and daemon parity static contract: PASS")
