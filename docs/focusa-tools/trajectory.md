# Trajectory Tool Index

Trajectory tools provide per-project north-star orientation. They are advisory projection tools, not planners or executors.

Navigation metaphor: Trajectory is the route model — current functional state, desired destination/outcome, and waypoint goals. `project_root` is the project folder/container that keeps the route attached to the right project.

Official ladder: **HLT** (High-Level Trajectory) → **MLG** (Mid-Level Goal) → **STG** (Short-Term Goal) → **Waypoints** → Workpoint. Models defer to the operator while actively offering HLT-aligned MLGs, STGs, and Waypoints as route guidance.

- [`focusa_trajectory_view`](tools/focusa_trajectory_view.md)
- [`focusa_trajectory_define_goal`](tools/focusa_trajectory_define_goal.md)
- [`focusa_trajectory_assess`](tools/focusa_trajectory_assess.md)
- [`focusa_trajectory_propose_workpoint`](tools/focusa_trajectory_propose_workpoint.md)
- [`focusa_trajectory_checkpoint`](tools/focusa_trajectory_checkpoint.md)
- [`focusa_trajectory_resume`](tools/focusa_trajectory_resume.md)

Boundary: Trajectory proposes and orients; Workpoint checkpoint/resume remains canonical continuation authority, Beads remains task authority, and operator steering wins.

Hierarchy: Trajectory exposes HLT/MLG/STG/Waypoint fields plus high/mid/low similarity keys for advisory grouping, but session authority remains `project_root + continuity_id`. Same high-level groups never merge sessions when mid/low goals or continuity IDs differ.
