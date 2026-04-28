# Spec89 Urgent Focusa Tool Failure Fix Validation — 2026-04-28

## Scope

Urgent failure lane under `focusa-bcyd.9`.

Validated fixes for inventory findings F1–F3 from `docs/evidence/SPEC89_TOOL_FAILURE_INVENTORY_2026-04-28.md`.

## Fixes validated

### F1 — Workpoint checkpoint reason taxonomy mismatch

- API now accepts `checkpoint_reason=operator_checkpoint`.
- CLI now passes `--reason operator_checkpoint` through as `operator_checkpoint`.
- Pi tool schema describes `operator_checkpoint` as supported.

Live API result:

```json
{"canonical":true,"idempotent_replay":false,"status":"accepted"}
```

HTTP: `200`.

Live CLI result:

```json
{"canonical":true,"idempotent_replay":false,"status":"accepted"}
```

### F2 — Workpoint validation error reporting

Unsupported reason `not_a_reason` now returns a typed validation envelope instead of a generic schema failure.

Live API result:

```json
{
  "status":"validation_rejected",
  "field":"checkpoint_reason",
  "rejected_value":"not_a_reason",
  "allowed_values":["manual","operator_checkpoint","session_start","session_resume","before_compact","after_compact","context_overflow","model_switch","fork","unknown"],
  "retry_posture":"do_not_retry_unchanged"
}
```

HTTP: `422`.

Pi tool summary path was patched to render `workpoint checkpoint validation_rejected` with field, allowed values, and retry posture.

### F3 — Workpoint drift false positive

The stress no-drift case no longer trips `do_not_drift_boundary` on substring token matches such as `tools` inside `FocusaToolSuite`.

Regression unit test added: `drift_classifier_does_not_match_boundary_tokens_inside_compound_words`.

### F4 — Operator directive constraints with `must not`

Pi constraint validation now treats `source=operator directive` or constraint text beginning `Operator directive` as a discovered requirement, so operator-authored `must/must not` language is not mistaken for an agent self-imposed task.

Regression stress case added: `focus_update_constraint` writes `Operator directive must not demote existing Focusa tools.` through Focus State update and expects `status=accepted`.

### F5 — Focus State false-offline recovery regression

The live stress suite now exercises Focus State writes after daemon restart/health verification. Current run did not reproduce false-offline behavior.

## Validation commands

```bash
cargo test -p focusa-api workpoint --target-dir /tmp/focusa-cargo-target
cargo test -p focusa-cli workpoint --target-dir /tmp/focusa-cargo-target
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
cargo build --release -p focusa-api -p focusa-cli --target-dir /home/wirebot/focusa/target
systemctl restart focusa-daemon.service
./tests/focusa_tool_stress_test.sh
```

## Results

- `cargo test -p focusa-api workpoint`: 8 passed, 0 failed.
- `cargo test -p focusa-cli workpoint`: 2 passed, 0 failed.
- Pi extension TypeScript check: passed.
- Release build: passed.
- Daemon restart health: HTTP 200, `ok=true`.
- Full stress suite before F4/F5 regression expansion: `passed=36 failed=0`.
- Full stress suite after F4/F5 regression expansion: `passed=37 failed=0`.

## Artifact snippets

Raw live validation artifacts were written under `/tmp/focusa-failure-fix-validation/` during the run.
