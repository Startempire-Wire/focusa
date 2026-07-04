# Project Directory Detection

Status: core authority requirement for GH #4 / `focusa-gh-4-perpetua-scope`.

## Problem

Focusa must scope parent projects, child repositories, subdomain apps, and folder-based projects consistently before Workpoint/Trajectory authority is accepted. Perpetua vs Focusa is the visible failure, but the root cause is generic directory detection.

## Detector inputs

Directory detection must consider, in order:

1. explicit safe `project_root`
2. `.focusa-project.json` marker nearest to the active folder
3. current ask alias/domain hints, e.g. `child.example.com`
4. repo fingerprint / `.git` root
5. `.beads` root
6. workspace markers (`Cargo.toml`, `package.json`, etc.)
7. durable project-switch ledger / persisted identity as corroboration only
8. unsafe-root refusal for broad roots like `/root`, `/home`, `/tmp`

## Authority rule

The canonical project root must be selected by the reusable detector before any surface treats Workpoint/Trajectory state as canonical. Similarity/fallback trajectories may orient, but may not merge authority across project roots or continuity IDs.

## Surfaces that must obey this rule

- `focusa_project_identity`
- `focusa_project_verify`
- `focusa_project_card`
- `focusa_workpoint_resume`
- `focusa_workpoint_checkpoint`
- `focusa_trajectory_view`
- Pi Focus Slice / current-ask scope detection
- project switch ledger / alias-domain observations

## Acceptance proof

- API detector markers: `crates/focusa-api/src/routes/project.rs`
- Pi detector markers: `apps/pi-extension/src/state.ts`
- Static guard: `tests/spec_focusa_g4_perpetua_scope_static_test.sh`
- Portable path guard: `tests/spec96_portable_identity_paths_static_test.sh`

## Non-goals

No project-specific hardcoded root workaround. Detector logic must work for any parent/child/subdomain/folder project layout.
