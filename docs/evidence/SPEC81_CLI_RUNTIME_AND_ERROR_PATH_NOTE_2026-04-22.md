# SPEC81 CLI Runtime + Error Path Note

**Date:** 2026-04-22  
**Spec:** `docs/81-focusa-llm-tool-suite-and-cli-development-reset-spec.md`  
**Task:** `focusa-p5hr.5`

## What was tested

The new high-level CLI commands were tested in both success and failure modes:
- `focusa metacognition loop run`
- `focusa metacognition promote`
- `focusa metacognition doctor`
- `focusa lineage compare`

## Runtime test file

- `tests/spec81_cli_high_order_runtime_test.sh`

## What the test checks

### Success cases
- JSON output works
- human-readable output works
- commands run against the live daemon

### Error cases
- blank required CLI input returns typed JSON code `CLI_INPUT_ERROR`
- missing upstream records return typed JSON code `API_HTTP_ERROR`

## Key code

- loop command surface: `crates/focusa-cli/src/commands/metacognition.rs:55`
- promote command surface: `crates/focusa-cli/src/commands/metacognition.rs:57`
- doctor command surface: `crates/focusa-cli/src/commands/metacognition.rs:68`
- local typed input errors: `crates/focusa-cli/src/commands/metacognition.rs:112`
- loop human summary: `crates/focusa-cli/src/commands/metacognition.rs:239`
- promote human summary: `crates/focusa-cli/src/commands/metacognition.rs:319`
- doctor human summary: `crates/focusa-cli/src/commands/metacognition.rs:397`
- lineage compare command: `crates/focusa-cli/src/commands/lineage.rs:43`
- lineage compare summary: `crates/focusa-cli/src/commands/lineage.rs:146`
- CLI JSON error classification: `crates/focusa-cli/src/main.rs:172`

## Command run

```bash
bash tests/spec81_cli_high_order_runtime_test.sh
```

## Result

Pass. 12 checks passed, 0 failed.
