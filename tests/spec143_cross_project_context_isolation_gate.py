#!/usr/bin/env python3
"""Release gate for GitHub #109 cross-project prediction/trajectory/ECS isolation."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
TOOLS = (ROOT / "apps/pi-extension/src/tools.ts").read_text()
TURNS = (ROOT / "apps/pi-extension/src/turns.ts").read_text()
ECS = (ROOT / "crates/focusa-api/src/routes/ecs.rs").read_text()
VISUAL = (ROOT / "crates/focusa-api/src/routes/visual_workflow.rs").read_text()
DAEMON = (ROOT / "crates/focusa-core/src/runtime/daemon.rs").read_text()
TYPES = (ROOT / "crates/focusa-core/src/types.rs").read_text()
PREDICTIONS = (ROOT / "crates/focusa-api/src/routes/predictions.rs").read_text()

for token in [
    "handleTrajectoryMatchesCurrentScope",
    "candidateRoot !== current.projectRoot",
    "candidateContinuity !== current.continuityId",
    "project_root: getSessionCwd()",
    "continuity_id: getContinuityId()",
]:
    assert token in TURNS, f"missing scoped ECS handle defense: {token}"

for token in [
    "typedTrajectoryScopeMatches",
    "cachedTrajectoryForScope",
    "sameWorkstream(body.scope, scope)",
    "response scope differs from requested project/workstream",
]:
    assert token in TOOLS, f"missing Pi client scope defense: {token}"

assert "request_scope_matches(&request_scope, &scope)" in PREDICTIONS
for scoped_key in ["&body.scope", "&scope"]:
    assert re.search(
        rf"\.prediction_store\s*\.recent\({re.escape(scoped_key)},\s*\d+\)",
        PREDICTIONS,
    ), f"missing scoped prediction read for {scoped_key}"
assert "pub fn trajectory_ladder_context_for_scope" in TYPES
for source in [ECS, VISUAL, DAEMON]:
    assert "trajectory_ladder_context_for_scope" in source

print("Spec143 cross-project context isolation gate: PASS")
