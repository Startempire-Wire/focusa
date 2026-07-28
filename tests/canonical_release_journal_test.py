#!/usr/bin/env python3
"""Offline contract tests for the canonical release journal client."""

import importlib.util
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/canonical-release-journal.py"
spec = importlib.util.spec_from_file_location("canonical_release_journal", SCRIPT)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)

improved = module.metric_comparison(80, 100, "seconds", True)
assert improved == {
    "current": 80,
    "baseline": 100,
    "delta": -20.0,
    "unit": "seconds",
    "direction": "improved",
}
assert module.metric_comparison(120, 100, "seconds", True)["direction"] == "degraded"
assert module.metric_comparison(100, 100, "seconds", True)["direction"] == "unchanged"
assert module.metric_comparison(None, 100, "seconds", True)["direction"] == "not_comparable"
assert module.metric_comparison(61, 60, "count", False)["direction"] == "improved"

payload = module.event(
    "v0.9.136",
    "plan",
    1,
    event_id="focusa:v0.9.136:plan:test",
    estimates={"total_elapsed_seconds": 1800},
    measurements={"candidate_commit": "abc"},
    evidence_refs=["test:offline"],
)
assert payload["schema"] == "agent-kb.release_journal.event.v1"
assert payload["release_id"] == "focusa:v0.9.136"
assert payload["phase"] == "plan"
assert payload["sequence"] == 1
json.dumps(payload, sort_keys=True)

actuals = {"total_elapsed_seconds": 900, "remote_pipeline_seconds": 600, "asset_count": 60, "problems_count": 1}
estimates = {"total_elapsed_seconds": 1200, "remote_pipeline_seconds": 500, "asset_count": 60, "problems_count": 0}
deltas = module.estimate_deltas(actuals, estimates)
assert deltas["total_elapsed_seconds"]["direction"] == "improved"
assert deltas["remote_pipeline_seconds"]["direction"] == "degraded"
assert deltas["asset_count"]["direction"] == "unchanged"
assert deltas["problems_count"]["direction"] == "degraded"

help_run = subprocess.run([sys.executable, str(SCRIPT), "--help"], capture_output=True, text=True)
assert help_run.returncode == 0
for command in ("backfill", "plan", "benchmark", "progress", "problem", "finalize", "history"):
    assert command in help_run.stdout

release_script = (ROOT / "scripts/create-dev-release-tag.sh").read_text()
for lifecycle_command in ("journal_client plan", "journal_client benchmark", "journal_client progress", "journal_client problem", "journal_client finalize"):
    assert lifecycle_command in release_script
assert "FOCUSA_RELEASE_JOURNAL_MODE" in release_script
assert "run-release-learning-guards.py" in release_script
assert "journal_client history --release-id" in release_script
assert "Canonical release journal plan resumed" in release_script
assert "Resuming exact stamped release surfaces" in release_script
assert "RELEASE_RETRY_DIRTY" in release_script
assert "--tag" in release_script

client_source = SCRIPT.read_text()
for learning_binding in ("retrieve_release_lessons", "record_release_predictions", "capture_release_lesson", "evaluate_stage_prediction"):
    assert learning_binding in client_source

guards = json.loads((ROOT / "config/release-learning-guards.json").read_text())
assert guards["schema"] == "focusa.release_learning_guards.v1"
classes = [row["failure_class"] for row in guards["guards"]]
assert len(classes) >= 6
assert len(classes) == len(set(classes))
assert all(row["lesson_ref"] and row["command"] for row in guards["guards"])

print("canonical release journal client contract: passed")
