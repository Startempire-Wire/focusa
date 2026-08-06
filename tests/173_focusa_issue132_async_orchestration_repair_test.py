#!/usr/bin/env python3
"""Static release gate for GitHub #132 asynchronous orchestration repair."""

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    value = (ROOT / path).read_text(encoding="utf-8")
    assert value.strip(), f"empty required repair surface: {path}"
    return value


silent = read("crates/focusa-cli/src/commands/silent.rs")
main = read("crates/focusa-cli/src/main.rs")
modules = read("crates/focusa-cli/src/commands/mod.rs")
work_loop = read("crates/focusa-cli/src/commands/work_loop.rs")
doctor = read("crates/focusa-cli/src/commands/doctor.rs")
api_caps = read("crates/focusa-api/src/routes/silent_sessions_capabilities.rs")
api_config = read("crates/focusa-api/src/routes/silent_sessions_config_read.rs")
api_loop = read("crates/focusa-api/src/routes/work_loop.rs")
core_daemon = read("crates/focusa-core/src/runtime/daemon.rs")
bd_adapter = read("crates/focusa-core/src/work_item/adapters/bd.rs")
api_server = read("crates/focusa-api/src/server.rs")
bd_adapter = read("crates/focusa-core/src/work_item/adapters/bd.rs")
scheduler = read("crates/focusa-core/src/work_item/scheduler.rs")
cli_manifest = read("crates/focusa-cli/Cargo.toml")
route_generator = read("scripts/generate-spec152-route-entitlement-table.py")
route_table = read("crates/focusa-api/src/middleware/entitlement_routes.rs")

# The CLI must use the same canonical /v1 paths mounted by the daemon.
assert not re.search(r'"/(?:silent-sessions|harnesses|providers)(?:[/"?])', silent)
for route in (
    "/v1/silent-sessions",
    "/v1/silent-sessions/profiles",
    "/v1/silent-sessions/presets",
    "/v1/silent-sessions/capabilities",
    "/v1/harnesses",
    "/v1/providers",
    "/models/preflight",
):
    assert route in silent, route
assert 'get_probe("/v1/health")' in silent
assert "Model(ModelCmd)" in silent
assert "unsupported_model" in api_caps
assert "FOCUSA_PI_SUPPORTED_MODELS" in api_caps
assert "local_pi_isolated" in api_config
assert '"preset_id": "conservative"' in api_config

# Work Loop is an explicit CLI surface with scope, writer, and fencing inputs.
assert "pub mod work_loop;" in modules
assert 'name = "work-loop"' in main
assert "WorkLoop(commands::work_loop::WorkLoopCmd)" in main
assert "Commands::WorkLoop(cmd)" in main
for operation in (
    "Status",
    "WriterStatus",
    "Enable",
    "Checkpoint",
    "Context",
    "SelectNext",
    "Pause",
    "Resume",
    "Stop",
):
    assert f"    {operation}" in work_loop, operation
assert "idempotency_key" in work_loop
assert "ApiClient::with_timeout_secs(60)" in work_loop
assert "WRITER_LEASE_TTL_MS: i64 = 120_000" in api_loop
assert "WORK_ITEM_PROVIDER_SELECTION_TIMEOUT_MS: u64 = 3_000" in core_daemon
assert "from_millis(WORK_ITEM_PROVIDER_SELECTION_TIMEOUT_MS)" in core_daemon
assert 'get("dependents")' in bd_adapter
assert "show_values(&query.project_root, &child_ids)" in bd_adapter
assert '"idempotency-key", &driver_idempotency_key' in api_server
assert '"idempotency_key": driver_idempotency_key' in api_server
assert "active.expires_at =" in api_server
assert "writer_lease_expiry(now)" in api_server
assert "stale_driver" in api_loop
assert "existing.child.try_wait()" in api_loop
for header in (
    "x-scope-project-root",
    "x-scope-continuity-id",
    "x-focusa-writer-id",
    "x-focusa-fencing-token",
    "x-focusa-approval",
):
    assert header in work_loop, header

# An approved enable may recover only an execution scope whose canonical
# Workpoint disappeared; ordinary live scopes remain fenced.
assert "active_scope_orphaned" in api_loop
assert "orphaned_scope_recovered" in api_loop
assert "stale_inert_or_orphaned_scope_rebinds_only_at_safe_boundaries" in api_loop
assert 'parts.uri.path() == "/v1/work-loop/enable"' in api_loop
assert 'value == "approved"' in api_loop
for route in (
    "/v1/work-loop/enable",
    "/v1/work-loop/checkpoint",
    "/v1/work-loop/context",
    "/v1/work-loop/select-next",
    "/v1/work-loop/pause",
    "/v1/work-loop/resume",
    "/v1/work-loop/stop",
):
    assert route in route_generator, route
    assert f'template: "{route}"' in route_table, route

# Doctor must fail closed on absent/degraded/empty orchestration contracts.
for route in (
    "/v1/silent-sessions?limit=1",
    "/v1/silent-sessions/profiles",
    "/v1/silent-sessions/presets",
    "/v1/silent-sessions/capabilities",
):
    assert route in doctor, route
assert "canonical_orchestration_check" in doctor
assert '"status": "blocked"' in doctor
assert "degraded" in doctor and "canonical" in doctor

# Parent-child is hierarchy only. Scheduler traverses descendants and evaluates
# only the normalized true dependency vector.
assert 'if relation == "parent-child"' in bd_adapter
assert "parent = Some(make_ref(dependency_id))" in bd_adapter
assert "dependencies.push" in bd_adapter
assert "root_query_reaches_nested_ready_leaves_before_parent_gates" in scheduler
assert "is_descendant_of" in scheduler

# Focused CLI compilation has its direct dependency and regression coverage.
assert "focusa-license = { workspace = true }" in cli_manifest
for path in (
    "crates/focusa-cli/tests/work_loop_cli_e2e.rs",
    "crates/focusa-cli/tests/silent_model_preflight_e2e.rs",
    "crates/focusa-cli/tests/silent_read_parity_e2e.rs",
    "crates/focusa-cli/tests/silent_doctor_e2e.rs",
):
    assert (ROOT / path).is_file(), path

print("GitHub #132 asynchronous orchestration repair static gate: PASS")
