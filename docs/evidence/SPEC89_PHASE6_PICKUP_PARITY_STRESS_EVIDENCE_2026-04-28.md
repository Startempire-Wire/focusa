# Spec89 Phase 6 Pickup, Parity, and Operational Stress Evidence — 2026-04-28

Active phase: `focusa-bcyd.7`.

## Pickup/desirability changes

Recent tool descriptions and visible summaries now emphasize safety/payoff:

- `focusa_tool_doctor`: first diagnostic entrypoint for blocked/degraded/stale tool state.
- `focusa_active_object_resolve`: resolves candidates without inventing canonical refs.
- `focusa_evidence_capture`: keeps evidence handles out of raw transcript and links to Workpoint.
- `focusa_work_loop_writer_status`: shows writer ownership without mutation.
- `focusa_state_hygiene_*`: proposal-first non-destructive hygiene flow.
- All Focusa tools receive `details.tool_result_v1` with retry/side-effect/canonical/degraded guidance.

## Parity and stress validation

Commands passed:

```bash
./tests/spec89_tool_envelope_contract_test.sh
cargo check -p focusa-api -p focusa-cli -p focusa-core --target-dir /tmp/focusa-cargo-target
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
./tests/focusa_tool_stress_test.sh
```

Final live stress result:

```text
passed=38 failed=0
```

## Degraded fallback and compaction survival

- Workpoint canonical/degraded semantics remain explicit in `tool_result_v1` and Workpoint resume packets.
- Non-canonical fallback is not silently promoted.
- Long-session compaction survival remains anchored to Spec88 Workpoint continuity and the active Pi extension wrapper; next real compaction cycle should continue to be observed operationally.

## Performance gates

Heavy read endpoints exercised in stress:

- lineage head/tree/path
- focus snapshots/diff
- ontology primitives/world/slices
- work-loop status

All passed in live stress run after one transient work-loop timeout retry.

## Operational rollout

Release daemon was previously rebuilt/restarted during Phase 2/3 validation; current Phase 6 gates confirm live daemon and CLI/Pi-equivalent surfaces remain healthy.
