# Focusa Native Dogfood

Focusa dogfood is **not** WPUIAI deluxe dogfood. It validates Focusa as an agent cognition/continuity system.

## Goal

Stress test and improve all Focusa subsystems so LLM agents get better UX, stronger tool/cognitive ability, clearer recovery, durable continuity, and higher productivity over long timelines.

## Script

```bash
bash tests/focusa_dogfood_test.sh
```

Optional wider gates:

```bash
FOCUSA_DOGFOOD_MUTATING_LOOP=1 bash tests/focusa_dogfood_test.sh
FOCUSA_DOGFOOD_SLOW=1 bash tests/focusa_dogfood_test.sh
FOCUSA_DOGFOOD_KEEP_ARTIFACTS=1 bash tests/focusa_dogfood_test.sh
```

Environment knobs:

- `FOCUSA_API_BASE_URL` — daemon base URL, default `http://127.0.0.1:8787`.
- `FOCUSA_DOGFOOD_PROJECT_ROOT` — project authority root, default repo root.
- `FOCUSA_DOGFOOD_CONTINUITY_ID` — stable logical dogfood run id.
- `FOCUSA_DOGFOOD_WRITER_ID` — writer id for daemon state writes.
- `FOCUSA_DOGFOOD_MUTATING_LOOP=1` — also run the existing broader `tests/focusa_tool_stress_test.sh`.
- `FOCUSA_DOGFOOD_SLOW=1` — also run `cargo test --workspace`.
- `FOCUSA_DOGFOOD_KEEP_ARTIFACTS=1` — keep `/tmp/focusa-dogfood.*` artifacts.

## Coverage

The dogfood script exercises these Focusa-native gates:

1. **Static tool UX contract**
   - `apps/pi-extension && npx tsc --noEmit`
   - `node scripts/validate-focusa-tool-contracts.mjs --json`

2. **Daemon health and inspectability**
   - `/v1/health`
   - `/v1/status?summary_only=true`
   - `/v1/resource/mode`
   - `/v1/ontology/tool-contracts`

3. **Trajectory clarity**
   - `/v1/trajectory/view`
   - `/v1/trajectory/assess`

4. **Workpoint continuity**
   - `/v1/workpoint/checkpoint`
   - `/v1/workpoint/current`
   - `/v1/workpoint/evidence/link`
   - `/v1/workpoint/resume`

5. **Evidence and bounded context recovery**
   - `/v1/traverse` recent evidence slice
   - `/v1/ontology/context` active mission slice

6. **Metacognition and prediction loop**
   - `/v1/predict/record`
   - `/v1/predict/evaluate`
   - `/v1/metacognition/capture`
   - `/v1/metacognition/retrieve`

7. **Resource pressure**
   - `tests/spec96_lowmem_surgical_agent_stress_test.sh`
   - Confirms LowMem keeps tools callable, hot routes recover, full-payload cold routes degrade explicitly, Workpoint evidence remains bounded, and summary/traverse recovery works.

8. **Optional broader live stress**
   - `tests/focusa_tool_stress_test.sh`
   - More mutating, broader API/CLI coverage.

9. **Optional slow workspace gate**
   - `cargo test --workspace`

## Pass criteria

A Focusa dogfood run passes when:

- Agent-facing tool contracts are typed and documented.
- The daemon is reachable and inspectable.
- Trajectory view/assessment returns actionable project-bound state.
- Workpoint checkpoint/current/evidence/resume roundtrip works.
- Evidence/context recovery can be performed from bounded slices, not transcript memory.
- Prediction/metacog surfaces accept bounded reusable learning signals.
- LowMem/resource pressure does not hide official tools or cause health restart storms.
- Explicit bounded degradation (`status=pending`, `failure_class=resource_exhausted`, `retry_posture=safe_retry`) counts as an agent-UX pass because the system returned actionable recovery instead of ambiguity.
- Failures include concrete envelopes/artifacts in `/tmp/focusa-dogfood.*`.

## Non-goals

- It does not validate WPUIAI Mimic/Agentic Loop flows.
- It does not prove live release binaries were rebuilt/restarted unless paired with release proof.
- It does not run destructive service/data operations.

## Recommended release posture

Use this before claiming agent-UX readiness:

```bash
bash tests/focusa_dogfood_test.sh
FOCUSA_DOGFOOD_MUTATING_LOOP=1 FOCUSA_DOGFOOD_SLOW=1 bash tests/focusa_dogfood_test.sh
```

Then capture evidence with `focusa_evidence_capture` or `focusa_workpoint_link_evidence` using the script path and preserved artifact directory.
