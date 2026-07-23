#!/usr/bin/env python3
"""Real Focusa E2E: compiled daemon + CLI + HTTP + durable restart."""

import json
import os
import signal
import subprocess
import tempfile
import time
import urllib.parse
import urllib.request
from pathlib import Path

DAEMON = os.environ.get("FOCUSA_E2E_DAEMON")
CLI = os.environ.get("FOCUSA_E2E_CLI")
PORT = int(os.environ.get("FOCUSA_E2E_PORT", "18941"))
BASE = f"http://127.0.0.1:{PORT}"

if not DAEMON or not Path(DAEMON).is_file():
    raise SystemExit("FOCUSA_E2E_DAEMON must name the current compiled daemon")
if not CLI or not Path(CLI).is_file():
    raise SystemExit("FOCUSA_E2E_CLI must name the current compiled CLI")


def request(method: str, path: str, body=None, timeout=15):
    data = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(
        BASE + path,
        data=data,
        method=method,
        headers={"content-type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=timeout) as response:
        payload = response.read()
        return response.status, json.loads(payload) if payload else {}


def wait_ready():
    for _ in range(150):
        try:
            status, payload = request("GET", "/v1/health", timeout=1)
            if status == 200 and payload:
                return
        except Exception:
            time.sleep(0.1)
    raise AssertionError("daemon did not become healthy")


def start_daemon(data_dir: str):
    env = os.environ.copy()
    env.update({"FOCUSA_BIND": f"127.0.0.1:{PORT}", "FOCUSA_DATA_DIR": data_dir})
    proc = subprocess.Popen(
        [DAEMON], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
    )
    try:
        wait_ready()
    except Exception:
        output = proc.stdout.read() if proc.poll() is not None else ""
        proc.terminate()
        raise AssertionError(f"daemon startup failed: {output[-4000:]}")
    return proc


def contains_value(value, expected):
    if value == expected:
        return True
    if isinstance(value, dict):
        return any(contains_value(item, expected) for item in value.values())
    if isinstance(value, list):
        return any(contains_value(item, expected) for item in value)
    return False


def stop_daemon(proc):
    proc.send_signal(signal.SIGTERM)
    try:
        proc.wait(timeout=10)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=5)
    if proc.stdout:
        proc.stdout.close()


with tempfile.TemporaryDirectory(prefix="focusa-real-e2e-") as root:
    project = Path(root) / "project"
    data_dir = Path(root) / "data"
    project.mkdir()
    (project / ".beads").mkdir()
    data_dir.mkdir()
    (project / ".focusa-project.json").write_text(
        json.dumps(
            {
                "schema": "focusa.project.v1",
                "project_id": "real-e2e",
                "canonical_name": "Real E2E",
                "project_root": str(project),
                "workspace_kind": "real-e2e",
            }
        )
    )
    continuity = f"real-e2e-{time.time_ns()}"
    quoted_root = urllib.parse.quote(str(project), safe="")
    quoted_cont = urllib.parse.quote(continuity, safe="")

    daemon = start_daemon(str(data_dir))
    try:
        status, checkpoint = request(
            "POST",
            "/v1/workpoint/checkpoint",
            {
                "project_root": str(project),
                "continuity_id": continuity,
                "mission": "Complete a real product journey",
                "current_action": "real_e2e",
                "next_slice": "Define trajectory and attach evidence",
                "canonical": True,
            },
        )
        assert status == 200 and checkpoint.get("canonical") is True, checkpoint
        workpoint_id = checkpoint["workpoint_id"]

        status, goal = request(
            "POST",
            "/v1/trajectory/define-goal",
            {
                "project_root": str(project),
                "continuity_id": continuity,
                "long_term_goal": "Ship a verified real E2E journey",
                "desired_end_state": "Trajectory, Workpoint, evidence and restart persistence agree",
                "current_state": "Fresh isolated daemon",
                "mid_level_goal": "Exercise canonical APIs",
                "short_term_goal": "Persist and resume",
                "waypoints": ["checkpoint", "evidence", "restart", "resume"],
                "goal_source": "operator",
                "operator_confirmed": True,
            },
        )
        assert status == 200 and goal.get("canonical") is True, goal

        status, linked = request(
            "POST",
            "/v1/workpoint/evidence/link",
            {
                "workpoint_id": workpoint_id,
                "target_ref": "real-product-journey",
                "result": "checkpoint and trajectory created by real daemon",
                "evidence_ref": "real-e2e:phase-1",
            },
        )
        assert status == 200 and linked.get("canonical") is True, linked

        status, cognition = request(
            "GET",
            f"/v1/context-cognition?project_root={quoted_root}&continuity_id={quoted_cont}",
        )
        assert status == 200 and contains_value(
            cognition, "focusa.context_cognition_packet.v1"
        ), cognition
        assert contains_value(cognition, continuity) and contains_value(
            cognition, str(project)
        ), cognition

        status, packet = request(
            "POST",
            "/v1/compaction/build",
            {
                "resume_source": "before_compaction",
                "project_root": str(project),
                "continuity_id": continuity,
                "session_id": "real-e2e-session",
                "current_ask": "Preserve the real product journey",
                "omitted_sections": ["raw_tool_history"],
                "omitted_tokens": 1200,
                "rehydrate_refs": ["real-e2e:phase-1"],
            },
        )
        assert (
            status == 200
            and packet.get("schema_version") == "focusa.compaction_mission_packet.v1"
        ), packet
        assert packet.get("trajectory", {}).get("hlt_status") == "canonical_explicit", (
            packet
        )
        assert packet.get("workpoint", {}).get("action_authority") is True, packet
        packet_id = packet["packet_id"]
        status, inspected = request("GET", f"/v1/compaction/inspect/{packet_id}")
        assert status == 200 and contains_value(inspected, packet_id), inspected

        status, pairing = request(
            "POST",
            "/v1/device/pair/start",
            {
                "device_name": "Real E2E Mac",
                "platform": "macos",
                "daemon_base_url": BASE,
                "scopes": ["read", "write"],
            },
        )
        assert status == 200 and pairing.get("code") and pairing.get("device_id"), (
            pairing
        )
        status, completed = request(
            "POST",
            "/v1/device/pair/complete",
            {
                "code": pairing["code"],
                "host": "real-e2e-host",
                "operator_id": "real-e2e-operator",
                "completed_by": "real-e2e",
            },
        )
        assert status == 200 and completed.get("token"), completed
        status, paired_status = request(
            "GET", f"/v1/device/pair/status?code={urllib.parse.quote(pairing['code'])}"
        )
        assert status == 200 and paired_status.get("status") == "completed", (
            paired_status
        )

        status, inventory = request("GET", "/v1/update/status")
        assert (
            status == 200 and inventory.get("schema") == "focusa.update_inventory.v1"
        ), inventory
        assert inventory.get("continuous_currency", {}).get("enabled") is True, (
            inventory
        )
        status, update_plan = request("GET", "/v1/update/plan")
        assert status == 200 and update_plan.get("schema") == "focusa.update_plan.v1", (
            update_plan
        )

        cli_env = os.environ.copy()
        cli_env["FOCUSA_API_URL"] = BASE
        cli_env["FOCUSA_BASE_URL"] = BASE
        cli = subprocess.run(
            [CLI, "--json", "status"],
            env=cli_env,
            text=True,
            capture_output=True,
            timeout=20,
        )
        assert cli.returncode == 0, cli.stdout + cli.stderr
        cli_payload = json.loads(cli.stdout)
        assert cli_payload, cli_payload
    finally:
        stop_daemon(daemon)

    daemon = start_daemon(str(data_dir))
    try:
        status, resumed = request(
            "POST",
            "/v1/workpoint/resume",
            {
                "project_root": str(project),
                "continuity_id": continuity,
                "workpoint_id": workpoint_id,
                "mode": "full_json",
            },
        )
        assert status == 200 and resumed.get("canonical") is True, resumed
        assert resumed.get("workpoint_id") == workpoint_id, resumed

        status, trajectory = request(
            "GET",
            f"/v1/trajectory/view?project_root={quoted_root}&continuity_id={quoted_cont}&mode=summary",
        )
        assert status == 200 and trajectory.get("canonical") is True, trajectory
        assert contains_value(trajectory, "Ship a verified real E2E journey"), (
            trajectory
        )
    finally:
        stop_daemon(daemon)

print(
    json.dumps(
        {
            "schema": "focusa.real_product_e2e.v1",
            "status": "pass",
            "surfaces": [
                "daemon",
                "cli",
                "http",
                "trajectory",
                "workpoint",
                "evidence",
                "context-cognition",
                "compaction",
                "device-pairing",
                "update",
                "restart-persistence",
            ],
        }
    )
)
