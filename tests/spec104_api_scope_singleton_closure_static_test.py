#!/usr/bin/env python3
"""Spec104 API scoped-state singleton closure static proof.

Short non-building guard for the API/core/menubar speedrun slice.
"""
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[1]

turn = (ROOT / "crates/focusa-api/src/routes/turn.rs").read_text()
snapshots = (ROOT / "crates/focusa-api/src/routes/snapshots.rs").read_text()
server = (ROOT / "crates/focusa-api/src/server.rs").read_text()
inventory = json.loads((ROOT / "config/spec104-scoped-state-inventory.json").read_text())

assert "static RECENT_COMPLETED_TURNS" not in turn
assert "OnceLock<Mutex<VecDeque<String>>>" not in turn
assert "recent_completed_turns_by_scope" in turn
assert "require_workstream_key()" in turn

assert "static SNAPSHOTS" not in snapshots
assert "snapshot_store()" not in snapshots
assert "snapshots_by_scope" in snapshots
assert "snapshot_dir(state, scope)" in snapshots

assert "recent_completed_turns_by_scope" in server
assert "snapshots_by_scope" in server
assert "WorkstreamKey" in server

entries = {(e["path"], e["symbol"]): e for e in inventory["entries"]}
assert entries[("crates/focusa-api/src/routes/turn.rs", "RECENT_COMPLETED_TURNS")]["status"] == "eliminated"
assert entries[("crates/focusa-api/src/routes/snapshots.rs", "SNAPSHOTS")]["status"] == "eliminated"

print("spec104 api scope singleton closure static proof: ok")
