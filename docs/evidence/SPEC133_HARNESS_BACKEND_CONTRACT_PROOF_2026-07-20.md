# Spec 133 harness adapter and backend capability proof

Work item: `focusa-a6yq6.4.3`

Spec slice: Spec 133 §14 and §27 adapter/backend capability and version negotiation

## Implemented contract

- `crates/focusa-core/src/silent_session_protocol.rs`
  - shared `native` / `emulated` / `heuristic` / `unsupported` capability truth;
  - required support levels distinguish merely available, deterministic, and native behavior;
  - highest-common protocol version negotiation fails closed for empty or incompatible offers.
- `crates/focusa-harness-adapters/src/contract.rs`
  - versioned `HarnessAdapter` interface;
  - all 17 required §14 harness capabilities are explicit serialized fields;
  - required unsupported capabilities return typed negotiation failures.
- `crates/focusa-harness-adapters/src/pi_rpc.rs`
  - Pi RPC capability declaration reflects the published LF-delimited JSONL protocol;
  - prompt, steering, follow-up, abort, state/model query, native session switch, and semantic event translation use typed protocol frames;
  - Pi's missing upstream version handshake, special-key control, hard pause, and subscription entitlement probe are explicit limitations;
  - launch construction pins RPC mode, provider, model, thinking level, and approved noninteractive trust instead of inheriting ambient values.
- `crates/focusa-harness-adapters/src/fake.rs`
  - deterministic adapter implements the full contract with repeatable event parsing, state, model, and control logs.
- `crates/focusa-harness-adapters/src/generic.rs`
  - generic RPC is deny-by-default until a reviewed protocol profile opts into capabilities;
  - generic PTY declares merged streams and heuristic semantic/steering posture.
- `crates/focusa-session-runner/src/backend.rs`
  - all 15 required §14 process-backend capabilities are explicit serialized fields;
  - the direct POSIX backend binds the protected owner-scoped supervisor and declares only proved detach, client independence, and process-tree kill behavior;
  - unavailable generic PTY and Windows Job/ConPTY backends reject required capabilities rather than silently falling back.

Unknown additive Pi events are preserved as `harness.unknown`; no adapter output becomes a canonical daemon fact by itself.

## Focused runtime evidence

```text
$ CARGO_INCREMENTAL=0 cargo test -p focusa-harness-adapters --test adapter_contract_runtime
running 4 tests
... 4 passed; 0 failed

$ CARGO_INCREMENTAL=0 cargo test -p focusa-session-runner --test protected_runner_e2e
running 1 test
... 1 passed; 0 failed

$ CARGO_INCREMENTAL=0 cargo test -p focusa-session-runner backend::tests
running 3 tests
... 3 passed; 0 failed

$ CARGO_INCREMENTAL=0 cargo test -p focusa-core silent_session_protocol::tests
running 2 tests
... 2 passed; 0 failed
```

The protected-runner E2E executes through `DirectProcessBackend`, proves owner-scoped process launch and process-group identity, performs the authenticated runner handshake, evaluates exact adoption, and terminates the owned tree.

## Strict lint and format evidence

```text
$ CARGO_INCREMENTAL=0 cargo clippy -p focusa-core -p focusa-harness-adapters -p focusa-session-runner --all-targets -- -D warnings
Finished `dev` profile ...

$ cargo fmt --all -- --check
exit 0
```

No local build artifact is a release or deployment artifact.
