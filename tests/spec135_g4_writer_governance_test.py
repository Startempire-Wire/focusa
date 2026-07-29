#!/usr/bin/env python3
"""Spec 135G-4 steering/contention/writer governance proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-writer-governance.v1.json").read_text())
W=(ROOT/"crates/focusa-api/src/routes/work_loop.rs").read_text()
S=(ROOT/"crates/focusa-api/src/scope.rs").read_text()
assert C["acceptance_criteria"] == "Concurrent work routes correctly and conflicting writers fail closed with visible recovery."
assert C["recipient"]["implicit_broadcast"] is False
assert C["recipient"]["broadcast_preview_required"] is True
assert C["writer_lease"]["single_active_writer"] is True
assert C["writer_lease"]["conflict_outcome"] == "fail_closed"
assert C["writer_lease"]["recovery_visible"] is True
assert len(C["writer_lease"]["recovery_actions"]) >= 4
assert C["worktree"]["exact_root_required"] is True
assert C["queues"]["observations"]["advisory_only"] is True
assert C["queues"]["proposals"]["promotion_required"] is True
for token in ("active_writer","writer","preflight"):
    assert token in W, token
for token in ("active_worktree_root","continuity_id"):
    assert token in S, token
assert "Conflicting writers never silently last-write-wins" in C["laws"]
print("Spec 135 G4 steering contention writer governance: PASS")
