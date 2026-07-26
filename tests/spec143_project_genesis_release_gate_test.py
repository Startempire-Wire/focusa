#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
route = (ROOT / "crates/focusa-api/src/routes/project_genesis.rs").read_text()
support = (ROOT / "crates/focusa-api/src/routes/project_genesis_support.rs").read_text()
workpoint = (ROOT / "crates/focusa-api/src/routes/workpoint.rs").read_text()
server = (ROOT / "crates/focusa-api/src/server.rs").read_text()
cli = (ROOT / "crates/focusa-cli/src/commands/project.rs").read_text()
pi_tools = (ROOT / "apps/pi-extension/src/tools.ts").read_text()
pi_session = (ROOT / "apps/pi-extension/src/session.ts").read_text()
operation_registry = (ROOT / "docs/contracts/spec135/generated-contract-v1/operation-registry.json").read_text()
api_docs = (ROOT / "docs/current/API_REFERENCE_CURRENT.md").read_text()
cli_docs = (ROOT / "docs/current/CLI_REFERENCE_CURRENT.md").read_text()

for endpoint in ("start", "resume", "status", "commit"):
    assert f'/v1/project/genesis/{endpoint}' in route, endpoint
    assert f'/v1/project/genesis/{endpoint}' in api_docs, endpoint
assert ".merge(routes::project_genesis::router())" in server
assert "pub(crate) async fn materialize_workpoint_events" in workpoint

for link in (
    "project_identity",
    "bootstrap_receipt",
    "hlt",
    "specification_and_acceptance",
    "current_and_desired_state",
    "mlg",
    "stg",
    "waypoints",
    "task_provider_and_task_graph",
    "first_workpoint_candidate",
    "coordination_owner",
    "readiness_receipt",
):
    assert f'"{link}"' in support or f'"{link}"' in route, link

assert '"hlt_impasse"' in support
assert "answer one concise HLT intent question" in support
assert "discover_beads_tasks" in support
assert '"provider_neutral"' in support
assert "allow_task_decomposition" in support
assert "operator_steering_precedence" in support
assert "FocusaEvent::TrajectoryGoalDefined" in route
assert "FocusaEvent::WorkpointCheckpointProposed" in route
assert "FocusaEvent::WorkpointCheckpointPromoted" in route
assert "materialize_workpoint_events" in route
assert route.index('packet["status"] = json!("ready")') < route.index("write_json_atomic(&marker_path, &marker)")
assert 'marker["genesis_binding"]' in route
assert '"marker_guard": "verified"' in route
assert "already_committed" in route
for choice in (
    "View current work",
    "Coordinate with that agent",
    "Take over with confirmation",
    "Continue read-only",
):
    assert choice in route, choice
assert "writer lease" not in route
assert "takeover_confirmation_required" in route

assert "ProjectGenesisCmd" in cli
for command in ("genesis start", "genesis resume", "genesis status", "genesis commit"):
    assert command in cli_docs, command
assert 'name: "focusa_project_genesis"' in pi_tools
assert "ensureProjectGenesis" in pi_session
assert 'focusaFetch("/project/genesis/start"' in pi_session
assert 'focusaFetch("/project/genesis/commit"' in pi_session
assert "session_start_genesis_ready" in pi_session
assert "without creating a placeholder Workpoint" in pi_session
for operation in (
    "focusa.project.genesis.start",
    "focusa.project.genesis.resume",
    "focusa.project.genesis.status",
    "focusa.project.genesis.commit",
):
    assert operation in operation_registry, operation
assert '{ triggerTurn: true }' not in route

for path in (
    ROOT / "crates/focusa-api/src/routes/project_genesis.rs",
    ROOT / "crates/focusa-api/src/routes/project_genesis_support.rs",
    ROOT / "crates/focusa-api/src/routes/project_genesis_tests.rs",
):
    assert len(path.read_text().splitlines()) < 500, path

print("Spec143 Project Genesis release gate: PASS")
