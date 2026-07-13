#!/usr/bin/env python3
"""Five strict, independent compiled-product E2Es mapped directly to Focusa specs."""

import json
import os
import signal
import subprocess
import tempfile
import time
import unittest
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

DAEMON = os.environ.get("FOCUSA_E2E_DAEMON", "")
BASE_PORT = int(os.environ.get("FOCUSA_STRICT_E2E_PORT", "18950"))


def request(base, method, path, body=None, timeout=15, allow_error=False):
    data = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(base + path, data=data, method=method, headers={"content-type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as response:
            raw = response.read()
            return response.status, json.loads(raw) if raw else {}
    except urllib.error.HTTPError as error:
        raw = error.read()
        payload = json.loads(raw) if raw else {}
        if allow_error:
            return error.code, payload
        raise


def contains(value, expected):
    if value == expected:
        return True
    if isinstance(value, dict):
        return any(contains(item, expected) for item in value.values())
    if isinstance(value, list):
        return any(contains(item, expected) for item in value)
    return False


class StrictSpecProductE2E(unittest.TestCase):
    counter = 0

    def setUp(self):
        if not DAEMON or not Path(DAEMON).is_file():
            self.fail("FOCUSA_E2E_DAEMON must point to the current compiled daemon")
        type(self).counter += 1
        self.port = BASE_PORT + type(self).counter
        self.base = f"http://127.0.0.1:{self.port}"
        self.temp = tempfile.TemporaryDirectory(prefix="focusa-strict-e2e-")
        self.root = Path(self.temp.name)
        self.project = self.root / "project"
        self.data = self.root / "data"
        self.project.mkdir()
        (self.project / ".beads").mkdir()
        self.data.mkdir()
        (self.project / ".focusa-project.json").write_text(json.dumps({
            "schema": "focusa.project.v1", "project_id": "strict-e2e",
            "canonical_name": "Strict E2E", "project_root": str(self.project),
            "workspace_kind": "strict-e2e",
        }))
        self.continuity = f"strict-e2e-{time.time_ns()}"
        self.proc = self.start_daemon()

    def tearDown(self):
        self.stop_daemon()
        self.temp.cleanup()

    def start_daemon(self):
        env = os.environ.copy()
        env.update({"FOCUSA_BIND": f"127.0.0.1:{self.port}", "FOCUSA_DATA_DIR": str(self.data)})
        proc = subprocess.Popen([DAEMON], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        for _ in range(150):
            try:
                if request(self.base, "GET", "/v1/health", timeout=1)[0] == 200:
                    return proc
            except Exception:
                time.sleep(0.1)
        output = proc.stdout.read() if proc.poll() is not None else ""
        proc.terminate()
        self.fail(f"daemon failed to start: {output[-3000:]}")

    def stop_daemon(self):
        if not getattr(self, "proc", None) or self.proc.poll() is not None:
            return
        self.proc.send_signal(signal.SIGTERM)
        try:
            self.proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            self.proc.wait(timeout=5)
        if self.proc.stdout:
            self.proc.stdout.close()

    def restart_daemon(self):
        self.stop_daemon()
        self.proc = self.start_daemon()

    def create_canonical_context(self):
        _, checkpoint = request(self.base, "POST", "/v1/workpoint/checkpoint", {
            "project_root": str(self.project), "continuity_id": self.continuity,
            "mission": "Strict spec comparison", "current_action": "strict_e2e",
            "next_slice": "Prove canonical cross-surface state", "canonical": True,
        })
        workpoint_id = checkpoint["workpoint_id"]
        _, trajectory = request(self.base, "POST", "/v1/trajectory/define-goal", {
            "project_root": str(self.project), "continuity_id": self.continuity,
            "long_term_goal": "Prove strict cross-surface specification behavior",
            "desired_end_state": "All authoritative surfaces agree after restart",
            "current_state": "Fresh compiled daemon", "mid_level_goal": "Exercise strict E2E",
            "short_term_goal": "Verify five specification boundaries",
            "waypoints": ["context", "evidence", "compaction", "pairing", "inventory"],
            "goal_source": "operator", "operator_confirmed": True,
        })
        return workpoint_id, trajectory["trajectory_id"]

    # Spec 100/130: Context Cognition must resolve canonical authority already created in the same scope.
    def test_1_context_cognition_resolves_canonical_workpoint_and_trajectory(self):
        workpoint_id, trajectory_id = self.create_canonical_context()
        query = urllib.parse.urlencode({"project_root": str(self.project), "continuity_id": self.continuity})
        status, cognition = request(self.base, "GET", f"/v1/context-cognition?{query}")
        self.assertEqual(status, 200)
        packet = cognition.get("packet", cognition)
        self.assertEqual(packet.get("scope_status"), "matched", cognition)
        self.assertEqual(packet.get("scope", {}).get("workpoint_id"), workpoint_id, cognition)
        self.assertEqual(packet.get("scope", {}).get("trajectory_id"), trajectory_id, cognition)

    # Spec 96/130: linked evidence must remain in the canonical resume packet after process restart.
    def test_2_workpoint_evidence_survives_restart(self):
        workpoint_id, _ = self.create_canonical_context()
        evidence_ref = "strict-e2e:evidence-survives-restart"
        status, linked = request(self.base, "POST", "/v1/workpoint/evidence/link", {
            "workpoint_id": workpoint_id, "target_ref": "strict-spec-product-e2e",
            "result": "durable evidence proof", "evidence_ref": evidence_ref,
        })
        self.assertEqual(status, 200, linked)
        self.restart_daemon()
        status, resumed = request(self.base, "POST", "/v1/workpoint/resume", {
            "project_root": str(self.project), "continuity_id": self.continuity,
            "workpoint_id": workpoint_id, "mode": "full_json",
        })
        self.assertEqual(status, 200)
        self.assertTrue(contains(resumed, evidence_ref), resumed)
        self.assertTrue(contains(resumed, "durable evidence proof"), resumed)

    # Spec 130: packet durability is part of replay, not an in-process cache illusion.
    def test_3_compaction_packet_survives_restart_and_replays(self):
        self.create_canonical_context()
        status, packet = request(self.base, "POST", "/v1/compaction/build", {
            "resume_source": "before_compaction", "project_root": str(self.project),
            "continuity_id": self.continuity, "session_id": "strict-e2e-session",
            "current_ask": "Persist this packet", "rehydrate_refs": ["strict-e2e:packet"],
        })
        self.assertEqual(status, 200)
        packet_id = packet["packet_id"]
        self.restart_daemon()
        status, restored = request(self.base, "GET", f"/v1/compaction/packet/{packet_id}", allow_error=True)
        self.assertEqual(status, 200, restored)
        self.assertTrue(contains(restored, packet_id), restored)
        status, replayed = request(self.base, "POST", "/v1/compaction/replay", {"packet_id": packet_id}, allow_error=True)
        self.assertEqual(status, 200, replayed)
        self.assertTrue(contains(replayed, packet_id), replayed)

    # Device pairing security contract: unsafe runtime paths cannot become durable host labels.
    def test_4_device_pairing_rejects_unsafe_host_scope(self):
        _, pairing = request(self.base, "POST", "/v1/device/pair/start", {
            "device_name": "Strict E2E Mac", "platform": "macos",
            "daemon_base_url": self.base, "scopes": ["read", "write"],
        })
        status, rejected = request(self.base, "POST", "/v1/device/pair/complete", {
            "code": pairing["code"], "host": "/home/example/.cargo",
            "operator_id": "strict-e2e", "completed_by": "strict-e2e",
        }, allow_error=True)
        self.assertEqual(status, 422, rejected)
        self.assertIn(rejected.get("failure_class"), {"scope_mismatch", "unsafe_host"}, rejected)

    # Spec 128 inventory table: every managed/protected surface must be visible, not only binaries.
    def test_5_update_inventory_covers_every_spec128_part(self):
        status, inventory = request(self.base, "GET", "/v1/update/status")
        self.assertEqual(status, 200)
        actual = {item.get("part") for item in inventory.get("parts", [])}
        expected = {
            "daemon", "cli", "tui", "service_definition", "service_overrides",
            "runtime_home", "env", "license_files", "source_checkout",
            "release_assets", "desktop_app", "agent_extension", "public_installer",
        }
        self.assertFalse(expected - actual, {"missing": sorted(expected - actual), "actual": sorted(str(x) for x in actual)})


if __name__ == "__main__":
    unittest.main(verbosity=2)
