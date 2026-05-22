# Spec96 LowMem Surgical-Agent Stress

`tests/spec96_lowmem_surgical_agent_stress_test.sh` forces LowMem and verifies:

- no official Focusa tool disappears from the contract registry;
- representative hot routes stay callable;
- a cold route pressure probe does not create a healthcheck restart storm or uptime reset;
- cold full-payload routes expose explicit degradation/opt-in metadata;
- a fresh-agent surgical task completes using `surgical_summary_only` ontology context, identity axes, rehydrate refs, and a targeted `focusa_traverse` evidence slice.

Companion coverage:

- `tests/spec96_lowmem_surgical_agent_static_test.sh` guards the scripted proof shape and docs hooks.
- `tests/spec96_lowmem_tool_dependencies_runtime_test.sh` uses an isolated temp directory so repeated runs do not fail on stale `/tmp` ownership, and captures live-proof JSON before failing.

Validation evidence, 2026-05-21:

```bash
/tmp/run_lowmem_validate_root.sh
```

The validation started an isolated current-source daemon on `127.0.0.1:8790` with `FOCUSA_DATA_DIR=/tmp/...` and `CARGO_TARGET_DIR=/tmp/...` because the `wirebot` shell cannot access the root-owned Cargo toolchain. The active production daemon on `127.0.0.1:8787` was not restarted.

Result:

```text
SPEC96 LowMem surgical-agent static test: PASS
SPEC96 LowMem tool dependency runtime test: PASS
SPEC96 LowMem surgical-agent stress test: PASS
```
