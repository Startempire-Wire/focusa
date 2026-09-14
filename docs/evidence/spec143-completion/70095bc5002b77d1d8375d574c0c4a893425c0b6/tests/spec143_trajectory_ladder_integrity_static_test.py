#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
trajectory = (ROOT / "crates/focusa-api/src/routes/trajectory.rs").read_text()
ontology = (ROOT / "crates/focusa-api/src/routes/ontology.rs").read_text()
types = (ROOT / "crates/focusa-core/src/types.rs").read_text()
project = (ROOT / "crates/focusa-api/src/routes/project.rs").read_text()
tui = (ROOT / "crates/focusa-tui/src/views/help_overlay.rs").read_text()

for forbidden in [
    "bootstrap_degraded_placeholder",
    "Close active gap:",
    "Advance STG:",
    "Validate MLG:",
    "Maintain and improve {project_label}",
]:
    assert forbidden not in trajectory, f"forbidden Trajectory synthesis remains: {forbidden}"

assert "let short_term_goal = persisted_short_term_goal.map(str::to_string);" in trajectory
assert "let mid_level_goal = persisted_mid_level_goal.map(str::to_string);" in trajectory
assert "let persisted_trajectory = persisted_exact_trajectory;" in trajectory
assert "HLT_IMPASSE: explicit operator HLT commitment required" in trajectory
assert "persisted_prior_project_trajectory" in trajectory

assert '"milestone"' not in ontology
assert ontology.count('"trajectory_waypoint"') >= 6
assert "current milestone" not in tui.lower()
assert "TrajectoryMilestoneRecord" not in types
assert "TrajectoryMilestoneStatus" not in types
assert "pub waypoints: Vec<TrajectoryWaypointRecord>" in types
assert '#[serde(alias = "milestone_id")]' in types
assert 'canonical.get("milestones").is_none()' in types

assert '"focusa.project.v2"' in project
assert '"HLT_IMPASSE"' in project
assert '"/v1/project/trajectory-guard"' in project
assert "write_json_file_atomic" in project
assert "append_trajectory_ladder_events" in project
print("Spec143 Trajectory Ladder integrity static gate: PASS")
