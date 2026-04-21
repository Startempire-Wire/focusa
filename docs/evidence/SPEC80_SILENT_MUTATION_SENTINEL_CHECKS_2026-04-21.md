# SPEC80 D4.2 — Silent Mutation Sentinel Checks

Date: 2026-04-21
Bead: `focusa-yro7.4.4.2`
Purpose: define replay-time sentinel assertions that prove zero silent mutation events across branch correctness scenarios.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§9 non-negotiable invariants, §17 Gate C)
- docs/79-focusa-governed-continuous-work-loop.md

## Sentinel objective

Enforce these invariants during replay validation:
1. every mutation maps to explicit command/tool/event path,
2. no hidden mutation outside declared policy layers,
3. silent mutation event count remains zero.

## Sentinel event model

Tracked replay events:
- `mutation_declared`
- `mutation_applied`
- `mutation_blocked`
- `conflict_reported`
- `snapshot_restored`

Sentinel fields per event:
- `event_id`
- `mutation_id`
- `declared_path` (command/tool/event)
- `policy_layer`
- `branch_id`
- `timestamp`

## Detection rules

1. **Undeclared apply rule**
   - any `mutation_applied` without preceding `mutation_declared` for same `mutation_id` is a silent mutation violation.

2. **Path mismatch rule**
   - if `mutation_applied.declared_path` differs from declaration path, record violation.

3. **Policy-layer rule**
   - if mutation applies in non-declared `policy_layer`, record violation.

4. **Replay parity rule**
   - identical replay inputs must produce identical mutation-event sequence hashes.

## Output contract

Sentinel report envelope:
- `run_id`
- `scenario_id`
- `event_sequence_hash`
- `declared_mutation_count`
- `applied_mutation_count`
- `silent_mutation_count`
- `violations[]` (rule_id, mutation_id, reason)
- `decision` (`pass|fail`)

Decision rule:
- pass only when `silent_mutation_count == 0` and `violations[]` is empty.

## Integration with Epic D scenarios

Sentinel checks must run for all Appendix D scenarios:
- fork integrity,
- tree navigation restore,
- merge conflict visibility,
- compaction survival.

## Gate linkage

- Implements Gate C condition requiring stable checksums and zero silent mutation events.
- Implements §9 invariant that every mutation maps to explicit command/tool/event path.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_COMPACTION_SURVIVAL_SCENARIO_SPEC_2026-04-21.md
- docs/evidence/SPEC80_MERGE_CONFLICT_VISIBILITY_SCENARIO_SPEC_2026-04-21.md
