# SPEC80 B3.2 — Planned Endpoint Backlog Derivation

Date: 2026-04-21
Bead: `focusa-yro7.2.3.2`
Label: `planned-extension`

Purpose: derive explicit implementation backlog from Appendix B rows marked ❌.

## Derived backlog rows

1. `POST /v1/focus/snapshots`
- Tool: `focusa_tree_snapshot_state`
- Required bead linkage: `focusa-yro7.2.1.2`, `focusa-yro7.4.2.2`

2. `POST /v1/focus/snapshots/restore`
- Tool: `focusa_tree_restore_state`
- Required bead linkage: `focusa-yro7.2.1.2`, `focusa-yro7.4.2.1`

3. `POST /v1/focus/snapshots/diff`
- Tool: `focusa_tree_diff_context`
- Required bead linkage: `focusa-yro7.2.1.2`, `focusa-yro7.4.1.2`

4. `POST /v1/metacognition/capture`
- Tool: `focusa_metacog_capture`
- Required bead linkage: `focusa-yro7.2.2.1`, `focusa-yro7.5.3.1`

5. `POST /v1/metacognition/retrieve`
- Tool: `focusa_metacog_retrieve`
- Required bead linkage: `focusa-yro7.2.2.1`, `focusa-yro7.4.3.1`

6. `POST /v1/metacognition/reflect`
- Tool: `focusa_metacog_reflect`
- Required bead linkage: `focusa-yro7.2.2.1` (with `/v1/reflect/*` compatibility note)

7. `POST /v1/metacognition/adjust`
- Tool: `focusa_metacog_plan_adjust`
- Required bead linkage: `focusa-yro7.2.2.2`, `focusa-yro7.5.4.2`

8. `POST /v1/metacognition/evaluate`
- Tool: `focusa_metacog_evaluate_outcome`
- Required bead linkage: `focusa-yro7.2.2.2`, `focusa-yro7.5.4.1`

## CLI parity backlog (Appendix B fallback column)

- `focusa lineage head|path --json`
- `focusa state snapshot create|restore|diff --json`
- `focusa metacognition capture|retrieve|reflect|adjust|evaluate --json`

Primary execution lane: Epic C (`focusa-yro7.3.*`).

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§15, §20.1)
- docs/evidence/SPEC80_SECTION20_DECOMPOSITION_LANES_2026-04-21.md
