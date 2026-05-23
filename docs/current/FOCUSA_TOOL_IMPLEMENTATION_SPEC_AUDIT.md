# Focusa Tool Implementation-to-Spec Audit

Purpose: compare the 58 current `focusa_*` tools against registered contracts, API routes, CLI parity, docs, result-envelope rules, and model-routing affordances.

## Audit command

```bash
node scripts/audit-focusa-tool-implementation-spec-gaps.mjs --json
```

The audit checks:

- Pi tool registrations match `docs/current/focusa-tool-contracts.json` and `apps/pi-extension/src/tool-contracts.ts`.
- Every declared API route exists in the Rust API route inventory.
- Every declared concrete `focusa ...` CLI command maps to an implemented CLI command/subcommand source.
- Full-parity contracts include API and CLI paths.
- Every per-tool doc includes declared API routes, CLI commands, and the `tool_result_v1` contract summary.
- `tool_result_v1` wrapping is installed before tool registration.
- Focus State family next-tool hints route outward to project identity, trajectory, and Workpoint instead of creating a scratch/decide loop.
- Friendly onboarding and choreography docs include the project route concepts.

## Latest result

```json
{
  "status": "passed",
  "tool_count": 58,
  "contract_count": 58,
  "failures": [],
  "warnings": []
}
```

Evidence artifacts:

- `/tmp/focusa-tool-implementation-spec-audit.json` — static implementation/spec audit passed.
- `/tmp/focusa-tool-contracts-live-final-uplift-pass.json` — live contract proof passed (`payload_equal=true`, 58/58).
- `/tmp/focusa-tool-stress-smoke.log` — bounded live stress passed (`passed=39 failed=0`).
- `/tmp/focusa-cli-parity-smoke.log` — concrete CLI parity smoke passed (`passed=15 failed=0`).
- `/tmp/focusa-tool-suite-safe-smoke.json` — safe suite had 0 failures; latest warning was process-memory pressure after repeated smoke/audit runs.

## Gaps found and filled in this pass

| Gap | Repair |
|---|---|
| Focus State tools claimed CLI parity through generic `focusa focus`, but CLI had only Focus Stack push/pop/set. | Added `focusa focus update` with slot flags for decisions, constraints, failures, intent, current focus, next steps, open questions, recent results, and notes. |
| Workpoint CLI checkpoint/resume/current did not expose scoped `project_root + continuity_id`, while canonical Workpoint API now requires safe scope. | Added `--project-root` and `--continuity-id` to checkpoint/current/resume; checkpoint also sends `active_object_refs` from `--target-ref`. |
| Metacognition recent helper tools had API/Pi parity but no concrete CLI subcommands. | Added `focusa metacognition recent-reflections` and `focusa metacognition recent-adjustments`. |
| Lineage Intelligence extract tool had Pi/API behavior but no concrete CLI command. | Added `focusa lineage extract` for bounded decisions/constraints/risks extraction and metacog next-tool hints. |
| Snapshot/tree tools used API routes but contracts marked them Pi-only with generic lineage CLI hints. | Updated contracts to full parity and added/declared concrete `focusa state snapshot create/recent/restore/diff/compare-latest` commands. |
| Contract docs had weak or stale CLI/API summaries for some tools. | Regenerated per-tool contract summaries and registry table from `focusa-tool-contracts.json`. |
| Model route hints allowed Focus State tools to remain too central. | Updated affordance catalog and Utility Card to route models through project identity → trajectory → Workpoint → evidence → prediction/metacog. |

## Uplift opportunities after no remaining hard gaps

- Add live smoke fixtures for the newly concrete CLI parity surfaces (`focus update`, scoped Workpoint CLI, metacog recent, lineage extract, snapshot compare-latest).
- Expose the choreography map through a machine-readable endpoint or Focus Slice section, not only docs and startup card text.
- Add per-tool success/failure example payloads to the contract registry for better model self-correction.
- Add weighted next-tool edges instead of family-level next-tool arrays so `focusa_evidence_capture` can point differently than `focusa_workpoint_checkpoint`.
- Track tool-chain outcomes as prediction/metacog evidence to learn which tool routes actually improve project results.
