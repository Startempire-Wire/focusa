# Spec 133 §3.5 provider/model safety proof — 2026-07-20

## Scope

Work item: `focusa-a6yq6.4.5`

Normative source: `docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md` §16 and Phase 3.

This slice implements deterministic provider/model policy in the harness-adapter boundary. The daemon remains canonical, and launching a runner does not grant project-mutation authority.

## Implemented contracts

- `crates/focusa-harness-adapters/src/model_safety.rs`
  - versioned evidence, verdict, runtime-confirmation, and model-switch schemas;
  - complete typed checks for provider configuration, authentication availability/type, entitlement, exact model availability, thinking support, context window, rate posture, billing/usage budget, and catalog freshness;
  - provenance and freshness validation for every check;
  - strict fail-closed handling for blocked, stale, missing, duplicate, or unknown required evidence;
  - explicit allowlist-only fallback with a typed trigger, trigger evidence, and operator-notification reference;
  - exact-selection prohibition on fallback;
  - requested/effective/observed binding transitions on `SilentSessionRun` without synthesizing observed truth;
  - runtime `model.mismatch` verdict requiring mutation barrier, controlled abort, blocked state, and operator notification;
  - `model_switch` proof requiring Workpoint checkpoint, config revision, preflight, refreshed bootstrap, runtime confirmation, event, receipt, and a new generation unless safe in-place switching is proven.
- `crates/focusa-harness-adapters/src/pi_rpc.rs`
  - strict entitlement preflight now blocks as `entitlement_unknown` because Pi RPC truthfully declares its entitlement probe unsupported;
  - required runtime model confirmation blocks when deterministic model observation is unavailable.
- `crates/focusa-harness-adapters/tests/model_safety_runtime.rs`
  - deterministic strict-preflight, exact-match/mismatch, run-persistence, fallback, evidence-freshness, and model-switch proof coverage.
- `crates/focusa-harness-adapters/tests/adapter_contract_runtime.rs`
  - Pi RPC unsupported-entitlement fail-closed regression proof.

## Verification

```text
CARGO_INCREMENTAL=0 cargo test -p focusa-harness-adapters
  adapter_contract_runtime: 6 passed
  model_safety_runtime: 5 passed
  doc tests: passed

CARGO_INCREMENTAL=0 cargo clippy -p focusa-harness-adapters --tests -- -D warnings
  passed

CARGO_INCREMENTAL=0 cargo test -p focusa-core silent_session_bootstrap::tests --lib
  3 passed; existing exact model/bootstrap/lease/Context Authority mutation barrier remains green

cargo fmt --all -- --check
  passed

git diff --check
  passed
```

## Fail-closed boundaries and remaining gate work

Pi RPC does not expose subscription-entitlement truth. Focusa therefore does not infer entitlement from configured-model discovery or ambient authentication; a profile requiring entitlement proof is blocked until a deterministic probe supplies fresh evidence.

This slice provides and tests the model-safety state machine and integrates strict capability gating into the Pi adapter. It does not claim the Phase 3 final gate: real Pi provider/bootstrap/stream/control proof, daemon-facade parity (`focusa-a6yq6.4.6`), and the combined `.4.7` gate remain separate work.

No merge, push, install, deploy, release, runtime termination, or mutation of unrelated dirty files occurred.
