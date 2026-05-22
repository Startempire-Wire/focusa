# Trajectory Tool Index

Trajectory tools provide per-project north-star orientation. They are advisory projection tools, not planners or executors.

- [`focusa_trajectory_view`](tools/focusa_trajectory_view.md)
- [`focusa_trajectory_define_goal`](tools/focusa_trajectory_define_goal.md)
- [`focusa_trajectory_assess`](tools/focusa_trajectory_assess.md)
- [`focusa_trajectory_propose_workpoint`](tools/focusa_trajectory_propose_workpoint.md)
- [`focusa_trajectory_checkpoint`](tools/focusa_trajectory_checkpoint.md)
- [`focusa_trajectory_resume`](tools/focusa_trajectory_resume.md)

Boundary: Trajectory proposes and orients; Workpoint checkpoint/resume remains canonical continuation authority, Beads remains task authority, and operator steering wins.

Hierarchy: Trajectory can expose high/mid/low goal similarity for advisory grouping, but session authority remains `project_root + continuity_id`. Same high-level groups never merge sessions when mid/low goals or continuity IDs differ.
