# Spec 133 Phase 8.1 — generic RPC and PTY adapters

Date: 2026-07-23
Bead: `focusa-a6yq6.9.1`
Scope: generic optional adapter truth

## Generic RPC

The base descriptor remains deny-by-default: JSONL framing alone grants no command or semantic capabilities.

A reviewed `GenericRpcProtocolProfile` must explicitly declare:

- profile/event/output fields;
- verified semantic event kinds;
- control method mappings;
- full `HarnessCapabilities` support matrix.

Frames with undeclared/unverified kinds are emitted only as `generic_rpc.frame`, runtime-observed raw payloads with no semantic label. A structured semantic label requires both native structured-events and native semantic-state capabilities plus an exact verified-kind mapping.

RPC controls require both a declared capability and exact method mapping. Unsupported or unmapped prompt/steering/follow-up/key operations fail closed.

## Generic PTY

PTY chunks are emitted as runtime-observed `terminal.output` with `stdout_stderr_merged = true` and `semantic_claim_verified = false`.

Prompt/blocker recognition is optional heuristic metadata only:

- provenance is `terminal_inferred`;
- confidence remains below `0.5`;
- `verified = false`;
- no heuristic text becomes a structured verified event.

PTY controls are limited to newline-delivered text and explicit `ENTER`, `ESCAPE`, and `CTRL_C` byte sequences. Unknown keys fail closed.

## Local non-building proof

Per operator policy, no local Cargo, CI, compilation, or tests were run.

```bash
rustfmt --edition 2024 --check \
  crates/focusa-harness-adapters/src/generic.rs
git diff --check
```

Result: passed.

## Required server proof

```bash
cargo test -p focusa-harness-adapters generic -- --nocapture
cargo test -p focusa-harness-adapters
cargo clippy -p focusa-harness-adapters --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Server tests must prove deny-by-default RPC, native capability requirements, undeclared raw framing, control mapping rejection, merged PTY output, bounded heuristic confidence/provenance, and special-key bounds.

## Gate disposition

Implementation and local static review are complete. Build/test closure remains server-owned and must pass before this bead is marked fully proven.
