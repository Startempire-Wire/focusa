# Spec89 Phase 4 Work-loop UX and Metacog Quality Evidence — 2026-04-28

Active phase: `focusa-bcyd.5`.

## Work-loop hardening

Implemented in `apps/pi-extension/src/tools.ts`:

- `focusa_work_loop_writer_status`: read-only writer ownership surface. Reports `active_writer`, authorship mode, and which actions require writer ownership.
- `focusa_work_loop_control.preflight`: returns intended route, writer id, and `mutates=false` without changing loop state.
- Existing work-loop blocked taxonomy remains surfaced through `explainWorkLoopResult` and the Phase 1 `tool_result_v1` envelope.
- Work-loop checkpoint remains distinct from Workpoint checkpoint by tool name, family, and envelope side effects.

## Metacog hardening

Implemented in `apps/pi-extension/src/tools.ts`:

- `metacogQualityGate()` scores content length, rationale, confidence, and evidence refs.
- Metacog tool results include `quality_gate`, `evidence_refs`, and `suggested_metrics` in details.
- `focusa_metacog_capture` accepts `evidence_refs` and carries them into the quality gate.
- Suggested metrics: `retrieval_reuse`, `promotion_precision`, `failure_recurrence`.

## Validation

Commands passed:

```bash
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
./tests/spec89_tool_envelope_contract_test.sh
```

Envelope skeleton now checks:

- 40 current `focusa_*` tools.
- `FocusaToolResultV1` helper.
- `withToolResultEnvelope` wrapper.
- `metacogQualityGate` helper.
- `focusa_work_loop_writer_status` tool.

## Phase 4 acceptance summary

- Agents can read writer ownership without mutation.
- Control actions have mutation-free preflight.
- Work-loop blocked state remains typed through envelope/error/retry fields.
- Metacog outputs carry quality gate, evidence refs, and metric suggestions.
- Promotion policy is represented as quality-gate recommendation; deeper reducer enforcement remains a later hardening extension if required.
