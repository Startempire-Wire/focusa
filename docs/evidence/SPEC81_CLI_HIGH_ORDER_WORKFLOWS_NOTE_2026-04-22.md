# SPEC81 CLI High-Order Workflows Note

**Date:** 2026-04-22  
**Spec:** `docs/81-focusa-llm-tool-suite-and-cli-development-reset-spec.md`  
**Task:** `focusa-p5hr.3`

## What was added

New higher-level CLI workflows:
- `focusa metacognition loop run`
- `focusa metacognition promote`
- `focusa metacognition doctor`
- `focusa lineage compare`

## Key code

- metacognition command surface: `crates/focusa-cli/src/commands/metacognition.rs:9`
- loop subcommand: `crates/focusa-cli/src/commands/metacognition.rs:55`
- local typed input errors: `crates/focusa-cli/src/commands/metacognition.rs:112`
- loop workflow payload/result: `crates/focusa-cli/src/commands/metacognition.rs:227`
- promote workflow: `crates/focusa-cli/src/commands/metacognition.rs:255`
- doctor workflow: `crates/focusa-cli/src/commands/metacognition.rs:315`
- lineage compare command: `crates/focusa-cli/src/commands/lineage.rs:43`
- lineage compare execution: `crates/focusa-cli/src/commands/lineage.rs:129`
- CLI input error classification: `crates/focusa-cli/src/main.rs:172`
- retrieve payload now exposes confidence/kind for doctor: `crates/focusa-api/src/routes/metacognition.rs:351`

## Live smoke checks run

```bash
cargo run -q -p focusa-cli -- --json metacognition loop run --kind workflow_smoke --content 'spec81 loop smoke' --current-ask 'spec81 loop smoke' --turn-range '1-3' --failure-class retry_drift --observed-metric latency_drop
cargo run -q -p focusa-cli -- --json metacognition promote --reflection-id <reflection_id> --observed-metric stability_gain
cargo run -q -p focusa-cli -- --json metacognition doctor --current-ask 'spec81'
cargo run -q -p focusa-cli -- --json lineage compare --from-snapshot-id <snap1> --to-snapshot-id <snap2>
```

## Live results

- loop run: returned workflow `metacognition_loop_run`, result `improved`, promote `true`
- promote: returned workflow `metacognition_promote`, decision `promote`
- doctor: returned workflow `metacognition_doctor` with candidate counts and kind breakdown
- lineage compare: returned `status=ok` with compare delta fields

## Build check

```bash
cargo check -p focusa-cli -p focusa-api
```

Result: pass.
