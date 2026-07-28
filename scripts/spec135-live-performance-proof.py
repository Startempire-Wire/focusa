#!/usr/bin/env python3
"""Measure live health and Pi Mission Canvas render budgets for Spec 135D/Q3."""
import json
import statistics
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

R = Path(__file__).resolve().parents[1]
budgets = json.loads((R / "docs/contracts/spec135-q3-performance-budgets.v1.yaml").read_text())["budgets"]
health_samples = []
for _ in range(12):
    started = time.perf_counter()
    with urllib.request.urlopen("http://127.0.0.1:8787/v1/health", timeout=2) as response:
        body = response.read()
        assert response.status == 200 and body
    health_samples.append((time.perf_counter() - started) * 1000)
health_samples.sort()
health_p95 = health_samples[min(len(health_samples) - 1, int(len(health_samples) * 0.95))]
commands = [
    (["node", "tests/mission-canvas-performance.test.mjs"], R / "apps/pi-extension", "mission_canvas_render"),
    ([sys.executable, "tests/spec135_durable_event_stream_test.py"], R, "stream_replay"),
    ([sys.executable, "tests/spec135_context_retrieval_test.py"], R, "bounded_retrieval"),
    ([sys.executable, "tests/spec135_workspace_live_refresh_test.py"], R, "bounded_live_refresh"),
    (["npx", "--yes", "tsx", "tests/utility_card_session_isolation_test.mts"], R, "tool_output_isolation"),
]
checks = {}
for command, cwd, name in commands:
    run = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    checks[name] = {
        "status": "passed" if run.returncode == 0 else "blocked",
        "result": (run.stdout or run.stderr).strip()[-240:],
    }
result = {
    "schema": "focusa.spec135.live_performance_proof.v1",
    "status": "passed"
    if health_p95 <= budgets["daemon_health_ms"]["target"] and all(row["status"] == "passed" for row in checks.values())
    else "blocked",
    "daemon_health": {
        "samples": len(health_samples),
        "p95_ms": round(health_p95, 3),
        "budget_ms": budgets["daemon_health_ms"]["target"],
    },
    "runtime_checks": checks,
    "max_projected_rows": budgets["rendered_rows"]["maximum"],
    "evidence_ref": "scripts/spec135-live-performance-proof.py",
}
print(json.dumps(result, indent=2, sort_keys=True))
sys.exit(0 if result["status"] == "passed" else 1)
