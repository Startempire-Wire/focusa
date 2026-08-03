#!/usr/bin/env python3
"""Spec 135E-1 cross-spec migration/amendment/rollback closure."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-cross-spec-closure.v1.json").read_text())
T=(ROOT/"apps/pi-extension/src/turns.ts").read_text()
W=(ROOT/"apps/pi-extension/src/mission-canvas-widget.ts").read_text()
assert all(r["passed"] for r in C["migration_contracts"])
assert C["authority"]["presentation_clients_own_state"] is False
assert C["authority"]["parallel_runtime"] is False
assert C["rollback"]["exact_snapshot"] is True
assert C["rollback"]["original_state_preserved_on_failure"] is True
assert C["rollback"]["silent_empty_initialization"] is False
assert 'setWidget("focusa"' not in T
assert "focusa-mission-canvas-work-rail" in W
for removal in C["legacy_removals"]: assert removal["removed"] and removal["canonical"]
for ref in C["proof_refs"]: assert (ROOT/ref).exists(), ref
print("Spec 135 E1 cross-spec migration/amendment/rollback closure: PASS")
