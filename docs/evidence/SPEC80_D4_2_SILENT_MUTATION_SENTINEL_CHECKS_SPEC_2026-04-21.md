# SPEC80 D4.2 — Silent Mutation Sentinel Checks Spec

Date: 2026-04-21
Bead: `focusa-yro7.4.4.2`
Label: `documented-authority`

Purpose: define sentinel checks proving zero silent mutation events across replay and compaction scenarios.

## Sentinel rule

Every state mutation must map to explicit command/tool/event path; un-attributed mutation is a gate failure.

## Check set

1. Replay mutation attribution
- For each replay op, verify one attributable mutation event record exists.

2. Restore mutation attribution
- For exact/merge restore, verify explicit restore event and associated deltas.

3. Compaction mutation attribution
- Verify compaction emits explicit operation event and derived effects.

4. Negative sentinel
- Scan event stream for state deltas without operation anchor.

## Failure signatures

- `SILENT_MUTATION_EVENT`
- `MISSING_MUTATION_TRACE`
- `UNATTRIBUTED_STATE_DELTA`

## Report schema

```json
{
  "run_id": "string",
  "mutations_total": 0,
  "attributed_mutations": 0,
  "silent_mutations": 0,
  "gate_pass": true,
  "violations": []
}
```

## Gate linkage

Supports Gate C and Gate E integrity requirements.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§9 invariant 1, §10 Gate C/E)
- docs/evidence/SPEC80_D1_1_FORK_INTEGRITY_SCENARIO_SPEC_2026-04-21.md
