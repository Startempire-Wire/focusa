#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAIN = (ROOT / "crates/focusa-cli/src/main.rs").read_text()
MOD = (ROOT / "crates/focusa-cli/src/commands/mod.rs").read_text()
CLI = (ROOT / "crates/focusa-cli/src/commands/silent.rs").read_text()

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
    "/silent-sessions/preflight",
    "/silent-sessions/config/resolve",
    "/silent-sessions/profiles",
    "/silent-sessions/presets",
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
]
for route in required_routes:
    assert route in CLI, route

assert 'const CLI_SCHEMA: &str = "focusa.silent_cli.v1"' in CLI
assert '"process_status"' in CLI
assert '"completion_status"' in CLI
assert '"side_effects"' in CLI
assert '"session_id"' in CLI
assert '"run_id"' in CLI
assert "fn redact(" in CLI
for marker in ["secret", "token", "credential", "authorization", "api_key"]:
    assert f'"{marker}"' in CLI
assert "max_polls" in CLI and ".clamp(1, 10_000)" in CLI
assert "inspect_side_effects_first" in CLI
assert "idempotency_key" in CLI

print("Spec133 full silent CLI static contract: PASS")
