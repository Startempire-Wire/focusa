# Spec 130A proactive-compaction gates — source/runtime proof (2026-07-21)

## Scope

This evidence covers the Pi-extension proactive-compaction control path in:

- `apps/pi-extension/src/auto-compaction.ts`
- `apps/pi-extension/src/index.ts`
- `apps/pi-extension/src/compaction.ts`
- `apps/pi-extension/tests/auto-compaction-resilience.test.mjs`
- `apps/pi-extension/tests/spec101-bloatgaurd-pressure-circuit-runtime.test.mjs`
- `tests/spec130a_proactive_compaction_runtime_test.sh`

It does **not** establish installed-extension, real-provider billing, or live long-session acceptance. Those gates remain open.

## Implemented controls

1. `Symbol.for("focusa.compaction.coordinator.v1")` owns a process-wide, versioned first-owner lease binding adapter instance, extension build/source, attachment/native session, registered handlers, and active epoch.
2. A second independently evaluated Focusa extension module emits one bounded diagnostic identifying the active owner and installation repair, returns before registering any handler, and causes `index.ts` to register nothing else from that duplicate package.
3. There is exactly one extension-owned `ctx.compact()` call site. The former hard/auto tier call sites in `compaction.ts` route through `requestCoordinatedCompaction()`.
4. Each epoch has a deterministic SHA-256 identity, explicit coordinator state, and `nativeCompactionCallCount`; invocation is blocked once the count reaches one. A retry is a distinct epoch linked by `retryOfEpochId`.
5. Context pressure is necessary but not sufficient. A conservative preflight uses Pi's exported `findCutPoint()`, `estimateTokens()`, and bundled compaction defaults.
6. The eligibility handler registers before Focusa's authoritative checkpoint/summary handler. `session_before_compact` applies a second gate to Pi's live `CompactionPreparation` before daemon checkpoint work or any summarization model call. Native automatic compaction is observed and subjected to the same exact gate; explicit manual compaction outranks automatic ROI optimization.
7. ROI uses a fail-closed maximum summary allowance: `0.8 * reserveTokens` for history or `1.3 * reserveTokens` for history plus split-turn prefix, plus bounded continuation overhead.
8. Empty sessions, already-compacted leaves, migration-invalid entries, insufficient history, insufficient reclaim, and negative ROI do not run summarization. Native `Nothing to compact` and `Already compacted` are terminal for unchanged semantic context.
9. At most one linked transient retry is permitted after the failed epoch settles and its primary error is persisted. The retry waits for the configured cooldown, then rechecks idle state, pending input, live pressure, semantic context, and intervening native/manual completion.
10. The initiating provider error remains `primary_error`; duplicate settlement and undefined-`signal`/re-entrancy failures are emitted separately as secondary classifications.
11. Durable custom entries record coordinator state, native-call count, attempt start/completion, failure/rejection, retry scheduling/suppression, token savings, and net-positive outcome.
12. Compaction instructions preserve current ask, `project_root + continuity_id`, Workpoint/Trajectory authority, evidence, blockers, exact next action, and do-not-drift boundaries.

## Static and compile proof

Run from `apps/pi-extension`:

```bash
npm run typecheck
npx eslint src/auto-compaction.ts src/compaction.ts src/index.ts tests/auto-compaction-resilience.test.mjs tests/spec101-bloatgaurd-pressure-circuit-runtime.test.mjs --max-warnings=0
node --test tests/auto-compaction-resilience.test.mjs tests/spec101-bloatgaurd-pressure-circuit-runtime.test.mjs
```

Result:

```text
typecheck: PASS
strict targeted ESLint: PASS
auto-compaction resilience: 10/10 PASS
Spec 101 coordinated pressure circuit: PASS
```

Targeted Prettier and repository `git diff --check` also pass.

## Emitted-JavaScript runtime proof

The reproducible test `tests/spec130a_proactive_compaction_runtime_test.sh` emits the extension to an isolated temporary directory, links only to the worktree's existing `node_modules`, and exercises deterministic fake Pi lifecycle/context adapters. No installation or provider call occurs.

```bash
tests/spec130a_proactive_compaction_runtime_test.sh
```

### Eligibility and ROI

```text
status: pass
compactableTokens: 75000
estimatedOverheadTokens: 14132
estimatedNetSavingsTokens: 60868
empty_session: rejected
already_compacted: rejected
```

### Registration, terminal suppression, and durable events

```text
status: pass
three independently evaluated module instances: one owner, two duplicates
owner handler registrations: present
duplicate handler registrations: zero
duplicate diagnostics: one bounded warning with active owner + repair action
concurrent agent_settled events: one native call
eligible events: attempt_started, attempt_completed
unchanged empty-context events: preflight_rejected (once)
unchanged empty-context notices: one
```

### Exact live-preparation rejection

```text
status: pass
events: attempt_started, eligibility_rejected
reason: insufficient_reclaim
compactableTokens: 25
estimatedOverheadTokens: 14132
retry_scheduled: false
```

### Linked transient retry and error precedence

```text
status: pass
attempt 1: attempt_started -> attempt_failed(primary WebSocket error) -> retry_scheduled
secondary callback: secondary_duplicate_settlement / secondary_reentrancy / undefined signal
attempt 2: distinct deterministic epoch_id, retry_of_epoch_id=<attempt-1>, attempt_started -> attempt_completed
native_compaction_call_count: 1 in each epoch
```

## Upstream reconciliation

`git fetch --prune origin` advanced the inspected upstream tip to `7839b234e9986273826790f1802fa6e32f74e1da`. The execution branch was 137 commits ahead and 49 behind `origin/main`, with extensive concurrent dirty state; no pull or merge into that worktree was safe. The compaction-scoped patch was instead applied cleanly to an isolated branch created directly from the fetched `origin/main`. Inspection found the normative Spec 130A addendum upstream but no newer single-coordinator source implementation to import.

## Wider Pi-extension suite

A full `node --test tests/*.test.mjs` run on the upstream-reconciled publish tree passed 19/19 tests. The dirty execution worktree separately passed 22/22 because it contains three additional unrelated local tests not present on `origin/main`. The pre-existing encoded-space test-path defect in `tests/spec104-pi-runtime-isolation.test.mjs` was corrected with Node's standard `fileURLToPath()` conversion.

No release, publication, deployment, or installed-extension mutation was performed. Read-only parity inspection confirms the active installed source is still older and unequal:

```text
worktree auto-compaction.ts sha256: 49976f4b9ad4e0aeaa3391ec51d3d936fcdc7e41e2d3a2b3d862970fa06cad92
installed auto-compaction.ts sha256: 4bba46005056190f3100868e093d16da81289f1fc29266566fa1dad21f311a57
```

Therefore this evidence is not an installed-runtime closure claim.

## Remaining acceptance

Before Spec 130A proactive compaction can be closed, run an installed-runtime long-session proof that demonstrates:

- one eligible native compaction call per epoch;
- no model call for exact-gate rejection or terminal no-op;
- actual post-compaction context release and productive continuation;
- provider token/cache telemetry and net-positive cost;
- crash/resume preservation of authoritative continuation fields;
- no duplicate warnings or retries across native/manual compaction races.
